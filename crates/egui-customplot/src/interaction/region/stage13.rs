use crate::{
    PlotLayer, PlotPoint, PlotTransform, YAxis,
    axis::{clipped_data_rect, point_in_ring_transformed, valid_extent},
};

use super::{HoveredRegion, RegionKind, base_region};

pub(super) fn pick_stage13_region(
    layer: PlotLayer<'_>,
    layer_id: egui::IdSalt,
    layer_index: usize,
    y_axis: YAxis,
    point: PlotPoint,
    transform: PlotTransform,
) -> Option<HoveredRegion> {
    match layer {
        PlotLayer::ConfusionChart(ch) => {
            let data = ch.data;
            let n = data.n_classes();
            for row in 0..n {
                if point[1] < row as f64 + 0.5 || point[1] > row as f64 + 1.5 {
                    continue;
                }
                for col in 0..n {
                    if point[0] < col as f64 + 0.5 || point[0] > col as f64 + 1.5 {
                        continue;
                    }
                    let lo = [col as f64 + 0.5, row as f64 + 0.5];
                    let hi = [col as f64 + 1.5, row as f64 + 1.5];
                    if !valid_extent(lo, hi, transform) {
                        return None;
                    }
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::ConfusionCell,
                        row * n + col,
                        point,
                    );
                    hit.grid_index = Some([col, row]);
                    hit.scalar_value = Some(data.display_value(row, col, ch.normalization));
                    hit.screen_rect = clipped_data_rect(lo, hi, transform);
                    return Some(hit);
                }
            }
            if ch.show_summaries {
                let nf = n as f64;
                for row in 0..n {
                    if point[1] < row as f64 + 0.5 || point[1] > row as f64 + 1.5 {
                        continue;
                    }
                    for sub_c in 0..2 {
                        let x0 = nf + 0.75 + sub_c as f64 * 0.9;
                        if point[0] >= x0 && point[0] <= x0 + 0.9 {
                            let lo = [x0, row as f64 + 0.5];
                            let hi = [x0 + 0.9, row as f64 + 1.5];
                            let mut hit = base_region(
                                layer_id,
                                layer_index,
                                y_axis,
                                RegionKind::ConfusionCell,
                                n * n + row * 2 + sub_c,
                                point,
                            );
                            hit.grid_index = Some([n + sub_c, row]);
                            hit.scalar_value = data
                                .row_recall(row)
                                .map(|r| (if sub_c == 0 { r } else { 1.0 - r }) * 100.0);
                            hit.screen_rect = clipped_data_rect(lo, hi, transform);
                            return Some(hit);
                        }
                    }
                }
                for col in 0..n {
                    if point[0] < col as f64 + 0.5 || point[0] > col as f64 + 1.5 {
                        continue;
                    }
                    for sub_r in 0..2 {
                        let y0 = nf + 0.75 + sub_r as f64 * 0.9;
                        if point[1] >= y0 && point[1] <= y0 + 0.9 {
                            let lo = [col as f64 + 0.5, y0];
                            let hi = [col as f64 + 1.5, y0 + 0.9];
                            let mut hit = base_region(
                                layer_id,
                                layer_index,
                                y_axis,
                                RegionKind::ConfusionCell,
                                n * n + n * 2 + sub_r * n + col,
                                point,
                            );
                            hit.grid_index = Some([col, n + sub_r]);
                            hit.scalar_value = data
                                .col_precision(col)
                                .map(|p| (if sub_r == 0 { p } else { 1.0 - p }) * 100.0);
                            hit.screen_rect = clipped_data_rect(lo, hi, transform);
                            return Some(hit);
                        }
                    }
                }
            }
            None
        }
        PlotLayer::CurvilinearPcolor(pc) => {
            let data = pc.data;
            let cols = data.cell_cols();
            for row in 0..data.cell_rows() {
                for col in 0..cols {
                    let Some(quad) = data.quad_nodes(row, col) else {
                        continue;
                    };
                    if point_in_ring_transformed(point, &quad, transform) {
                        let mut hit = base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::CurvilinearCell,
                            row * cols + col,
                            point,
                        );
                        hit.grid_index = Some([col, row]);
                        hit.scalar_value = data.node_value(row, col);
                        return Some(hit);
                    }
                }
            }
            None
        }
        _ => None,
    }
}
