use egui::{Painter, Pos2};

use super::pattern::apply_pattern;
use crate::{
    PlotDiagnostics, PlotTransform, Quiver, axis::project_vector_segment,
    model::ResolvedSeriesStyle,
};

pub(super) fn paint_quiver(
    painter: &Painter,
    quiver: Quiver<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    if !style.stroke.width.is_finite() || style.stroke.width < 0.0 || !style.pattern.is_valid() {
        diagnostics.numeric_rejections += 1;
        return;
    }
    if style.stroke.is_empty() {
        return;
    }
    let head_valid = quiver.head_size.iter().all(|v| v.is_finite() && *v > 0.0);
    if quiver.show_heads && quiver.head_size.iter().any(|v| !v.is_finite() || *v < 0.0) {
        diagnostics.numeric_rejections += 1;
    }
    let mut shafts = Vec::new();
    let mut heads = Vec::new();
    for &arrow in quiver.data.arrows() {
        diagnostics.visited_quiver_arrows += 1;
        let Some(end) = arrow.endpoint() else {
            continue;
        };
        if !transform.is_valid_point(arrow.origin) || !transform.is_valid_point(end) {
            continue;
        }
        let segment = match project_vector_segment(arrow.origin, end, transform) {
            Ok(Some(segment)) => segment,
            Ok(None) => continue,
            Err(()) => {
                diagnostics.numeric_rejections += 1;
                continue;
            }
        };
        shafts.push(segment.shaft.to_vec());
        if quiver.show_heads && head_valid && segment.end_visible {
            let factor = (0.4 * segment.length / f64::from(quiver.head_size[0])).min(1.0);
            let length = f64::from(quiver.head_size[0]) * factor;
            let half_width = f64::from(quiver.head_size[1]) * factor * 0.5;
            // Anchor at the real endpoint, never at a clipped shaft intersection.
            let tip = transform.position_from_point(end);
            let [dx, dy] = segment.direction;
            let wing = |sign: f64| {
                egui::pos2(
                    (f64::from(tip.x) - dx * length - sign * dy * half_width) as f32,
                    (f64::from(tip.y) - dy * length + sign * dx * half_width) as f32,
                )
            };
            let head = vec![wing(1.0), tip, wing(-1.0)];
            if head.iter().all(|p| p.is_finite()) {
                heads.push(head);
            } else {
                diagnostics.numeric_rejections += 1;
            }
        }
    }
    let patterned = apply_pattern(shafts, style.pattern);
    diagnostics.pattern_truncated |= patterned.truncated;
    diagnostics.output_quiver_shafts += patterned.paths.len();
    diagnostics.output_quiver_heads += heads.len();
    for path in patterned.paths.into_iter().chain(heads) {
        diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        painter.add(egui::Shape::line(path, style.stroke));
    }
}
