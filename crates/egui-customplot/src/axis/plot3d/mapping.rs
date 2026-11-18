use crate::{AxisScale, BoundsError};

/// A pure, strictly increasing pair of transforms over a built-in validity domain.
/// This keeps host data summaries usable across views. Explicit ticks are expressed in data units.
#[derive(Clone, Copy, Debug)]
pub struct ScaleMapping3D {
    domain: AxisScale,
    functions: Option<ScaleFunctions>,
}

#[derive(Clone, Copy, Debug)]
struct ScaleFunctions {
    forward: fn(f64) -> Option<f64>,
    inverse: fn(f64) -> Option<f64>,
}

impl From<AxisScale> for ScaleMapping3D {
    fn from(domain: AxisScale) -> Self {
        Self {
            domain,
            functions: None,
        }
    }
}

impl ScaleMapping3D {
    /// Functions must be deterministic mutual inverses throughout the chosen domain.
    pub fn new(
        domain: AxisScale,
        forward: fn(f64) -> Option<f64>,
        inverse: fn(f64) -> Option<f64>,
    ) -> Result<Self, BoundsError> {
        let result = Self {
            domain,
            functions: Some(ScaleFunctions { forward, inverse }),
        };
        result
            .valid_range(domain.default_range())
            .then_some(result)
            .ok_or(BoundsError)
    }
    pub fn domain(self) -> AxisScale {
        self.domain
    }
    pub fn forward(self, x: f64) -> Option<f64> {
        if let Some(f) = self.functions {
            self.domain.forward(x)?;
            (f.forward)(x).filter(|v| v.is_finite())
        } else {
            self.domain.forward(x)
        }
    }
    pub fn inverse(self, x: f64) -> Option<f64> {
        if let Some(f) = self.functions {
            let value = (f.inverse)(x)?;
            self.domain.forward(value)?;
            Some(value)
        } else {
            self.domain.inverse(x)
        }
    }
    pub(crate) fn function_key(self) -> Option<[usize; 2]> {
        self.functions
            .map(|f| [f.forward as usize, f.inverse as usize])
    }
    pub(crate) fn valid_range(self, [a, b]: [f64; 2]) -> bool {
        (b - a).is_finite()
            && self
                .forward(a)
                .zip(self.forward(b))
                .is_some_and(|(x, y)| y > x && (y - x).is_finite())
    }

    pub(crate) fn padded_range(self, extent: Option<[f64; 2]>) -> [f64; 2] {
        if let Some([a, b]) = extent
            && let Some((x, y)) = self.forward(a).zip(self.forward(b))
        {
            let pad = if self.functions.is_none() && self.domain == AxisScale::Log10 {
                1.25_f64.log10()
            } else if y - x <= 1e-12 {
                0.5
            } else {
                (y - x) * 0.06
            };
            let range = [
                self.inverse(x - pad).unwrap_or(a).min(a),
                self.inverse(y + pad).unwrap_or(b).max(b),
            ];
            if self.valid_range(range) {
                return range;
            }
            if self.valid_range([a, b]) {
                return [a, b];
            }
        }
        self.domain.default_range()
    }
}
