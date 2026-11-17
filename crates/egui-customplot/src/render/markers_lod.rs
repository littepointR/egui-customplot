use std::ops::Range;

use egui::{Color32, Pos2, Vec2};

use super::markers::{PreparedMarkers, marker_outline, prepare_markers, push_shaped_marker};
use super::markers_grid_tree::collect_indexed_grid_markers;
use crate::{
    AxisScale, MarkerSampling, MarkerShape, MarkerStyle, PlotData, PlotPoint, PlotTransform,
    data::QueryStats,
};

const AUTO_PIXEL_GRID_THRESHOLD: usize = 16_384;

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_markers_with_sampling(
    data: &PlotData,
    transform: PlotTransform,
    style: MarkerStyle,
    color: Color32,
    colors: Option<&[Color32]>,
    radii: Option<&[f32]>,
    pixels_per_point: f32,
    sampling: MarkerSampling,
) -> PreparedMarkers {
    let points = data.points();
    if points.is_empty() {
        return PreparedMarkers::default();
    }
    let ppp = pixels_per_point.max(0.01);
    let fringe = 1.0 / ppp;
    let ranges_opt = candidate_ranges(data, points, transform, style.radius, fringe, radii);
    let candidate_count = ranges_opt
        .as_ref()
        .map_or(points.len(), |r| r.iter().map(Range::len).sum());

    let use_grid = match sampling {
        MarkerSampling::Exact => false,
        MarkerSampling::PixelGrid => candidate_count > 0,
        MarkerSampling::Auto => candidate_count >= AUTO_PIXEL_GRID_THRESHOLD,
    };
    if !use_grid {
        let Some(ranges) = ranges_opt.as_deref() else {
            return prepare_markers(points, transform, style, color, colors, radii, ppp);
        };
        return prepare_ranges_exact(points, ranges, transform, style, color, colors, radii, ppp);
    }
    let full = Range {
        start: 0,
        end: points.len(),
    };
    let ranges = ranges_opt.as_deref().unwrap_or(std::slice::from_ref(&full));
    if radii.is_none()
        && colors.is_none()
        && style.radius.is_finite()
        && style.radius > 0.0
        && color != Color32::TRANSPARENT
        && let Some(index) = data.line_index()
    {
        let grid = collect_indexed_grid_markers(
            index,
            points,
            ranges,
            transform,
            style.radius,
            color,
            ppp,
            fringe,
        );
        return emit_chosen_marker_mesh(
            grid.chosen,
            grid.visited_points,
            grid.numeric_rejections,
            grid.reduced,
            style,
            radii,
            ppp,
            fringe,
        );
    }
    prepare_markers_pixel_grid(
        points, ranges, transform, style, color, colors, radii, ppp, fringe,
    )
}

