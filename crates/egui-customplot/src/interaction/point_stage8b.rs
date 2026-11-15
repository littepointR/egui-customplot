use egui::{Color32, Pos2};

use super::quiver::pick_quiver_data;
use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, PlotTransform, axis::SeriesTransforms,
    model::ResolvedSeriesStyle,
};

pub(super) fn pick_stage8b_point(
    layer: PlotLayer<'_>,
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    let transform = transforms.for_axis(layer.y_axis());
    match layer {
        PlotLayer::TriPlot(tri) => {
            diagnostics.geometry.pick_queries += 1;
            let visible = (resolved.stroke.width.is_finite()
                && resolved.stroke.width > 0.0
                && resolved.stroke.color != Color32::TRANSPARENT
                && resolved.pattern.is_valid())
                || (tri.show_markers
                    && resolved.marker.radius.is_finite()
                    && resolved.marker.radius > 0.0
                    && resolved.stroke.color != Color32::TRANSPARENT);
            if !visible {
                return Some(None);
            }
            let mut best = None;
            for (idx, &pt) in tri.data.vertices().iter().enumerate() {
                diagnostics.geometry.visited_items += 1;
                consider_point(&mut best, idx, pt, transform, position, radius);
            }
            Some(best)
        }
        PlotLayer::Voronoi(vor) => {
            diagnostics.geometry.pick_queries += 1;
            if resolved.stroke.color == Color32::TRANSPARENT {
                return Some(None);
            }
            let mut best = None;
            for (idx, &pt) in vor.data.sites().iter().enumerate() {
                diagnostics.geometry.visited_items += 1;
                consider_point(&mut best, idx, pt, transform, position, radius);
            }
            Some(best)
        }
        PlotLayer::Streamlines(stream) => {
            diagnostics.flow.pick_queries += 1;
            if !resolved.stroke.width.is_finite()
                || resolved.stroke.is_empty()
                || !resolved.pattern.is_valid()
            {
                return Some(None);
            }
            let mut best = None;
            let mut idx = 0usize;
            for line in stream.data.lines() {
                for &pt in line {
                    diagnostics.flow.visited_items += 1;
                    consider_point(&mut best, idx, pt, transform, position, radius);
                    idx += 1;
                }
            }
            Some(best)
        }
        PlotLayer::Feather(feather) => {
            diagnostics.flow.pick_queries += 1;
            let (hit, visited) =
                pick_quiver_data(feather.data.quiver(), transform, position, radius, resolved);
            diagnostics.flow.visited_items += visited;
            Some(hit)
        }
        PlotLayer::Spy(spy) => {
            diagnostics.geometry.pick_queries += 1;
            if resolved.stroke.color == Color32::TRANSPARENT
                || !resolved.marker.radius.is_finite()
                || resolved.marker.radius <= 0.0
            {
                return Some(None);
            }
            let r = radius + 1.0;
            let p0 = transform.point_from_position(egui::pos2(position.x - r, position.y - r));
            let p1 = transform.point_from_position(egui::pos2(position.x + r, position.y + r));
            if !p0[0].is_finite() || !p0[1].is_finite() || !p1[0].is_finite() || !p1[1].is_finite()
            {
                return Some(None);
            }
            let xr = [p0[0].min(p1[0]), p0[0].max(p1[0])];
            let yr = [p0[1].min(p1[1]), p0[1].max(p1[1])];
            let mut best = None;
            spy.data.visit_window(xr, yr, |idx, pt| {
                diagnostics.geometry.visited_items += 1;
                consider_point(&mut best, idx, pt, transform, position, radius);
            });
            Some(best)
        }
        PlotLayer::Comet(comet) => {
            diagnostics.flow.pick_queries += 1;
            let pts = comet.data.points();
            if pts.is_empty() || resolved.stroke.color == Color32::TRANSPARENT {
                return Some(None);
            }
            let progress = comet
                .playback
                .progress_points()
                .clamp(0.0, (pts.len() - 1) as f64);
            let whole = (progress.floor() as usize).min(pts.len() - 1);
            let mut best = None;
            for (idx, &pt) in pts[..=whole].iter().enumerate() {
                diagnostics.flow.visited_items += 1;
                consider_point(&mut best, idx, pt, transform, position, radius);
            }
            let frac = progress - whole as f64;
            if frac > 0.0 && whole + 1 < pts.len() {
                let a = pts[whole];
                let b = pts[whole + 1];
                if transform.is_valid_point(a) && transform.is_valid_point(b) {
                    let head = [a[0] + frac * (b[0] - a[0]), a[1] + frac * (b[1] - a[1])];
                    diagnostics.flow.visited_items += 1;
                    consider_point(&mut best, whole + 1, head, transform, position, radius);
                }
            }
            Some(best)
        }
        _ => None,
    }
}

fn consider_point(
    best: &mut Option<(usize, PlotPoint, f32)>,
    index: usize,
    pt: PlotPoint,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
) {
    if !transform.is_valid_point(pt) {
        return;
    }
    let screen = transform.position_from_point(pt);
    if !transform.rect().contains(screen) {
        return;
    }
    let dist = screen.distance(position);
    if dist <= radius && best.is_none_or(|b| dist < b.2) {
        *best = Some((index, pt, dist));
    }
}
