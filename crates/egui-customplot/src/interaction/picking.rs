use std::time::Instant;

use egui::{Color32, IdSalt, Pos2, Response, Stroke};

use crate::{PlotDiagnostics, PlotPoint, PlotState, PlotTransform, Series};

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
) -> Option<HoveredPoint> {
    let start = Instant::now();
    let mut closest: Option<HoveredPoint> = None;
    for (series_index, s) in series.iter().enumerate() {
        if !state.series_visible(s.id(series_index)) {
            continue;
        }
        for (point_index, point) in s.data().points().iter().enumerate() {
            diagnostics.visited_pick_points += 1;
            if !transform.is_valid_point(*point) || !point_visible(*s, point_index, series_index) {
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

pub(crate) fn show_tooltip(
    response: &Response,
    series: &[Series<'_>],
    point: Option<&HoveredPoint>,
    position: PlotPoint,
) {
    response.clone().on_hover_ui_at_pointer(|ui| {
        if let Some(point) = point {
            let name = series[point.series_index].name();
            if !name.is_empty() {
                ui.strong(name);
            }
            ui.monospace(format!(
                "[{}]  x={:.8e}  y={:.8e}",
                point.point_index, point.point[0], point.point[1]
            ));
        } else {
            ui.monospace(format!("x={:.8e}  y={:.8e}", position[0], position[1]));
        }
    });
}

fn point_visible(series: Series<'_>, index: usize, series_index: usize) -> bool {
    match series {
        Series::Line(line) => {
            let stroke = line
                .stroke
                .unwrap_or(Stroke::new(1.5, series.color(series_index)));
            (stroke.width.is_finite()
                && stroke.width > 0.0
                && stroke.color != Color32::TRANSPARENT
                && line.pattern.is_valid())
                || line.markers.is_some_and(|style| {
                    style.radius.is_finite()
                        && style.radius > 0.0
                        && stroke.color != Color32::TRANSPARENT
                })
        }
        Series::Scatter(scatter) => {
            let radius = scatter.radii.map_or(scatter.style.radius, |r| r[index]);
            radius.is_finite()
                && radius > 0.0
                && scatter
                    .colors
                    .map_or(series.color(series_index), |c| c[index])
                    != Color32::TRANSPARENT
        }
    }
}
