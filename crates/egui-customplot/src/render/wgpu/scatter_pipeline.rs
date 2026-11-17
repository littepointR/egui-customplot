use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use bytemuck::{Pod, Zeroable};
use egui::Color32;
use egui_wgpu::{Callback, wgpu};

use super::{
    WgpuPlotConfig,
    resources::{WgpuHostTag, lock_versions},
    scatter_callback::ScatterCallback,
    waveform_pipeline::{bgl_buffer, egui_blend_target},
};
use crate::{
    AxisScale, MarkerSampling, MarkerShape, MarkerStyle, PlotData, PlotDiagnostics, PlotTransform,
    RenderBackend,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct GpuScatterInstance {
    x: f32,
    y: f32,
    radius: f32,
    rgba: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ScatterUniforms {
    pub(super) rect: [f32; 4],
    pub(super) screen: [f32; 4],
    pub(super) axis_min: [f32; 4],
    pub(super) ref_origin: [f32; 4],
    pub(super) default_color: [f32; 4],
}

pub(super) struct ScatterDrawSlot {
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) first_instance: u32,
    pub(super) instance_count: u32,
}

pub(super) struct ScatterGpuState {
    pub(super) bgl: wgpu::BindGroupLayout,
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) slots: HashMap<u64, ScatterDrawSlot>,
    pub(super) slot_lru: VecDeque<u64>,
}

