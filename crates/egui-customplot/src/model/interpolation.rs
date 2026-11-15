/// Connection between consecutive valid samples, in their original order.
/// Affects line geometry only: markers, auto bounds and picking remain raw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineInterpolation {
    /// Straight segments (in transformed coordinates on log axes).
    #[default]
    Linear,
    /// Hold y[i] to x[i+1], then jump to y[i+1] (MATLAB-style stairs).
    /// Starts at the first sample and ends at the last; no extrapolated plateau.
    StepAfter,
    /// Jump to y[i+1] at x[i], then hold to x[i+1]. No endpoint extension.
    StepBefore,
}
