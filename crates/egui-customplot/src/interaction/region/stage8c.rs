use egui::Color32;

use super::{HoveredRegion, RegionKind, base_region};
use crate::{
    BarOrientation, PlotLayer, PlotPoint, PlotTransform, YAxis,
    axis::{clipped_data_rect, point_in_ring_transformed, valid_extent},
};

#[inline]
fn orient(o: BarOrientation, pos: f64, val: f64) -> PlotPoint {
    match o {
        BarOrientation::Vertical => [pos, val],
        BarOrientation::Horizontal => [val, pos],
    }
}

#[inline]
fn split_coords(o: BarOrientation, point: PlotPoint) -> (f64, f64) {
    match o {
        BarOrientation::Vertical => (point[0], point[1]),
        BarOrientation::Horizontal => (point[1], point[0]),
    }
}

pub(super) fn pick_stage8c_region(
    layer: PlotLayer<'_>,
    layer_id: egui::IdSalt,
    layer_index: usize,
    y_axis: YAxis,
    point: PlotPoint,
    _p_norm: [f64; 2],
    transform: PlotTransform,
) -> Option<HoveredRegion> {
    match layer {
        PlotLayer::Histogram2(hist) => {
            let nx = hist.data.nx();
            let ny = hist.data.ny();
            let x_edges = hist.data.x_edges();
            let y_edges = hist.data.y_edges();
            for row in 0..ny {
                if point[1] < y_edges[row] || point[1] > y_edges[row + 1] {
                    continue;
                }
                for col in 0..nx {
                    if point[0] < x_edges[col] || point[0] > x_edges[col + 1] {
                        continue;
                    }
                    let count = hist.data.counts()[row * nx + col];
                    if count == 0 {
                        return None;
                    }
                    let lo = [x_edges[col], y_edges[row]];
                    let hi = [x_edges[col + 1], y_edges[row + 1]];
                    if !valid_extent(lo, hi, transform) {
                        return None;
                    }
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::Histogram2Bin,
                        row * nx + col,
                        point,
                    );
                    hit.grid_index = Some([col, row]);
                    hit.scalar_value = Some(count as f64);
                    hit.screen_rect = clipped_data_rect(lo, hi, transform);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::BinScatter(bins) => {
            for (idx, cell) in bins.data.cells().iter().enumerate().rev() {
                if point[0] < cell.x0
                    || point[0] > cell.x1
                    || point[1] < cell.y0
                    || point[1] > cell.y1
                {
                    continue;
                }
                let lo = [cell.x0, cell.y0];
                let hi = [cell.x1, cell.y1];
                if !valid_extent(lo, hi, transform) {
                    return None;
                }
                let mut hit = base_region(
                    layer_id,
                    layer_index,
                    y_axis,
                    RegionKind::BinScatterCell,
                    idx,
                    point,
                );
                hit.scalar_value = Some(cell.count as f64);
                hit.screen_rect = clipped_data_rect(lo, hi, transform);
                return Some(hit);
            }
            None
        }
        PlotLayer::BoxChart(boxes) => {
            let (pos_c, val_c) = split_coords(boxes.orientation, point);
            for (idx, b) in boxes.data.boxes().iter().enumerate().rev() {
                let hw = b.width * 0.5;
                if pos_c >= b.position - hw
                    && pos_c <= b.position + hw
                    && val_c >= b.q1
                    && val_c <= b.q3
                {
                    let lo = orient(boxes.orientation, b.position - hw, b.q1);
                    let hi = orient(boxes.orientation, b.position + hw, b.q3);
                    if valid_extent(lo, hi, transform) {
                        let mut hit = base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::BoxChart,
                            idx,
                            point,
                        );
                        hit.scalar_value = Some(b.median);
                        hit.screen_rect = clipped_data_rect(lo, hi, transform);
                        return Some(hit);
                    }
                }
            }
            None
        }
        PlotLayer::ViolinPlot(v) => {
            let (pos_c, val_c) = split_coords(v.orientation, point);
            for (idx, entry) in v.data.entries().iter().enumerate().rev() {
                let y = &entry.y;
                if y.len() < 2 || val_c < y[0] || val_c > y[y.len() - 1] {
                    continue;
                }
                let half_max = entry.half_widths.iter().copied().fold(0.0_f64, f64::max);
                if (pos_c - entry.summary.position).abs() <= half_max {
                    let mut poly = Vec::with_capacity(y.len() * 2);
                    for (&yi, &hw) in y.iter().zip(entry.half_widths.iter()) {
                        poly.push(orient(v.orientation, entry.summary.position + hw, yi));
                    }
                    for (&yi, &hw) in y.iter().zip(entry.half_widths.iter()).rev() {
                        poly.push(orient(v.orientation, entry.summary.position - hw, yi));
                    }
                    if point_in_ring_transformed(point, &poly, transform) {
                        let mut hit = base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::Violin,
                            idx,
                            point,
                        );
                        hit.scalar_value = Some(entry.summary.median);
                        return Some(hit);
                    }
                }
            }
            None
        }
        PlotLayer::Raincloud(r) => {
            let (_, val_c) = split_coords(r.orientation, point);
            for (idx, entry) in r.data.entries().iter().enumerate().rev() {
                let y = &entry.y;
                if y.len() < 2 || val_c < y[0] || val_c > y[y.len() - 1] {
                    continue;
                }
                let mut poly = Vec::with_capacity(y.len() + 2);
                for (&yi, &hw) in y.iter().zip(entry.half_widths.iter()) {
                    poly.push(orient(r.orientation, entry.summary.position + hw, yi));
                }
                poly.push(orient(
                    r.orientation,
                    entry.summary.position,
                    y[y.len() - 1],
                ));
                poly.push(orient(r.orientation, entry.summary.position, y[0]));
                if point_in_ring_transformed(point, &poly, transform) {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::Raincloud,
                        idx,
                        point,
                    );
                    hit.scalar_value = Some(entry.summary.median);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::PieChart(pie) => {
            for slice in pie.data.slices().iter().rev() {
                let mid = 0.5 * (slice.start_angle + slice.end_angle);
                let offset = if slice.explode { 0.08_f64 } else { 0.0 };
                let cx = offset * f64::from(mid.cos());
                let cy = offset * f64::from(mid.sin());
                let dx = point[0] - cx;
                let dy = point[1] - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let inner = f64::from(pie.inner_ratio);
                if dist < inner || dist > 1.0 {
                    continue;
                }
                let ang = dy.atan2(dx) as f32;
                let tau = std::f32::consts::TAU;
                let rel = (ang - slice.start_angle).rem_euclid(tau);
                let span = (slice.end_angle - slice.start_angle).max(0.0);
                if rel <= span {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::PieSlice,
                        slice.index,
                        point,
                    );
                    hit.scalar_value = Some(slice.value);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::WordCloud(wc) => {
            for (idx, w) in wc.data.words().iter().enumerate().rev() {
                if (point[0] - w.center[0]).abs() <= w.half_size[0]
                    && (point[1] - w.center[1]).abs() <= w.half_size[1]
                {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::WordCloudItem,
                        idx,
                        point,
                    );
                    hit.scalar_value = Some(w.weight);
                    return Some(hit);
                }
            }
            None
        }
        PlotLayer::BubbleCloud(bc) => {
            for (idx, b) in bc.data.bubbles().iter().enumerate().rev() {
                let dx = point[0] - b.center[0];
                let dy = point[1] - b.center[1];
                if dx * dx + dy * dy <= b.radius * b.radius {
                    let mut hit = base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::BubbleCloudItem,
                        idx,
                        point,
                    );
                    hit.scalar_value = Some(b.value);
                    hit.rgba = Some(bc.palette.map_or([0, 0, 0, 255], |p| {
                        let c = p[b.group % p.len()];
                        [c.r(), c.g(), c.b(), c.a()]
                    }));
                    return Some(hit);
                }
            }
            None
        }
        other => {
            let _ = Color32::TRANSPARENT;
            super::stage13::pick_stage13_region(
                other,
                layer_id,
                layer_index,
                y_axis,
                point,
                transform,
            )
            .or_else(|| {
                super::stage14::pick_stage14_region(
                    other,
                    layer_id,
                    layer_index,
                    y_axis,
                    point,
                    transform,
                )
            })
            .or_else(|| {
                super::stage16::pick_stage16_region(
                    other,
                    layer_id,
                    layer_index,
                    y_axis,
                    point,
                    transform,
                )
            })
        }
    }
}
