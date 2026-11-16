use egui::{Color32, Painter, Pos2, Shape, Stroke};

use super::super::pattern::apply_pattern;
use crate::{
    CompassPlot, MarkerShape, NegativeRadiusMode, PolarBubble, PolarDiagnostics, PolarHistogram,
    PolarLayer, PolarLine, PolarScatter, PolarState, PolarTransform,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_polar_layers(
    painter: &Painter,
    state: &PolarState,
    layers: &[PolarLayer<'_>],
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    _pixels_per_point: f32,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut PolarDiagnostics,
) {
    for (idx, layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        if !state.layer_visible(id) {
            continue;
        }
        let resolved = layer.resolved_style(idx, style);
        match *layer {
            PolarLayer::Histogram(h) => {
                paint_histogram(painter, h, transform, resolved, diagnostics);
            }
            PolarLayer::Line(l) => {
                paint_line(painter, l, transform, neg_mode, resolved, diagnostics);
            }
            PolarLayer::Scatter(s) => {
                paint_scatter(painter, s, transform, neg_mode, resolved, diagnostics);
            }
            PolarLayer::Bubble(b) => {
                paint_bubble(painter, b, transform, neg_mode, resolved, diagnostics);
            }
            PolarLayer::Compass(c) => {
                paint_compass(painter, c, transform, resolved, diagnostics);
            }
        }
    }
}

fn paint_histogram(
    painter: &Painter,
    hist: PolarHistogram<'_>,
    transform: PolarTransform,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PolarDiagnostics,
) {
    let vals = hist.data.values(hist.normalization);
    let edges = hist.data.edges();
    let [r_lo, r_hi] = transform.r_lim();
    let span_r = (r_hi - r_lo).max(1e-12);
    let max_r = transform.pixel_radius();
    let fill = resolved.stroke.color.gamma_multiply(hist.alpha);
    for (i, &v) in vals.iter().enumerate() {
        diagnostics.visited_items += 1;
        if v <= r_lo || r_hi <= r_lo {
            continue;
        }
        let top = v.min(r_hi);
        let base = 0.0_f64.clamp(r_lo, r_hi);
        if top <= base {
            continue;
        }
        let inner_px = ((base - r_lo) / span_r) as f32 * max_r;
        let outer_px = ((top - r_lo) / span_r) as f32 * max_r;
        let e0 = edges[i];
        let e1 = edges[i + 1];
        let mesh = super::sector_mesh(transform, e0, e1 - e0, inner_px, outer_px, fill);
        if !mesh.is_empty() {
            painter.add(Shape::mesh(mesh));
            diagnostics.output_sectors += 1;
            if let Some(border) = hist.border
                && border.width > 0.0
                && border.color != Color32::TRANSPARENT
            {
                let mut arc = super::arc_points(transform, e0, e1 - e0, outer_px, 16);
                let s1 = transform.screen_angle(e1);
                let s0 = transform.screen_angle(e0);
                let c = transform.center();
                arc.push(Pos2::new(
                    c.x + inner_px * s1.cos() as f32,
                    c.y + inner_px * s1.sin() as f32,
                ));
                arc.push(Pos2::new(
                    c.x + inner_px * s0.cos() as f32,
                    c.y + inner_px * s0.sin() as f32,
                ));
                if let Some(&first) = arc.first() {
                    arc.push(first);
                }
                painter.add(Shape::line(arc, border));
            }
        }
    }
}

fn paint_line(
    painter: &Painter,
    line: PolarLine<'_>,
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PolarDiagnostics,
) {
    if resolved.stroke.color == Color32::TRANSPARENT || resolved.stroke.width <= 0.0 {
        return;
    }
    let pts = line.data.points();
    let mut paths: Vec<Vec<Pos2>> = Vec::new();
    let mut current: Vec<Pos2> = Vec::new();
    let mut prev_raw: Option<[f64; 2]> = None;

    for &[th, r] in pts {
        diagnostics.visited_items += 1;
        if !th.is_finite() || !r.is_finite() {
            if current.len() >= 2 {
                paths.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            prev_raw = None;
            continue;
        }
        if let Some([p_th, p_r]) = prev_raw {
            let steps = (((th - p_th).abs() / 0.08).ceil() as usize).clamp(1, 32);
            for s in 1..=steps {
                let frac = s as f64 / steps as f64;
                let it = p_th + (th - p_th) * frac;
                let ir = p_r + (r - p_r) * frac;
                if let Some(pos) = transform.position_from_polar(it, ir, neg_mode) {
                    current.push(pos);
                } else {
                    diagnostics.clipped_points += 1;
                    if current.len() >= 2 {
                        paths.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                }
            }
        } else if let Some(pos) = transform.position_from_polar(th, r, neg_mode) {
            current.push(pos);
        } else {
            diagnostics.clipped_points += 1;
        }
        prev_raw = Some([th, r]);
    }
    if current.len() >= 2 {
        paths.push(current);
    }
    let patterned = apply_pattern(paths, resolved.pattern);
    diagnostics.output_paths += patterned.paths.len();
    for path in patterned.paths {
        painter.add(Shape::line(path, resolved.stroke));
    }
    if line.show_markers && resolved.marker.radius > 0.0 {
        for &[th, r] in pts {
            if let Some(pos) = transform.position_from_polar(th, r, neg_mode) {
                draw_polar_marker(
                    painter,
                    pos,
                    resolved.marker.radius,
                    resolved.marker.shape,
                    resolved.stroke.color,
                );
                diagnostics.output_markers += 1;
            }
        }
    }
}

fn paint_scatter(
    painter: &Painter,
    scatter: PolarScatter<'_>,
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PolarDiagnostics,
) {
    let pts = scatter.data.points();
    let default_c = resolved.stroke.color;
    let default_r = resolved.marker.radius.max(2.0);
    for (i, &[th, r]) in pts.iter().enumerate() {
        diagnostics.visited_items += 1;
        let Some(pos) = transform.position_from_polar(th, r, neg_mode) else {
            diagnostics.clipped_points += 1;
            continue;
        };
        let color = scatter
            .colors
            .and_then(|c| c.get(i).copied())
            .unwrap_or(default_c);
        let radius = scatter
            .radii
            .and_then(|rs| rs.get(i).copied())
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(default_r);
        if color != Color32::TRANSPARENT {
            draw_polar_marker(painter, pos, radius, resolved.marker.shape, color);
            diagnostics.output_markers += 1;
        }
    }
}

fn paint_bubble(
    painter: &Painter,
    bubble: PolarBubble<'_>,
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PolarDiagnostics,
) {
    for pt in bubble.data.points() {
        diagnostics.visited_items += 1;
        if !pt.size.is_finite() || pt.size < 0.0 {
            continue;
        }
        let Some(pos) = transform.position_from_polar(pt.theta, pt.r, neg_mode) else {
            diagnostics.clipped_points += 1;
            continue;
        };
        let base_c = match (bubble.color_scale, pt.scalar) {
            (Some(cs), Some(v)) => cs.color(v).unwrap_or(resolved.stroke.color),
            _ => resolved.stroke.color,
        };
        let fill = base_c.gamma_multiply(bubble.alpha);
        let rad = bubble.data.pixel_radius(pt.size);
        let stroke = bubble.border.unwrap_or(Stroke::NONE);
        painter.circle(pos, rad, fill, stroke);
        diagnostics.output_markers += 1;
    }
}

fn paint_compass(
    painter: &Painter,
    compass: CompassPlot<'_>,
    transform: PolarTransform,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PolarDiagnostics,
) {
    let [r_lo, _] = transform.r_lim();
    let origin_r = 0.0_f64.max(r_lo);
    let head_len = f64::from(compass.head_size[0].max(2.0));
    let head_half = f64::from(compass.head_size[1].max(1.0)) * 0.5;
    for &[th, rho] in compass.data.arrows() {
        diagnostics.visited_items += 1;
        if !th.is_finite() || !rho.is_finite() {
            continue;
        }
        let Some(start) =
            transform.position_from_polar(th, origin_r, NegativeRadiusMode::ShiftMinRadius)
        else {
            continue;
        };
        let Some(end) = transform.position_from_polar(th, rho, NegativeRadiusMode::ShiftMinRadius)
        else {
            diagnostics.clipped_points += 1;
            continue;
        };
        let delta = end - start;
        let len = delta.length();
        if len <= 1e-3 {
            continue;
        }
        painter.line_segment([start, end], resolved.stroke);
        diagnostics.output_arrows += 1;
        if compass.show_heads {
            let dir = [f64::from(delta.x / len), f64::from(delta.y / len)];
            let scale = (f64::from(len) * 0.45 / head_len).min(1.0);
            let hl = head_len * scale;
            let hw = head_half * scale;
            let left = Pos2::new(
                (f64::from(end.x) - dir[0] * hl - dir[1] * hw) as f32,
                (f64::from(end.y) - dir[1] * hl + dir[0] * hw) as f32,
            );
            let right = Pos2::new(
                (f64::from(end.x) - dir[0] * hl + dir[1] * hw) as f32,
                (f64::from(end.y) - dir[1] * hl - dir[0] * hw) as f32,
            );
            painter.add(Shape::line(vec![left, end, right], resolved.stroke));
        }
    }
}

fn draw_polar_marker(
    painter: &Painter,
    pos: Pos2,
    radius: f32,
    shape: MarkerShape,
    color: Color32,
) {
    match shape {
        MarkerShape::Circle => {
            painter.circle_filled(pos, radius, color);
        }
        MarkerShape::Square => {
            let r = egui::Rect::from_center_size(pos, egui::vec2(radius * 1.8, radius * 1.8));
            painter.rect_filled(r, 0.0, color);
        }
        MarkerShape::Diamond => {
            let pts = vec![
                Pos2::new(pos.x, pos.y - radius),
                Pos2::new(pos.x + radius, pos.y),
                Pos2::new(pos.x, pos.y + radius),
                Pos2::new(pos.x - radius, pos.y),
            ];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        _ => {
            painter.circle_filled(pos, radius, color);
        }
    }
}
