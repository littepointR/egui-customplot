use egui::{Color32, Pos2};

use crate::{
    Area, Contours, PlotDiagnostics, PlotPoint, PlotTransform, model::ResolvedSeriesStyle,
};

pub(super) fn pick_area_band(
    area: Area<'_>,
    band: &crate::AreaBandData,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<(usize, PlotPoint, f32)> {
    if !area.show_stroke
        || !resolved.stroke.width.is_finite()
        || resolved.stroke.width <= 0.0
        || resolved.stroke.color == Color32::TRANSPARENT
        || !resolved.pattern.is_valid()
    {
        return None;
    }
    let x = band.x();
    if x.is_empty() {
        return None;
    }
    let rect = transform.rect();
    let xa =
        transform.point_from_position(egui::pos2(position.x - radius - 1.0, rect.center().y))[0];
    let xb =
        transform.point_from_position(egui::pos2(position.x + radius + 1.0, rect.center().y))[0];
    let (start, end) = if band.is_x_monotonic() && xa.is_finite() && xb.is_finite() {
        let lo = xa.min(xb);
        let hi = xa.max(xb);
        let s = x.partition_point(|v| *v < lo).saturating_sub(1);
        let e = (x.partition_point(|v| *v <= hi) + 1).min(x.len());
        (s, e)
    } else {
        (0, x.len())
    };
    let mut best: Option<(usize, PlotPoint, f32)> = None;
    for (i, (&xv, &yv)) in x
        .iter()
        .zip(band.upper().iter())
        .enumerate()
        .take(end)
        .skip(start)
    {
        diagnostics.visited_pick_points += 1;
        let pt = [xv, yv];
        if !transform.is_valid_point(pt) {
            continue;
        }
        let screen = transform.position_from_point(pt);
        if !transform.rect().contains(screen) {
            continue;
        }
        let dist = screen.distance(position);
        if dist <= radius && best.is_none_or(|b| dist < b.2) {
            best = Some((i, pt, dist));
        }
    }
    best
}

pub(super) fn pick_contours(
    contours: Contours<'_>,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<(usize, PlotPoint, f32)> {
    diagnostics.contours.pick_queries += 1;
    if !resolved.stroke.width.is_finite()
        || resolved.stroke.width <= 0.0
        || !resolved.pattern.is_valid()
    {
        return None;
    }
    let mut best: Option<(usize, PlotPoint, f32)> = None;
    let mut vertex_index = 0usize;
    for line in contours.data.lines() {
        let color = contours
            .color_scale
            .map_or(Some(resolved.stroke.color), |scale| scale.color(line.level));
        if color.is_none_or(|c| c == Color32::TRANSPARENT) {
            vertex_index += line.points.len();
            continue;
        }
        for &pt in &line.points {
            let idx = vertex_index;
            vertex_index += 1;
            diagnostics.visited_pick_points += 1;
            if !transform.is_valid_point(pt) {
                continue;
            }
            let screen = transform.position_from_point(pt);
            if !transform.rect().contains(screen) {
                continue;
            }
            let dist = screen.distance(position);
            if dist <= radius && best.is_none_or(|b| dist < b.2) {
                best = Some((idx, pt, dist));
            }
        }
    }
    best
}
