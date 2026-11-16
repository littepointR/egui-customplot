use std::{
    collections::{HashMap, VecDeque},
    num::NonZeroU64,
    sync::Arc,
};

use bytemuck::{Pod, Zeroable};
use egui_wgpu::{Callback, wgpu};

use super::{
    WgpuPlotConfig,
    resources::{MAX_WAVE_COLUMNS, WgpuHostTag, lock_versions},
    waveform_callback::{WaveUpload, WaveformCallback},
};
use crate::{
    AxisScale, PlotDiagnostics, PlotTransform, RenderBackend, UniformWaveform, axis::PlotBounds,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct WaveUniforms {
    pub(super) rect: [f32; 4],
    pub(super) screen: [f32; 4],
    pub(super) x_map: [f32; 4],
    pub(super) y_map: [f32; 4],
    pub(super) color: [f32; 4],
    pub(super) counts: [u32; 4],
}

pub(super) struct WaveDrawSlot {
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) reduced_buffer: wgpu::Buffer,
    pub(super) compute_bind_group: Option<(u64, wgpu::BindGroup)>,
    pub(super) render_bind_group: wgpu::BindGroup,
    pub(super) segment_count: u32,
    pub(super) columns: u32,
}

pub(super) struct WaveformGpuState {
    pub(super) compute_bgl: wgpu::BindGroupLayout,
    pub(super) render_bgl: wgpu::BindGroupLayout,
    pub(super) compute_pipeline: wgpu::ComputePipeline,
    pub(super) render_pipeline: wgpu::RenderPipeline,
    pub(super) slots: HashMap<u64, WaveDrawSlot>,
    pub(super) slot_lru: VecDeque<u64>,
}

