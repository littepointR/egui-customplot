use std::sync::Arc;

use super::control::{ControlError, poly_roots_aberth};
use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComplexRoot {
    pub re: f64,
    pub im: f64,
    pub multiplicity: usize,
}

impl ComplexRoot {
    pub fn magnitude(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn angle_rad(self) -> f64 {
        self.im.atan2(self.re)
    }
}

/// Discrete/continuous zero-pole dataset (`zplane`, `EXT-20`) with root multiplicity clustering.
#[derive(Clone, Debug)]
pub struct PoleZeroData {
    zeros: Arc<[ComplexRoot]>,
    poles: Arc<[ComplexRoot]>,
    discrete: bool,
    stable: bool,
}

impl PoleZeroData {
    pub fn from_zeros_poles(zeros: &[[f64; 2]], poles: &[[f64; 2]]) -> Result<Self, ControlError> {
        Self::from_roots_with_tol(zeros, poles, 1e-4, true)
    }

    pub fn from_roots_with_tol(
        zeros: &[[f64; 2]],
        poles: &[[f64; 2]],
        tol: f64,
        discrete: bool,
    ) -> Result<Self, ControlError> {
        if zeros.is_empty() && poles.is_empty() {
            return Err(ControlError::Empty);
        }
        if zeros
            .iter()
            .chain(poles.iter())
            .any(|p| !p[0].is_finite() || !p[1].is_finite())
            || !tol.is_finite()
            || tol <= 0.0
        {
            return Err(ControlError::NonFinite);
        }
        Ok(Self {
            zeros: cluster_roots(zeros, tol).into(),
            poles: cluster_roots(poles, tol).into(),
            discrete,
            stable: poles.iter().all(|&[re, im]| {
                if discrete {
                    re.hypot(im) < 1.0
                } else {
                    re < 0.0
                }
            }),
        })
    }

    /// MATLAB `zplane(b, a)` convention: `H(z) = sum(b_k z^-k) / sum(a_k z^-k)`
    /// padded with zeros at `z = 0` when numerator and denominator degrees differ.
    pub fn from_transfer_fn(b: &[f64], a: &[f64]) -> Result<Self, ControlError> {
        if b.is_empty() || a.is_empty() {
            return Err(ControlError::Empty);
        }
        if b.iter().chain(a.iter()).any(|v| !v.is_finite()) {
            return Err(ControlError::NonFinite);
        }
        let max_len = b.len().max(a.len());
        let mut b_pad = b.to_vec();
        let mut a_pad = a.to_vec();
        b_pad.resize(max_len, 0.0);
        a_pad.resize(max_len, 0.0);
        let zeros = poly_roots_aberth(&b_pad)?;
        let poles = poly_roots_aberth(&a_pad)?;
        Self::from_roots_with_tol(&zeros, &poles, 1e-4, true)
    }

    pub fn is_discrete(&self) -> bool {
        self.discrete
    }
    pub fn zeros(&self) -> &[ComplexRoot] {
        &self.zeros
    }
    pub fn poles(&self) -> &[ComplexRoot] {
        &self.poles
    }
    pub fn is_causal_stable(&self) -> bool {
        self.stable
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.zeros) + std::mem::size_of_val(&*self.poles)
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [-1.15_f64; 2];
        let mut max = [1.15_f64; 2];
        for r in self.zeros.iter().chain(self.poles.iter()) {
            if scales[0].forward(r.re).is_none() || scales[1].forward(r.im).is_none() {
                continue;
            }
            min[0] = min[0].min(r.re - 0.15);
            max[0] = max[0].max(r.re + 0.15);
            min[1] = min[1].min(r.im - 0.15);
            max[1] = max[1].max(r.im + 0.15);
        }
        if scales[0] != AxisScale::Linear || scales[1] != AxisScale::Linear {
            min = [f64::INFINITY; 2];
            max = [f64::NEG_INFINITY; 2];
            let mut count = 0usize;
            for r in self.zeros.iter().chain(self.poles.iter()) {
                if scales[0].forward(r.re).is_some() && scales[1].forward(r.im).is_some() {
                    min[0] = min[0].min(r.re);
                    max[0] = max[0].max(r.re);
                    min[1] = min[1].min(r.im);
                    max[1] = max[1].max(r.im);
                    count += 1;
                }
            }
            if count == 0 {
                return None;
            }
        }
        PlotBounds::new(min, max).ok()
    }
}

fn cluster_roots(roots: &[[f64; 2]], tol: f64) -> Vec<ComplexRoot> {
    let mut clusters: Vec<ComplexRoot> = Vec::new();
    for &[re, im] in roots {
        if let Some(existing) = clusters
            .iter_mut()
            .find(|c| (c.re - re).hypot(c.im - im) <= tol)
        {
            let m = existing.multiplicity as f64;
            existing.re = (existing.re * m + re) / (m + 1.0);
            existing.im = (existing.im * m + im) / (m + 1.0);
            existing.multiplicity += 1;
        } else {
            clusters.push(ComplexRoot {
                re,
                im,
                multiplicity: 1,
            });
        }
    }
    clusters
}
