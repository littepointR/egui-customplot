use egui::{Painter, Pos2, Rect, Stroke};

use crate::{
    ErrorBars, PlotDiagnostics, PlotTransform,
    axis::{clipped_axis_segment, valid_extent},
};

pub(crate) fn paint_error_bars(
    painter: &Painter,
    bars: ErrorBars<'_>,
    transform: PlotTransform,
    stroke: Stroke,
    diagnostics: &mut PlotDiagnostics,
) {
    if !stroke.width.is_finite() || stroke.width < 0.0 {
        diagnostics.numeric_rejections += 1;
        return;
    }
    if stroke.is_empty() {
        return;
    }
    let cap = if bars.cap_width.is_finite() && bars.cap_width >= 0.0 {
        bars.cap_width
    } else {
        diagnostics.numeric_rejections += 1;
        0.0
    };
    for &bar in bars.data.bars() {
        diagnostics.visited_error_bars += 1;
        let Some((lo, hi)) = bar.extent() else {
            continue;
        };
        if !valid_extent(lo, hi, transform) {
            continue;
        }
        for axis in 0..2 {
            if lo[axis] == hi[axis] {
                continue;
            }
            let mut a = bar.point;
            let mut b = bar.point;
            a[axis] = lo[axis];
            b[axis] = hi[axis];
            if let Some(pair) = clipped_axis_segment(a, b, axis, transform) {
                submit(painter, pair, stroke, diagnostics);
            }
            if cap > 0.0 {
                for endpoint in [a, b] {
                    let view = transform.bounds();
                    // Caps belong only to the original endpoints, never to a
                    // stem's intersections with the view boundary.
                    if endpoint[axis] < view.min()[axis] || endpoint[axis] > view.max()[axis] {
                        continue;
                    }
                    let pos = transform.position_from_point(endpoint);
                    if let Some(pair) = clipped_cap(pos, axis, cap, transform.rect()) {
                        submit(painter, pair, stroke, diagnostics);
                    }
                }
            }
        }
    }
}

fn clipped_cap(position: Pos2, stem_axis: usize, width: f32, rect: Rect) -> Option<[Pos2; 2]> {
    if !position.is_finite() {
        return None;
    }
    let mut a = position;
    let mut b = position;
    let perpendicular = 1 - stem_axis;
    a[perpendicular] = (a[perpendicular] - width * 0.5).max(rect.min[perpendicular]);
    b[perpendicular] = (b[perpendicular] + width * 0.5).min(rect.max[perpendicular]);
    (a[perpendicular] < b[perpendicular] && rect.contains(a) && rect.contains(b)).then_some([a, b])
}

fn submit(painter: &Painter, pair: [Pos2; 2], stroke: Stroke, diagnostics: &mut PlotDiagnostics) {
    painter.line_segment(pair, stroke);
    diagnostics.output_error_bar_segments += 1;
}
