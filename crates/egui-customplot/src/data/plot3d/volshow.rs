use crate::ColorMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VolumeRenderMode {
    DirectAlpha,
    MaximumIntensity,
    IsosurfaceRay { threshold: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferStop {
    pub position: f64,
    pub color: [u8; 3],
    pub alpha: f32,
}

/// Compact piecewise-linear RGBA transfer-function LUT for direct volume raycasting (`volshow`, `EXT-23`).
#[derive(Clone, Copy, Debug)]
pub struct VolumeTransferFunction {
    lut: [[u8; 4]; 32],
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Default for VolumeTransferFunction {
    fn default() -> Self {
        Self::ramp(ColorMap::Viridis, 0.0, 0.85)
    }
}

impl VolumeTransferFunction {
    pub fn ramp(map: ColorMap, min_alpha: f32, max_alpha: f32) -> Self {
        let a0 = min_alpha.clamp(0.0, 1.0);
        let a1 = max_alpha.clamp(0.0, 1.0);
        let lut = std::array::from_fn(|i| {
            let t = i as f64 / 31.0;
            let c = map.sample(t);
            let a = a0 + (a1 - a0) * (t as f32).powf(1.3);
            [c.r(), c.g(), c.b(), to_u8(a)]
        });
        Self { lut }
    }

    pub fn gaussian_band(map: ColorMap, center: f64, width: f64, peak_alpha: f32) -> Self {
        let w = width.abs().max(0.02);
        let peak = peak_alpha.clamp(0.0, 1.0);
        let lut = std::array::from_fn(|i| {
            let t = i as f64 / 31.0;
            let c = map.sample(t);
            let d = (t - center) / w;
            let a = peak * (-0.5 * d * d).exp() as f32;
            [c.r(), c.g(), c.b(), to_u8(a)]
        });
        Self { lut }
    }

    pub fn from_stops(stops: &[TransferStop]) -> Self {
        if stops.is_empty() {
            return Self::default();
        }
        let mut sorted = stops.to_vec();
        sorted.sort_by(|a, b| a.position.total_cmp(&b.position));
        let lut = std::array::from_fn(|i| {
            let t = i as f64 / 31.0;
            if t <= sorted[0].position {
                let s = &sorted[0];
                return [s.color[0], s.color[1], s.color[2], to_u8(s.alpha)];
            }
            let last = &sorted[sorted.len() - 1];
            if t >= last.position {
                return [
                    last.color[0],
                    last.color[1],
                    last.color[2],
                    to_u8(last.alpha),
                ];
            }
            for w in sorted.windows(2) {
                if t <= w[1].position {
                    let span = (w[1].position - w[0].position).max(1e-12);
                    let f = ((t - w[0].position) / span) as f32;
                    let c = |a: u8, b: u8| {
                        (f32::from(a) + (f32::from(b) - f32::from(a)) * f)
                            .clamp(0.0, 255.0)
                            .round() as u8
                    };
                    return [
                        c(w[0].color[0], w[1].color[0]),
                        c(w[0].color[1], w[1].color[1]),
                        c(w[0].color[2], w[1].color[2]),
                        to_u8(w[0].alpha + (w[1].alpha - w[0].alpha) * f),
                    ];
                }
            }
            [0; 4]
        });
        Self { lut }
    }

    #[inline]
    pub(crate) fn lut_bytes(&self) -> &[[u8; 4]; 32] {
        &self.lut
    }

    #[inline]
    pub fn sample_norm(&self, t: f64) -> [f32; 4] {
        let pos = t.clamp(0.0, 1.0) * 31.0;
        let idx = (pos as usize).min(30);
        let f = (pos as f32) - idx as f32;
        let a = self.lut[idx];
        let b = self.lut[idx + 1];
        let lerp = |c0: u8, c1: u8| (f32::from(c0) + (f32::from(c1) - f32::from(c0)) * f) / 255.0;
        [
            lerp(a[0], b[0]),
            lerp(a[1], b[1]),
            lerp(a[2], b[2]),
            lerp(a[3], b[3]),
        ]
    }
}
