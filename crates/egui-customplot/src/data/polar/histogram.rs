use std::{
    f64::consts::{PI, TAU},
    sync::Arc,
};

use super::{PolarCurveData, PolarError};

/// Normalization modes matching MATLAB `polarhistogram` (`ML-046`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PolarHistNormalization {
    #[default]
    Count,
    Probability,
    CountDensity,
    Pdf,
    CumCount,
    Cdf,
}

/// Immutable angular histogram (`polarhistogram`, `ML-046`) wrapping finite angles modulo `2*pi`.
#[derive(Clone, Debug)]
pub struct PolarHistogramData {
    edges: Arc<[f64]>,
    counts: Arc<[usize]>,
    finite_total: usize,
    non_finite: usize,
}

impl PolarHistogramData {
    pub fn new(angles_rad: &[f64], nbins: usize) -> Result<Self, PolarError> {
        if nbins == 0 {
            return Err(PolarError::InvalidBins);
        }
        let step = TAU / nbins as f64;
        let edges: Vec<f64> = (0..=nbins).map(|i| i as f64 * step).collect();
        Self::with_edges(angles_rad, &edges)
    }

    pub fn with_edges(angles_rad: &[f64], edges: &[f64]) -> Result<Self, PolarError> {
        if edges.len() < 2
            || edges.iter().any(|e| !e.is_finite())
            || edges.windows(2).any(|w| w[1] <= w[0])
        {
            return Err(PolarError::InvalidEdges);
        }
        let span = edges[edges.len() - 1] - edges[0];
        if span > TAU + 1e-6 {
            return Err(PolarError::InvalidEdges);
        }
        let nbins = edges.len() - 1;
        let mut counts = vec![0usize; nbins];
        let mut finite_total = 0usize;
        let mut non_finite = 0usize;
        let origin = edges[0];
        let full_circle = (span - TAU).abs() <= 1e-6;
        for &theta in angles_rad {
            if !theta.is_finite() {
                non_finite += 1;
                continue;
            }
            let delta = (theta - origin).rem_euclid(TAU);
            let shifted = origin + delta;
            if shifted > edges[nbins] + 1e-12 && !full_circle {
                continue;
            }
            finite_total += 1;
            let idx = if full_circle && (delta >= TAU - 1e-12 || shifted >= edges[nbins]) {
                nbins - 1
            } else {
                match edges.binary_search_by(|e| e.total_cmp(&shifted)) {
                    Ok(exact) => exact.min(nbins - 1),
                    Err(ins) => ins.saturating_sub(1).min(nbins - 1),
                }
            };
            counts[idx] += 1;
        }
        Ok(Self {
            edges: Arc::from(edges),
            counts: Arc::from(counts),
            finite_total,
            non_finite,
        })
    }

    pub fn edges(&self) -> &[f64] {
        &self.edges
    }
    pub fn counts(&self) -> &[usize] {
        &self.counts
    }
    pub fn bin_count(&self) -> usize {
        self.counts.len()
    }
    pub fn finite_total(&self) -> usize {
        self.finite_total
    }
    pub fn non_finite(&self) -> usize {
        self.non_finite
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.edges.as_ref()) + std::mem::size_of_val(self.counts.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.counts.as_ptr() as usize, self.sample_bytes())
    }

    pub fn values(&self, norm: PolarHistNormalization) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.counts.len());
        let total = self.finite_total.max(1) as f64;
        let mut running = 0usize;
        for (i, &c) in self.counts.iter().enumerate() {
            running += c;
            let w = (self.edges[i + 1] - self.edges[i]).max(1e-12);
            let v = match norm {
                PolarHistNormalization::Count => c as f64,
                PolarHistNormalization::Probability => c as f64 / total,
                PolarHistNormalization::CountDensity => c as f64 / w,
                PolarHistNormalization::Pdf => c as f64 / (total * w),
                PolarHistNormalization::CumCount => running as f64,
                PolarHistNormalization::Cdf => running as f64 / total,
            };
            out.push(v);
        }
        out
    }

    pub fn max_value(&self, norm: PolarHistNormalization) -> f64 {
        self.values(norm)
            .into_iter()
            .fold(0.0_f64, |acc, v| acc.max(v))
    }
}