impl ScatterGpuState {
    pub fn new(device: &wgpu::Device, config: WgpuPlotConfig) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("customplot_scatter_wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scatter.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("customplot_scatter_bgl"),
            entries: &[bgl_buffer(
                0,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                wgpu::BufferBindingType::Uniform,
                size_of::<ScatterUniforms>(),
            )],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("customplot_scatter_pl"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let fs_entry = if config.target_format.is_srgb() {
            "fs_scatter_linear"
        } else {
            "fs_scatter_gamma"
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("customplot_scatter_rp"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_scatter"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<GpuScatterInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32,
                        2 => Uint32
                    ],
                })],
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
            bgl,
            pipeline,
            slots: HashMap::new(),
            slot_lru: VecDeque::new(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn try_paint_scatter(
    painter: &egui::Painter,
    tag: &WgpuHostTag,
    draw_key: u64,
    data: &PlotData,
    transform: PlotTransform,
    style: MarkerStyle,
    default_color: Color32,
    colors: Option<&[Color32]>,
    radii: Option<&[f32]>,
    sampling: MarkerSampling,
    backend: RenderBackend,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) -> bool {
    if colors.is_some() || radii.is_some() {
        // Per-point varying colors/radii can vary independently from PlotData::version;
        // let CPU pixel-grid LOD handle them safely without extra hashing overhead.
        return false;
    }
    let count = data.len();
    if count == 0
        || !style.radius.is_finite()
        || style.radius <= 0.0
        || default_color == Color32::TRANSPARENT
    {
        return false;
    }
    let scales = transform.scales();
    let Some((ext_min, ext_max)) = data.extent(scales) else {
        return true;
    };
    let rect = transform.rect();
    let ppp = pixels_per_point.max(0.01);
    let pad = style.radius + 1.0 / ppp + 1.0;
    let raw_sx0 = crate::data::project_axis_raw(transform, 0, ext_min[0]).unwrap_or(rect.left());
    let raw_sx1 = crate::data::project_axis_raw(transform, 0, ext_max[0]).unwrap_or(rect.right());
    let raw_sy0 = crate::data::project_axis_raw(transform, 1, ext_min[1]).unwrap_or(rect.top());
    let raw_sy1 = crate::data::project_axis_raw(transform, 1, ext_max[1]).unwrap_or(rect.bottom());
    if raw_sx0.max(raw_sx1) < rect.left() - pad
        || raw_sx0.min(raw_sx1) > rect.right() + pad
        || raw_sy0.max(raw_sy1) < rect.top() - pad
        || raw_sy0.min(raw_sy1) > rect.bottom() + pad
    {
        return true;
    }
    let (first_instance, end_instance) = if let Some(index) = data.line_index() {
        let xa = transform.point_from_position(egui::pos2(rect.left() - pad, rect.center().y))[0];
        let xb = transform.point_from_position(egui::pos2(rect.right() + pad, rect.center().y))[0];
        let x_bounds = if xa.is_finite() && xb.is_finite() {
            [xa.min(xb), xa.max(xb)]
        } else {
            [f64::NEG_INFINITY, f64::INFINITY]
        };
        let ranges = index.candidates(
            data.points(),
            scales,
            x_bounds,
            &mut crate::data::QueryStats::default(),
        );
        if ranges.is_empty() {
            return true;
        }
        (
            ranges.first().expect("non-empty").start as u32,
            ranges.last().expect("non-empty").end as u32,
        )
    } else {
        (0u32, count as u32)
    };
    let vis_len = (end_instance - first_instance) as usize;
    let sx0 = raw_sx0.clamp(rect.left(), rect.right());
    let sx1 = raw_sx1.clamp(rect.left(), rect.right());
    let sy0 = raw_sy0.clamp(rect.top(), rect.bottom());
    let sy1 = raw_sy1.clamp(rect.top(), rect.bottom());
    let box_px_area = (((sx1 - sx0).abs() * ppp).max(1.0)) * (((sy1 - sy0).abs() * ppp).max(1.0));
    if sampling == MarkerSampling::Auto
        && backend == RenderBackend::Auto
        && data.line_index().is_some()
        && (vis_len as f32) > (box_px_area * 32.0).max(65_536.0)
    {
        return false;
    }
    let stride = if sampling == MarkerSampling::Auto
        && (vis_len as f32) > (box_px_area * 32.0).max(8_192.0)
    {
        ((vis_len as f32) / (box_px_area * 32.0).max(8_192.0))
            .ceil()
            .clamp(1.0, 65_535.0) as u32
    } else {
        1u32
    };
    let total_bytes = count.saturating_mul(size_of::<GpuScatterInstance>());
    let max_allowed = tag
        .max_buffer_bytes
        .min(tag.budget_bytes.load(std::sync::atomic::Ordering::Relaxed));
    if total_bytes > max_allowed {
        return false;
    }
    let bounds = transform.bounds();
    let Some(ax0) = scales[0].forward(bounds.min()[0]) else {
        return false;
    };
    let Some(ax1) = scales[0].forward(bounds.max()[0]) else {
        return false;
    };
    let Some(ay0) = scales[1].forward(bounds.min()[1]) else {
        return false;
    };
    let Some(ay1) = scales[1].forward(bounds.max()[1]) else {
        return false;
    };
    let span_x = ax1 - ax0;
    let span_y = ay1 - ay0;
    if !span_x.is_finite() || !span_y.is_finite() || span_x == 0.0 || span_y == 0.0 {
        return false;
    }
    let ref_pt = ext_min;
    let ref_x = if scales[0] == AxisScale::Linear {
        ref_pt[0]
    } else {
        0.0
    };
    let ref_y = if scales[1] == AxisScale::Linear {
        ref_pt[1]
    } else {
        0.0
    };
    let ref_rel_x = if scales[0] == AxisScale::Linear {
        ((ref_x - ax0) / span_x) as f32
    } else {
        0.0
    };
    let ref_rel_y = if scales[1] == AxisScale::Linear {
        ((ref_y - ay0) / span_y) as f32
    } else {
        0.0
    };
    let source_id = (data.identity() as usize as u64) ^ 0xA5A5_5A5A_1234_5678;
    let version = data.version();
    let committed = lock_versions(&tag.committed_versions)
        .get(&source_id)
        .copied();
    let (upload, uploaded_bytes) = if committed == Some(version) {
        (None, 0)
    } else {
        let instances: Vec<GpuScatterInstance> = data
            .points()
            .iter()
            .map(|p| GpuScatterInstance {
                x: if scales[0] == AxisScale::Linear {
                    (p[0] - ref_x) as f32
                } else {
                    p[0] as f32
                },
                y: if scales[1] == AxisScale::Linear {
                    (p[1] - ref_y) as f32
                } else {
                    p[1] as f32
                },
                radius: -1.0,
                rgba: 0,
            })
            .collect();
        (Some(Arc::from(instances)), total_bytes)
    };

    let shape_id = match style.shape {
        MarkerShape::Circle => 0.0,
        MarkerShape::Square => 1.0,
        MarkerShape::Diamond => 2.0,
        MarkerShape::TriangleUp => 3.0,
        MarkerShape::TriangleDown => 4.0,
        MarkerShape::Plus => 5.0,
        MarkerShape::Cross => 6.0,
        MarkerShape::Asterisk => 7.0,
        MarkerShape::Pentagram => 8.0,
        MarkerShape::Hexagram => 9.0,
        MarkerShape::CircleHollow => 10.0,
        MarkerShape::SquareHollow => 11.0,
        MarkerShape::DiamondHollow => 12.0,
        MarkerShape::TriangleUpHollow => 13.0,
        MarkerShape::TriangleDownHollow => 14.0,
    };
    let rev = transform.reversed();
    let flags = u32::from(rev[0])
        | (u32::from(rev[1]) << 1)
        | (u32::from(scales[0] == AxisScale::Log10) << 2)
        | (u32::from(scales[1] == AxisScale::Log10) << 3);
    let packed_flags = flags | (stride << 4);

    diagnostics.wgpu_series += 1;
    diagnostics.wgpu_uploaded_bytes += uploaded_bytes;
    diagnostics.wgpu_retained_bytes += total_bytes;
    diagnostics.output_markers += vis_len.div_ceil(stride as usize);

    let clip = painter.clip_rect().intersect(rect);
    if !clip.is_positive() {
        return true;
    }
    painter.add(egui::Shape::Callback(Callback::new_paint_callback(
        clip,
        ScatterCallback {
            draw_key,
            source_id,
            version,
            total_bytes,
            upload,
            first_instance,
            instance_count: end_instance - first_instance,
            rect,
            default_radius: style.radius,
            default_color: default_color.to_array().map(|b| f32::from(b) / 255.0),
            axis_min: [
                ax0 as f32,
                (1.0 / span_x) as f32,
                ay0 as f32,
                (1.0 / span_y) as f32,
            ],
            ref_origin: [ref_rel_x, ref_rel_y, shape_id, packed_flags as f32],
        },
    )));
    true
}
