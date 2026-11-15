use egui::Color32;

use crate::AxisScale;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMap {
    #[default]
    Viridis,
    Gray,
    BlueRed,
}

impl ColorMap {
    /// Piecewise interpolation in opaque sRGB; not a perceptual-uniformity claim.
    pub fn sample(self, fraction: f64) -> Color32 {
        let stops = match self {
            Self::Viridis => [
                [68, 1, 84],
                [59, 82, 139],
                [33, 145, 140],
                [94, 201, 98],
                [253, 231, 37],
            ],
            Self::Gray => [[0; 3], [64; 3], [128; 3], [191; 3], [255; 3]],
            Self::BlueRed => [
                [33, 102, 172],
                [146, 197, 222],
                [247, 247, 247],
                [244, 165, 130],
                [178, 24, 43],
            ],
        };
        let t = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        } * 4.0;
        let index = (t.floor() as usize).min(3);
        let blend = t - index as f64;
        let rgb: [u8; 3] = std::array::from_fn(|channel| {
            (f64::from(stops[index][channel]) * (1.0 - blend)
                + f64::from(stops[index + 1][channel]) * blend)
                .round() as u8
        });
        Color32::from_rgb(rgb[0], rgb[1], rgb[2])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorScaleError;
impl std::fmt::Display for ColorScaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("color range must be finite, increasing and valid in its scale")
    }
}
impl std::error::Error for ColorScaleError {}

/// Shared numeric mapping for heatmaps and color bars. Outliers clamp; missing
/// values and nonpositive log values return None and produce no filled cell.
#[derive(Clone, Copy, Debug)]
pub struct ColorScale {
    range: [f64; 2],
    scale: AxisScale,
    map: ColorMap,
    transformed: [f64; 2],
}

impl ColorScale {
    pub fn new(range: [f64; 2]) -> Result<Self, ColorScaleError> {
        Self::with_scale(range, AxisScale::Linear)
    }
    pub fn with_scale(range: [f64; 2], scale: AxisScale) -> Result<Self, ColorScaleError> {
        let transformed = [
            scale.forward(range[0]).ok_or(ColorScaleError)?,
            scale.forward(range[1]).ok_or(ColorScaleError)?,
        ];
        let span = transformed[1] - transformed[0];
        if !span.is_finite() || span <= 0.0 {
            return Err(ColorScaleError);
        }
        Ok(Self {
            range,
            scale,
            map: ColorMap::default(),
            transformed,
        })
    }
    pub fn color_map(mut self, map: ColorMap) -> Self {
        self.map = map;
        self
    }
    pub fn range(self) -> [f64; 2] {
        self.range
    }
    pub fn scale(self) -> AxisScale {
        self.scale
    }
    pub fn map(self) -> ColorMap {
        self.map
    }
    pub fn fraction(self, value: f64) -> Option<f64> {
        let value = self.scale.forward(value)?;
        // Clamp before subtracting so extreme finite outliers cannot overflow.
        Some(
            (value.clamp(self.transformed[0], self.transformed[1]) - self.transformed[0])
                / (self.transformed[1] - self.transformed[0]),
        )
    }
    pub fn color(self, value: f64) -> Option<Color32> {
        self.fraction(value).map(|t| self.map.sample(t))
    }
}
