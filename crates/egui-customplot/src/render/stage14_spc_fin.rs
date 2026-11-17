use egui::{Align2, Color32, FontId, Painter, Shape, Stroke, epaint};

use super::{line::prepare_line, markers::prepare_markers, pattern::apply_pattern};
use crate::{
    Candlestick, ControlChart, LineInterpolation, LinePattern, MarkerShape, MarkerStyle, OhlcBars,
    PlotDiagnostics, PlotPoint, PlotTransform, Waterfall2D, WaterfallKind,
    axis::{clipped_axis_segment, clipped_data_rect, rect_edges, valid_extent},
    model::ResolvedSeriesStyle,
};

pub(super) fn paint_control_chart(
    painter: &Painter,
    chart: ControlChart<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let pts = chart.data.points();
    if pts.is_empty() {
        return;
    }
    let ucl_pts: Vec<PlotPoint> = pts.iter().map(|p| [p.x, p.ucl]).collect();
    let lcl_pts: Vec<PlotPoint> = pts.iter().map(|p| [p.x, p.lcl]).collect();
    let cl_pts: Vec<PlotPoint> = pts.iter().map(|p| [p.x, p.cl]).collect();
    let val_pts: Vec<PlotPoint> = pts.iter().map(|p| [p.x, p.value]).collect();

    let dashed = LinePattern::Dashed {
        length: 6.0,
        gap: 4.0,
    };
    for (line_pts, stroke) in [
        (&ucl_pts, Stroke::new(1.3, chart.limit_color)),
        (&lcl_pts, Stroke::new(1.3, chart.limit_color)),
    ] {
        let prep = prepare_line(line_pts, transform, LineInterpolation::StepAfter);
        for path in apply_pattern(prep.paths, dashed).paths {
            painter.add(epaint::PathShape::line(path, stroke));
        }
    }
    let cl_prep = prepare_line(&cl_pts, transform, LineInterpolation::Linear);
    let cl_stroke = Stroke::new(1.3, chart.center_color);
    for path in cl_prep.paths {
        painter.add(epaint::PathShape::line(path, cl_stroke));
    }

    let val_prep = prepare_line(&val_pts, transform, LineInterpolation::Linear);
    diagnostics.visited_segments += val_prep.visited_segments;
    for path in apply_pattern(val_prep.paths, style.pattern).paths {
        painter.add(epaint::PathShape::line(path, style.stroke));
    }

    let ok_pts: Vec<PlotPoint> = pts
        .iter()
        .filter(|p| p.violation.is_none())
        .map(|p| [p.x, p.value])
        .collect();
    let bad_pts: Vec<PlotPoint> = pts
        .iter()
        .filter(|p| p.violation.is_some())
        .map(|p| [p.x, p.value])
        .collect();

    let ok_markers = prepare_markers(
        &ok_pts,
        transform,
        style.marker,
        style.stroke.color,
        None,
        None,
        pixels_per_point,
    );
    diagnostics.visited_marker_points += ok_markers.visited_points;
    if !ok_markers.mesh.is_empty() {
        diagnostics.output_markers += ok_markers.output_markers;
        painter.add(Shape::mesh(ok_markers.mesh));
    }
    if !bad_pts.is_empty() {
        let bad_style = MarkerStyle {
            shape: MarkerShape::Diamond,
            radius: (style.marker.radius * 1.45).max(5.5),
        };
        let bad_markers = prepare_markers(
            &bad_pts,
            transform,
            bad_style,
            chart.violation_color,
            None,
            None,
            pixels_per_point,
        );
        diagnostics.visited_marker_points += bad_markers.visited_points;
        if !bad_markers.mesh.is_empty() {
            diagnostics.output_markers += bad_markers.output_markers;
            painter.add(Shape::mesh(bad_markers.mesh));
        }
    }

    if chart.show_labels
        && let Some(last) = pts.last()
    {
        let font = FontId::proportional(10.0);
        for (y, tag, col) in [
            (last.ucl, "UCL", chart.limit_color),
            (last.cl, "CL", chart.center_color),
            (last.lcl, "LCL", chart.limit_color),
        ] {
            if transform.is_valid_point([last.x, y]) {
                let p = transform.position_from_point([last.x, y]);
                if transform.rect().contains(p) {
                    painter.text(
                        p + egui::vec2(4.0, -4.0),
                        Align2::LEFT_BOTTOM,
                        tag,
                        font.clone(),
                        col,
                    );
                }
            }
        }
    }
}

