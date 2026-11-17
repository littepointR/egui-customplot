use egui::IdSalt;

use super::{HoveredRegion, RegionKind, base_region};
use crate::{
    BarOrientation, PlotLayer, PlotPoint, PlotTransform, WaterfallKind, YAxis,
    axis::{clipped_data_rect, valid_extent},
};

pub(super) fn pick_stage14_region(
    layer: PlotLayer<'_>,
    layer_id: IdSalt,
    layer_index: usize,
    y_axis: YAxis,
    point: PlotPoint,
    transform: PlotTransform,
) -> Option<HoveredRegion> {
    match layer {
        PlotLayer::DendrogramPlot(dendro) => {
            let max_d = dendro.data.max_distance().max(1.0);
            let leaf_tol = 0.22;
            let dist_tol = max_d * 0.04;
            let (leaf_coord, dist_coord) = match dendro.orientation {
                BarOrientation::Vertical => (point[0], point[1]),
                BarOrientation::Horizontal => (point[1], point[0]),
            };
            for branch in dendro.data.branches().iter().rev() {
                let on_top = leaf_coord >= branch.x_left - leaf_tol
                    && leaf_coord <= branch.x_right + leaf_tol
                    && (dist_coord - branch.y_top).abs() <= dist_tol;
                let on_left = (leaf_coord - branch.x_left).abs() <= leaf_tol
                    && dist_coord >= branch.y_left - dist_tol
                    && dist_coord <= branch.y_top + dist_tol;
                let on_right = (leaf_coord - branch.x_right).abs() <= leaf_tol
                    && dist_coord >= branch.y_right - dist_tol
                    && dist_coord <= branch.y_top + dist_tol;
                if on_top || on_left || on_right {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::DendrogramBranch,
                        branch.index,
                        point,
                    );
                    hit.scalar_value = Some(branch.y_top);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::Candlestick(candle) => {
            for (idx, b) in candle.data.bars().iter().enumerate().rev() {
                let x0 = b.x - 0.5 * b.width;
                let x1 = b.x + 0.5 * b.width;
                if point[0] >= x0 && point[0] <= x1 && point[1] >= b.low && point[1] <= b.high {
                    let body_lo = b.open.min(b.close);
                    let body_hi = b.open.max(b.close);
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::OhlcCandle,
                        idx,
                        point,
                    );
                    hit.scalar_value = Some(b.close);
                    let col = if b.is_up() {
                        candle.up_color
                    } else {
                        candle.down_color
                    };
                    hit.rgba = Some([col.r(), col.g(), col.b(), col.a()]);
                    if body_hi > body_lo && valid_extent([x0, body_lo], [x1, body_hi], transform) {
                        hit.screen_rect =
                            clipped_data_rect([x0, body_lo], [x1, body_hi], transform);
                    }
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::Waterfall2D(wf) => {
            for (idx, col) in wf.data.columns().iter().enumerate().rev() {
                let x0 = col.x - 0.5 * col.width;
                let x1 = col.x + 0.5 * col.width;
                let y0 = col.start.min(col.end);
                let y1 = col.start.max(col.end);
                if point[0] >= x0 && point[0] <= x1 && point[1] >= y0 && point[1] <= y1 {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::WaterfallBar,
                        idx,
                        point,
                    );
                    hit.scalar_value = Some(col.delta);
                    let rgba = match col.kind {
                        WaterfallKind::Positive => wf.positive_color,
                        WaterfallKind::Negative => wf.negative_color,
                        WaterfallKind::Total => wf.total_color,
                    };
                    hit.rgba = Some([rgba.r(), rgba.g(), rgba.b(), rgba.a()]);
                    if y1 > y0 && valid_extent([x0, y0], [x1, y1], transform) {
                        hit.screen_rect = clipped_data_rect([x0, y0], [x1, y1], transform);
                    }
                    return Some(hit);
                }
            }
            None
        }
        _ => None,
    }
}
