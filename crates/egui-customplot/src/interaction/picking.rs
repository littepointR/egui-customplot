use std::time::Instant;

use egui::{Color32, IdSalt, Pos2};

use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, PlotState, Series, YAxis, axis::SeriesTransforms,
    data::QueryStats, model::ResolvedPlotStyle, model::ResolvedSeriesStyle,
};

#[derive(Clone, Debug)]
pub struct HoveredPoint {
    pub y_axis: YAxis,
    pub series_id: IdSalt,
    pub series_index: usize,
    pub point_index: usize,
    pub point: PlotPoint,
    pub screen_distance: f32,
}

pub(super) struct PickOptions {
    pub position: Pos2,
    pub radius: f32,
    pub first_layer: usize,
}

/// Pick raw samples, using an optional X index, never reduced render geometry.
pub(super) fn pick_point(
    state: &PlotState,
    series: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    options: PickOptions,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
) -> Option<HoveredPoint> {
    let start = Instant::now();
    let PickOptions {
        position,
        radius,
        first_layer,
    } = options;
    let mut closest: Option<HoveredPoint> = None;
    for (series_index, s) in series.iter().enumerate().skip(first_layer) {
        if !state.series_visible(s.id(series_index)) {
            continue;
        }
        let resolved = s.resolved_style(series_index, style);
        if let Some(hit) = super::point_aux::pick_aux_point(
            *s,
            transforms,
            position,
            radius,
            resolved,
            diagnostics,
        )
        .or_else(|| {
            super::point_stage8b::pick_stage8b_point(
                *s,
                transforms,
                position,
                radius,
                resolved,
                diagnostics,
            )
        })
        .or_else(|| {
            super::point_stage8c::pick_stage8c_point(
                *s,
                transforms,
                position,
                radius,
                resolved,
                diagnostics,
            )
        }) {
            if let Some((point_index, point, distance)) = hit
                && closest
                    .as_ref()
                    .is_none_or(|p| distance < p.screen_distance)
            {
                closest = Some(HoveredPoint {
                    y_axis: transforms.effective_axis(s.y_axis()),
                    series_id: s.id(series_index),
                    series_index,
                    point_index,
                    point,
                    screen_distance: distance,
                });
            }
            continue;
        }
        let Some(data) = s.point_data() else { continue };
        let transform = transforms.for_axis(s.y_axis());
        if let PlotLayer::Stems(stems) = *s
            && !stems.baseline_valid(transform.scales())
        {
            continue;
        }
        if let PlotLayer::Area(area) = *s
            && (!area.show_stroke || !area.baseline_valid(transform.scales()))
        {
            continue;
        }
        let ranges = if let Some(index) = data.line_index() {
            // A one-point guard conservatively covers f32 projection rounding;
            // the final raw-point distance filter still uses the exact radius.
            let rect = transform.rect();
            let a = transform
                .point_from_position(egui::pos2(position.x - radius - 1.0, rect.center().y))[0];
            let b = transform
                .point_from_position(egui::pos2(position.x + radius + 1.0, rect.center().y))[0];
            let mut stats = QueryStats::default();
            // Do not clamp to the data bounds: just-outside points may round
            // onto the f32 screen edge and are eligible in the reference path.
            // An inverse overflow must broaden, not accidentally empty, a query.
            let x = if a.is_finite() && b.is_finite() {
                [a.min(b), a.max(b)]
            } else {
                [f64::NEG_INFINITY, f64::INFINITY]
            };
            let ranges = index.candidates(data.points(), transform.scales(), x, &mut stats);
            diagnostics.pick_index_runs += stats.visited_runs;
            diagnostics.pick_index_comparisons += stats.comparisons;
            diagnostics.pick_query_capacity_bytes +=
                ranges.capacity() * std::mem::size_of::<std::ops::Range<usize>>();
            ranges
        } else {
            std::iter::once(0..data.len()).collect()
        };
        for point_index in ranges.into_iter().flatten() {
            let point = data.points()[point_index];
            diagnostics.visited_pick_points += 1;
            if !transform.is_valid_point(point) || !point_visible(*s, point_index, resolved) {
                continue;
            }
            let screen = transform.position_from_point(point);
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
                    y_axis: transforms.effective_axis(s.y_axis()),
                    series_id: s.id(series_index),
                    series_index,
                    point_index,
                    point,
                    screen_distance: distance,
                });
            }
        }
    }
    diagnostics.pick_duration = start.elapsed();
    closest
}

fn point_visible(layer: PlotLayer<'_>, index: usize, resolved: ResolvedSeriesStyle) -> bool {
    if let PlotLayer::Stems(stems) = layer {
        let mut resolved = resolved;
        if stems.data.points()[index][stems.orientation.axis()] == stems.baseline {
            resolved.stroke.width = 0.0; // Only an enabled marker can show a zero rod.
        }
        return line_point_visible(resolved);
    }
    if let PlotLayer::Area(area) = layer {
        return area.show_stroke
            && line_point_visible(ResolvedSeriesStyle {
                line_markers: false,
                ..resolved
            });
    }
    let PlotLayer::Series(series) = layer else {
        return false;
    };
    match series {
        Series::Line(_) => line_point_visible(resolved),
        Series::Scatter(scatter) => {
            let radius = scatter.radii.map_or(resolved.marker.radius, |r| r[index]);
            radius.is_finite()
                && radius > 0.0
                && scatter.colors.map_or(resolved.stroke.color, |c| c[index])
                    != Color32::TRANSPARENT
        }
    }
}

fn line_point_visible(resolved: ResolvedSeriesStyle) -> bool {
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
