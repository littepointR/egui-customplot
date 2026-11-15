use egui::{Color32, Painter, Pos2, Shape, Stroke};

use super::{line::prepare_line_iter, mesh::stroke_valid};
use crate::{
    Comet, LineInterpolation, PlotDiagnostics, PlotPoint, PlotTransform, model::ResolvedSeriesStyle,
};

pub(super) fn paint_comet(
    painter: &Painter,
    comet: Comet<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    let pts = comet.data.points();
    if pts.is_empty() {
        return;
    }
    if !style.stroke.width.is_finite() || style.stroke.width < 0.0 {
        diagnostics.numeric_rejections += 1;
        return;
    }

    let max_idx = (pts.len() - 1) as f64;
    let progress = comet.playback.progress_points().clamp(0.0, max_idx);
    let whole = (progress.floor() as usize).min(pts.len() - 1);
    let frac = progress - whole as f64;
    let head_pt = if frac > 0.0
        && whole + 1 < pts.len()
        && transform.is_valid_point(pts[whole])
        && transform.is_valid_point(pts[whole + 1])
    {
        let a = pts[whole];
        let b = pts[whole + 1];
        Some([a[0] + frac * (b[0] - a[0]), a[1] + frac * (b[1] - a[1])])
    } else {
        None
    };

    diagnostics.flow.visited_items += whole + 1;
    let base_color = style.stroke.color;
    let base_width = style.stroke.width.max(1.0);

    if comet.show_full_track && pts.len() >= 2 {
        let track_stroke = Stroke::new(
            (base_width * 0.65).max(1.0),
            base_color.gamma_multiply(0.25),
        );
        paint_polyline(
            painter,
            pts.iter().copied(),
            transform,
            track_stroke,
            diagnostics,
        );
    }

    let tail_pts = (max_idx * comet.playback.tail_fraction_value()).max(1.0);
    let tail_start = ((progress - tail_pts).floor().max(0.0) as usize).min(whole);

    if tail_start >= 1 {
        let body_color = comet
            .body_color
            .unwrap_or_else(|| base_color.gamma_multiply(0.55));
        let body_stroke = Stroke::new(base_width, body_color);
        paint_polyline(
            painter,
            pts[..=tail_start].iter().copied(),
            transform,
            body_stroke,
            diagnostics,
        );
    }

    let tail_color = comet.tail_color.unwrap_or(base_color);
    let tail_stroke = Stroke::new((base_width * 1.4).max(1.5), tail_color);
    paint_polyline(
        painter,
        pts[tail_start..=whole].iter().copied().chain(head_pt),
        transform,
        tail_stroke,
        diagnostics,
    );

    let head = head_pt.unwrap_or(pts[whole]);
    if !comet.head_radius.is_finite() || comet.head_radius < 0.0 {
        diagnostics.numeric_rejections += 1;
    } else if comet.head_radius > 0.0 && transform.is_valid_point(head) {
        let pos = transform.position_from_point(head);
        let head_color = comet.head_color.unwrap_or(base_color);
        if pos.is_finite()
            && head_color != Color32::TRANSPARENT
            && transform.rect().expand(comet.head_radius).contains(pos)
        {
            diagnostics.flow.output_heads += 1;
            painter.circle_filled(pos, comet.head_radius, head_color);
        }
    }
}

fn paint_polyline(
    painter: &Painter,
    points: impl IntoIterator<Item = PlotPoint>,
    transform: PlotTransform,
    stroke: Stroke,
    diagnostics: &mut PlotDiagnostics,
) {
    if !stroke_valid(stroke) {
        return;
    }
    let prepared = prepare_line_iter(points, transform, LineInterpolation::Linear);
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    diagnostics.flow.output_paths += prepared.paths.len();
    for path in prepared.paths {
        diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        painter.add(Shape::line(path, stroke));
    }
}
