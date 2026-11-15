use std::sync::Arc;

use crate::{Bar, BarData, HistogramData, HistogramError, HistogramNormalization};

/// Legacy `hist`-compatible wrapper built over [`HistogramData`], producing fixed-count bins and bin centers.
#[derive(Clone, Debug)]
pub struct LegacyHistData {
    inner: HistogramData,
    centers: Arc<[f64]>,
    widths: Arc<[f64]>,
}

impl LegacyHistData {
    /// Partitions finite `samples` into `nbins` equal-width bins (`nbins >= 1`).
    pub fn new(samples: &[f64], nbins: usize) -> Result<Self, HistogramError> {
        if nbins == 0 {
            return Err(HistogramError::InvalidEdges);
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        let mut finite = 0usize;
        for &s in samples {
            if s.is_finite() {
                lo = lo.min(s);
                hi = hi.max(s);
                finite += 1;
            }
        }
        if finite == 0 {
            lo = 0.0;
            hi = 1.0;
        } else if (hi - lo).abs() <= f64::EPSILON {
            lo -= 0.5;
            hi += 0.5;
        }
        let step = (hi - lo) / nbins as f64;
        let mut edges = Vec::with_capacity(nbins + 1);
        for i in 0..nbins {
            edges.push(lo + step * i as f64);
        }
        // Extend top edge slightly so half-open binning captures the exact maximum sample.
        edges.push(hi + (hi.abs().max(1.0)) * 1e-12);
        let inner = HistogramData::new(samples, edges)?;
        let centers: Arc<[f64]> = (0..nbins).map(|i| lo + step * (i as f64 + 0.5)).collect();
        let widths: Arc<[f64]> = vec![step; nbins].into();
        Ok(Self {
            inner,
            centers,
            widths,
        })
    }

    /// Bins finite `samples` by nearest distance to strictly increasing `centers`.
    pub fn with_centers(samples: &[f64], centers: &[f64]) -> Result<Self, HistogramError> {
        if centers.is_empty() {
            return Err(HistogramError::InvalidEdges);
        }
        if centers.iter().any(|c| !c.is_finite()) || centers.windows(2).any(|w| w[0] >= w[1]) {
            return Err(HistogramError::InvalidEdges);
        }
        let mut min_s = centers[0];
        let mut max_s = centers[centers.len() - 1];
        for &s in samples {
            if s.is_finite() {
                min_s = min_s.min(s);
                max_s = max_s.max(s);
            }
        }
        let mut widths = Vec::with_capacity(centers.len());
        let mut edges = Vec::with_capacity(centers.len() + 1);
        let default_w = if centers.len() == 1 {
            1.0
        } else {
            centers
                .windows(2)
                .map(|w| w[1] - w[0])
                .fold(f64::INFINITY, f64::min)
        };
        let left_bound = min_s.min(centers[0] - default_w * 0.5) - 1e-9;
        edges.push(left_bound);
        for w in centers.windows(2) {
            edges.push(0.5 * (w[0] + w[1]));
        }
        let right_bound = max_s.max(centers[centers.len() - 1] + default_w * 0.5) + 1e-9;
        edges.push(right_bound.max(edges[edges.len() - 1] + 1e-9));
        for i in 0..centers.len() {
            let w = if centers.len() == 1 {
                default_w
            } else if i == 0 {
                centers[1] - centers[0]
            } else if i + 1 == centers.len() {
                centers[i] - centers[i - 1]
            } else {
                0.5 * (centers[i + 1] - centers[i - 1])
            };
            widths.push(w);
        }
        let inner = HistogramData::new(samples, edges)?;
        Ok(Self {
            inner,
            centers: centers.into(),
            widths: widths.into(),
        })
    }

    pub fn centers(&self) -> &[f64] {
        &self.centers
    }

    pub fn counts(&self) -> &[usize] {
        self.inner.counts()
    }

    pub fn histogram(&self) -> &HistogramData {
        &self.inner
    }

    pub fn to_bars(&self) -> BarData {
        let bars: Arc<[Bar]> = self
            .centers
            .iter()
            .zip(self.inner.counts().iter())
            .zip(self.widths.iter())
            .map(|((&c, &cnt), &w)| Bar::new(c, cnt as f64).width(w * 0.95).baseline(0.0))
            .collect();
        BarData::new(bars)
    }

    pub fn normalized_bars(&self, norm: HistogramNormalization) -> Result<BarData, HistogramError> {
        self.inner.to_bars(norm)
    }
}
