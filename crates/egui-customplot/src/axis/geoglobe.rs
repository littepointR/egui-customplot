use egui::{Pos2, Rect, Vec2, pos2};

/// Orthographic 3D WGS84 spherical Earth camera transform (`geoglobe` / `geoplot3`, `EXT-25`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoGlobeTransform {
    center_lat_deg: f64,
    center_lon_deg: f64,
    zoom: f64,
    rect: Rect,
    radius_px: f32,
    alt_exaggeration: f64,
}

impl GeoGlobeTransform {
    pub const EARTH_MEAN_RADIUS_M: f64 = 6_371_000.0;

    pub fn new(
        rect: Rect,
        center_lat_deg: f64,
        center_lon_deg: f64,
        zoom: f64,
        alt_exaggeration: f64,
    ) -> Self {
        let lat = if center_lat_deg.is_finite() {
            center_lat_deg.clamp(-89.9, 89.9)
        } else {
            25.0
        };
        let lon = if center_lon_deg.is_finite() {
            super::wrap_lon_180(center_lon_deg)
        } else {
            100.0
        };
        let z = if zoom.is_finite() && zoom > 0.0 {
            zoom.clamp(0.4, 8.0)
        } else {
            1.0
        };
        let ex = if alt_exaggeration.is_finite() && alt_exaggeration >= 0.0 {
            alt_exaggeration
        } else {
            1.0
        };
        let side = rect.width().min(rect.height()).max(4.0);
        let radius_px = side * 0.42 * z as f32;
        Self {
            center_lat_deg: lat,
            center_lon_deg: lon,
            zoom: z,
            rect,
            radius_px,
            alt_exaggeration: ex,
        }
    }

    pub fn center_lat_deg(self) -> f64 {
        self.center_lat_deg
    }
    pub fn center_lon_deg(self) -> f64 {
        self.center_lon_deg
    }
    pub fn zoom(self) -> f64 {
        self.zoom
    }
    pub fn rect(self) -> Rect {
        self.rect
    }
    pub fn screen_center(self) -> Pos2 {
        self.rect.center()
    }
    pub fn radius_px(self) -> f32 {
        self.radius_px
    }

    /// Camera-space unit vector `(x_east, y_north, z_view)` for `(lat_deg, lon_deg)`.
    /// Points on the visible front hemisphere satisfy `z_view >= -0.02`.
    pub fn view_dir(self, lat_deg: f64, lon_deg: f64) -> Option<[f64; 3]> {
        if !lat_deg.is_finite() || !lon_deg.is_finite() {
            return None;
        }
        let phi = lat_deg.clamp(-90.0, 90.0).to_radians();
        let phi0 = self.center_lat_deg.to_radians();
        let dlon = super::shortest_dlon(self.center_lon_deg, lon_deg).to_radians();
        let (sp, cp) = phi.sin_cos();
        let (sp0, cp0) = phi0.sin_cos();
        let (sdl, cdl) = dlon.sin_cos();
        let x = cp * sdl;
        let y = cp0 * sp - sp0 * cp * cdl;
        let z = sp0 * sp + cp0 * cp * cdl;
        Some([x, y, z])
    }

    /// Projects `(lat_deg, lon_deg, alt_m)` with front-hemisphere limb culling.
    /// Returns `Some((screen_pos, view_z))` or `None` if occluded behind the Earth sphere.
    pub fn project(self, lat_deg: f64, lon_deg: f64, alt_m: f64) -> Option<(Pos2, f64)> {
        let [vx, vy, vz] = self.view_dir(lat_deg, lon_deg)?;
        let alt = if alt_m.is_finite() {
            alt_m.max(-12_000.0)
        } else {
            0.0
        };
        let r_norm = 1.0 + (alt / Self::EARTH_MEAN_RADIUS_M) * self.alt_exaggeration;
        let [px, py, pz] = [vx * r_norm, vy * r_norm, vz * r_norm];
        // Occluded if point sits on far side AND projected inside the unit Earth limb disk.
        if pz < 0.0 && px.hypot(py) < 1.0 {
            return None;
        }
        let c = self.rect.center();
        let r = f64::from(self.radius_px);
        let sx = (f64::from(c.x) + px * r) as f32;
        let sy = (f64::from(c.y) - py * r) as f32;
        (sx.is_finite() && sy.is_finite()).then_some((pos2(sx, sy), pz))
    }

    /// Inverts visible screen position inside the Earth sphere to `(lat_deg, lon_deg)`.
    pub fn screen_to_lat_lon(self, pos: Pos2) -> Option<[f64; 2]> {
        let c = self.rect.center();
        let r = f64::from(self.radius_px).max(1.0);
        let x = f64::from(pos.x - c.x) / r;
        let y = -f64::from(pos.y - c.y) / r;
        let rho2 = x * x + y * y;
        if rho2 > 1.0 {
            return None;
        }
        let rho = rho2.sqrt();
        if rho < 1e-12 {
            return Some([self.center_lat_deg, self.center_lon_deg]);
        }
        let c_ang = rho.min(1.0).asin();
        let (sc, cc) = c_ang.sin_cos();
        let phi0 = self.center_lat_deg.to_radians();
        let (sp0, cp0) = phi0.sin_cos();
        let phi = (cc * sp0 + (y * sc * cp0) / rho).clamp(-1.0, 1.0).asin();
        let lon = self.center_lon_deg.to_radians() + (x * sc).atan2(rho * cp0 * cc - y * sp0 * sc);
        Some([phi.to_degrees(), super::wrap_lon_180(lon.to_degrees())])
    }

    pub fn orbit(self, delta_px: Vec2) -> (f64, f64) {
        let deg_per_px = 110.0 / f64::from(self.radius_px).max(20.0);
        let next_lon =
            super::wrap_lon_180(self.center_lon_deg - f64::from(delta_px.x) * deg_per_px);
        let next_lat =
            (self.center_lat_deg + f64::from(delta_px.y) * deg_per_px).clamp(-85.0, 85.0);
        (next_lat, next_lon)
    }
}
