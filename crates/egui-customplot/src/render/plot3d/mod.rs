mod axes;
mod custom;
mod layers;
mod meshes_contours;
mod scene;
mod solid_layers;
mod stage13;
mod surfaces;
mod volshow;
mod volshow_raycast;
mod volume_flow;
mod zbuffer;
mod zbuffer_cache;
mod zbuffer_key;

use egui::Painter;

pub(crate) use zbuffer_cache::ZBufferCache;

use crate::{
    Axis3D, BoxStyle3D, ConstantPlane3D, DepthMode, Layer3D, Plot3DDiagnostics, Plot3DState,
    Text3D, Transform3D, model::ResolvedPlotStyle,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_plot3d(
    painter: &Painter,
    state: &mut Plot3DState,
    layers: &[Layer3D<'_>],
    planes: &[ConstantPlane3D],
    texts: &[Text3D],
    axes: [&Axis3D; 3],
    transform: Transform3D,
    depth_mode: DepthMode,
    box_style: BoxStyle3D,
    show_grid: bool,
    title: &str,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut Plot3DDiagnostics,
) {
    state.edges = axes::paint_axes_3d(
        painter,
        state.previous_edges(),
        axes,
        transform,
        box_style,
        show_grid,
        title,
        style,
    );
    let area = transform.rect();
    if depth_mode == DepthMode::ZBuffer {
        let (w, h) = zbuffer::zbuffer_dims(area, crate::render::export_density(painter.ctx()));
        let key = zbuffer_key::compute_zbuffer_key(
            state, layers, planes, texts, transform, box_style, style, w, h,
        );
        if let Some(cache) = state.zbuffer_cache.as_ref().filter(|c| c.key == key) {
            zbuffer_cache::emit_cached_zbuffer(cache, painter, area, style.text_color, diagnostics);
            return;
        }
        let before_render = *diagnostics;
        let mut scene = scene::Scene3D::default();
        layers::collect_layers_3d(
            &mut scene,
            state,
            layers,
            planes,
            texts,
            transform,
            style,
            diagnostics,
        );
        state.zbuffer_cache = zbuffer::emit_zbuffer(
            &scene,
            layers,
            state,
            transform,
            painter,
            area,
            style.text_color,
            key,
            (w, h),
            before_render,
            diagnostics,
        );
        return;
    }

    let mut scene = scene::Scene3D::default();
    layers::collect_layers_3d(
        &mut scene,
        state,
        layers,
        planes,
        texts,
        transform,
        style,
        diagnostics,
    );
    volshow::paint_volumes_painter(layers, state, transform, painter, area, diagnostics);
    scene.emit_sorted(painter, area, style.text_color, diagnostics);
}
