/// Coordinate semantics, shared by drawing, ticks, navigation and picking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AxisScale {
    #[default]
    Linear,
    Log10,
}

impl AxisScale {
    pub fn forward(self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        match self {
            Self::Linear => Some(value),
            Self::Log10 => (value > 0.0).then(|| value.log10()),
        }
    }

    pub fn inverse(self, value: f64) -> Option<f64> {
        let result = match self {
            Self::Linear => value,
            Self::Log10 => 10.0_f64.powf(value),
        };
        (result.is_finite() && (self == Self::Linear || result > 0.0)).then_some(result)
    }

    pub(crate) fn default_range(self) -> [f64; 2] {
        match self {
            Self::Linear => [-1.0, 1.0],
            Self::Log10 => [0.1, 10.0],
        }
    }
}
