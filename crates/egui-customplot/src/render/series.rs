use std::{collections::HashSet, time::Instant};

use egui::{Color32, IdSalt, Pos2, Ui};

use super::{
    line_query::prepare_series_line, markers_lod::prepare_markers_with_sampling,
    pattern::apply_pattern,
};
use crate::{
    MarkerSampling, PlotCache, PlotDiagnostics, PlotLayer, RenderBackend, Series,
    axis::SeriesTransforms, model::ResolvedPlotStyle,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_series(
    ui: &Ui,
    plot_id: egui::Id,
    hidden_series: &HashSet<IdSalt>,
    series: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
    mut cache: Option<&mut PlotCache>,
    mut auto_cache: Option<&mut PlotCache>,
) {
    let painter = ui.painter().with_clip_rect(transforms.left.rect());
    let start = Instant::now();
    let query_start = diagnostics.query_duration;
    for (index, s) in series.iter().enumerate() {
        if hidden_series.contains(&s.id(index)) {
            continue;
        }
        let transform = transforms.for_axis(s.y_axis());
        let resolved = s.resolved_style(index, style);
        let s = match *s {
            PlotLayer::Custom(layer, _) => {
                layer.paint(&crate::LayerPaintContext {
                    painter: &painter,
                    transform,
                    stroke: resolved.stroke,
                    pixels_per_point: ui.ctx().pixels_per_point(),
                    exporting: super::is_capturing_snapshot(ui.ctx()),
                });
                continue;
            }
            PlotLayer::StreamLine(line) => {
                super::fast_series::paint_stream_line(
                    &painter,
                    line,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                    cache.as_deref_mut(),
                    auto_cache.as_deref_mut(),
                );
                continue;
            }
            PlotLayer::WaveformLine(line) => {
                let draw_key = plot_id.with((ui.ctx().viewport_id(), index)).value();
                super::fast_series::paint_waveform_line(
                    ui.ctx(),
                    &painter,
                    draw_key,
                    line,
                    transform,
                    resolved,
                    ui.ctx().pixels_per_point(),
                    diagnostics,
                    cache.as_deref_mut(),
                    auto_cache.as_deref_mut(),
                );
                continue;
            }
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
                    auto_cache.as_deref_mut(),
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
            other => {
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
        };
        let color = resolved.stroke.color;
        let markers = match s {
            Series::Line(line) => {
                if line.backend == RenderBackend::Wgpu {
                    diagnostics.wgpu_fallback_series += 1;
                }
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
                            auto_cache.as_deref_mut(),
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
                    .then_some((resolved.marker, None, None, line.marker_sampling))
            }
            Series::Scatter(scatter) => {
                #[cfg(feature = "wgpu")]
                let painted_wgpu = {
                    let draw_key = plot_id.with((ui.ctx().viewport_id(), index)).value();
                    super::wgpu::try_paint_scatter_wgpu(
                        ui.ctx(),
                        &painter,
                        draw_key,
                        scatter.data,
                        transform,
                        resolved.marker,
                        color,
                        scatter.colors,
                        scatter.radii,
                        scatter.sampling,
                        scatter.backend,
                        diagnostics,
                    )
                };
                #[cfg(not(feature = "wgpu"))]
                let painted_wgpu = false;
                if painted_wgpu {
                    None
                } else {
                    if scatter.backend == RenderBackend::Wgpu {
                        diagnostics.wgpu_fallback_series += 1;
                    }
                    Some((
                        resolved.marker,
                        scatter.colors,
                        scatter.radii,
                        scatter.sampling,
                    ))
                }
            }
        };
        if let Some((style, colors, radii, marker_sampling)) = markers {
            let prepared = prepare_markers_with_sampling(
                s.data(),
                transform,
                style,
                color,
                colors,
                radii,
                ui.ctx(),
                marker_sampling,
            );
            diagnostics.visited_marker_points += prepared.visited_points;
            diagnostics.output_markers += prepared.output_markers;
            diagnostics.numeric_rejections += prepared.numeric_rejections;
            if marker_sampling == MarkerSampling::Auto && prepared.reduced {
                diagnostics.marker_lod_series += 1;
            }
            diagnostics.geometry_capacity_bytes += prepared.mesh.vertices.capacity()
                * std::mem::size_of::<egui::epaint::Vertex>()
                + prepared.mesh.indices.capacity() * std::mem::size_of::<u32>();
            if !prepared.discs.is_empty() {
                diagnostics.geometry_capacity_bytes +=
                    prepared.discs.capacity() * std::mem::size_of::<egui::Shape>();
                painter.add(egui::Shape::Vec(prepared.discs));
            } else if !prepared.mesh.is_empty() {
                painter.add(egui::Shape::mesh(prepared.mesh));
            }
        }
    }
    diagnostics.prepare_duration = start
        .elapsed()
        .saturating_sub(diagnostics.query_duration - query_start);
}
