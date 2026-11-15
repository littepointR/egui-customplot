use egui::{Painter, Pos2, Shape, Stroke};

use super::{
    line::prepare_line, mesh::stroke_valid, pattern::apply_pattern, quiver::paint_quiver_data,
};
use crate::{
    Feather, LineInterpolation, PlotDiagnostics, PlotPoint, PlotTransform, Streamlines,
    axis::project_boundary_segment, model::ResolvedSeriesStyle,
};

pub(super) fn paint_streamlines(
    painter: &Painter,
    stream: Streamlines<'_>,
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

    let mut all_paths = Vec::new();
    for line in stream.data.lines() {
        let prepared = prepare_line(line, transform, LineInterpolation::Linear);
        diagnostics.flow.visited_items += prepared.visited_segments;
        diagnostics.numeric_rejections += prepared.numeric_rejections;
        all_paths.extend(prepared.paths);
    }
    let patterned = apply_pattern(all_paths, style.pattern);
    diagnostics.pattern_truncated |= patterned.truncated;
    diagnostics.flow.output_paths += patterned.paths.len();
    for path in patterned.paths {
        diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        painter.add(Shape::line(path, style.stroke));
    }

    if !stream.show_arrows {
        return;
    }
    if stream.arrow_size.iter().any(|v| !v.is_finite() || *v < 0.0) {
        diagnostics.numeric_rejections += 1;
        return;
    }
    if !stream.arrow_size.iter().all(|v| *v > 0.0) {
        return;
    }

    let rect = transform.rect();
    for &(anchor, dir) in stream.data.arrows() {
        diagnostics.flow.visited_items += 1;
        if !transform.is_valid_point(anchor) {
            continue;
        }
        let tip = transform.position_from_point(anchor);
        if !tip.is_finite() || !rect.contains(tip) {
            continue;
        }
        let Some([dx, dy]) = screen_direction(anchor, dir, transform) else {
            continue;
        };
        let len = f64::from(stream.arrow_size[0]);
        let half_w = f64::from(stream.arrow_size[1]) * 0.5;
        let wing = |sign: f64| {
            egui::pos2(
                (f64::from(tip.x) - dx * len - sign * dy * half_w) as f32,
                (f64::from(tip.y) - dy * len + sign * dx * half_w) as f32,
            )
        };
        let head = vec![wing(1.0), tip, wing(-1.0)];
        if head.iter().all(|p| p.is_finite()) {
            diagnostics.flow.output_heads += 1;
            diagnostics.geometry_capacity_bytes += head.capacity() * std::mem::size_of::<Pos2>();
            painter.add(Shape::line(head, style.stroke));
        } else {
            diagnostics.numeric_rejections += 1;
        }
    }
}

pub(super) fn paint_feather(
    painter: &Painter,
    feather: Feather<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    let baseline = feather.baseline_stroke.unwrap_or_else(|| {
        Stroke::new(
            (style.stroke.width * 0.6).max(1.0),
            style.stroke.color.gamma_multiply(0.55),
        )
    });
    if let Some([x0, x1]) = feather.data.x_span()
        && stroke_valid(baseline)
    {
        let y = feather.data.baseline_y();
        if let Ok(Some([pa, pb])) = project_boundary_segment([x0, y], [x1, y], transform) {
            diagnostics.flow.output_paths += 1;
            painter.add(Shape::line_segment([pa, pb], baseline));
        }
    }

    let stats = paint_quiver_data(
        painter,
        feather.data.quiver(),
        feather.show_heads,
        feather.head_size,
        transform,
        style,
    );
    diagnostics.flow.visited_items += stats.visited;
    diagnostics.flow.output_paths += stats.output_shafts;
    diagnostics.flow.output_heads += stats.output_heads;
    diagnostics.geometry_capacity_bytes += stats.capacity_bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
    diagnostics.pattern_truncated |= stats.pattern_truncated;
}

fn screen_direction(
    anchor: PlotPoint,
    dir: [f64; 2],
    transform: PlotTransform,
) -> Option<[f64; 2]> {
    let mag = dir[0].hypot(dir[1]);
    if !mag.is_finite() || mag <= 0.0 {
        return None;
    }
    let span = transform.bounds().span();
    let ux = dir[0] / mag;
    let uy = dir[1] / mag;
    let hx = (span[0] * 1e-4).max(anchor[0].abs() * 1e-5).max(1e-12);
    let hy = (span[1] * 1e-4).max(anchor[1].abs() * 1e-5).max(1e-12);
    let mut probe = [anchor[0] + ux * hx, anchor[1] + uy * hy];
    let mut sign = 1.0;
    if !transform.is_valid_point(probe) {
        probe = [anchor[0] - ux * hx, anchor[1] - uy * hy];
        sign = -1.0;
        if !transform.is_valid_point(probe) {
            return None;
        }
    }
    let p0 = transform.position_from_point(anchor);
    let p1 = transform.position_from_point(probe);
    let sx = sign * f64::from(p1.x - p0.x);
    let sy = sign * f64::from(p1.y - p0.y);
    let slen = sx.hypot(sy);
    if !slen.is_finite() || slen <= 1e-12 {
        return None;
    }
    Some([sx / slen, sy / slen])
}
