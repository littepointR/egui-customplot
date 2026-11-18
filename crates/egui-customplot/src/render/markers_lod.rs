use std::ops::Range;

use egui::{Color32, Pos2};

use super::markers::{
    PreparedMarkers, marker_outline, prepare_markers, prepare_selected_markers, push_shaped_marker,
};
use super::markers_grid_tree::collect_indexed_grid_markers;
use crate::{
    AxisScale, MarkerSampling, MarkerStyle, PlotData, PlotPoint, PlotTransform, data::QueryStats,
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
    ctx: &egui::Context,
    sampling: MarkerSampling,
) -> PreparedMarkers {
    let points = data.points();
    if points.is_empty() {
        return PreparedMarkers::default();
    }
    let ppp = ctx.pixels_per_point().max(0.01);
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
        return prepare_selected_markers(
            grid.chosen,
            grid.visited_points,
            grid.numeric_rejections,
            grid.reduced,
            style,
            radii,
            ctx,
        );
    }
    prepare_markers_pixel_grid(points, ranges, transform, style, color, colors, radii, ctx)
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
    ctx: &egui::Context,
) -> PreparedMarkers {
    let ppp = ctx.pixels_per_point().max(0.01);
    let fringe = 1.0 / ppp;
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
    prepare_selected_markers(
        chosen,
        result.visited_points,
        result.numeric_rejections,
        reduced,
        style,
        radii,
        ctx,
    )
}
