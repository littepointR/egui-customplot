use egui::{Color32, Pos2};

use super::quiver::pick_quiver_data;
use crate::{
    Area, Contours, ErrorBars, PlotDiagnostics, PlotLayer, PlotPoint, PlotTransform,
    axis::{SeriesTransforms, valid_extent},
    model::{AreaSource, ResolvedSeriesStyle},
};

pub(super) fn pick_aux_point(
    layer: PlotLayer<'_>,
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    match layer {
        PlotLayer::Quiver(quiver) => {
            let (hit, visited) = pick_quiver_data(
                quiver.data,
                transforms.for_axis(quiver.y_axis),
                position,
                radius,
                resolved,
            );
            diagnostics.visited_pick_quiver_arrows += visited;
            Some(hit)
        }
        PlotLayer::ErrorBars(bars) => Some(pick_error_bars(
            bars,
            transforms.for_axis(bars.y_axis),
            position,
            radius,
            resolved,
            diagnostics,
        )),
        PlotLayer::Area(area) => match area.source {
            AreaSource::Band(band) => Some(pick_area_band(
                area,
                band,
                transforms.for_axis(area.y_axis),
                position,
                radius,
                resolved,
                diagnostics,
            )),
            AreaSource::Curve { .. } => None,
        },
        PlotLayer::Contours(contours) => Some(pick_contours(
            contours,
            transforms.for_axis(contours.y_axis),
            position,
            radius,
            resolved,
            diagnostics,
        )),
        _ => None,
    }
}

fn pick_error_bars(
    bars: ErrorBars<'_>,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<(usize, PlotPoint, f32)> {
    let stroke = resolved.stroke;
    if !stroke.width.is_finite() || stroke.is_empty() {
        return None;
    }
    let mut best: Option<(usize, PlotPoint, f32)> = None;
    for (point_index, &bar) in bars.data.bars().iter().enumerate() {
        diagnostics.visited_pick_points += 1;
        let Some((lo, hi)) = bar.extent() else {
            continue;
        };
        if !bar.has_error() || !valid_extent(lo, hi, transform) {
            continue;
        }
        let screen = transform.position_from_point(bar.point);
        if !transform.rect().contains(screen) {
            continue;
        }
        let distance = screen.distance(position);
        if distance <= radius && best.is_none_or(|b| distance < b.2) {
            best = Some((point_index, bar.point, distance));
        }
    }
    best
}

fn pick_area_band(
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

fn pick_contours(
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
