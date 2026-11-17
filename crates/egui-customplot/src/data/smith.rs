use std::sync::Arc;

use crate::axis::SmithTransform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmithError {
    Empty,
    LengthMismatch,
    NonFinite,
    InvalidReferenceImpedance,
}

impl std::fmt::Display for SmithError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("Smith trace inputs must not be empty"),
            Self::LengthMismatch => {
                f.write_str("frequency and reflection arrays must match length")
            }
            Self::NonFinite => f.write_str("Smith trace inputs must be finite"),
            Self::InvalidReferenceImpedance => {
                f.write_str("reference impedance Z0 must be finite and positive")
            }
        }
    }
}

impl std::error::Error for SmithError {}

#[derive(Clone, Debug)]
pub struct SmithTrace {
    pub name: String,
    pub freq_hz: Arc<[f64]>,
    pub gamma: Arc<[[f64; 2]]>,
    pub z0_ohms: f64,
}

impl SmithTrace {
    pub fn from_s_params(
        name: impl Into<String>,
        freq_hz: &[f64],
        gamma: &[[f64; 2]],
        z0_ohms: f64,
    ) -> Result<Self, SmithError> {
        if freq_hz.is_empty() {
            return Err(SmithError::Empty);
        }
        if freq_hz.len() != gamma.len() {
            return Err(SmithError::LengthMismatch);
        }
        if !z0_ohms.is_finite() || z0_ohms <= 0.0 {
            return Err(SmithError::InvalidReferenceImpedance);
        }
        if freq_hz.iter().any(|f| !f.is_finite())
            || gamma.iter().any(|g| !g[0].is_finite() || !g[1].is_finite())
        {
            return Err(SmithError::NonFinite);
        }
        Ok(Self {
            name: name.into(),
            freq_hz: Arc::from(freq_hz),
            gamma: Arc::from(gamma),
            z0_ohms,
        })
    }

    pub fn from_impedance(
        name: impl Into<String>,
        freq_hz: &[f64],
        z_ohms: &[[f64; 2]],
        z0_ohms: f64,
    ) -> Result<Self, SmithError> {
        if freq_hz.is_empty() {
            return Err(SmithError::Empty);
        }
        if freq_hz.len() != z_ohms.len() {
            return Err(SmithError::LengthMismatch);
        }
        if !z0_ohms.is_finite() || z0_ohms <= 0.0 {
            return Err(SmithError::InvalidReferenceImpedance);
        }
        let mut gamma = Vec::with_capacity(z_ohms.len());
        for &[r, x] in z_ohms {
            if !r.is_finite() || !x.is_finite() {
                return Err(SmithError::NonFinite);
            }
            let g = SmithTransform::z_to_gamma([r / z0_ohms, x / z0_ohms])
                .ok_or(SmithError::NonFinite)?;
            gamma.push(g);
        }
        Self::from_s_params(name, freq_hz, &gamma, z0_ohms)
    }

    pub fn len(&self) -> usize {
        self.gamma.len()
    }
    pub fn is_empty(&self) -> bool {
        self.gamma.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct SmithPlotData {
    traces: Arc<[SmithTrace]>,
    vswr_circles: Arc<[f64]>,
}

impl SmithPlotData {
    pub fn new(traces: Vec<SmithTrace>) -> Result<Self, SmithError> {
        if traces.is_empty() {
            return Err(SmithError::Empty);
        }
        Ok(Self {
            traces: traces.into(),
            vswr_circles: Arc::from([]),
        })
    }

    pub fn from_s11(freq_hz: &[f64], gamma: &[[f64; 2]]) -> Result<Self, SmithError> {
        Self::new(vec![SmithTrace::from_s_params(
            "S11", freq_hz, gamma, 50.0,
        )?])
    }

    pub fn from_impedance(
        freq_hz: &[f64],
        z_ohms: &[[f64; 2]],
        z0_ohms: f64,
    ) -> Result<Self, SmithError> {
        Self::new(vec![SmithTrace::from_impedance(
            "Z", freq_hz, z_ohms, z0_ohms,
        )?])
    }

    pub fn with_vswr_circles(mut self, vswrs: &[f64]) -> Self {
        let valid: Vec<f64> = vswrs
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 1.0)
            .collect();
        self.vswr_circles = valid.into();
        self
    }

    pub fn traces(&self) -> &[SmithTrace] {
        &self.traces
    }
    pub fn vswr_circles(&self) -> &[f64] {
        &self.vswr_circles
    }
    pub fn sample_bytes(&self) -> usize {
        self.traces
            .iter()
            .map(|t| (t.freq_hz.len() + t.gamma.len() * 2) * 8)
            .sum()
    }
}
