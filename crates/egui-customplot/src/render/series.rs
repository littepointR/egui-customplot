use std::time::Instant;

use egui::{Color32, Pos2, Stroke, Ui};

use super::{line::prepare_line, markers::prepare_markers, pattern::apply_pattern};
use crate::{PlotDiagnostics, PlotState, PlotTransform, Series};

pub(crate) fn paint_series(
    ui: &Ui,
    state: &PlotState,
    series: &[Series<'_>],
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let painter = ui.painter().with_clip_rect(transform.rect());
    let start = Instant::now();
    for (index, s) in series.iter().enumerate() {
        if !state.series_visible(s.id(index)) {
            continue;
        }
        let color = s.color(index);
        let markers = match *s {
            Series::Line(line) => {
                let stroke = line.stroke.unwrap_or(Stroke::new(1.5, color));
                if stroke.width.is_finite()
                    && stroke.width > 0.0
                    && stroke.color != Color32::TRANSPARENT
                {
                    if line.pattern.is_valid() {
                        let prepared = prepare_line(line.data.points(), transform);
                        diagnostics.visited_segments += prepared.visited_segments;
                        diagnostics.numeric_rejections += prepared.numeric_rejections;
                        let patterned = apply_pattern(prepared.paths, line.pattern);
                        diagnostics.pattern_truncated |= patterned.truncated;
                        diagnostics.output_paths += patterned.paths.len();
                        for path in patterned.paths {
                            diagnostics.output_points += path.len();
                            diagnostics.geometry_capacity_bytes +=
                                path.capacity() * std::mem::size_of::<Pos2>();
                            painter.add(egui::Shape::line(path, stroke));
                        }
                    } else {
                        diagnostics.numeric_rejections += 1;
                    }
                }
                line.markers.map(|style| (style, None, None))
            }
            Series::Scatter(scatter) => Some((scatter.style, scatter.colors, scatter.radii)),
        };
        if let Some((style, colors, radii)) = markers {
            let prepared = prepare_markers(
                s.data().points(),
                transform,
                style,
                color,
                colors,
                radii,
                ui.ctx().pixels_per_point(),
            );
            diagnostics.visited_marker_points += prepared.visited_points;
            diagnostics.output_markers += prepared.output_markers;
            diagnostics.numeric_rejections += prepared.numeric_rejections;
            diagnostics.geometry_capacity_bytes += prepared.mesh.vertices.capacity()
                * std::mem::size_of::<egui::epaint::Vertex>()
                + prepared.mesh.indices.capacity() * std::mem::size_of::<u32>();
            if !prepared.mesh.is_empty() {
                painter.add(egui::Shape::mesh(prepared.mesh));
            }
        }
    }
    diagnostics.prepare_duration = start.elapsed();
}
