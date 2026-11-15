use egui::{Painter, Pos2};

use super::{markers::prepare_markers, pattern::apply_pattern};
use crate::{
    PlotDiagnostics, PlotTransform, Stems, axis::clipped_axis_segment, model::ResolvedSeriesStyle,
};

pub(super) fn paint_stems(
    painter: &Painter,
    stems: Stems<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if !stems.baseline_valid(transform.scales()) {
        return;
    }
    let axis = stems.orientation.axis();
    let stroke_valid =
        style.stroke.width.is_finite() && style.stroke.width > 0.0 && !style.stroke.is_empty();
    if stroke_valid && style.pattern.is_valid() {
        let mut paths = Vec::new();
        for &point in stems.data.points() {
            diagnostics.visited_stems += 1;
            if !transform.is_valid_point(point) {
                continue;
            }
            let mut base = point;
            base[axis] = stems.baseline;
            if let Some(mut pair) = clipped_axis_segment(base, point, axis, transform) {
                // The shared clipper sorts data coordinates. Restore baseline
                // to endpoint order so negative rods start their dash there too.
                if base[axis] > point[axis] {
                    pair.swap(0, 1);
                }
                paths.push(pair.to_vec());
            }
        }
        let patterned = apply_pattern(paths, style.pattern);
        diagnostics.pattern_truncated |= patterned.truncated;
        for path in patterned.paths {
            diagnostics.output_stem_segments += 1;
            diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
            painter.add(egui::Shape::line(path, style.stroke));
        }
    } else if !style.stroke.width.is_finite() || !style.pattern.is_valid() {
        diagnostics.numeric_rejections += 1;
    }
    paint_baseline(painter, stems, transform, style, diagnostics);
    if style.line_markers {
        let prepared = prepare_markers(
            stems.data.points(),
            transform,
            style.marker,
            style.stroke.color,
            None,
            None,
            pixels_per_point,
        );
        diagnostics.visited_marker_points += prepared.visited_points;
        diagnostics.output_markers += prepared.output_markers;
        diagnostics.numeric_rejections += prepared.numeric_rejections;
        diagnostics.geometry_capacity_bytes += prepared.mesh.vertices.capacity()
            * std::mem::size_of::<egui::epaint::Vertex>()
            + prepared.mesh.indices.capacity() * std::mem::size_of::<u32>();
        if !prepared.mesh.is_empty() {
            painter.add(egui::Shape::mesh(prepared.mesh));
        }
    }
}

fn paint_baseline(
    painter: &Painter,
    stems: Stems<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    if !stems.show_baseline {
        return;
    }
    let stroke = stems.baseline_stroke.unwrap_or(style.stroke);
    if !stroke.width.is_finite() || stroke.width < 0.0 {
        diagnostics.numeric_rejections += 1;
    } else if !stroke.is_empty()
        && let Some((mut min, mut max)) = stems.data.extent(transform.scales())
    {
        let axis = stems.orientation.axis();
        min[axis] = stems.baseline;
        max[axis] = stems.baseline;
        if let Some(pair) = clipped_axis_segment(min, max, 1 - axis, transform) {
            painter.line_segment(pair, stroke);
            diagnostics.output_stem_baselines += 1;
        }
    }
}
