use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaError {
    LengthMismatch,
    InvalidBaseline,
}

impl std::fmt::Display for AreaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LengthMismatch => "area x and y series must have identical lengths",
            Self::InvalidBaseline => "area stacking baseline must be finite",
        })
    }
}

impl std::error::Error for AreaError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AreaStackMode {
    #[default]
    Cumulative,
    /// Stacks nonnegative values upward and negative values downward from baseline.
    PosNegSplit,
}

/// Immutable host-owned lower/upper band for stacked or bounded area plots.
#[derive(Clone, Debug)]
pub struct AreaBandData {
    x: Arc<[f64]>,
    lower: Arc<[f64]>,
    upper: Arc<[f64]>,
    extents: CachedExtents,
    non_finite: usize,
    x_monotonic: bool,
}

impl AreaBandData {
    pub fn from_band(
        x: impl Into<Arc<[f64]>>,
        lower: impl Into<Arc<[f64]>>,
        upper: impl Into<Arc<[f64]>>,
    ) -> Result<Self, AreaError> {
        let (x, lower, upper) = (x.into(), lower.into(), upper.into());
        if x.len() != lower.len() || x.len() != upper.len() {
            return Err(AreaError::LengthMismatch);
        }
        let x_monotonic = x
            .windows(2)
            .all(|w| w[0].is_finite() && w[1].is_finite() && w[0] <= w[1]);
        let mut extents = CachedExtents::default();
        let mut non_finite = 0;
        for i in 0..x.len() {
            let (xv, lo, hi) = (x[i], lower[i], upper[i]);
            if !xv.is_finite() || !lo.is_finite() || !hi.is_finite() {
                non_finite += 1;
                extents.push(None);
            } else {
                extents.push(Some(([xv, lo.min(hi)], [xv, lo.max(hi)])));
            }
        }
        Ok(Self {
            x,
            lower,
            upper,
            extents,
            non_finite,
            x_monotonic,
        })
    }

    /// Preprocess stacked bands once outside UI passes, sharing one `x` buffer.
    pub fn stack(
        x: impl Into<Arc<[f64]>>,
        series: &[&[f64]],
        baseline: f64,
        mode: AreaStackMode,
    ) -> Result<Vec<Self>, AreaError> {
        if !baseline.is_finite() {
            return Err(AreaError::InvalidBaseline);
        }
        let x: Arc<[f64]> = x.into();
        for s in series {
            if s.len() != x.len() {
                return Err(AreaError::LengthMismatch);
            }
        }
        let n = x.len();
        let mut pos_acc = vec![baseline; n];
        let mut neg_acc = vec![baseline; n];
        let mut bands = Vec::with_capacity(series.len());
        for s in series {
            let mut lower = Vec::with_capacity(n);
            let mut upper = Vec::with_capacity(n);
            for i in 0..n {
                let y = s[i];
                if !x[i].is_finite() || !y.is_finite() {
                    lower.push(f64::NAN);
                    upper.push(f64::NAN);
                    continue;
                }
                match mode {
                    AreaStackMode::Cumulative => {
                        let base = pos_acc[i];
                        let next = base + y;
                        if !next.is_finite() {
                            lower.push(f64::NAN);
                            upper.push(f64::NAN);
                        } else {
                            lower.push(base);
                            upper.push(next);
                            pos_acc[i] = next;
                        }
                    }
                    AreaStackMode::PosNegSplit => {
                        if y >= 0.0 {
                            let base = pos_acc[i];
                            let next = base + y;
                            if !next.is_finite() {
                                lower.push(f64::NAN);
                                upper.push(f64::NAN);
                            } else {
                                lower.push(base);
                                upper.push(next);
                                pos_acc[i] = next;
                            }
                        } else {
                            let top = neg_acc[i];
                            let bottom = top + y;
                            if !bottom.is_finite() {
                                lower.push(f64::NAN);
                                upper.push(f64::NAN);
                            } else {
                                lower.push(bottom);
                                upper.push(top);
                                neg_acc[i] = bottom;
                            }
                        }
                    }
                }
            }
            bands.push(Self::from_band(Arc::clone(&x), lower, upper)?);
        }
        Ok(bands)
    }

    pub fn x(&self) -> &[f64] {
        &self.x
    }
    pub fn lower(&self) -> &[f64] {
        &self.lower
    }
    pub fn upper(&self) -> &[f64] {
        &self.upper
    }
    pub fn len(&self) -> usize {
        self.x.len()
    }
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }
    pub fn non_finite_samples(&self) -> usize {
        self.non_finite
    }
    pub fn sample_bytes(&self) -> usize {
        (self.x.len() + self.lower.len() + self.upper.len()) * 8
    }
    pub(crate) fn allocations(&self) -> [(usize, usize); 3] {
        [&self.x, &self.lower, &self.upper].map(|v| (v.as_ptr() as usize, v.len() * 8))
    }
    pub(crate) fn is_x_monotonic(&self) -> bool {
        self.x_monotonic
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }
}
