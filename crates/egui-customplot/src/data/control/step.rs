use crate::{PlotData, TransferFunction};

/// Input errors for a continuous, zero-initial-state unit-step response.
#[derive(Debug, PartialEq, Eq)]
pub enum StepResponseError {
    InvalidTimeGrid,
    UnsupportedSystem,
    NonFiniteResponse,
}

impl std::fmt::Display for StepResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidTimeGrid => "expected a positive finite duration and 2..=1,000,001 samples",
            Self::UnsupportedSystem => "expected a stable, proper real transfer function of order one or two with zero delay",
            Self::NonFiniteResponse => "step response exceeds the numeric range",
        })
    }
}
impl std::error::Error for StepResponseError {}

/// Sampled unit-step response of a stable first- or second-order continuous system.
/// The grid includes t=0+ (including direct feedthrough) and the requested end time.
#[derive(Clone, Debug)]
pub struct StepResponseData {
    curve: PlotData,
    steady_state: f64,
    overshoot_percent: Option<f64>,
    settling_time: Option<f64>,
}

impl StepResponseData {
    pub fn new(
        tf: &TransferFunction,
        end_seconds: f64,
        samples: usize,
    ) -> Result<Self, StepResponseError> {
        if !end_seconds.is_finite() || end_seconds <= 0.0 || !(2..=1_000_001).contains(&samples) {
            return Err(StepResponseError::InvalidTimeGrid);
        }
        let den = tf.den();
        let num = tf.num();
        if tf.delay_sec() != 0.0 || num.len() > den.len() || !(2..=3).contains(&den.len()) {
            return Err(StepResponseError::UnsupportedSystem);
        }
        let lead = den[0];
        let direct = if num.len() == den.len() {
            num[0] / lead
        } else {
            0.0
        };
        let steady = num[num.len() - 1] / den[den.len() - 1];
        let decay = den[1] / lead;
        let constant = den[den.len() - 1] / lead;
        if decay <= 0.0 || constant <= 0.0 {
            return Err(StepResponseError::UnsupportedSystem);
        }
        if ![steady, direct, decay, constant]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(StepResponseError::NonFiniteResponse);
        }
        let excursion = steady - direct;
        let mut points = Vec::with_capacity(samples);
        let mut max_excess = 0.0_f64;
        let mut last_outside = None;
        // Initial derivative from the coefficient of 1/s in the expansion of G(s).
        let slope = num.iter().rev().nth(1).copied().unwrap_or(0.0) / lead - direct * decay;
        for i in 0..samples {
            let t = end_seconds * (i as f64 / (samples - 1) as f64);
            let y = if i == 0 {
                direct
            } else if den.len() == 2 {
                direct - excursion * (-decay * t).exp_m1()
            } else {
                steady + transient(t, decay * 0.5, constant, -excursion, slope)
            };
            if !y.is_finite() {
                return Err(StepResponseError::NonFiniteResponse);
            }
            if excursion != 0.0 {
                let excess = (y - steady) / excursion;
                max_excess = max_excess.max(excess);
                if excess.abs() > 0.02 {
                    last_outside = Some(i);
                }
            }
            points.push([t, y]);
        }
        let settling_time = last_outside.and_then(|i| points.get(i + 1)).map(|p| p[0]);
        Ok(Self {
            curve: PlotData::new(points),
            steady_state: steady,
            overshoot_percent: (excursion != 0.0).then_some(100.0 * max_excess),
            settling_time,
        })
    }

    pub fn curve(&self) -> &PlotData {
        &self.curve
    }
    pub fn steady_state(&self) -> f64 {
        self.steady_state
    }
    /// Peak excess on this grid relative to the signed step excursion.
    pub fn overshoot_percent(&self) -> Option<f64> {
        self.overshoot_percent
    }
    /// First sample after the last excursion outside the 2% band on the observed grid.
    /// None when the observation ends outside the band or the step excursion is zero.
    pub fn settling_time(&self) -> Option<f64> {
        self.settling_time
    }
}

// Solve x'' + 2*a*x' + c*x = 0 from the initial displacement and velocity.
fn transient(t: f64, a: f64, c: f64, x: f64, v: f64) -> f64 {
    let disc = a.mul_add(a, -c);
    if disc == 0.0 {
        return (-a * t).exp() * (x + (v + a * x) * t);
    }
    if disc < 0.0 {
        let w = (-disc).sqrt();
        let (sin, cos) = (w * t).sin_cos();
        return (-a * t).exp() * (x * cos + (v + a * x) * (sin / w));
    }
    let gap = 2.0 * disc.sqrt();
    let fast = -a - disc.sqrt();
    let slow = c / fast;
    (slow * t).exp() * (x * (-gap * t).exp() - (v - fast * x) * (-gap * t).exp_m1() / gap)
}
