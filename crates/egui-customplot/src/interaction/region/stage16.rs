use egui::IdSalt;

use super::{HoveredRegion, RegionKind, base_region};
use crate::{
    PlotLayer, PlotPoint, PlotTransform, YAxis,
    axis::{clipped_data_rect, valid_extent},
};

pub(super) fn pick_stage16_region(
    layer: PlotLayer<'_>,
    layer_id: IdSalt,
    layer_index: usize,
    y_axis: YAxis,
    point: PlotPoint,
    transform: PlotTransform,
) -> Option<HoveredRegion> {
    match layer {
        PlotLayer::TreemapChart(tm) => {
            for r in tm.data.rects() {
                if point[0] >= r.x0 && point[0] <= r.x1 && point[1] >= r.y0 && point[1] <= r.y1 {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::TreemapCell,
                        r.index,
                        point,
                    );
                    hit.scalar_value = Some(r.value);
                    if valid_extent([r.x0, r.y0], [r.x1, r.y1], transform) {
                        hit.screen_rect = clipped_data_rect([r.x0, r.y0], [r.x1, r.y1], transform);
                    }
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::SunburstChart(sb) => {
            let r = point[0].hypot(point[1]);
            let a = point[1].atan2(point[0]).rem_euclid(std::f64::consts::TAU);
            for sec in sb.data.sectors().iter().rev() {
                if r >= sec.r_in && r <= sec.r_out && a >= sec.start_rad && a <= sec.end_rad {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::SunburstSector,
                        sec.index,
                        point,
                    );
                    hit.scalar_value = Some(sec.value);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::SankeyChart(sankey) => {
            for rib in sankey.data.ribbons() {
                let x0 = rib.top_curve.first().map_or(0.0, |p| p[0]);
                let x1 = rib.top_curve.last().map_or(0.0, |p| p[0]);
                if point[0] < x0 || point[0] > x1 || (x1 - x0) <= 1e-12 {
                    continue;
                }
                let u = ((point[0] - x0) / (x1 - x0)).clamp(0.0, 1.0);
                let idx = (u * (rib.top_curve.len() - 1) as f64).round() as usize;
                let y_top = rib.top_curve[idx][1];
                let y_bot = rib.bottom_curve[idx][1];
                if point[1] >= y_bot && point[1] <= y_top {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::SankeyRibbon,
                        rib.index,
                        point,
                    );
                    hit.scalar_value = Some(rib.value);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::GanttChart(gantt) => {
            for (idx, t) in gantt.data.tasks().iter().enumerate() {
                let y = idx as f64 + 1.0;
                if (point[1] - y).abs() <= 0.34 && point[0] >= t.start && point[0] <= t.end {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::GanttTaskBar,
                        idx,
                        point,
                    );
                    hit.scalar_value = Some(t.progress * 100.0);
                    if !t.milestone
                        && valid_extent([t.start, y - 0.32], [t.end, y + 0.32], transform)
                    {
                        hit.screen_rect =
                            clipped_data_rect([t.start, y - 0.32], [t.end, y + 0.32], transform);
                    }
                    return Some(hit);
                }
            }
            None
        }
        _ => None,
    }
}