pub(super) fn paint_candlestick(
    painter: &Painter,
    candle: Candlestick<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    for &b in candle.data.bars() {
        diagnostics.visited_bars += 1;
        let up = b.is_up();
        let color = if up {
            candle.up_color
        } else {
            candle.down_color
        };
        let body_lo = b.open.min(b.close);
        let body_hi = b.open.max(b.close);
        let x0 = b.x - 0.5 * b.width;
        let x1 = b.x + 0.5 * b.width;
        let wick_stroke = Stroke::new(candle.wick_width, color);

        for (y_a, y_b) in [(b.low, body_lo), (body_hi, b.high)] {
            if y_b > y_a
                && let Some(pair) = clipped_axis_segment([b.x, y_a], [b.x, y_b], 1, transform)
            {
                painter.line_segment(pair, wick_stroke);
            }
        }
        if (body_hi - body_lo).abs() <= 1e-12 {
            if let Some(pair) = clipped_axis_segment([x0, body_lo], [x1, body_lo], 0, transform) {
                painter.line_segment(pair, Stroke::new(candle.wick_width.max(1.5), color));
                diagnostics.output_bars += 1;
            }
            continue;
        }
        let lo = [x0, body_lo];
        let hi = [x1, body_hi];
        if !valid_extent(lo, hi, transform) {
            continue;
        }
        if !(up && candle.hollow_up)
            && let Some(rect) = clipped_data_rect(lo, hi, transform)
        {
            painter.rect_filled(rect, 0.0, color);
        }
        let border = Stroke::new(candle.wick_width.max(1.1), color);
        for (a, c, axis) in rect_edges(lo, hi) {
            if let Some(pair) = clipped_axis_segment(a, c, axis, transform) {
                painter.line_segment(pair, border);
            }
        }
        diagnostics.output_bars += 1;
    }
}

pub(super) fn paint_ohlc_bars(
    painter: &Painter,
    ohlc: OhlcBars<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    for &b in ohlc.data.bars() {
        diagnostics.visited_bars += 1;
        let color = if b.is_up() {
            ohlc.up_color
        } else {
            ohlc.down_color
        };
        let stroke = Stroke::new(ohlc.width, color);
        let mut drawn = false;
        if let Some(pair) = clipped_axis_segment([b.x, b.low], [b.x, b.high], 1, transform) {
            painter.line_segment(pair, stroke);
            drawn = true;
        }
        if let Some(pair) =
            clipped_axis_segment([b.x - 0.5 * b.width, b.open], [b.x, b.open], 0, transform)
        {
            painter.line_segment(pair, stroke);
            drawn = true;
        }
        if let Some(pair) =
            clipped_axis_segment([b.x, b.close], [b.x + 0.5 * b.width, b.close], 0, transform)
        {
            painter.line_segment(pair, stroke);
            drawn = true;
        }
        diagnostics.output_bars += usize::from(drawn);
    }
}

pub(super) fn paint_waterfall2d(
    painter: &Painter,
    wf: Waterfall2D<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let cols = wf.data.columns();
    let conn_stroke = Stroke::new(1.0, text_color.gamma_multiply(0.45));
    let conn_pattern = LinePattern::Dashed {
        length: 4.0,
        gap: 3.0,
    };
    let font = FontId::proportional(9.5);
    for (i, c) in cols.iter().enumerate() {
        diagnostics.visited_bars += 1;
        let color = match c.kind {
            WaterfallKind::Positive => wf.positive_color,
            WaterfallKind::Negative => wf.negative_color,
            WaterfallKind::Total => wf.total_color,
        };
        let y0 = c.start.min(c.end);
        let y1 = c.start.max(c.end);
        let x0 = c.x - 0.5 * c.width;
        let x1 = c.x + 0.5 * c.width;
        if y1 > y0
            && valid_extent([x0, y0], [x1, y1], transform)
            && let Some(rect) = clipped_data_rect([x0, y0], [x1, y1], transform)
        {
            painter.rect_filled(rect, 0.0, color);
            diagnostics.output_bars += 1;
        }
        if wf.show_connectors && i + 1 < cols.len() {
            let next = &cols[i + 1];
            let bridge_y = c.end;
            if let Some(pair) = clipped_axis_segment(
                [x1, bridge_y],
                [next.x - 0.5 * next.width, bridge_y],
                0,
                transform,
            ) {
                for seg in apply_pattern(vec![pair.to_vec()], conn_pattern).paths {
                    painter.add(epaint::PathShape::line(seg, conn_stroke));
                }
            }
        }
        if wf.show_values && cols.len() <= 24 && transform.is_valid_point([c.x, c.end]) {
            let pos = transform.position_from_point([c.x, y1]);
            if transform.rect().contains(pos) {
                let txt = if matches!(c.kind, WaterfallKind::Total) {
                    format!("{:.1}", c.end)
                } else {
                    format!("{:+.1}", c.delta)
                };
                painter.text(
                    pos + egui::vec2(0.0, -3.0),
                    Align2::CENTER_BOTTOM,
                    txt,
                    font.clone(),
                    text_color,
                );
            }
        }
    }
}
