use std::{error::Error, fmt};

use super::AxisScale;

/// Invalid, degenerate, non-finite, or unrepresentable coordinate range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundsError;

impl fmt::Display for BoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounds must have finite endpoints and finite, positive spans")
    }
}

impl Error for BoundsError {}

/// A validated, nondegenerate rectangle in f64 data coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotBounds {
    pub(super) min: [f64; 2],
    pub(super) max: [f64; 2],
}

impl Default for PlotBounds {
    fn default() -> Self {
        Self {
            min: [-1.0; 2],
            max: [1.0; 2],
        }
    }
}

impl PlotBounds {
    pub fn new(min: [f64; 2], max: [f64; 2]) -> Result<Self, BoundsError> {
        for axis in 0..2 {
            let span = max[axis] - min[axis];
            if !min[axis].is_finite() || !max[axis].is_finite() || !span.is_finite() || span <= 0.0
            {
                return Err(BoundsError);
            }
        }
        Ok(Self { min, max })
    }

    pub fn min(self) -> [f64; 2] {
        self.min
    }
    pub fn max(self) -> [f64; 2] {
        self.max
    }
    pub fn span(self) -> [f64; 2] {
        [self.max[0] - self.min[0], self.max[1] - self.min[1]]
    }

    pub(crate) fn fit(min: [f64; 2], max: [f64; 2]) -> Result<Self, BoundsError> {
        let mut lo = min;
        let mut hi = max;
        for axis in 0..2 {
            let span = hi[axis] - lo[axis];
            let pad = if span == 0.0 {
                (lo[axis].abs() * 0.05).max(0.5)
            } else {
                span * 0.05
            };
            // Padding is best-effort; don't overflow otherwise valid data.
            let padded_lo = lo[axis] - pad;
            let padded_hi = hi[axis] + pad;
            if padded_lo.is_finite() && padded_hi.is_finite() && (padded_hi - padded_lo).is_finite()
            {
                lo[axis] = padded_lo;
                hi[axis] = padded_hi;
            }
        }
        Self::new(lo, hi)
    }

    pub(crate) fn fit_scaled(
        min: [f64; 2],
        max: [f64; 2],
        scales: [AxisScale; 2],
    ) -> Result<Self, BoundsError> {
        let lo = [
            scales[0].forward(min[0]).ok_or(BoundsError)?,
            scales[1].forward(min[1]).ok_or(BoundsError)?,
        ];
        let hi = [
            scales[0].forward(max[0]).ok_or(BoundsError)?,
            scales[1].forward(max[1]).ok_or(BoundsError)?,
        ];
        let padded = Self::fit(lo, hi)?;
        // Preserve extreme valid endpoints when padding leaves the log domain.
        let low = [
            scales[0].inverse(padded.min[0]).unwrap_or(min[0]),
            scales[1].inverse(padded.min[1]).unwrap_or(min[1]),
        ];
        let high = [
            scales[0].inverse(padded.max[0]).unwrap_or(max[0]),
            scales[1].inverse(padded.max[1]).unwrap_or(max[1]),
        ];
        Self::new(low, high)
    }

    pub(crate) fn default_scaled(scales: [AxisScale; 2]) -> Self {
        let x = scales[0].default_range();
        let y = scales[1].default_range();
        Self {
            min: [x[0], y[0]],
            max: [x[1], y[1]],
        }
    }

    pub(crate) fn fit_optional_ranges(
        ranges: [Option<[f64; 2]>; 2],
        scales: [AxisScale; 2],
    ) -> Result<Self, BoundsError> {
        let mut result = Self::default_scaled(scales);
        for axis in 0..2 {
            if let Some([lo, hi]) = ranges[axis] {
                let fitted = Self::fit_scaled([lo; 2], [hi; 2], [scales[axis]; 2])?;
                result.min[axis] = fitted.min[0];
                result.max[axis] = fitted.max[0];
            }
        }
        Ok(result)
    }

    /// Translate in data coordinates. Invalid/overflowing navigation is rejected.
    pub fn translated(self, delta: [f64; 2]) -> Result<Self, BoundsError> {
        Self::new(
            [self.min[0] + delta[0], self.min[1] + delta[1]],
            [self.max[0] + delta[0], self.max[1] + delta[1]],
        )
    }

    /// Scale the spans around an anchor; factors below one zoom in.
    pub fn zoomed(self, anchor: [f64; 2], factor: f64) -> Result<Self, BoundsError> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(BoundsError);
        }
        let mut min = self.min;
        let mut max = self.max;
        for axis in 0..2 {
            min[axis] = anchor[axis] + (min[axis] - anchor[axis]) * factor;
            max[axis] = anchor[axis] + (max[axis] - anchor[axis]) * factor;
        }
        Self::new(min, max)
    }
}
