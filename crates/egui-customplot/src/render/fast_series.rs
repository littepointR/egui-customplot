use std::time::Instant;

use egui::{Color32, Painter, Pos2, Shape};

use super::{
    cache::Key,
    line::{PreparedLine, prepare_line_auto_unindexed_iter, prepare_line_iter},
    pattern::apply_pattern,
};
use crate::{
    LineInterpolation, LineSampling, PlotCache, PlotDiagnostics, PlotTransform, RenderBackend,
    StreamLine, WaveformLine, data::AUTO_INDEX_MIN_POINTS, model::ResolvedSeriesStyle,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_stream_line(
    painter: &Painter,
    line: StreamLine<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
    cache: Option<&mut PlotCache>,
    auto_cache: Option<&mut PlotCache>,
) {
    if line.backend == RenderBackend::Wgpu {
        diagnostics.wgpu_fallback_series += 1;
    }
    let stroke = resolved.stroke;
    if !stroke.width.is_finite() || stroke.width <= 0.0 || stroke.color == Color32::TRANSPARENT {
        return;
    }
    if !resolved.pattern.is_valid() {
        diagnostics.numeric_rejections += 1;
        return;
    }
    let key = Key::from_parts(
        1,
        line.data.version(),
        transform,
        pixels_per_point,
        line.sampling,
        line.interpolation,
        true,
    );
    let prepared = lookup_or_prepare(
        key,
        line.sampling,
        line.data.len(),
        diagnostics,
        cache,
        auto_cache,
        |diagnostics| {
            let reduce = matches!(line.sampling, LineSampling::MinMax | LineSampling::Auto);
            let start = Instant::now();
            let q = line.data.query(transform, pixels_per_point, reduce);
            diagnostics.query_duration += start.elapsed();
            diagnostics.indexed_line_series += usize::from(q.monotonic);
            if line.sampling == LineSampling::Auto && (q.stats.reduced || !q.monotonic) {
                diagnostics.auto_lod_series +=
                    usize::from(q.stats.reduced || line.data.len() >= 32_768);
            }
            diagnostics.query_comparisons += q.stats.comparisons;
            diagnostics.query_index_nodes += q.stats.visited_nodes;
            diagnostics.query_scanned_points += q.stats.scanned_points;
            diagnostics.query_selected_points += q.stats.selected_points;
            diagnostics.query_runs += q.stats.visited_runs;
            diagnostics.query_capacity_bytes += q.capacity_bytes;
            if !q.monotonic
                && line.sampling == LineSampling::Auto
                && line.data.len() >= 32_768
                && line.interpolation == LineInterpolation::Linear
            {
                let (a, b) = line.data.as_slices();
                prepare_line_auto_unindexed_iter(
                    a.iter().chain(b.iter()).copied(),
                    transform,
                    pixels_per_point,
                )
            } else {
                prepare_line_iter(
                    q.slots
                        .into_iter()
                        .map(|slot| line.data.point_at_slot(slot)),
                    transform,
                    line.interpolation,
                )
            }
        },
    );
    submit_prepared(painter, prepared, resolved, diagnostics);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_waveform_line(
    ui_ctx: &egui::Context,
    painter: &Painter,
    draw_key: u64,
    line: WaveformLine<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
    cache: Option<&mut PlotCache>,
    auto_cache: Option<&mut PlotCache>,
) {
    #[cfg(feature = "wgpu")]
    if super::wgpu::try_paint_waveform_wgpu(
        ui_ctx,
        painter,
        draw_key,
        line,
        transform,
        resolved.stroke,
        resolved.pattern,
        diagnostics,
    ) {
        return;
    }
    #[cfg(not(feature = "wgpu"))]
    let _ = (ui_ctx, draw_key);
    if line.backend == RenderBackend::Wgpu {
        diagnostics.wgpu_fallback_series += 1;
    }
    let stroke = resolved.stroke;
    if !stroke.width.is_finite() || stroke.width <= 0.0 || stroke.color == Color32::TRANSPARENT {
        return;
    }
    if !resolved.pattern.is_valid() {
        diagnostics.numeric_rejections += 1;
        return;
    }
    let key = Key::from_parts(
        2,
        line.data.version(),
        transform,
        pixels_per_point,
        line.sampling,
        line.interpolation,
        true,
    );
    let prepared = lookup_or_prepare(
        key,
        line.sampling,
        line.data.len(),
        diagnostics,
        cache,
        auto_cache,
        |diagnostics| {
            let reduce = matches!(line.sampling, LineSampling::MinMax | LineSampling::Auto);
            let start = Instant::now();
            let q = line.data.query(transform, pixels_per_point, reduce);
            diagnostics.query_duration += start.elapsed();
            diagnostics.indexed_line_series += 1;
            if line.sampling == LineSampling::Auto && q.stats.reduced {
                diagnostics.auto_lod_series += 1;
            }
            diagnostics.query_comparisons += q.stats.comparisons;
            diagnostics.query_index_nodes += q.stats.visited_nodes;
            diagnostics.query_scanned_points += q.stats.scanned_points;
            diagnostics.query_selected_points += q.stats.selected_points;
            diagnostics.query_runs += q.stats.visited_runs;
            diagnostics.query_capacity_bytes += q.capacity_bytes;
            prepare_line_iter(
                q.indices
                    .into_iter()
                    .map(|i| line.data.point_at(i as usize)),
                transform,
                line.interpolation,
            )
        },
    );
    submit_prepared(painter, prepared, resolved, diagnostics);
}

fn lookup_or_prepare(
    key: Key,
    sampling: LineSampling,
    len: usize,
    diagnostics: &mut PlotDiagnostics,
    cache: Option<&mut PlotCache>,
    auto_cache: Option<&mut PlotCache>,
    prepare: impl FnOnce(&mut PlotDiagnostics) -> PreparedLine,
) -> PreparedLine {
    if let Some(cache) = cache {
        if let Some(mut prepared) = cache.get(&key) {
            diagnostics.line_cache_hits += 1;
            prepared.visited_segments = 0;
            return prepared;
        }
        diagnostics.line_cache_misses += 1;
        let prepared = prepare(diagnostics);
        if !cache.insert(key, &prepared) {
            diagnostics.line_cache_rejections += 1;
        }
        return prepared;
    }
    if sampling == LineSampling::Auto
        && len >= AUTO_INDEX_MIN_POINTS
        && let Some(auto) = auto_cache
    {
        if let Some(mut prepared) = auto.get(&key) {
            prepared.visited_segments = 0;
            return prepared;
        }
        let prepared = prepare(diagnostics);
        auto.insert(key, &prepared);
        return prepared;
    }
    prepare(diagnostics)
}

fn submit_prepared(
    painter: &Painter,
    prepared: PreparedLine,
    resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    diagnostics.visited_segments += prepared.visited_segments;
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    let patterned = apply_pattern(prepared.paths, resolved.pattern);
    diagnostics.pattern_truncated |= patterned.truncated;
    diagnostics.output_paths += patterned.paths.len();
    for path in patterned.paths {
        diagnostics.output_points += path.len();
        diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        painter.add(Shape::line(path, resolved.stroke));
    }
}