fn candidate_ranges(
    data: &PlotData,
    points: &[PlotPoint],
    transform: PlotTransform,
    radius: f32,
    fringe: f32,
    radii: Option<&[f32]>,
) -> Option<Vec<Range<usize>>> {
    if radii.is_some() || !radius.is_finite() || radius < 0.0 {
        return None;
    }
    if let Some(index) = data.line_index() {
        let rect = transform.rect();
        let pad = radius.max(0.0) + fringe + 1.0;
        let xa = transform.point_from_position(egui::pos2(rect.left() - pad, rect.center().y))[0];
        let xb = transform.point_from_position(egui::pos2(rect.right() + pad, rect.center().y))[0];
        if xa.is_finite() && xb.is_finite() {
            return Some(index.candidates(
                points,
                transform.scales(),
                [xa.min(xb), xa.max(xb)],
                &mut QueryStats::default(),
            ));
        }
        return Some(index.runs_for_scales(transform.scales()).to_vec());
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn prepare_ranges_exact(
    points: &[PlotPoint],
    ranges: &[Range<usize>],
    transform: PlotTransform,
    style: MarkerStyle,
    color: Color32,
    colors: Option<&[Color32]>,
    radii: Option<&[f32]>,
    ppp: f32,
) -> PreparedMarkers {
    let mut result = PreparedMarkers::default();
    let outline = marker_outline(style.shape);
    let fringe = 1.0 / ppp;
    for range in ranges {
        for index in range.clone() {
            result.visited_points += 1;
            let point = points[index];
            if !transform.is_valid_point(point) {
                continue;
            }
            let c = colors.map_or(color, |s| s[index]);
            let r = radii.map_or(style.radius, |s| s[index]);
            if !r.is_finite() || r < 0.0 {
                result.numeric_rejections += 1;
                continue;
            }
            if r == 0.0 || c == Color32::TRANSPARENT {
                continue;
            }
            let center = transform.position_from_point(point);
            if !center.is_finite() {
                result.numeric_rejections += 1;
                continue;
            }
            if !transform.rect().expand(r + fringe).contains(center) {
                continue;
            }
            if !push_shaped_marker(
                &mut result.mesh,
                center,
                r,
                c,
                style.shape,
                &outline,
                fringe,
            ) {
                result.numeric_rejections += 1;
                continue;
            }
            result.output_markers += 1;
        }
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn prepare_markers_pixel_grid(
    points: &[PlotPoint],
    ranges: &[Range<usize>],
    transform: PlotTransform,
    style: MarkerStyle,
    color: Color32,
    colors: Option<&[Color32]>,
    radii: Option<&[f32]>,
    ppp: f32,
    fringe: f32,
) -> PreparedMarkers {
    let mut result = PreparedMarkers::default();
    let rect = transform.rect();
    let cell = (1.0 / ppp).max(0.5);
    let cols = (rect.width() / cell).ceil().clamp(1.0, 2048.0) as usize;
    let rows = (rect.height() / cell).ceil().clamp(1.0, 2048.0) as usize;
    let scale_x = cols as f32 / rect.width().max(1e-3);
    let scale_y = rows as f32 / rect.height().max(1e-3);
    let mut occupied = vec![0u64; (cols * rows).div_ceil(64)];
    let mut chosen: Vec<(Pos2, f32, Color32)> = Vec::new();
    let mut eligible = 0usize;

    let scales = transform.scales();
    let fast_uniform = scales == [AxisScale::Linear; 2]
        && radii.is_none()
        && colors.is_none()
        && style.radius.is_finite()
        && style.radius > 0.0
        && color != Color32::TRANSPARENT;

    if fast_uniform {
        let bounds = transform.bounds();
        let min_x = bounds.min()[0];
        let min_y = bounds.min()[1];
        let rev = transform.reversed();
        let inv_x = f64::from(rect.width()) / (bounds.max()[0] - min_x);
        let inv_y = f64::from(rect.height()) / (bounds.max()[1] - min_y);
        let x0 = if rev[0] {
            f64::from(rect.right())
        } else {
            f64::from(rect.left())
        };
        let mx = if rev[0] { -inv_x } else { inv_x };
        let y0 = if rev[1] {
            f64::from(rect.top())
        } else {
            f64::from(rect.bottom())
        };
        let my = if rev[1] { inv_y } else { -inv_y };
        let pad = style.radius + fringe;
        let min_px = rect.left() - pad;
        let max_px = rect.right() + pad;
        let min_py = rect.top() - pad;
        let max_py = rect.bottom() + pad;
        let rect_l = rect.left();
        let rect_t = rect.top();
        let max_gx = cols as isize - 1;
        let max_gy = rows as isize - 1;

        for range in ranges.iter().rev() {
            for &point in points[range.clone()].iter().rev() {
                result.visited_points += 1;
                if !point[0].is_finite() || !point[1].is_finite() {
                    continue;
                }
                let sx = (x0 + (point[0] - min_x) * mx) as f32;
                let sy = (y0 + (point[1] - min_y) * my) as f32;
                if !(sx >= min_px && sx <= max_px && sy >= min_py && sy <= max_py) {
                    if !sx.is_finite() || !sy.is_finite() {
                        result.numeric_rejections += 1;
                    }
                    continue;
                }
                eligible += 1;
                let gx = (((sx - rect_l) * scale_x) as isize).clamp(0, max_gx) as usize;
                let gy = (((sy - rect_t) * scale_y) as isize).clamp(0, max_gy) as usize;
                let bit = gy * cols + gx;
                let word = &mut occupied[bit >> 6];
                let mask = 1u64 << (bit & 63);
                if (*word & mask) == 0 {
                    *word |= mask;
                    chosen.push((Pos2::new(sx, sy), style.radius, color));
                }
            }
        }
    } else {
        for range in ranges.iter().rev() {
            for index in range.clone().rev() {
                result.visited_points += 1;
                let point = points[index];
                if !transform.is_valid_point(point) {
                    continue;
                }
                let c = colors.map_or(color, |slice| slice[index]);
                let r = radii.map_or(style.radius, |slice| slice[index]);
                if !r.is_finite() || r < 0.0 {
                    result.numeric_rejections += 1;
                    continue;
                }
                if r == 0.0 || c == Color32::TRANSPARENT {
                    continue;
                }
                let center = transform.position_from_point(point);
                if !center.is_finite() {
                    result.numeric_rejections += 1;
                    continue;
                }
                if !rect.expand(r + fringe).contains(center) {
                    continue;
                }
                eligible += 1;
                let gx = (((center.x - rect.left()) * scale_x) as isize).clamp(0, cols as isize - 1)
                    as usize;
                let gy = (((center.y - rect.top()) * scale_y) as isize).clamp(0, rows as isize - 1)
                    as usize;
                let bit = gy * cols + gx;
                let word = &mut occupied[bit >> 6];
                let mask = 1u64 << (bit & 63);
                if (*word & mask) == 0 {
                    *word |= mask;
                    chosen.push((center, r, c));
                }
            }
        }
    }

    chosen.reverse();
    let reduced = chosen.len() < eligible;
    emit_chosen_marker_mesh(
        chosen,
        result.visited_points,
        result.numeric_rejections,
        reduced,
        style,
        radii,
        ppp,
        fringe,
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_chosen_marker_mesh(
    chosen: Vec<(Pos2, f32, Color32)>,
    visited_points: usize,
    numeric_rejections: usize,
    reduced: bool,
    style: MarkerStyle,
    radii: Option<&[f32]>,
    ppp: f32,
    fringe: f32,
) -> PreparedMarkers {
    let mut result = PreparedMarkers {
        visited_points,
        numeric_rejections,
        reduced,
        ..PreparedMarkers::default()
    };
    let outline = if chosen.len() > 2_048
        && style.shape == MarkerShape::Circle
        && style.radius * ppp <= 5.0
        && radii.is_none()
    {
        (0..8)
            .map(|i| Vec2::angled(i as f32 * std::f32::consts::TAU / 8.0))
            .collect()
    } else {
        marker_outline(style.shape)
    };
    let n = outline.len();
    result.mesh.vertices.reserve(chosen.len() * (1 + 2 * n));
    result.mesh.indices.reserve(chosen.len() * 9 * n);
    if radii.is_none() && !style.shape.is_hollow() {
        let inner: Vec<Vec2> = outline.iter().map(|u| *u * style.radius).collect();
        let outer: Vec<Vec2> = outline
            .iter()
            .map(|u| *u * style.radius + u.normalized() * fringe)
            .collect();
        let n_u32 = n as u32;
        for (center, _, c) in chosen {
            if result.mesh.vertices.len() > (u32::MAX as usize).saturating_sub(64) {
                result.numeric_rejections += 1;
                continue;
            }
            let base = result.mesh.vertices.len() as u32;
            result.mesh.colored_vertex(center, c);
            for off in &inner {
                result.mesh.colored_vertex(center + *off, c);
            }
            for off in &outer {
                result
                    .mesh
                    .colored_vertex(center + *off, Color32::TRANSPARENT);
            }
            for edge in 0..n_u32 {
                let next = (edge + 1) % n_u32;
                result
                    .mesh
                    .add_triangle(base, base + 1 + edge, base + 1 + next);
                result.mesh.add_triangle(
                    base + 1 + edge,
                    base + 1 + n_u32 + edge,
                    base + 1 + n_u32 + next,
                );
                result
                    .mesh
                    .add_triangle(base + 1 + edge, base + 1 + n_u32 + next, base + 1 + next);
            }
            result.output_markers += 1;
        }
    } else {
        for (center, r, c) in chosen {
            if !push_shaped_marker(
                &mut result.mesh,
                center,
                r,
                c,
                style.shape,
                &outline,
                fringe,
            ) {
                result.numeric_rejections += 1;
                continue;
            }
            result.output_markers += 1;
        }
    }
    result
}
