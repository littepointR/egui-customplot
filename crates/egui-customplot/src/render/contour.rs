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
            let pill_bg = if plot_style.background == Color32::TRANSPARENT {
                Color32::from_rgba_unmultiplied(248, 249, 252, 225)
            } else {
                plot_style.background.gamma_multiply(0.92)
            };
            let bg_l = color_luma(pill_bg);
            let text_fg = if (color_luma(color) - bg_l).abs() >= 95.0 {
                color
            } else if (color_luma(plot_style.text_color) - bg_l).abs() >= 95.0 {
                plot_style.text_color
            } else if bg_l > 128.0 {
                Color32::from_gray(24)
            } else {
                Color32::from_gray(242)
            };
            let galley = painter.layout_no_wrap(text, font.clone(), text_fg);
            let rect = Align2::CENTER_CENTER.anchor_size(pos, galley.size());
            let pill_rect = rect.expand2(egui::vec2(3.5, 1.5));
            painter.rect_filled(pill_rect, 2.5, pill_bg);
            painter.rect_stroke(
                pill_rect,
                2.5,
                Stroke::new(0.8, text_fg.gamma_multiply(0.25)),
                egui::StrokeKind::Inside,
            );
            painter.galley(rect.min, galley, text_fg);
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

#[inline]
fn color_luma(c: Color32) -> f32 {
    0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
}