impl WaveformGpuState {
    pub fn new(device: &wgpu::Device, config: WgpuPlotConfig) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("customplot_waveform_wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("waveform.wgsl").into()),
        });
        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("customplot_wave_compute_bgl"),
            entries: &[
                bgl_buffer(
                    0,
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::BufferBindingType::Uniform,
                    size_of::<WaveUniforms>(),
                ),
                bgl_buffer(
                    1,
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    4,
                ),
                bgl_buffer(
                    2,
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::BufferBindingType::Storage { read_only: false },
                    8,
                ),
            ],
        });
        let render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("customplot_wave_render_bgl"),
            entries: &[
                bgl_buffer(
                    0,
                    wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    wgpu::BufferBindingType::Uniform,
                    size_of::<WaveUniforms>(),
                ),
                bgl_buffer(
                    1,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    8,
                ),
            ],
        });
        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("customplot_wave_compute_pl"),
            bind_group_layouts: &[Some(&compute_bgl)],
            immediate_size: 0,
        });
        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("customplot_wave_render_pl"),
            bind_group_layouts: &[Some(&render_bgl)],
            immediate_size: 0,
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("customplot_wave_cs"),
            layout: Some(&compute_layout),
            module: &module,
            entry_point: Some("cs_reduce"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let fs_entry = if config.target_format.is_srgb() {
            "fs_line_linear"
        } else {
            "fs_line_gamma"
        };
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("customplot_wave_rp"),
            layout: Some(&render_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_line"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: config.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: config.msaa_samples.max(1),
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(fs_entry),
                targets: &[Some(egui_blend_target(config.target_format))],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            compute_bgl,
            render_bgl,
            compute_pipeline,
            render_pipeline,
            slots: HashMap::new(),
            slot_lru: VecDeque::new(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn try_paint_waveform(
    painter: &egui::Painter,
    tag: &WgpuHostTag,
    draw_key: u64,
    wave: &UniformWaveform,
    backend: RenderBackend,
    transform: PlotTransform,
    stroke: egui::Stroke,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) -> bool {
    let total_bytes = wave.sample_bytes();
    let max_allowed = tag
        .max_binding_bytes
        .min(tag.budget_bytes.load(std::sync::atomic::Ordering::Relaxed));
    if wave.len() < 2 || total_bytes > max_allowed {
        return false;
    }
    let bounds = transform.bounds();
    let scales = transform.scales();
    let Some((start_idx, end_idx)) = visible_wave_indices(wave, bounds, scales) else {
        return true; // completely offscreen, nothing to draw
    };
    let visible_count = end_idx - start_idx;
    let ppp = pixels_per_point.max(0.01);
    let rect = transform.rect();
    let sx0 = crate::data::project_axis_raw(transform, 0, wave.x_at(start_idx))
        .unwrap_or(rect.left())
        .clamp(rect.left(), rect.right());
    let sx1 = crate::data::project_axis_raw(transform, 0, wave.x_at(end_idx - 1))
        .unwrap_or(rect.right())
        .clamp(rect.left(), rect.right());
    let wave_px_cols = (((sx1 - sx0).abs() * ppp).ceil() as usize).max(1);
    if backend == RenderBackend::Auto && (visible_count <= 4_096 || wave_px_cols <= 64) {
        return false;
    }
    let axis_lo_x = scales[0].forward(bounds.min()[0]).expect("valid");
    let axis_hi_x = scales[0].forward(bounds.max()[0]).expect("valid");
    let axis_lo_y = scales[1].forward(bounds.min()[1]).expect("valid");
    let axis_hi_y = scales[1].forward(bounds.max()[1]).expect("valid");
    let span_x = axis_hi_x - axis_lo_x;
    let span_y = axis_hi_y - axis_lo_y;
    if !span_x.is_finite() || !span_y.is_finite() || span_x == 0.0 || span_y == 0.0 {
        return false;
    }
    let x_map = if scales[0] == AxisScale::Log10 {
        [
            wave.x_at(start_idx) as f32,
            wave.dx() as f32,
            axis_lo_x as f32,
            (1.0 / span_x) as f32,
        ]
    } else {
        [
            ((wave.x_at(start_idx) - axis_lo_x) / span_x) as f32,
            (wave.dx() / span_x) as f32,
            0.0,
            0.0,
        ]
    };
    let y_map = if scales[1] == AxisScale::Log10 {
        [axis_lo_y as f32, (1.0 / span_y) as f32, 0.0, 0.0]
    } else {
        let y_ref = (0.5 * (axis_lo_y + axis_hi_y)).clamp(-1.0e30, 1.0e30) as f32;
        let y_offset = ((f64::from(y_ref) - axis_lo_y) / span_y) as f32;
        [y_ref, (1.0 / span_y) as f32, y_offset, 0.0]
    };
    let rev = transform.reversed();
    let flags = u32::from(rev[0])
        | (u32::from(rev[1]) << 1)
        | (u32::from(scales[0] == AxisScale::Log10) << 2)
        | (u32::from(scales[1] == AxisScale::Log10) << 3);

    let max_cols = (visible_count / 2).clamp(1, MAX_WAVE_COLUMNS) as f32;
    let plot_cols = (rect.width() * ppp).ceil().clamp(1.0, max_cols) as u32;
    let min_wgs = (visible_count / 32_768).clamp(1, 256) as u32;
    let columns = (wave_px_cols as u32).max(min_wgs).min(plot_cols).max(1);

    let committed = lock_versions(&tag.committed_versions)
        .get(&wave.source_id())
        .copied();
    let (upload, uploaded_bytes) = if committed == Some(wave.samples_version()) {
        (WaveUpload::None, 0)
    } else if committed == Some(wave.prev_samples_version())
        && let Some(range) = wave.dirty_sample_range()
    {
        let slice = &wave.samples()[range.clone()];
        (
            WaveUpload::Partial {
                offset_samples: range.start,
                data: Arc::from(slice),
            },
            std::mem::size_of_val(slice),
        )
    } else {
        (WaveUpload::Full(Arc::from(wave.samples())), total_bytes)
    };

    let c = stroke.color.to_array().map(|b| f32::from(b) / 255.0);
    diagnostics.wgpu_series += 1;
    diagnostics.wgpu_uploaded_bytes += uploaded_bytes;
    diagnostics.wgpu_retained_bytes += total_bytes;
    diagnostics.output_paths += 1;
    diagnostics.output_points += (columns * 4) as usize;

    let clip = painter.clip_rect().intersect(transform.rect());
    if !clip.is_positive() {
        return true;
    }
    painter.add(egui::Shape::Callback(Callback::new_paint_callback(
        clip,
        WaveformCallback {
            draw_key,
            source_id: wave.source_id(),
            samples_version: wave.samples_version(),
            total_bytes,
            upload,
            rect: transform.rect(),
            stroke_width: stroke.width,
            color: c,
            x_map,
            y_map,
            start_idx: start_idx as u32,
            end_idx: end_idx as u32,
            columns,
            flags,
        },
    )));
    true
}

fn visible_wave_indices(
    wave: &UniformWaveform,
    bounds: PlotBounds,
    scales: [AxisScale; 2],
) -> Option<(usize, usize)> {
    let x_lo = bounds.min()[0].min(bounds.max()[0]);
    let x_hi = bounds.min()[0].max(bounds.max()[0]);
    let max_x = wave.x_at(wave.len() - 1);
    if x_hi < wave.x0() || x_lo > max_x {
        return None;
    }
    let raw_lo = ((x_lo - wave.x0()) / wave.dx()).floor();
    let raw_hi = ((x_hi - wave.x0()) / wave.dx()).ceil();
    let mut lo = if raw_lo <= 0.0 {
        0
    } else {
        (raw_lo as usize).saturating_sub(1)
    };
    if scales[0] == AxisScale::Log10 && wave.x_at(lo) <= 0.0 {
        let pos_approx = ((-wave.x0()) / wave.dx()).floor().max(0.0) as usize;
        lo = lo.max(pos_approx);
        while lo < wave.len() && wave.x_at(lo) <= 0.0 {
            lo += 1;
        }
    }
    let hi = if !raw_hi.is_finite() || raw_hi >= wave.len() as f64 {
        wave.len()
    } else {
        ((raw_hi.max(0.0) as usize) + 2).min(wave.len())
    };
    (lo + 1 < hi).then_some((lo, hi))
}

pub(super) fn bgl_buffer(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BufferBindingType,
    min_size: usize,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            has_dynamic_offset: false,
            min_binding_size: NonZeroU64::new(min_size as u64),
            ty,
        },
        count: None,
    }
}

pub(super) fn egui_blend_target(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }
}
