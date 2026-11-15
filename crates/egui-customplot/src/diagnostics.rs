use std::time::Duration;

/// Per-show-pass counters for area, fill, patch, rectangle and polyshape layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PatchDiagnostics {
    pub input_items: usize,
    pub invalid_items: usize,
    pub domain_invalid_items: usize,
    pub invalid_area_baselines: usize,
    pub unique_bytes: usize,
    pub visited_items: usize,
    pub output_meshes: usize,
    pub output_triangles: usize,
    pub output_strokes: usize,
    pub pick_queries: usize,
}

/// Per-show-pass counters for contour line, filled contour band and label layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContourDiagnostics {
    pub input_lines: usize,
    pub input_bands: usize,
    pub domain_invalid_items: usize,
    pub unique_bytes: usize,
    pub visited_items: usize,
    pub output_paths: usize,
    pub output_meshes: usize,
    pub output_labels: usize,
    pub pick_queries: usize,
}

/// Per-show-pass counters for pcolor fields and raster image layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageDiagnostics {
    pub input_cells: usize,
    pub non_finite_cells: usize,
    pub color_invalid_cells: usize,
    pub domain_invalid_cells: usize,
    pub unique_bytes: usize,
    pub visited_cells: usize,
    pub output_cells: usize,
    pub output_meshes: usize,
    pub pick_queries: usize,
}

/// Per-show-pass counters for triangulation, Voronoi, alphaShape and spy layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeometryDiagnostics {
    pub input_edges: usize,
    pub input_triangles: usize,
    pub input_spy_nonzeros: usize,
    pub domain_invalid_items: usize,
    pub unique_bytes: usize,
    pub visited_items: usize,
    pub output_paths: usize,
    pub output_meshes: usize,
    pub output_markers: usize,
    pub pick_queries: usize,
}

/// Per-show-pass counters for streamline, streamslice, feather and comet layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlowDiagnostics {
    pub input_streamlines: usize,
    pub input_feather_vectors: usize,
    pub input_comet_points: usize,
    pub domain_invalid_items: usize,
    pub unique_bytes: usize,
    pub visited_items: usize,
    pub output_paths: usize,
    pub output_heads: usize,
    pub pick_queries: usize,
}

/// Statistical, categorical, pie/cloud, and bivariate counters (Stage 8C).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatDiagnostics {
    pub input_bubbles: usize,
    pub output_bubbles: usize,
    pub input_boxes: usize,
    pub output_boxes: usize,
    pub input_violins: usize,
    pub output_violins: usize,
    pub input_slices: usize,
    pub output_slices: usize,
    pub input_clouds: usize,
    pub output_clouds: usize,
    pub input_bins: usize,
    pub output_bins: usize,
    pub input_parallel_rows: usize,
    pub output_parallel_segments: usize,
}

/// Per-show-pass counters, not process/GPU memory measurements.
#[derive(Clone, Debug, Default)]
pub struct PlotDiagnostics {
    pub patches: PatchDiagnostics,
    pub contours: ContourDiagnostics,
    pub images: ImageDiagnostics,
    pub geometry: GeometryDiagnostics,
    pub flow: FlowDiagnostics,
    pub stats: StatDiagnostics,
    pub input_quiver_arrows: usize,
    /// Nonfinite origins/vectors or overflowed true endpoints.
    pub invalid_quiver_arrows: usize,
    /// Otherwise-valid arrows whose origin or true endpoint violates a log domain.
    pub domain_invalid_quiver_arrows: usize,
    pub unique_quiver_bytes: usize,
    pub visited_quiver_arrows: usize,
    pub visited_pick_quiver_arrows: usize,
    /// Shaft paths after dashing, excluding solid heads.
    pub output_quiver_shafts: usize,
    pub output_quiver_heads: usize,
    /// Stem descriptors whose baseline is nonfinite or outside the value domain.
    /// Raw endpoint diagnostics remain in input_points / domain_invalid_points.
    pub invalid_stem_baselines: usize,
    pub visited_stems: usize,
    /// Rod paths after dashing, not original endpoint count.
    pub output_stem_segments: usize,
    pub output_stem_baselines: usize,
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
