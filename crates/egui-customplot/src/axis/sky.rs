use egui::Pos2;

/// Zenith-centered azimuthal projection (`skyplot`, `EXT-07`).
/// Elevation `90°` maps to `center`; `min_elevation_deg` maps to outer ring `radius_px`.
/// Azimuth `0°` points North (top, `-y`), increasing clockwise (`90°` = East, `+x`).
#[derive(Clone, Copy, Debug)]
pub struct SkyTransform {
    center: Pos2,
    radius_px: f32,
    min_elevation_deg: f64,
}

impl SkyTransform {
    pub fn new(center: Pos2, radius_px: f32, min_elevation_deg: f64) -> Self {
        Self {
            center,
            radius_px: radius_px.max(1.0),
            min_elevation_deg: if min_elevation_deg.is_finite() {
                min_elevation_deg.clamp(0.0, 85.0)
            } else {
                0.0
            },
        }
    }

    pub fn center(self) -> Pos2 {
        self.center
    }
    pub fn radius_px(self) -> f32 {
        self.radius_px
    }
    pub fn min_elevation_deg(self) -> f64 {
        self.min_elevation_deg
    }

    pub fn radius_fraction(self, elevation_deg: f64) -> Option<f64> {
        if !elevation_deg.is_finite()
            || elevation_deg < self.min_elevation_deg
            || elevation_deg > 90.0
        {
            return None;
        }
        Some((90.0 - elevation_deg) / (90.0 - self.min_elevation_deg))
    }

    pub fn project(self, azimuth_deg: f64, elevation_deg: f64) -> Option<Pos2> {
        if !azimuth_deg.is_finite() {
            return None;
        }
        let r_norm = self.radius_fraction(elevation_deg)?;
        let r_px = r_norm * f64::from(self.radius_px);
        let rad = azimuth_deg.to_radians();
        let x = self.center.x + (r_px * rad.sin()) as f32;
        let y = self.center.y - (r_px * rad.cos()) as f32;
        (x.is_finite() && y.is_finite()).then_some(Pos2::new(x, y))
    }

    pub fn unproject(self, pos: Pos2) -> Option<[f64; 2]> {
        let dx = f64::from(pos.x - self.center.x);
        let dy = f64::from(pos.y - self.center.y);
        let dist = dx.hypot(dy);
        if dist > f64::from(self.radius_px) + 0.5 {
            return None;
        }
        let r_norm = (dist / f64::from(self.radius_px)).clamp(0.0, 1.0);
        let elevation = 90.0 - r_norm * (90.0 - self.min_elevation_deg);
        let azimuth = if dist < 1e-6 {
            0.0
        } else {
            dx.atan2(-dy).to_degrees().rem_euclid(360.0)
        };
        Some([azimuth, elevation])
    }
}
