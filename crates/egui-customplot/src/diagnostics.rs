use std::time::Duration;

/// Per-show-pass counters, not process/GPU memory measurements.
#[derive(Clone, Debug, Default)]
pub struct PlotDiagnostics {
    pub input_points: usize,
    pub unique_sample_bytes: usize,
    pub non_finite_points: usize,
    /// Finite samples outside the configured coordinate domains, e.g. log x<=0.
    pub domain_invalid_points: usize,
    pub visited_segments: usize,
    pub visited_marker_points: usize,
    pub visited_pick_points: usize,
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
}
