use egui::{Align2, Color32, Painter, Pos2, Shape, Stroke, Ui};

use super::{
    line::prepare_line,
    mesh::{MeshBatch, stroke_valid},
    pattern::apply_pattern,
};
use crate::{
    Contours, FilledContours, LineInterpolation, PlotDiagnostics, PlotTransform,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

pub(super) fn paint_contours(
    ui: &Ui,
    painter: &Painter,
    contours: Contours<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    plot_style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut PlotDiagnostics,
) {
    let stroke = resolved.stroke;
    if !stroke.width.is_finite() || stroke.width <= 0.0 || !resolved.pattern.is_valid() {
        if !resolved.pattern.is_valid() {
            diagnostics.numeric_rejections += 1;
        }
        return;
    }
    let font = egui::TextStyle::Small.resolve(ui.style());
    for line in contours.data.lines() {
        diagnostics.contours.visited_items += 1;
        let color = contours
            .color_scale
            .map_or(Some(stroke.color), |scale| scale.color(line.level));
        let Some(color) = color else {
            continue;
        };
        if color == Color32::TRANSPARENT {
            continue;
        }
        let prepared = prepare_line(&line.points, transform, LineInterpolation::Linear);
        diagnostics.visited_segments += prepared.visited_segments;
        diagnostics.numeric_rejections += prepared.numeric_rejections;
        let mut label_anchor: Option<(Pos2, f32)> = None;
        if contours.show_labels {
            for path in &prepared.paths {
                let len = polyline_length(path);
                if len >= 36.0
                    && label_anchor.is_none_or(|(_, best)| len > best)
                    && let Some(mid) = polyline_midpoint(path, len * 0.5)
                {
                    label_anchor = Some((mid, len));
                }
            }
        }
        let patterned = apply_pattern(prepared.paths, resolved.pattern);
        diagnostics.pattern_truncated |= patterned.truncated;
        let line_stroke = Stroke::new(stroke.width, color);
        for path in patterned.paths {
            diagnostics.contours.output_paths += 1;
            diagnostics.output_points += path.len();
            diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
            painter.add(Shape::line(path, line_stroke));
        }
        if let Some((pos, _)) = label_anchor {
            let text = format_level(line.level);
            let galley = painter.layout_no_wrap(text, font.clone(), color);
            let rect = Align2::CENTER_CENTER.anchor_size(pos, galley.size());
            if plot_style.background != Color32::TRANSPARENT {
                painter.rect_filled(
                    rect.expand2(egui::vec2(2.0, 1.0)),
                    2.0,
                    plot_style.background.gamma_multiply(0.85),
                );
            }
            painter.galley(rect.min, galley, color);
            diagnostics.contours.output_labels += 1;
        }
    }
}

pub(super) fn paint_filled_contours(
    painter: &Painter,
    fc: FilledContours<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let mut batch = MeshBatch::default();
    for band in fc.data.bands() {
        diagnostics.contours.visited_items += 1;
        let Some(color) = fc.colors.color(band.lower) else {
            continue;
        };
        if color == Color32::TRANSPARENT {
            continue;
        }
        for &[i0, i1, i2] in &band.triangles {
            let v0 = band.vertices[i0 as usize];
            let v1 = band.vertices[i1 as usize];
            let v2 = band.vertices[i2 as usize];
            if !transform.is_valid_point(v0)
                || !transform.is_valid_point(v1)
                || !transform.is_valid_point(v2)
            {
                continue;
            }
            let tri = [
                transform.normalized(v0),
                transform.normalized(v1),
                transform.normalized(v2),
            ];
            batch.add_flat_triangle(painter, tri, color, transform);
        }
    }
    let stats = batch.finish(painter);
    diagnostics.contours.output_meshes += stats.meshes;
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;

    if let Some(lines) = fc.border_lines
        && stroke_valid(fc.border)
    {
        for line in lines.lines() {
            let prepared = prepare_line(&line.points, transform, LineInterpolation::Linear);
            diagnostics.visited_segments += prepared.visited_segments;
            diagnostics.numeric_rejections += prepared.numeric_rejections;
            for path in prepared.paths {
                diagnostics.contours.output_paths += 1;
                diagnostics.output_points += path.len();
                diagnostics.geometry_capacity_bytes +=
                    path.capacity() * std::mem::size_of::<Pos2>();
                painter.add(Shape::line(path, fc.border));
            }
        }
    }
}

fn polyline_length(path: &[Pos2]) -> f32 {
    path.windows(2).map(|w| w[0].distance(w[1])).sum()
}

fn polyline_midpoint(path: &[Pos2], mut target: f32) -> Option<Pos2> {
    for w in path.windows(2) {
        let seg = w[0].distance(w[1]);
        if seg > 0.0 && target <= seg {
            return Some(w[0].lerp(w[1], target / seg));
        }
        target -= seg;
    }
    path.first().copied()
}

fn format_level(level: f64) -> String {
    if level == 0.0 {
        return "0".to_string();
    }
    if level.abs() >= 1e4 || level.abs() < 1e-3 {
        return format!("{level:.2e}");
    }
    let s = format!("{level:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
