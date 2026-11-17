//! Optional `egui-wgpu` hardware-accelerated compute & instanced rendering.
//! Reuses the host's `Device`, `Queue`, and `RenderPass` via `CallbackTrait`.

mod resources;
mod scatter_callback;
mod scatter_pipeline;
mod waveform_callback;
mod waveform_pipeline;

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use egui::Color32;
use egui_wgpu::{RenderState, wgpu};

use crate::{
    LineInterpolation, LinePattern, MarkerSampling, MarkerStyle, PlotData, PlotDiagnostics,
    PlotTransform, RenderBackend, WaveformLine,
};
use resources::{WgpuHostTag, WgpuPlotResources};

const DEFAULT_WGPU_BUDGET_BYTES: usize = 256 * 1024 * 1024;
const AUTO_WGPU_WAVEFORM_MIN_LEN: usize = 16_384;
const AUTO_WGPU_SCATTER_MIN_LEN: usize = 32_768;

/// Host `egui-wgpu` compute/instancing pipeline configuration and bounded VRAM budget.
///
/// 可选 `wgpu` GPU 加速管线配置（目标纹理格式、MSAA 采样与显存预算），复用宿主渲染上下文。
#[derive(Clone, Copy, Debug)]
pub struct WgpuPlotConfig {
    pub target_format: wgpu::TextureFormat,
    pub msaa_samples: u32,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub budget_bytes: usize,
}

impl WgpuPlotConfig {
    pub fn from_render_state(render_state: &RenderState) -> Self {
        Self {
            target_format: render_state.target_format,
            msaa_samples: 1,
            depth_format: None,
            budget_bytes: DEFAULT_WGPU_BUDGET_BYTES,
        }
    }
}

/// Registers GPU plot pipelines and bounded VRAM resources on the host's
/// `egui_wgpu::RenderState` and `egui::Context`. Returns `false` if device
/// limits do not support compute storage buffers.
///
/// 将大规模波形/散点 Compute + Instancing 管线与显存缓存注册到宿主 `RenderState`（硬件不支持存储缓冲区时安全返回 `false` 并回退 CPU 路径）。
pub fn register_wgpu(ctx: &egui::Context, render_state: &RenderState) -> bool {
    register_wgpu_with_config(
        ctx,
        render_state,
        WgpuPlotConfig::from_render_state(render_state),
    )
}

/// Registers GPU plot pipelines with explicit target format, MSAA, and VRAM budget.
///
/// 使用自定义纹理格式、MSAA 与显存上限注册 `wgpu` 加速渲染管线。
pub fn register_wgpu_with_config(
    ctx: &egui::Context,
    render_state: &RenderState,
    config: WgpuPlotConfig,
) -> bool {
    let limits = render_state.device.limits();
    if limits.max_storage_buffers_per_shader_stage < 2
        || limits.max_compute_workgroups_per_dimension == 0
    {
        return false;
    }
    let committed_versions = Arc::new(Mutex::new(HashMap::new()));
    let resources =
        WgpuPlotResources::new(&render_state.device, config, committed_versions.clone());
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(resources);
    let tag = WgpuHostTag {
        max_binding_bytes: limits.max_storage_buffer_binding_size as usize,
        max_buffer_bytes: limits.max_buffer_size as usize,
        budget_bytes: Arc::new(AtomicUsize::new(config.budget_bytes)),
        committed_versions,
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, tag));
    true
}

/// Adjusts the GPU sample buffer retention budget and immediately evicts excess VRAM buffers.
pub fn set_wgpu_budget(ctx: &egui::Context, render_state: &RenderState, budget_bytes: usize) {
    ctx.data(|d| {
        if let Some(tag) = d.get_temp::<WgpuHostTag>(egui::Id::NULL) {
            tag.budget_bytes.store(budget_bytes, Ordering::Relaxed);
        }
    });
    if let Some(res) = render_state
        .renderer
        .write()
        .callback_resources
        .get_mut::<WgpuPlotResources>()
    {
        res.set_budget(budget_bytes);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn try_paint_waveform_wgpu(
    ui_ctx: &egui::Context,
    painter: &egui::Painter,
    draw_key: u64,
    line: WaveformLine<'_>,
    transform: PlotTransform,
    stroke: egui::Stroke,
    pattern: LinePattern,
    diagnostics: &mut PlotDiagnostics,
) -> bool {
    if line.backend == RenderBackend::Painter
        || super::is_capturing_snapshot(ui_ctx)
        || transform.scales().iter().any(|s| s.is_probability())
    {
        return false;
    }
    if line.backend == RenderBackend::Auto && line.data.len() < AUTO_WGPU_WAVEFORM_MIN_LEN {
        return false;
    }
    if pattern != LinePattern::Solid
        || line.interpolation != LineInterpolation::Linear
        || !stroke.width.is_finite()
        || stroke.width <= 0.0
        || stroke.color == Color32::TRANSPARENT
    {
        return false;
    }
    let Some(tag) = ui_ctx.data(|d| d.get_temp::<WgpuHostTag>(egui::Id::NULL)) else {
        return false;
    };
    waveform_pipeline::try_paint_waveform(
        painter,
        &tag,
        draw_key,
        line.data,
        line.backend,
        transform,
        stroke,
        ui_ctx.pixels_per_point(),
        diagnostics,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn try_paint_scatter_wgpu(
    ui_ctx: &egui::Context,
    painter: &egui::Painter,
    draw_key: u64,
    data: &PlotData,
    transform: PlotTransform,
    style: MarkerStyle,
    default_color: Color32,
    colors: Option<&[Color32]>,
    radii: Option<&[f32]>,
    sampling: MarkerSampling,
    backend: RenderBackend,
    diagnostics: &mut PlotDiagnostics,
) -> bool {
    if backend == RenderBackend::Painter
        || sampling == MarkerSampling::PixelGrid
        || super::is_capturing_snapshot(ui_ctx)
        || transform.scales().iter().any(|s| s.is_probability())
    {
        return false;
    }
    if backend == RenderBackend::Auto && data.len() < AUTO_WGPU_SCATTER_MIN_LEN {
        return false;
    }
    let Some(tag) = ui_ctx.data(|d| d.get_temp::<WgpuHostTag>(egui::Id::NULL)) else {
        return false;
    };
    scatter_pipeline::try_paint_scatter(
        painter,
        &tag,
        draw_key,
        data,
        transform,
        style,
        default_color,
        colors,
        radii,
        sampling,
        backend,
        ui_ctx.pixels_per_point(),
        diagnostics,
    )
}
