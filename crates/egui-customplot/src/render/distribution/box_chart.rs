use egui::{Color32, Painter, Pos2, Shape, Stroke, StrokeKind};

use super::orient;
use crate::{
    BarOrientation, BoxChart, BoxSummary, MarkerShape, MarkerStyle, PlotDiagnostics, PlotTransform,
    axis::{clipped_axis_segment, clipped_data_rect, valid_extent},
    model::ResolvedSeriesStyle,
    render::{markers::prepare_markers, mesh::MeshBatch},
};

pub(in crate::render) fn paint_box_chart(
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
    let o = box_layer.orientation;

    for b in box_layer.data.boxes() {
        paint_single_box(
            painter,
            &mut batch,
            b,
            b.width,
            o,
            fill_color,
            line_stroke,
            box_layer.notched,
            transform,
        );
        diagnostics.stats.output_boxes += 1;
        if box_layer.show_mean {
            means.push(orient(o, b.position, b.mean));
        }
        if box_layer.show_outliers {
            for &out in b.outliers.iter() {
                outliers.push(orient(o, b.position, out));
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

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_single_box(
    painter: &Painter,
    batch: &mut MeshBatch,
    b: &BoxSummary,
    width: f64,
    orientation: BarOrientation,
    fill: Color32,
    stroke: Stroke,
    notched: bool,
    transform: PlotTransform,
) {
    let hw = width * 0.5;
    let lo = orient(orientation, b.position - hw, b.q1);
    let hi = orient(orientation, b.position + hw, b.q3);
    if !valid_extent(lo, hi, transform) {
        return;
    }
    let val_axis = if orientation == BarOrientation::Vertical {
        1
    } else {
        0
    };
    let pos_axis = 1 - val_axis;
    for (w0, w1) in [(b.whisker_low, b.q1), (b.q3, b.whisker_high)] {
        if let Some(seg) = clipped_axis_segment(
            orient(orientation, b.position, w0),
            orient(orientation, b.position, w1),
            val_axis,
            transform,
        ) {
            painter.line_segment(seg, stroke);
        }
    }
    let cap_hw = hw * 0.55;
    for cap_y in [b.whisker_low, b.whisker_high] {
        if let Some(seg) = clipped_axis_segment(
            orient(orientation, b.position - cap_hw, cap_y),
            orient(orientation, b.position + cap_hw, cap_y),
            pos_axis,
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
            orient(orientation, b.position - hw, b.q1),
            orient(orientation, b.position + hw, b.q1),
            orient(orientation, b.position + hw, nl),
            orient(orientation, b.position + pinch, b.median),
            orient(orientation, b.position + hw, nh),
            orient(orientation, b.position + hw, b.q3),
            orient(orientation, b.position - hw, b.q3),
            orient(orientation, b.position - hw, nh),
            orient(orientation, b.position - pinch, b.median),
            orient(orientation, b.position - hw, nl),
        ];
        if pts.iter().all(|p| transform.is_valid_point(*p)) {
            let center_n = transform.normalized(orient(orientation, b.position, b.median));
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
            orient(orientation, b.position - pinch, b.median),
            orient(orientation, b.position + pinch, b.median),
            pos_axis,
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
            orient(orientation, b.position - hw, b.median),
            orient(orientation, b.position + hw, b.median),
            pos_axis,
            transform,
        ) {
            painter.line_segment(
                seg,
                Stroke::new((stroke.width * 1.4).max(1.8), stroke.color),
            );
        }
    }
}
