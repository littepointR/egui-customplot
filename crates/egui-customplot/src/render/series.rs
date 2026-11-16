use std::time::Instant;

use egui::{Color32, Pos2, Ui};

use super::{line_query::prepare_series_line, markers::prepare_markers, pattern::apply_pattern};
use crate::{
    PlotCache, PlotDiagnostics, PlotLayer, PlotState, Series, axis::SeriesTransforms,
    model::ResolvedPlotStyle,
};

pub(crate) fn paint_series(
    ui: &Ui,
    state: &PlotState,
    series: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
    mut cache: Option<&mut PlotCache>,
) {
    let painter = ui.painter().with_clip_rect(transforms.left.rect());
    let start = Instant::now();
    let query_start = diagnostics.query_duration;
    for (index, s) in series.iter().enumerate() {
        if !state.series_visible(s.id(index)) {
            continue;
        }
        let transform = transforms.for_axis(s.y_axis());
        let resolved = s.resolved_style(index, style);
        let s = match *s {
            PlotLayer::Heatmap(heatmap) => {
                super::heatmap::paint_heatmap(&painter, heatmap, transform, diagnostics);
                continue;
            }
            PlotLayer::Bars(bars) => {
                super::bars::paint_bars(
                    &painter,
                    bars,
                    transform,
                    resolved.stroke.color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::ErrorBars(bars) => {
                super::error_bars::paint_error_bars(
                    &painter,
                    bars,
                    transform,
                    resolved.stroke,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Stems(stems) => {
                super::stems::paint_stems(
                    &painter,
                    stems,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Quiver(quiver) => {
                super::quiver::paint_quiver(&painter, quiver, transform, resolved, diagnostics);
                continue;
            }
            PlotLayer::Area(area) => {
                super::area::paint_area(
                    &painter,
                    area,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                    cache.as_deref_mut(),
                );
                continue;
            }
            PlotLayer::Fill(fill) => {
                super::polygon::paint_fill(
                    &painter,
                    fill,
                    transform,
                    resolved.stroke.color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Patch(patch) => {
                super::patch::paint_patch(
                    &painter,
                    patch,
                    transform,
                    resolved.stroke.color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Rectangles(rects) => {
                super::shapes::paint_rectangles(
                    &painter,
                    rects,
                    transform,
                    resolved.stroke.color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::PolyShape(poly) => {
                super::polygon::paint_polyshape(
                    &painter,
                    poly,
                    transform,
                    resolved.stroke.color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Contours(contours) => {
                super::contour::paint_contours(
                    ui,
                    &painter,
                    contours,
                    transform,
                    resolved,
                    style,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::FilledContours(fc) => {
                super::contour::paint_filled_contours(&painter, fc, transform, diagnostics);
                continue;
            }
            PlotLayer::Pcolor(pcolor) => {
                super::pcolor::paint_pcolor(&painter, pcolor, transform, diagnostics);
                continue;
            }
            PlotLayer::Image(image) => {
                super::image::paint_image(&painter, image, transform, diagnostics);
                continue;
            }
            PlotLayer::TriPlot(tri) => {
                super::triangulation::paint_triplot(
                    &painter,
                    tri,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Voronoi(vor) => {
                super::triangulation::paint_voronoi(
                    &painter,
                    vor,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                );
                continue;
            }
            PlotLayer::AlphaShape(alpha) => {
                super::triangulation::paint_alphashape(
                    &painter,
                    alpha,
                    transform,
                    resolved.stroke.color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Streamlines(stream) => {
                super::flow::paint_streamlines(&painter, stream, transform, resolved, diagnostics);
                continue;
            }
            PlotLayer::Feather(feather) => {
                super::flow::paint_feather(&painter, feather, transform, resolved, diagnostics);
                continue;
            }
            PlotLayer::Spy(spy) => {
                super::spy::paint_spy(
                    &painter,
                    spy,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Comet(comet) => {
                super::animation::paint_comet(&painter, comet, transform, resolved, diagnostics);
                continue;
            }
            other @ (PlotLayer::BubbleChart(_)
            | PlotLayer::SwarmChart(_)
            | PlotLayer::Histogram2(_)
            | PlotLayer::BinScatter(_)
            | PlotLayer::BoxChart(_)
            | PlotLayer::ViolinPlot(_)
            | PlotLayer::Raincloud(_)
            | PlotLayer::PieChart(_)
            | PlotLayer::WordCloud(_)
            | PlotLayer::BubbleCloud(_)
            | PlotLayer::CategoricalHeatmap(_)
            | PlotLayer::ParallelPlot(_)) => {
                super::stage8c::paint_stage8c_layer(
                    &painter,
                    other,
                    transform,
                    resolved,
                    style.text_color,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Graph(graph) => {
                super::graph::paint_graph(
                    &painter,
                    graph,
                    transform,
                    resolved,
                    style.text_color,
                    diagnostics,
                );
                continue;
            }
            PlotLayer::Series(s) => s,
        };
        let color = resolved.stroke.color;
        let markers = match s {
            Series::Line(line) => {
                let stroke = resolved.stroke;
                if stroke.width.is_finite()
                    && stroke.width > 0.0
                    && stroke.color != Color32::TRANSPARENT
                {
                    if resolved.pattern.is_valid() {
                        let prepared = prepare_series_line(
                            line,
                            transform,
                            ui.ctx().pixels_per_point(),
                            diagnostics,
                            cache.as_deref_mut(),
                        );
                        diagnostics.visited_segments += prepared.visited_segments;
                        diagnostics.numeric_rejections += prepared.numeric_rejections;
                        let patterned = apply_pattern(prepared.paths, resolved.pattern);
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
                resolved
                    .line_markers
                    .then_some((resolved.marker, None, None))
            }
            Series::Scatter(scatter) => Some((resolved.marker, scatter.colors, scatter.radii)),
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
    diagnostics.prepare_duration = start
        .elapsed()
        .saturating_sub(diagnostics.query_duration - query_start);
}
