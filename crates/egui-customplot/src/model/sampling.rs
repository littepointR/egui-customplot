/// Rendering policy for lines only. Markers and data-cursor values stay raw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineSampling {
    /// Reference path: visit every raw segment, no derived index required.
    #[default]
    Exact,
    /// Exact visible segments from a prebuilt monotonic-X index.
    Visible,
    /// Opt-in approximation: first/last and Y extrema in each physical screen
    /// column, in original order, never joining domain breaks. Column count is
    /// capped at 16,384 per run. Not an exact substitute for every segment.
    MinMax,
}
