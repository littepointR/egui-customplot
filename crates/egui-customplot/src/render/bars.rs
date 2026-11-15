use egui::{Color32, Painter, Stroke};

use crate::{
    Bars, PlotDiagnostics, PlotTransform,
    axis::{clipped_axis_segment, clipped_data_rect, rect_edges, valid_extent},
};

pub(crate) fn paint_bars(
    painter: &Painter,
    bars: Bars<'_>,
    transform: PlotTransform,
    color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let border = bars.border;
    let valid_border = border.width.is_finite() && border.width >= 0.0;
    if !valid_border {
        diagnostics.numeric_rejections += 1;
    }
    let border = if valid_border { border } else { Stroke::NONE };
    if color == Color32::TRANSPARENT && border.is_empty() {
        return;
    }
    for &bar in bars.data.bars() {
        diagnostics.visited_bars += 1;
        let Some((lo, hi)) = bar.extent(bars.orientation) else {
            continue;
        };
        if lo[0] == hi[0] || lo[1] == hi[1] || !valid_extent(lo, hi, transform) {
            continue;
        }
        let mut painted = false;
        if color != Color32::TRANSPARENT
            && let Some(rect) = clipped_data_rect(lo, hi, transform)
        {
            painter.rect_filled(rect, 0.0, color);
            painted = true;
        }
        if !border.is_empty() {
            for (a, b, axis) in rect_edges(lo, hi) {
                if let Some(pair) = clipped_axis_segment(a, b, axis, transform) {
                    painter.line_segment(pair, border);
                    painted = true;
                }
            }
        }
        diagnostics.output_bars += usize::from(painted);
    }
}
