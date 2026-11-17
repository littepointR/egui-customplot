use std::time::Instant;

use crate::{
    Line, LineInterpolation, LineSampling, PlotCache, PlotDiagnostics, PlotTransform,
    data::AUTO_INDEX_MIN_POINTS,
};

use super::cache::Key;
use super::line::{PreparedLine, prepare_line, prepare_line_auto_unindexed, prepare_line_iter};

pub(super) fn prepare_series_line(
    line: Line<'_>,
    transform: PlotTransform,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
    cache: Option<&mut PlotCache>,
    auto_cache: Option<&mut PlotCache>,
) -> PreparedLine {
    // Pre-style geometry is independent of color/width/dash/marker properties.
    // All transform, immutable data and sampling inputs belong to its key.
    if let Some(cache) = cache {
        let key = Key::new(line, transform, pixels_per_point);
        if let Some(mut prepared) = cache.get(&key) {
            diagnostics.line_cache_hits += 1;
            prepared.visited_segments = 0;
            if matches!(line.sampling, LineSampling::Visible | LineSampling::MinMax)
                && line.data.line_index().is_none()
            {
                diagnostics.lod_fallback_series += 1;
            }
            return prepared;
        }
        diagnostics.line_cache_misses += 1;
        let prepared = prepare_uncached_line(line, transform, pixels_per_point, diagnostics);
        if !cache.insert(key, &prepared) {
            diagnostics.line_cache_rejections += 1;
        }
        return prepared;
    }
    if line.sampling == LineSampling::Auto
        && line.data.len() >= AUTO_INDEX_MIN_POINTS
        && let Some(auto) = auto_cache
    {
        let key = Key::new(line, transform, pixels_per_point);
        if let Some(mut prepared) = auto.get(&key) {
            prepared.visited_segments = 0;
            return prepared;
        }
        let prepared = prepare_uncached_line(line, transform, pixels_per_point, diagnostics);
        auto.insert(key, &prepared);
        return prepared;
    }
    prepare_uncached_line(line, transform, pixels_per_point, diagnostics)
}

fn prepare_uncached_line(
    line: Line<'_>,
    transform: PlotTransform,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) -> PreparedLine {
    if line.sampling == LineSampling::Exact || transform.scales().iter().any(|s| s.is_probability())
    {
        return prepare_line(line.data.points(), transform, line.interpolation);
    }
    let Some(index) = line.data.line_index() else {
        if line.sampling == LineSampling::Auto {
            if line.data.len() >= 32_768 && line.interpolation == LineInterpolation::Linear {
                diagnostics.auto_lod_series += 1;
                return prepare_line_auto_unindexed(
                    line.data.points(),
                    transform,
                    pixels_per_point,
                );
            }
            return prepare_line(line.data.points(), transform, line.interpolation);
        }
        diagnostics.lod_fallback_series += 1;
        return prepare_line(line.data.points(), transform, line.interpolation);
    };
    let reduce = matches!(line.sampling, LineSampling::MinMax | LineSampling::Auto);
    let start = Instant::now();
    let query = index.query(line.data.points(), transform, pixels_per_point, reduce);
    diagnostics.query_duration += start.elapsed();
    diagnostics.indexed_line_series += 1;
    if line.sampling == LineSampling::Auto && query.stats.reduced {
        diagnostics.auto_lod_series += 1;
    }
    diagnostics.query_comparisons += query.stats.comparisons;
    diagnostics.query_index_nodes += query.stats.visited_nodes;
    diagnostics.query_scanned_points += query.stats.scanned_points;
    diagnostics.query_selected_points += query.stats.selected_points;
    diagnostics.query_runs += query.stats.visited_runs;
    diagnostics.query_capacity_bytes += query.capacity_bytes;
    let mut prepared = PreparedLine::default();
    for path in query.paths {
        let part = prepare_line_iter(
            path.into_iter().map(|i| line.data.points()[i]),
            transform,
            line.interpolation,
        );
        prepared.paths.extend(part.paths);
        prepared.visited_segments += part.visited_segments;
        prepared.numeric_rejections += part.numeric_rejections;
    }
    prepared
}
