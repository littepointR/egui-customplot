use std::sync::Arc;

use super::poly::{ControlError, TransferFunction, poly_roots_aberth};
use crate::{AxisScale, PlotBounds, PlotData, PlotPoint};

/// Evans root locus of `1 + K * G(s) = 0` (`rlocus`, `EXT-21`).
#[derive(Clone, Debug)]
pub struct RootLocusData {
    gains: Arc<[f64]>,
    open_loop_poles: Arc<[[f64; 2]]>,
    open_loop_zeros: Arc<[[f64; 2]]>,
    branches: Arc<[PlotData]>,
}

impl RootLocusData {
    pub fn auto_gains(
        tf: &TransferFunction,
        k_max: f64,
        n_steps: usize,
    ) -> Result<Self, ControlError> {
        if !k_max.is_finite() || k_max <= 0.0 {
            return Err(ControlError::InvalidFrequencyRange);
        }
        let n = n_steps.clamp(12, 1024);
        let mut gains = Vec::with_capacity(n);
        gains.push(0.0);
        let l_lo = (1e-4_f64).ln();
        let l_hi = k_max.ln();
        for i in 1..n {
            let t = (i - 1) as f64 / (n - 2).max(1) as f64;
            gains.push((l_lo + (l_hi - l_lo) * t).exp());
        }
        Self::from_transfer_fn(tf, &gains)
    }

    /// Returns `InvalidPolynomial` for a nonzero delay, whose characteristic equation is transcendental.
    pub fn from_transfer_fn(tf: &TransferFunction, gains: &[f64]) -> Result<Self, ControlError> {
        if gains.is_empty() {
            return Err(ControlError::Empty);
        }
        if gains.iter().any(|g| !g.is_finite() || *g < 0.0) {
            return Err(ControlError::NonFinite);
        }

        if tf.delay_sec() != 0.0 {
            return Err(ControlError::InvalidPolynomial);
        }

        let num = tf.num();
        let den = tf.den();
        let open_loop_poles = poly_roots_aberth(den)?;
        let open_loop_zeros = poly_roots_aberth(num)?;
        if open_loop_poles.is_empty() {
            return Err(ControlError::InvalidPolynomial);
        }

        let deg = den.len().max(num.len());
        let mut prev_roots = Vec::new();
        let mut per_branch: Vec<Vec<PlotPoint>> = Vec::new();

        for &k in gains {
            let mut poly = vec![0.0_f64; deg];
            let d_off = deg - den.len();
            for (i, &c) in den.iter().enumerate() {
                poly[d_off + i] += c;
            }
            let n_off = deg - num.len();
            for (i, &c) in num.iter().enumerate() {
                poly[n_off + i] += k * c;
            }
            let curr = poly_roots_aberth(&poly)?;
            if curr.is_empty() {
                continue;
            }
            if prev_roots.is_empty() {
                per_branch = curr.iter().map(|&r| vec![r]).collect();
                prev_roots = curr;
                continue;
            }
            let n_b = per_branch.len().min(curr.len());
            let matched = match_roots(&prev_roots, &curr);
            for (b_idx, r_idx) in matched.into_iter().take(n_b).enumerate() {
                per_branch[b_idx].push(curr[r_idx]);
            }
            prev_roots = curr;
        }

        let branches: Vec<PlotData> = per_branch
            .into_iter()
            .filter(|pts| !pts.is_empty())
            .map(PlotData::new_unindexed)
            .collect();

        Ok(Self {
            gains: Arc::from(gains),
            open_loop_poles: open_loop_poles.into(),
            open_loop_zeros: open_loop_zeros.into(),
            branches: branches.into(),
        })
    }

    pub fn gains(&self) -> &[f64] {
        &self.gains
    }
    pub fn open_loop_poles(&self) -> &[[f64; 2]] {
        &self.open_loop_poles
    }
    pub fn open_loop_zeros(&self) -> &[[f64; 2]] {
        &self.open_loop_zeros
    }
    pub fn branches(&self) -> &[PlotData] {
        &self.branches
    }
    pub fn sample_bytes(&self) -> usize {
        self.branches.iter().map(PlotData::sample_bytes).sum()
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for pt in self
            .open_loop_poles
            .iter()
            .chain(self.open_loop_zeros.iter())
            .copied()
            .chain(
                self.branches
                    .iter()
                    .flat_map(|b| b.points().iter().copied()),
            )
        {
            if scales[0].forward(pt[0]).is_none() || scales[1].forward(pt[1]).is_none() {
                continue;
            }
            min[0] = min[0].min(pt[0]);
            max[0] = max[0].max(pt[0]);
            min[1] = min[1].min(pt[1]);
            max[1] = max[1].max(pt[1]);
            count += 1;
        }
        (count > 0)
            .then(|| PlotBounds::new(min, max).ok())
            .flatten()
    }
}

fn match_roots(prev: &[[f64; 2]], curr: &[[f64; 2]]) -> Vec<usize> {
    let mut used = vec![false; curr.len()];
    let mut assign = Vec::with_capacity(prev.len());
    for &p in prev {
        let mut best_idx = 0usize;
        let mut best_d = f64::INFINITY;
        for (j, &c) in curr.iter().enumerate() {
            if !used[j] {
                let d = (p[0] - c[0]).hypot(p[1] - c[1]);
                if d < best_d {
                    best_d = d;
                    best_idx = j;
                }
            }
        }
        if best_idx < used.len() {
            used[best_idx] = true;
            assign.push(best_idx);
        }
    }
    assign
}
