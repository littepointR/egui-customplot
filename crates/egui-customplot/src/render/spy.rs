use egui::{Color32, Painter, Shape, epaint};

use super::markers::prepare_markers;
use crate::{PlotDiagnostics, PlotPoint, PlotTransform, Spy, model::ResolvedSeriesStyle};

pub(super) fn paint_spy(
    painter: &Painter,
    spy: Spy<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if !style.marker.radius.is_finite() || style.marker.radius < 0.0 {
        diagnostics.numeric_rejections += 1;
        return;
    }
    if style.marker.radius == 0.0 || style.stroke.color == Color32::TRANSPARENT {
        return;
    }

    let bounds = transform.bounds();
    let lo = bounds.min();
    let hi = bounds.max();
    let mut visible: Vec<PlotPoint> = Vec::new();
    spy.data
        .visit_window([lo[0], hi[0]], [lo[1], hi[1]], |_, pt| {
            visible.push(pt);
        });
    diagnostics.geometry.visited_items += visible.len();
    if visible.is_empty() {
        return;
    }

    let prepared = prepare_markers(
        &visible,
        transform,
        style.marker,
        style.stroke.color,
        None,
        None,
        pixels_per_point,
    );
    diagnostics.geometry.output_markers += prepared.output_markers;
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    diagnostics.geometry_capacity_bytes += prepared.mesh.vertices.capacity()
        * std::mem::size_of::<epaint::Vertex>()
        + prepared.mesh.indices.capacity() * std::mem::size_of::<u32>();
    if !prepared.mesh.is_empty() {
        diagnostics.geometry.output_meshes += 1;
        painter.add(Shape::mesh(prepared.mesh));
    }
}
