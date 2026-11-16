use egui::{Color32, Pos2};

use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, axis::SeriesTransforms, data::QueryStats,
    model::ResolvedSeriesStyle,
};

pub(super) fn pick_fast_point(
    layer: PlotLayer<'_>,
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    best_limit: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    let visible = stroke_visible(resolved);
    match layer {
        PlotLayer::StreamLine(line) => {
            if !visible {
                return Some(None);
            }
            let transform = transforms.for_axis(line.y_axis);
            let mut stats = QueryStats::default();
            let hit = line
                .data
                .pick(transform, position, radius, best_limit, &mut stats);
            diagnostics.visited_pick_points += stats.scanned_points;
            diagnostics.pick_index_runs += stats.visited_runs;
            diagnostics.pick_index_comparisons += stats.comparisons;
            Some(hit)
        }
        PlotLayer::WaveformLine(line) => {
            if !visible {
                return Some(None);
            }
            let transform = transforms.for_axis(line.y_axis);
            let mut stats = QueryStats::default();
            let hit = line
                .data
                .pick(transform, position, radius, best_limit, &mut stats);
            diagnostics.visited_pick_points += stats.scanned_points;
            diagnostics.pick_index_runs += stats.visited_runs;
            diagnostics.pick_index_comparisons += stats.comparisons;
            Some(hit)
        }
        _ => None,
    }
}

fn stroke_visible(resolved: ResolvedSeriesStyle) -> bool {
    let stroke = resolved.stroke;
    stroke.width.is_finite()
        && stroke.width > 0.0
        && stroke.color != Color32::TRANSPARENT
        && resolved.pattern.is_valid()
}
