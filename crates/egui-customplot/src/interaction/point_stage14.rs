use egui::Pos2;

use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, PlotTransform, axis::SeriesTransforms,
    model::ResolvedSeriesStyle,
};

pub(super) fn pick_stage14_point(
    layer: PlotLayer<'_>,
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    let transform = transforms.for_axis(layer.y_axis());
    match layer {
        PlotLayer::ProbabilityPlot(prob) => {
            let hit_r = radius.max(resolved.marker.radius);
            let mut best = None;
            for (idx, &pt) in prob.data.points().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, idx, pt, transform, position, hit_r);
            }
            Some(best)
        }
        PlotLayer::EcdfPlot(ecdf) => {
            let mut best = None;
            for (idx, &pt) in ecdf
                .data
                .event_points()
                .iter()
                .chain(ecdf.data.censored_marks().iter())
                .enumerate()
            {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, idx, pt, transform, position, radius);
            }
            Some(best)
        }
        PlotLayer::Biplot(bp) => {
            let mut vec_style = resolved;
            vec_style.stroke.color = bp.loading_color.unwrap_or(resolved.stroke.color);
            vec_style.stroke.width = resolved.stroke.width.max(1.5);
            let (mut best, visited_q) = super::quiver::pick_quiver_data(
                bp.data.loadings_quiver(),
                transform,
                position,
                radius,
                vec_style,
            );
            diagnostics.visited_pick_quiver_arrows += visited_q;
            if bp.show_scores {
                let n_vars = bp.data.n_vars();
                let hit_r = radius.max(resolved.marker.radius);
                for (i, &pt) in bp.data.scores().points().iter().enumerate() {
                    diagnostics.visited_pick_points += 1;
                    consider_point(&mut best, n_vars + i, pt, transform, position, hit_r);
                }
            }
            Some(best)
        }
        PlotLayer::ControlChart(chart) => {
            let hit_r = radius.max(resolved.marker.radius);
            let mut best = None;
            for (idx, spc) in chart.data.points().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(
                    &mut best,
                    idx,
                    [spc.x, spc.value],
                    transform,
                    position,
                    hit_r,
                );
            }
            Some(best)
        }
        PlotLayer::OhlcBars(ohlc) => {
            let mut best = None;
            for (idx, b) in ohlc.data.bars().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, idx, [b.x, b.close], transform, position, radius);
                consider_point(&mut best, idx, [b.x, b.open], transform, position, radius);
            }
            Some(best)
        }
        _ => None,
    }
}

fn consider_point(
    best: &mut Option<(usize, PlotPoint, f32)>,
    index: usize,
    point: PlotPoint,
    transform: PlotTransform,
    pointer: Pos2,
    radius: f32,
) {
    if !transform.is_valid_point(point) {
        return;
    }
    let pos = transform.position_from_point(point);
    if !pos.is_finite() || !transform.rect().expand(radius).contains(pos) {
        return;
    }
    let dist = pos.distance(pointer);
    if dist <= radius && best.as_ref().is_none_or(|cur| dist < cur.2) {
        *best = Some((index, point, dist));
    }
}
