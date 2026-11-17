mod box_chart;

use egui::{Color32, Painter, Pos2, Shape, Stroke};

pub(super) use box_chart::paint_box_chart;
use box_chart::paint_single_box;

use super::{markers::prepare_markers, mesh::MeshBatch};
use crate::{
    BarOrientation, MarkerShape, MarkerStyle, PlotDiagnostics, PlotPoint, PlotTransform, Raincloud,
    ViolinHalf, ViolinPlot, axis::clipped_axis_segment, model::ResolvedSeriesStyle,
};

#[inline]
pub(super) fn orient(o: BarOrientation, pos: f64, val: f64) -> PlotPoint {
    match o {
        BarOrientation::Vertical => [pos, val],
        BarOrientation::Horizontal => [val, pos],
    }
}

pub(super) fn paint_violin(
    painter: &Painter,
    violin: ViolinPlot<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    let color = resolved.stroke.color;
    let fill = color.gamma_multiply(violin.fill_alpha);
    let outline = Stroke::new(resolved.stroke.width.clamp(0.75, 3.5), color);
    let half = violin.data.options().half;
    let show_box = violin.show_box.unwrap_or(violin.data.options().show_box);
    let o = violin.orientation;
    let pos_axis = if o == BarOrientation::Vertical { 0 } else { 1 };
    let mut batch = MeshBatch::default();

    for entry in violin.data.entries() {
        let pos = entry.summary.position;
        let ys = &entry.y;
        let hw = &entry.half_widths;
        if ys.len() < 2 {
            continue;
        }
        let mut left_pts = Vec::with_capacity(ys.len());
        let mut right_pts = Vec::with_capacity(ys.len());
        for (&y, &w) in ys.iter().zip(hw.iter()) {
            let xl = match half {
                ViolinHalf::Both | ViolinHalf::Left => pos - w,
                ViolinHalf::Right => pos,
            };
            let xr = match half {
                ViolinHalf::Both | ViolinHalf::Right => pos + w,
                ViolinHalf::Left => pos,
            };
            let p_l = orient(o, xl, y);
            let p_r = orient(o, xr, y);
            if transform.is_valid_point(p_l) && transform.is_valid_point(p_r) {
                left_pts.push(transform.normalized(p_l));
                right_pts.push(transform.normalized(p_r));
            }
        }
        for i in 0..left_pts.len().saturating_sub(1) {
            batch.add_flat_triangle(
                painter,
                [left_pts[i], right_pts[i], right_pts[i + 1]],
                fill,
                transform,
            );
            batch.add_flat_triangle(
                painter,
                [left_pts[i], right_pts[i + 1], left_pts[i + 1]],
                fill,
                transform,
            );
        }
        let mut border: Vec<Pos2> = right_pts
            .iter()
            .map(|&n| transform.position_from_normalized(n))
            .collect();
        for &n in left_pts.iter().rev() {
            border.push(transform.position_from_normalized(n));
        }
        if let Some(&first) = border.first() {
            border.push(first);
            painter.add(Shape::line(border, outline));
        }
        if show_box {
            paint_single_box(
                painter,
                &mut batch,
                &entry.summary,
                entry.summary.width * 0.7,
                o,
                color.gamma_multiply(0.65),
                Stroke::new(1.2, color),
                false,
                transform,
            );
        } else if violin.show_median {
            let w = hw[hw.len() / 2].max(0.05);
            if let Some(seg) = clipped_axis_segment(
                orient(o, pos - w * 0.5, entry.summary.median),
                orient(o, pos + w * 0.5, entry.summary.median),
                pos_axis,
                transform,
            ) {
                painter.line_segment(seg, Stroke::new(2.0, Color32::WHITE));
            }
        }
        diagnostics.stats.output_violins += 1;
    }
    let stats = batch.finish(painter);
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}

pub(super) fn paint_raincloud(
    painter: &Painter,
    rain: Raincloud<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let color = resolved.stroke.color;
    let fill = color.gamma_multiply(rain.fill_alpha);
    let outline = Stroke::new(resolved.stroke.width.clamp(0.75, 3.0), color);
    let o = rain.orientation;
    let mut batch = MeshBatch::default();
    let mut drops: Vec<PlotPoint> = Vec::new();

    for entry in rain.data.entries() {
        let pos = entry.summary.position;
        let ys = &entry.y;
        let hw = &entry.half_widths;
        if ys.len() >= 2 {
            let mut spine = Vec::with_capacity(ys.len());
            let mut ridge = Vec::with_capacity(ys.len());
            for (&y, &w) in ys.iter().zip(hw.iter()) {
                let p0 = orient(o, pos, y);
                let p1 = orient(o, pos + w, y);
                if transform.is_valid_point(p0) && transform.is_valid_point(p1) {
                    spine.push(transform.normalized(p0));
                    ridge.push(transform.normalized(p1));
                }
            }
            for i in 0..spine.len().saturating_sub(1) {
                batch.add_flat_triangle(
                    painter,
                    [spine[i], ridge[i], ridge[i + 1]],
                    fill,
                    transform,
                );
                batch.add_flat_triangle(
                    painter,
                    [spine[i], ridge[i + 1], spine[i + 1]],
                    fill,
                    transform,
                );
            }
            let mut outline_pts: Vec<Pos2> = ridge
                .iter()
                .map(|&n| transform.position_from_normalized(n))
                .collect();
            for &n in spine.iter().rev() {
                outline_pts.push(transform.position_from_normalized(n));
            }
            if let Some(&first) = outline_pts.first() {
                outline_pts.push(first);
                painter.add(Shape::line(outline_pts, outline));
            }
        }
        paint_single_box(
            painter,
            &mut batch,
            &entry.summary,
            rain.data.width() * 0.14,
            o,
            fill,
            outline,
            false,
            transform,
        );
        for (&dx, &dy) in entry.drop_x.iter().zip(entry.drop_y.iter()) {
            drops.push(orient(o, dx, dy));
        }
        diagnostics.stats.output_violins += 1;
    }
    let stats = batch.finish(painter);
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;

    if !drops.is_empty() {
        let prepared = prepare_markers(
            &drops,
            transform,
            MarkerStyle {
                shape: MarkerShape::Circle,
                radius: rain.drop_radius,
            },
            color.gamma_multiply(0.85),
            None,
            None,
            pixels_per_point,
        );
        if !prepared.mesh.is_empty() {
            painter.add(Shape::mesh(prepared.mesh));
        }
    }
}
