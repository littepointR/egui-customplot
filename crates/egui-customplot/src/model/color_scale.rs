use egui::Color32;

use crate::AxisScale;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMap {
    #[default]
    Viridis,
    Parula,
    Turbo,
    Inferno,
    Plasma,
    Magma,
    Cividis,
    Hot,
    Coolwarm,
    Gray,
    BlueRed,
}

impl ColorMap {
    pub const ALL: [Self; 11] = [
        Self::Viridis,
        Self::Parula,
        Self::Turbo,
        Self::Inferno,
        Self::Plasma,
        Self::Magma,
        Self::Cividis,
        Self::Hot,
        Self::Coolwarm,
        Self::Gray,
        Self::BlueRed,
    ];

    /// Piecewise interpolation in opaque sRGB; not a perceptual-uniformity claim.
    pub fn sample(self, fraction: f64) -> Color32 {
        let stops: &[[u8; 3]] = match self {
            Self::Viridis => &[
                [68, 1, 84],
                [59, 82, 139],
                [33, 145, 140],
                [94, 201, 98],
                [253, 231, 37],
            ],
            Self::Parula => &[
                [53, 42, 135],
                [15, 92, 221],
                [18, 125, 216],
                [7, 156, 207],
                [21, 177, 180],
                [89, 189, 140],
                [165, 190, 107],
                [225, 185, 82],
                [249, 251, 14],
            ],
            Self::Turbo => &[
                [48, 18, 59],
                [70, 107, 227],
                [40, 188, 235],
                [50, 242, 152],
                [164, 252, 60],
                [238, 207, 58],
                [251, 126, 33],
                [208, 47, 5],
                [122, 4, 3],
            ],
            Self::Inferno => &[
                [0, 0, 4],
                [31, 12, 72],
                [85, 15, 109],
                [136, 34, 106],
                [186, 54, 85],
                [227, 89, 51],
                [249, 140, 10],
                [249, 201, 50],
                [252, 255, 164],
            ],
            Self::Plasma => &[
                [13, 8, 135],
                [75, 3, 161],
                [125, 3, 168],
                [168, 34, 150],
                [203, 70, 121],
                [229, 107, 93],
                [248, 148, 65],
                [253, 195, 40],
                [240, 249, 33],
            ],
            Self::Magma => &[
                [0, 0, 4],
                [28, 16, 68],
                [79, 18, 123],
                [129, 37, 129],
                [181, 54, 122],
                [229, 80, 100],
                [251, 135, 97],
                [254, 194, 135],
                [252, 253, 191],
            ],
            Self::Cividis => &[
                [0, 32, 76],
                [0, 54, 108],
                [42, 74, 108],
                [76, 94, 107],
                [107, 114, 110],
                [139, 136, 115],
                [174, 160, 114],
                [212, 187, 102],
                [253, 234, 69],
            ],
            Self::Hot => &[
                [10, 0, 0],
                [85, 0, 0],
                [170, 0, 0],
                [255, 0, 0],
                [255, 85, 0],
                [255, 170, 0],
                [255, 255, 0],
                [255, 255, 128],
                [255, 255, 255],
            ],
            Self::Coolwarm => &[
                [59, 76, 192],
                [98, 130, 234],
                [141, 176, 254],
                [184, 208, 249],
                [221, 221, 221],
                [245, 196, 173],
                [244, 154, 123],
                [222, 96, 77],
                [180, 4, 38],
            ],
            Self::Gray => &[[0; 3], [64; 3], [128; 3], [191; 3], [255; 3]],
            Self::BlueRed => &[
                [33, 102, 172],
                [146, 197, 222],
                [247, 247, 247],
                [244, 165, 130],
                [178, 24, 43],
            ],
        };
        let last = stops.len() - 1;
        let t = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        } * last as f64;
        let index = (t.floor() as usize).min(last - 1);
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
