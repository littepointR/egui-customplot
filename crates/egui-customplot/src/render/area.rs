use egui::{Color32, Painter, Pos2, Shape};

use super::{
    line::prepare_line_iter, line_query::prepare_series_line, mesh::MeshBatch,
    pattern::apply_pattern,
};
use crate::{
    Area, Line, LineInterpolation, PlotCache, PlotDiagnostics, PlotTransform,
    data::QueryStats,
    model::{AreaSource, ResolvedSeriesStyle},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_area(
    painter: &Painter,
    area: Area<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
    cache: Option<&mut PlotCache>,
    auto_cache: Option<&mut PlotCache>,
) {
    if !area.baseline_valid(transform.scales()) {
        return;
    }
    let fill = area.resolved_fill(resolved.stroke.color);
    let stroke = resolved.stroke;
    match area.source {
        AreaSource::Curve { data, baseline } => {
            if fill != Color32::TRANSPARENT && data.len() >= 2 {
                paint_curve_fill(painter, data, baseline, fill, transform, diagnostics);
            }
            if area.show_stroke
                && stroke.width.is_finite()
                && stroke.width > 0.0
                && stroke.color != Color32::TRANSPARENT
            {
                if resolved.pattern.is_valid() {
                    let prepared = prepare_series_line(
                        Line::new(data),
                        transform,
                        pixels_per_point,
                        diagnostics,
                        cache,
                        auto_cache,
                    );
                    diagnostics.visited_segments += prepared.visited_segments;
                    diagnostics.numeric_rejections += prepared.numeric_rejections;
                    let patterned = apply_pattern(prepared.paths, resolved.pattern);
                    diagnostics.pattern_truncated |= patterned.truncated;
                    for path in patterned.paths {
                        diagnostics.patches.output_strokes += 1;
                        diagnostics.output_points += path.len();
                        diagnostics.geometry_capacity_bytes +=
                            path.capacity() * std::mem::size_of::<Pos2>();
                        painter.add(Shape::line(path, stroke));
                    }
                } else {
                    diagnostics.numeric_rejections += 1;
                }
            }
        }
        AreaSource::Band(band) => {
            let x = band.x();
            if x.len() < 2 {
                return;
            }
            let (start, end) = if band.is_x_monotonic() {
                let view = transform.bounds();
                let lo_x = view.min()[0].min(view.max()[0]);
                let hi_x = view.min()[0].max(view.max()[0]);
                (
                    x.partition_point(|v| *v < lo_x).saturating_sub(1),
                    (x.partition_point(|v| *v <= hi_x) + 1).min(x.len()),
                )
            } else {
                (0, x.len())
            };
            if end.saturating_sub(start) < 2 {
                return;
            }
            if fill != Color32::TRANSPARENT {
                let mut batch = MeshBatch::default();
                for i in start..end - 1 {
                    diagnostics.patches.visited_items += 1;
                    let p0_lo = [x[i], band.lower()[i]];
                    let p1_lo = [x[i + 1], band.lower()[i + 1]];
                    let p0_hi = [x[i], band.upper()[i]];
                    let p1_hi = [x[i + 1], band.upper()[i + 1]];
                    if ![p0_lo, p1_lo, p0_hi, p1_hi]
                        .into_iter()
                        .all(|p| transform.is_valid_point(p))
                    {
                        continue;
                    }
                    emit_band_quad(
                        &mut batch,
                        painter,
                        transform.normalized(p0_lo),
                        transform.normalized(p1_lo),
                        transform.normalized(p0_hi),
                        transform.normalized(p1_hi),
                        fill,
                        transform,
                    );
                }
                let stats = batch.finish(painter);
                diagnostics.patches.output_meshes += stats.meshes;
                diagnostics.patches.output_triangles += stats.triangles;
                diagnostics.geometry_capacity_bytes += stats.bytes;
                diagnostics.numeric_rejections += stats.numeric_rejections;
            }
            if area.show_stroke
                && stroke.width.is_finite()
                && stroke.width > 0.0
                && stroke.color != Color32::TRANSPARENT
            {
                if resolved.pattern.is_valid() {
                    let prepared = prepare_line_iter(
                        (start..end).map(|i| [x[i], band.upper()[i]]),
                        transform,
                        LineInterpolation::Linear,
                    );
                    diagnostics.visited_segments += prepared.visited_segments;
                    diagnostics.numeric_rejections += prepared.numeric_rejections;
                    let patterned = apply_pattern(prepared.paths, resolved.pattern);
                    diagnostics.pattern_truncated |= patterned.truncated;
                    for path in patterned.paths {
                        diagnostics.patches.output_strokes += 1;
                        diagnostics.output_points += path.len();
                        diagnostics.geometry_capacity_bytes +=
                            path.capacity() * std::mem::size_of::<Pos2>();
                        painter.add(Shape::line(path, stroke));
                    }
                } else {
                    diagnostics.numeric_rejections += 1;
                }
            }
        }
    }
}

fn paint_curve_fill(
    painter: &Painter,
    data: &crate::PlotData,
    baseline: f64,
    fill: Color32,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let view = transform.bounds();
    let base_v = transform.normalized([view.min()[0], baseline])[1];
    if !base_v.is_finite() {
        return;
    }
    let pts = data.points();
    let mut batch = MeshBatch::default();
    let mut last_idx: Option<usize> = None;
    let mut emit_range = |range: std::ops::Range<usize>| {
        let start = range.start.saturating_sub(1);
        let end = range.end.min(pts.len()).saturating_sub(1);
        for i in start..end {
            if last_idx.is_some_and(|prev| i <= prev) {
                continue;
            }
            last_idx = Some(i);
            diagnostics.patches.visited_items += 1;
            let (p0, p1) = (pts[i], pts[i + 1]);
            if !transform.is_valid_point(p0) || !transform.is_valid_point(p1) {
                continue;
            }
            let n0 = transform.normalized(p0);
            let n1 = transform.normalized(p1);
            if n0[0].max(n1[0]) <= 0.0 || n0[0].min(n1[0]) >= 1.0 {
                continue;
            }
            let b0 = [n0[0], base_v];
            let b1 = [n1[0], base_v];
            emit_band_quad(&mut batch, painter, b0, b1, n0, n1, fill, transform);
        }
    };
    if let Some(index) = data.line_index() {
        let mut stats = QueryStats::default();
        let x = [
            view.min()[0].min(view.max()[0]),
            view.min()[0].max(view.max()[0]),
        ];
        let ranges = index.candidates(pts, transform.scales(), x, &mut stats);
        diagnostics.query_runs += stats.visited_runs;
        diagnostics.query_comparisons += stats.comparisons;
        for range in ranges {
            emit_range(range);
        }
    } else {
        emit_range(0..pts.len());
    }
    let stats = batch.finish(painter);
    diagnostics.patches.output_meshes += stats.meshes;
    diagnostics.patches.output_triangles += stats.triangles;
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}

#[allow(clippy::too_many_arguments)]
fn emit_band_quad(
    batch: &mut MeshBatch,
    painter: &Painter,
    lo0: [f64; 2],
    lo1: [f64; 2],
    hi0: [f64; 2],
    hi1: [f64; 2],
    color: Color32,
    transform: PlotTransform,
) {
    let d0 = hi0[1] - lo0[1];
    let d1 = hi1[1] - lo1[1];
    if (d0 > 0.0 && d1 < 0.0) || (d0 < 0.0 && d1 > 0.0) {
        let t = (d0 / (d0 - d1)).clamp(0.0, 1.0);
        let mid = [
            lo0[0] + t * (lo1[0] - lo0[0]),
            lo0[1] + t * (lo1[1] - lo0[1]),
        ];
        batch.add_flat_triangle(painter, [lo0, hi0, mid], color, transform);
        batch.add_flat_triangle(painter, [mid, hi1, lo1], color, transform);
    } else {
        batch.add_flat_triangle(painter, [lo0, hi0, hi1], color, transform);
        batch.add_flat_triangle(painter, [lo0, hi1, lo1], color, transform);
    }
}