/// Legacy MATLAB `rose` (`LG-006`) radial sector histogram, producing closed petal polylines from origin.
#[derive(Clone, Debug)]
pub struct RoseData {
    edges: Arc<[f64]>,
    counts: Arc<[usize]>,
    non_finite: usize,
}

impl RoseData {
    pub fn new(angles_rad: &[f64], nbins: usize) -> Result<Self, PolarError> {
        let inner = PolarHistogramData::new(angles_rad, nbins)?;
        Ok(Self {
            edges: Arc::from(inner.edges()),
            counts: Arc::from(inner.counts()),
            non_finite: inner.non_finite(),
        })
    }

    pub fn with_centers(angles_rad: &[f64], centers_rad: &[f64]) -> Result<Self, PolarError> {
        if centers_rad.is_empty() || centers_rad.iter().any(|c| !c.is_finite()) {
            return Err(PolarError::InvalidEdges);
        }
        let mut sorted: Vec<f64> = centers_rad.iter().map(|c| c.rem_euclid(TAU)).collect();
        sorted.sort_by(f64::total_cmp);
        sorted.dedup_by(|a, b| (*a - *b).abs() <= 1e-9);
        let n = sorted.len();
        let mut counts = vec![0usize; n];
        let mut non_finite = 0usize;
        for &theta in angles_rad {
            if !theta.is_finite() {
                non_finite += 1;
                continue;
            }
            let wrapped = theta.rem_euclid(TAU);
            let mut best = 0usize;
            let mut best_dist = f64::INFINITY;
            for (i, &c) in sorted.iter().enumerate() {
                let d = ((wrapped - c + PI).rem_euclid(TAU) - PI).abs();
                if d < best_dist {
                    best_dist = d;
                    best = i;
                }
            }
            counts[best] += 1;
        }
        let mut edges = Vec::with_capacity(n + 1);
        for i in 0..n {
            let prev = sorted[(i + n - 1) % n];
            let cur = sorted[i];
            let gap = (cur - prev).rem_euclid(TAU);
            let left = cur - 0.5 * if gap > 1e-9 { gap } else { TAU / n as f64 };
            edges.push(left);
        }
        let first_next = sorted[0] + TAU;
        let last = sorted[n - 1];
        let last_edge = edges[0] + TAU;
        let _ = (first_next, last);
        edges.push(last_edge);
        Ok(Self {
            edges: Arc::from(edges),
            counts: Arc::from(counts),
            non_finite,
        })
    }

    pub fn edges(&self) -> &[f64] {
        &self.edges
    }
    pub fn counts(&self) -> &[usize] {
        &self.counts
    }
    pub fn non_finite(&self) -> usize {
        self.non_finite
    }

    /// Generates classic `rose` radial petals `(e0, 0) -> (e0, count) -> arc -> (e1, count) -> (e1, 0)`.
    pub fn to_polar_curve(&self) -> PolarCurveData {
        let mut pts = Vec::with_capacity(self.counts.len() * 6);
        for (i, &c) in self.counts.iter().enumerate() {
            let r = c as f64;
            let e0 = self.edges[i];
            let e1 = self.edges[i + 1];
            pts.push([e0, 0.0]);
            pts.push([e0, r]);
            if r > 0.0 {
                let steps = ((e1 - e0).abs() / 0.12).ceil().clamp(1.0, 16.0) as usize;
                for s in 1..=steps {
                    let t = s as f64 / steps as f64;
                    pts.push([e0 + (e1 - e0) * t, r]);
                }
            } else {
                pts.push([e1, 0.0]);
            }
            pts.push([e1, 0.0]);
        }
        PolarCurveData::new(pts)
    }
}
