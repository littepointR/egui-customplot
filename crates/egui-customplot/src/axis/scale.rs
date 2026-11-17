/// Coordinate semantics, shared by drawing, ticks, navigation and picking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AxisScale {
    #[default]
    Linear,
    Log10,
    /// Normal probability scale `Phi^{-1}(p)` on open interval `(0, 1)` (`EXT-13`).
    Probit,
    /// Weibull probability scale `ln(-ln(1 - p))` on open interval `(0, 1)` (`EXT-13`).
    Weibull,
}

impl AxisScale {
    pub fn is_probability(self) -> bool {
        matches!(self, Self::Probit | Self::Weibull)
    }

    pub fn forward(self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        match self {
            Self::Linear => Some(value),
            Self::Log10 => (value > 0.0).then(|| value.log10()),
            Self::Probit => super::probability::norm_inv(value),
            Self::Weibull => super::probability::weibull_forward(value),
        }
    }

    pub fn inverse(self, value: f64) -> Option<f64> {
        let result = match self {
            Self::Linear => Some(value),
            Self::Log10 => {
                let r = 10.0_f64.powf(value);
                (r.is_finite() && r > 0.0).then_some(r)
            }
            Self::Probit => super::probability::norm_cdf_clamped(value),
            Self::Weibull => super::probability::weibull_inverse(value),
        }?;
        result.is_finite().then_some(result)
    }

    pub(crate) fn default_range(self) -> [f64; 2] {
        match self {
            Self::Linear => [-1.0, 1.0],
            Self::Log10 => [0.1, 10.0],
            Self::Probit | Self::Weibull => [0.01, 0.99],
        }
    }
}
