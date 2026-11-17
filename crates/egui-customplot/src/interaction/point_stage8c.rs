use egui::{Color32, Pos2};

use crate::{
    BarOrientation, PlotDiagnostics, PlotLayer, PlotPoint, PlotTransform, axis::SeriesTransforms,
    model::ResolvedSeriesStyle,
};

#[inline]
fn orient(o: BarOrientation, pos: f64, val: f64) -> PlotPoint {
    match o {
        BarOrientation::Vertical => [pos, val],
        BarOrientation::Horizontal => [val, pos],
    }
}

pub(super) fn pick_stage8c_point(
    layer: PlotLayer<'_>,
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    resolved: ResolvedSeriesStyle,
    _diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    let transform = transforms.for_axis(layer.y_axis());
    match layer {
        PlotLayer::BubbleChart(bubble) => {
            let mut best = None;
            for (idx, pt) in bubble.data.points().iter().enumerate() {
                if !pt.size.is_finite() || pt.size < 0.0 {
                    continue;
                }
                let hit_r = radius.max(bubble.data.pixel_radius(pt.size));
                consider_point(&mut best, idx, [pt.x, pt.y], transform, position, hit_r);
            }
            Some(best)
        }
        PlotLayer::SwarmChart(swarm) => {
            if resolved.stroke.color == Color32::TRANSPARENT {
                return Some(None);
            }
            let hit_r = radius.max(resolved.marker.radius);
            let mut best = None;
            for (idx, pt) in swarm.data.points().iter().enumerate() {
                consider_point(&mut best, idx, [pt.x, pt.y], transform, position, hit_r);
            }
            Some(best)
        }
        PlotLayer::BoxChart(b) => {
            let mut best = None;
            let mut idx = 0usize;
            for box_stat in b.data.boxes() {
                consider_point(
                    &mut best,
                    idx,
                    orient(b.orientation, box_stat.position, box_stat.median),
                    transform,
                    position,
                    radius,
                );
                idx += 1;
                if b.show_outliers {
                    for &out in box_stat.outliers.iter() {
                        consider_point(
                            &mut best,
                            idx,
                            orient(b.orientation, box_stat.position, out),
                            transform,
                            position,
                            radius,
                        );
                        idx += 1;
                    }
                }
            }
            Some(best)
        }
        PlotLayer::Raincloud(rain) => {
            let mut best = None;
            let mut idx = 0usize;
            for entry in rain.data.entries() {
                for (&x, &y) in entry.drop_x.iter().zip(entry.drop_y.iter()) {
                    consider_point(
                        &mut best,
                        idx,
                        orient(rain.orientation, x, y),
                        transform,
                        position,
                        radius,
                    );
                    idx += 1;
                }
            }
            Some(best)
        }
        PlotLayer::ParallelPlot(par) => {
            let mut best = None;
            let n_vars = par.data.n_vars();
            for row in 0..par.data.n_rows() {
                let norm = par.data.normalized_row(row);
                for (col, &y) in norm.iter().enumerate() {
                    if !y.is_finite() {
                        continue;
                    }
                    consider_point(
                        &mut best,
                        row * n_vars + col,
                        [col as f64, y],
                        transform,
                        position,
                        radius,
                    );
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
