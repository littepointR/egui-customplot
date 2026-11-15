use std::time::Duration;

/// Per-show-pass counters, not process/GPU memory measurements.
#[derive(Clone, Debug, Default)]
pub struct PlotDiagnostics {
    pub input_bars: usize,
    /// Invalid record payloads (including width/overflow), before log domains.
    pub invalid_bars: usize,
    pub domain_invalid_bars: usize,
    pub unique_bar_bytes: usize,
    pub visited_bars: usize,
    pub output_bars: usize,
    pub visited_pick_bars: usize,
    pub input_error_bars: usize,
    pub invalid_error_bars: usize,
    pub domain_invalid_error_bars: usize,
    pub unique_error_bar_bytes: usize,
    pub visited_error_bars: usize,
    pub output_error_bar_segments: usize,
    pub input_heatmap_cells: usize,
    pub non_finite_heatmap_cells: usize,
    /// Missing scalar values plus nonpositive values under log color mapping;
    /// this overlaps, and must not be added to, coordinate-domain invalid cells.
    pub color_invalid_heatmap_cells: usize,
    /// Cells with one or both coordinate edges outside a log domain.
    pub domain_invalid_heatmap_cells: usize,
    /// Distinct edge/value Arc payloads, excluding allocation headers.
    pub unique_grid_bytes: usize,
    pub visited_heatmap_cells: usize,
    pub output_heatmap_cells: usize,
    pub output_heatmap_meshes: usize,
    pub heatmap_pick_queries: usize,
    /// Shared ranges rejected by the current log domain or aspect constraint.
    pub invalid_linked_bounds: usize,
    pub input_points: usize,
    pub unique_sample_bytes: usize,
    /// Shared immutable line indexes, deduplicated separately from raw samples.
    pub unique_index_bytes: usize,
    pub non_finite_points: usize,
    /// Finite samples outside the configured coordinate domains, e.g. log x<=0.
    pub domain_invalid_points: usize,
    /// Selected sample pairs inspected, before step expansion; zero on cache hits.
    pub visited_segments: usize,
    pub visited_marker_points: usize,
    pub visited_pick_points: usize,
    pub indexed_line_series: usize,
    /// Non-Exact sampling requested without an index; reference path used.
    pub lod_fallback_series: usize,
    pub query_runs: usize,
    pub query_comparisons: usize,
    pub query_index_nodes: usize,
    /// Raw points inspected in partial tree blocks, not selected render points.
    pub query_scanned_points: usize,
    pub query_selected_points: usize,
    /// Candidate-range and selected-index Vec capacities for processed lines;
    /// excludes geometry and transient reallocation peaks. Summed across series.
    pub query_capacity_bytes: usize,
    pub pick_index_runs: usize,
    pub pick_index_comparisons: usize,
    pub pick_query_capacity_bytes: usize,
    pub line_cache_hits: usize,
    pub line_cache_misses: usize,
    /// Result exceeded cache budget; still rendered normally.
    pub line_cache_rejections: usize,
    /// All entries retained by the supplied host cache, not just this plot.
    pub line_cache_bytes: usize,
    pub output_points: usize,
    pub output_paths: usize,
    pub output_markers: usize,
    /// Visible annotation items submitted (a labeled reference line is one item).
    pub output_annotations: usize,
    /// Invalid coordinates, reversed/NaN intervals, or invalid annotation widths.
    pub annotation_rejections: usize,
    /// Line-point / marker vertex / marker index Vec capacities submitted to egui.
    pub geometry_capacity_bytes: usize,
    pub numeric_rejections: usize,
    pub auto_bounds_error: bool,
    pub invalid_manual_bounds: bool,
    pub invalid_right_y_range: bool,
    /// Right series used Left because no secondary axis was configured.
    pub unconfigured_right_axis_series: usize,
    /// Requested aspect expansion could not be represented; natural bounds used.
    pub aspect_constraint_failed: bool,
    /// Dashed stroke hit its per-series safety cap; remaining dashes were omitted.
    pub pattern_truncated: bool,
    pub duplicate_series_ids: usize,
    pub painting_skipped: bool,
    pub visible_series: usize,
    /// Geometry preparation and submission only; axes/picking/tessellation excluded.
    pub prepare_duration: Duration,
    pub pick_duration: Duration,
    pub query_duration: Duration,
}
