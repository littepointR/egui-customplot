use std::time::Instant;

use crate::{Line, LineSampling, PlotCache, PlotDiagnostics, PlotTransform};

use super::cache::Key;
use super::line::{PreparedLine, prepare_line, prepare_line_iter};

pub(super) fn prepare_series_line(
    line: Line<'_>,
    transform: PlotTransform,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
    cache: Option<&mut PlotCache>,
) -> PreparedLine {
    // Pre-style geometry is independent of color/width/dash/marker properties.
    // All transform, immutable data and sampling inputs belong to its key.
    if let Some(cache) = cache {
        let key = Key::new(line, transform, pixels_per_point);
        if let Some(mut prepared) = cache.get(&key) {
            diagnostics.line_cache_hits += 1;
            prepared.visited_segments = 0;
            if line.sampling != LineSampling::Exact && line.data.line_index().is_none() {
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
    prepare_uncached_line(line, transform, pixels_per_point, diagnostics)
}

fn prepare_uncached_line(
    line: Line<'_>,
    transform: PlotTransform,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) -> PreparedLine {
    if line.sampling == LineSampling::Exact {
        return prepare_line(line.data.points(), transform, line.interpolation);
    }
    let Some(index) = line.data.line_index() else {
        diagnostics.lod_fallback_series += 1;
        return prepare_line(line.data.points(), transform, line.interpolation);
    };
    let start = Instant::now();
    let query = index.query(
        line.data.points(),
        transform,
        pixels_per_point,
        line.sampling == LineSampling::MinMax,
    );
    diagnostics.query_duration += start.elapsed();
    diagnostics.indexed_line_series += 1;
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
