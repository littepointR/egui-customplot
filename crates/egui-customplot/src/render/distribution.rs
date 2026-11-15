use egui::{Color32, Painter, Pos2, Shape, Stroke, StrokeKind};

use super::{markers::prepare_markers, mesh::MeshBatch};
use crate::{
    BoxChart, BoxSummary, MarkerShape, MarkerStyle, PlotDiagnostics, PlotPoint, PlotTransform,
    Raincloud, ViolinHalf, ViolinPlot,
    axis::{clipped_axis_segment, clipped_data_rect, valid_extent},
    model::ResolvedSeriesStyle,
};

pub(super) fn paint_box_chart(
    painter: &Painter,
    box_layer: BoxChart<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let line_color = resolved.stroke.color;
    let fill_color = line_color.gamma_multiply(box_layer.fill_alpha);
    let line_stroke = Stroke::new(resolved.stroke.width.clamp(0.75, 4.0), line_color);
    let mut batch = MeshBatch::default();
    let mut outliers = Vec::new();
    let mut means = Vec::new();

    for b in box_layer.data.boxes() {
        paint_single_box(
            painter,
            &mut batch,
            b,
            b.width,
            fill_color,
            line_stroke,
            box_layer.notched,
            transform,
        );
        diagnostics.stats.output_boxes += 1;
        if box_layer.show_mean {
            means.push([b.position, b.mean]);
        }
        if box_layer.show_outliers {
            for &out in b.outliers.iter() {
                outliers.push([b.position, out]);
            }
        }
    }
    let stats = batch.finish(painter);
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;

    if !outliers.is_empty() {
        let prepared = prepare_markers(
            &outliers,
            transform,
            MarkerStyle {
                shape: MarkerShape::Circle,
                radius: 3.2,
            },
            line_color,
            None,
            None,
            pixels_per_point,
        );
        if !prepared.mesh.is_empty() {
            painter.add(Shape::mesh(prepared.mesh));
        }
    }
    if !means.is_empty() {
        let prepared = prepare_markers(
            &means,
            transform,
            MarkerStyle {
                shape: MarkerShape::Diamond,
                radius: 4.0,
            },
            Color32::WHITE,
            None,
            None,
            pixels_per_point,
        );
        if !prepared.mesh.is_empty() {
            painter.add(Shape::mesh(prepared.mesh));
        }
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
            if transform.is_valid_point([xl, y]) && transform.is_valid_point([xr, y]) {
                let nl = transform.normalized([xl, y]);
                let nr = transform.normalized([xr, y]);
                left_pts.push(nl);
                right_pts.push(nr);
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
                color.gamma_multiply(0.65),
                Stroke::new(1.2, color),
                false,
                transform,
            );
        } else if violin.show_median {
            let w = hw[hw.len() / 2].max(0.05);
            if let Some(seg) = clipped_axis_segment(
                [pos - w * 0.5, entry.summary.median],
                [pos + w * 0.5, entry.summary.median],
                0,
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
                let p0 = [pos, y];
                let p1 = [pos + w, y];
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
            fill,
            outline,
            false,
            transform,
        );
        for (&dx, &dy) in entry.drop_x.iter().zip(entry.drop_y.iter()) {
            drops.push([dx, dy]);
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

#[allow(clippy::too_many_arguments)]
fn paint_single_box(
    painter: &Painter,
    batch: &mut MeshBatch,
    b: &BoxSummary,
    width: f64,
    fill: Color32,
    stroke: Stroke,
    notched: bool,
    transform: PlotTransform,
) {
    let hw = width * 0.5;
    let lo = [b.position - hw, b.q1];
    let hi = [b.position + hw, b.q3];
    if !valid_extent(lo, hi, transform) {
        return;
    }
    // Lower and upper whiskers
    for (w0, w1) in [(b.whisker_low, b.q1), (b.q3, b.whisker_high)] {
        if let Some(seg) = clipped_axis_segment([b.position, w0], [b.position, w1], 1, transform) {
            painter.line_segment(seg, stroke);
        }
    }
    let cap_hw = hw * 0.55;
    for cap_y in [b.whisker_low, b.whisker_high] {
        if let Some(seg) = clipped_axis_segment(
            [b.position - cap_hw, cap_y],
            [b.position + cap_hw, cap_y],
            0,
            transform,
        ) {
            painter.line_segment(seg, stroke);
        }
    }
    if notched && b.q3 > b.q1 {
        let pinch = hw * 0.48;
        let nl = b.notch_low.clamp(b.q1, b.median);
        let nh = b.notch_high.clamp(b.median, b.q3);
        let pts = [
            [b.position - hw, b.q1],
            [b.position + hw, b.q1],
            [b.position + hw, nl],
            [b.position + pinch, b.median],
            [b.position + hw, nh],
            [b.position + hw, b.q3],
            [b.position - hw, b.q3],
            [b.position - hw, nh],
            [b.position - pinch, b.median],
            [b.position - hw, nl],
        ];
        if pts.iter().all(|p| transform.is_valid_point(*p)) {
            let center_n = transform.normalized([b.position, b.median]);
            let norms: Vec<[f64; 2]> = pts.iter().map(|p| transform.normalized(*p)).collect();
            for i in 0..norms.len() {
                let next = (i + 1) % norms.len();
                batch.add_flat_triangle(
                    painter,
                    [center_n, norms[i], norms[next]],
                    fill,
                    transform,
                );
            }
            let mut screen: Vec<Pos2> = norms
                .iter()
                .map(|n| transform.position_from_normalized(*n))
                .collect();
            screen.push(screen[0]);
            painter.add(Shape::line(screen, stroke));
        }
        if let Some(seg) = clipped_axis_segment(
            [b.position - pinch, b.median],
            [b.position + pinch, b.median],
            0,
            transform,
        ) {
            painter.line_segment(
                seg,
                Stroke::new((stroke.width * 1.4).max(1.8), stroke.color),
            );
        }
    } else if let Some(rect) = clipped_data_rect(lo, hi, transform) {
        painter.rect_filled(rect, 0.0, fill);
        painter.rect_stroke(rect, 0.0, stroke, StrokeKind::Inside);
        if let Some(seg) = clipped_axis_segment(
            [b.position - hw, b.median],
            [b.position + hw, b.median],
            0,
            transform,
        ) {
            painter.line_segment(
                seg,
                Stroke::new((stroke.width * 1.4).max(1.8), stroke.color),
            );
        }
    }
}
