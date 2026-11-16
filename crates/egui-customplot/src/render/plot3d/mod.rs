mod axes;
mod layers;
mod meshes_contours;
mod scene;
mod solid_layers;
mod surfaces;
mod zbuffer;

use egui::Painter;

use crate::{
    Axis3D, BoxStyle3D, ConstantPlane3D, DepthMode, Layer3D, Plot3DDiagnostics, Plot3DState,
    Text3D, Transform3D, model::ResolvedPlotStyle,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_plot3d(
    painter: &Painter,
    state: &Plot3DState,
    layers: &[Layer3D<'_>],
    planes: &[ConstantPlane3D],
    texts: &[Text3D],
    axes: [&Axis3D; 3],
    transform: Transform3D,
    depth_mode: DepthMode,
    box_style: BoxStyle3D,
    title: &str,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut Plot3DDiagnostics,
) {
    axes::paint_axes_3d(painter, axes, transform, box_style, title, style);
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
    match depth_mode {
        DepthMode::SortedPainter => {
            scene.emit_sorted(painter, transform.rect(), style.text_color, diagnostics);
        }
        DepthMode::ZBuffer => {
            zbuffer::emit_zbuffer(
                &scene,
                painter,
                transform.rect(),
                style.text_color,
                diagnostics,
            );
        }
    }
}
