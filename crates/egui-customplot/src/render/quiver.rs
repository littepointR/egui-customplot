use egui::{Painter, Pos2};

use super::pattern::apply_pattern;
use crate::{
    PlotDiagnostics, PlotTransform, Quiver, QuiverData, axis::project_vector_segment,
    model::ResolvedSeriesStyle,
};

pub(super) struct QuiverPaintStats {
    pub visited: usize,
    pub output_shafts: usize,
    pub output_heads: usize,
    pub capacity_bytes: usize,
    pub numeric_rejections: usize,
    pub pattern_truncated: bool,
}

pub(super) fn paint_quiver(
    painter: &Painter,
    quiver: Quiver<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    let stats = paint_quiver_data(
        painter,
        quiver.data,
        quiver.show_heads,
        quiver.head_size,
        transform,
        style,
    );
    diagnostics.visited_quiver_arrows += stats.visited;
    diagnostics.output_quiver_shafts += stats.output_shafts;
    diagnostics.output_quiver_heads += stats.output_heads;
    diagnostics.geometry_capacity_bytes += stats.capacity_bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
    diagnostics.pattern_truncated |= stats.pattern_truncated;
}

pub(super) fn paint_quiver_data(
    painter: &Painter,
    data: &QuiverData,
    show_heads: bool,
    head_size: [f32; 2],
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
) -> QuiverPaintStats {
    let mut stats = QuiverPaintStats {
        visited: 0,
        output_shafts: 0,
        output_heads: 0,
        capacity_bytes: 0,
        numeric_rejections: 0,
        pattern_truncated: false,
    };
    if !style.stroke.width.is_finite() || style.stroke.width < 0.0 || !style.pattern.is_valid() {
        stats.numeric_rejections += 1;
        return stats;
    }
    if style.stroke.is_empty() {
        return stats;
    }
    let head_valid = head_size.iter().all(|v| v.is_finite() && *v > 0.0);
    if show_heads && head_size.iter().any(|v| !v.is_finite() || *v < 0.0) {
        stats.numeric_rejections += 1;
    }
    let mut shafts = Vec::new();
    let mut heads = Vec::new();
    for &arrow in data.arrows() {
        stats.visited += 1;
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
                stats.numeric_rejections += 1;
                continue;
            }
        };
        shafts.push(segment.shaft.to_vec());
        if show_heads && head_valid && segment.end_visible {
            let factor = (0.4 * segment.length / f64::from(head_size[0])).min(1.0);
            let length = f64::from(head_size[0]) * factor;
            let half_width = f64::from(head_size[1]) * factor * 0.5;
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
                stats.numeric_rejections += 1;
            }
        }
    }
    let patterned = apply_pattern(shafts, style.pattern);
    stats.pattern_truncated |= patterned.truncated;
    stats.output_shafts += patterned.paths.len();
    stats.output_heads += heads.len();
    for path in patterned.paths.into_iter().chain(heads) {
        stats.capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        painter.add(egui::Shape::line(path, style.stroke));
    }
    stats
}
