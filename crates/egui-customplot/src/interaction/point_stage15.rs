use egui::Pos2;

use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, PlotTransform, axis::SeriesTransforms,
    model::ResolvedSeriesStyle,
};

pub(super) fn pick_stage15_point(
    layer: PlotLayer<'_>,
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    let transform = transforms.for_axis(layer.y_axis());
    match layer {
        PlotLayer::ZPlanePlot(zp) => {
            let hit_r = radius.max(resolved.marker.radius).max(6.0);
            let mut best = None;
            let n_z = zp.data.zeros().len();
            for (i, z) in zp.data.zeros().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, i, [z.re, z.im], transform, position, hit_r);
            }
            for (i, p) in zp.data.poles().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, n_z + i, [p.re, p.im], transform, position, hit_r);
            }
            Some(best)
        }
        PlotLayer::NyquistPlot(nyq) => {
            let mut best = None;
            for (i, &pt) in nyq.data.nyquist_curve().points().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, i, pt, transform, position, radius);
            }
            Some(best)
        }
        PlotLayer::NicholsPlot(nich) => {
            let mut best = None;
            for (i, &pt) in nich.data.nichols_curve().points().iter().enumerate() {
                diagnostics.visited_pick_points += 1;
                consider_point(&mut best, i, pt, transform, position, radius);
            }
            Some(best)
        }
        PlotLayer::RootLocus(rl) => {
            let mut best = None;
            let mut idx = 0usize;
            for branch in rl.data.branches() {
                for &pt in branch.points() {
                    diagnostics.visited_pick_points += 1;
                    consider_point(&mut best, idx, pt, transform, position, radius);
                    idx += 1;
                }
            }
            Some(best)
        }
        PlotLayer::RadarChart(radar) => {
            let hit_r = radius.max(6.0);
            let mut best = None;
            let mut idx = 0usize;
            for ser in radar.data.series() {
                for &pt in ser.vertices.iter() {
                    diagnostics.visited_pick_points += 1;
                    consider_point(&mut best, idx, pt, transform, position, hit_r);
                    idx += 1;
                }
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
