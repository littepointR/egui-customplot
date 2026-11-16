/// Rendering policy for lines only. Markers and data-cursor values stay raw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineSampling {
    /// Automatic policy: uses prebuilt or auto-built monotonic-X index to
    /// preserve peaks via pixel-column MinMax when visible samples exceed 4x
    /// physical screen columns, switches seamlessly to exact segments when zoomed
    /// in, and handles large unsorted polylines via consecutive pixel coalescing.
    #[default]
    Auto,
    /// Reference path: visit every raw segment, no derived index required.
    Exact,
    /// Exact visible segments from a prebuilt monotonic-X index.
    Visible,
    /// Opt-in approximation: first/last and Y extrema in each physical screen
    /// column, in original order, never joining domain breaks. Column count is
    /// capped at 16,384 per run. Not an exact substitute for every segment.
    /// With step interpolation it can also change jump times and hold durations.
    MinMax,
}

/// Rendering policy for point markers (`Scatter` and `Line` markers).
/// Raw-point hover picking is independent and always inspects raw coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerSampling {
    /// Exact below 16,384 points; screen pixel-cell deduplication above.
    #[default]
    Auto,
    /// Emit geometry for every visible marker sample.
    Exact,
    /// Deduplicate dense overlapping markers to one top-most representative per
    /// physical screen pixel cell.
    PixelGrid,
}

/// Preferred rendering backend for accelerated large-scale layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderBackend {
    /// Use registered `wgpu` GPU pipelines when available and profitable;
    /// otherwise fall back seamlessly to CPU `Painter`.
    #[default]
    Auto,
    /// Force CPU `egui::Painter` rendering.
    Painter,
    /// Prefer `wgpu` GPU pipelines when registered; falls back to `Painter`
    /// with a diagnostic counter if unavailable or during snapshot export.
    Wgpu,
}
