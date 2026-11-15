use std::sync::Arc;

use crate::{Bar, BarData};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistogramNormalization {
    #[default]
    Count,
    Probability,
    /// Probability divided by each bin's data-space width.
    Density,
    CumulativeCount,
    CumulativeProbability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistogramError {
    InvalidEdges,
    AllocationFailed,
    UnrepresentableBin,
}
impl std::fmt::Display for HistogramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid histogram: {self:?}")
    }
}
impl std::error::Error for HistogramError {}

/// Explicit CPU preprocessing, never performed by a plotting pass. Bins are
/// [left, right), except the last includes its right edge. Nonfinite and outside
/// observations are counted separately. Normalization uses in-range samples.
/// No automatic estimator, categorical aggregation or hidden rebinning.
#[derive(Clone, Debug)]
pub struct HistogramData {
    edges: Arc<[f64]>,
    counts: Arc<[usize]>,
    included: usize,
    non_finite: usize,
    outside: usize,
}

impl HistogramData {
    pub fn new(samples: &[f64], edges: impl Into<Arc<[f64]>>) -> Result<Self, HistogramError> {
        let edges = edges.into();
        if edges.len() < 2
            || edges.iter().any(|v| !v.is_finite())
            || edges
                .windows(2)
                .any(|p| p[0] >= p[1] || !(p[1] - p[0]).is_finite())
        {
            return Err(HistogramError::InvalidEdges);
        }
        let mut counts = Vec::new();
        counts
            .try_reserve_exact(edges.len() - 1)
            .map_err(|_| HistogramError::AllocationFailed)?;
        counts.resize(edges.len() - 1, 0);
        let (mut included, mut non_finite, mut outside) = (0, 0, 0);
        for &sample in samples {
            if !sample.is_finite() {
                non_finite += 1;
                continue;
            }
            if sample < edges[0] || sample > edges[edges.len() - 1] {
                outside += 1;
                continue;
            }
            let index = edges
                .partition_point(|e| *e <= sample)
                .saturating_sub(1)
                .min(counts.len() - 1);
            counts[index] += 1;
            included += 1;
        }
        Ok(Self {
            edges,
            counts: counts.into(),
            included,
            non_finite,
            outside,
        })
    }
    pub fn edges(&self) -> &[f64] {
        &self.edges
    }
    pub fn counts(&self) -> &[usize] {
        &self.counts
    }
    pub fn included_samples(&self) -> usize {
        self.included
    }
    pub fn non_finite_samples(&self) -> usize {
        self.non_finite
    }
    pub fn outside_samples(&self) -> usize {
        self.outside
    }

    /// Build host-owned bar records once when normalization changes. Empty bins
    /// have height zero; an empty in-range set normalizes to zero, never NaN.
    /// Returns an error when f64 cannot represent the derived center/width/value.
    pub fn to_bars(
        &self,
        normalization: HistogramNormalization,
    ) -> Result<BarData, HistogramError> {
        let mut bars = Vec::new();
        bars.try_reserve_exact(self.counts.len())
            .map_err(|_| HistogramError::AllocationFailed)?;
        let mut cumulative = 0;
        for (index, &count) in self.counts.iter().enumerate() {
            cumulative += count;
            let width = self.edges[index + 1] - self.edges[index];
            let value = match normalization {
                HistogramNormalization::Count => count as f64,
                HistogramNormalization::CumulativeCount => cumulative as f64,
                HistogramNormalization::Probability | HistogramNormalization::Density => {
                    let probability = if self.included == 0 {
                        0.0
                    } else {
                        count as f64 / self.included as f64
                    };
                    if normalization == HistogramNormalization::Density {
                        probability / width
                    } else {
                        probability
                    }
                }
                HistogramNormalization::CumulativeProbability => {
                    if self.included == 0 {
                        0.0
                    } else {
                        cumulative as f64 / self.included as f64
                    }
                }
            };
            let center = self.edges[index] + width * 0.5;
            let bar = Bar::new(center, value).width(width);
            if bar.extent(crate::BarOrientation::Vertical).is_none() {
                return Err(HistogramError::UnrepresentableBin);
            }
            bars.push(bar);
        }
        Ok(BarData::new(bars))
    }
}
