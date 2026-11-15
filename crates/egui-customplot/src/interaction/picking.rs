use std::time::Instant;

use egui::{Color32, IdSalt, Pos2};

use crate::{
    PlotDiagnostics, PlotPoint, PlotState, PlotTransform, Series,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

#[derive(Clone, Debug)]
pub struct HoveredPoint {
    pub series_id: IdSalt,
    pub series_index: usize,
    pub point_index: usize,
    pub point: PlotPoint,
    pub screen_distance: f32,
}

/// O(N) reference picking of raw samples, never the reduced render geometry.
pub(crate) fn pick_point(
    state: &PlotState,
    series: &[Series<'_>],
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
) -> Option<HoveredPoint> {
    let start = Instant::now();
    let mut closest: Option<HoveredPoint> = None;
    for (series_index, s) in series.iter().enumerate() {
        if !state.series_visible(s.id(series_index)) {
            continue;
        }
        let resolved = s.resolved_style(series_index, style);
        for (point_index, point) in s.data().points().iter().enumerate() {
            diagnostics.visited_pick_points += 1;
            if !transform.is_valid_point(*point) || !point_visible(*s, point_index, resolved) {
                continue;
            }
            let screen = transform.position_from_point(*point);
            if !transform.rect().contains(screen) {
                continue;
            }
            let distance = screen.distance(position);
            if distance <= radius
                && closest
                    .as_ref()
                    .is_none_or(|p| distance < p.screen_distance)
            {
                closest = Some(HoveredPoint {
                    series_id: s.id(series_index),
                    series_index,
                    point_index,
                    point: *point,
                    screen_distance: distance,
                });
            }
        }
    }
    diagnostics.pick_duration = start.elapsed();
    closest
}

fn point_visible(series: Series<'_>, index: usize, resolved: ResolvedSeriesStyle) -> bool {
    match series {
        Series::Line(_) => {
            let stroke = resolved.stroke;
            (stroke.width.is_finite()
                && stroke.width > 0.0
                && stroke.color != Color32::TRANSPARENT
                && resolved.pattern.is_valid())
                || (resolved.line_markers
                    && resolved.marker.radius.is_finite()
                    && resolved.marker.radius > 0.0
                    && stroke.color != Color32::TRANSPARENT)
        }
        Series::Scatter(scatter) => {
            let radius = scatter.radii.map_or(resolved.marker.radius, |r| r[index]);
            radius.is_finite()
                && radius > 0.0
                && scatter.colors.map_or(resolved.stroke.color, |c| c[index])
                    != Color32::TRANSPARENT
        }
    }
}
