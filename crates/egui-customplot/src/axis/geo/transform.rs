use egui::{Pos2, Rect};

use super::{GeoBounds, GeoPoint, GeoProjection, shortest_dlon, wrap_lon_180};

/// Aspect-preserving projection between `(lat, lon)` and screen pixels inside `plot_area`.
#[derive(Clone, Copy, Debug)]
pub struct GeoTransform {
    bounds: GeoBounds,
    projection: GeoProjection,
    viewport: Rect,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl GeoTransform {
    pub fn new(bounds: GeoBounds, projection: GeoProjection, plot_area: Rect) -> Option<Self> {
        if !plot_area.is_finite()
            || plot_area.width() <= 1.0
            || plot_area.height() <= 1.0
            || !bounds.lat_min.is_finite()
            || !bounds.lat_max.is_finite()
            || bounds.lat_max <= bounds.lat_min
            || !bounds.lon_span.is_finite()
            || bounds.lon_span <= 0.0
        {
            return None;
        }
        let x_half = (bounds.lon_span / 360.0) * 0.5;
        let x_min = -x_half;
        let x_max = x_half;
        let y_min = projection.lat_to_norm_y(bounds.lat_max);
        let y_max = projection.lat_to_norm_y(bounds.lat_min);
        let dx = (x_max - x_min).max(1e-9);
        let dy = (y_max - y_min).max(1e-9);
        let horiz_factor = match projection {
            GeoProjection::WebMercator => 1.0,
            GeoProjection::Equirectangular => {
                let mid_lat = 0.5 * (bounds.lat_min + bounds.lat_max);
                2.0 * mid_lat.to_radians().cos().abs().max(0.15)
            }
        };
        let desired_aspect = ((dx * horiz_factor) / dy).clamp(0.05, 20.0) as f32;
        let avail_aspect = plot_area.width() / plot_area.height();
        let (w, h) = if avail_aspect > desired_aspect {
            (plot_area.height() * desired_aspect, plot_area.height())
        } else {
            (plot_area.width(), plot_area.width() / desired_aspect)
        };
        let viewport = Rect::from_center_size(plot_area.center(), egui::vec2(w, h));
        Some(Self {
            bounds,
            projection,
            viewport,
            x_min,
            x_max,
            y_min,
            y_max,
        })
    }

    pub const fn bounds(self) -> GeoBounds {
        self.bounds
    }

    pub const fn projection(self) -> GeoProjection {
        self.projection
    }

    pub const fn viewport(self) -> Rect {
        self.viewport
    }

    pub(crate) fn norm_xy(self, pt: GeoPoint) -> Option<[f64; 2]> {
        if !pt.is_valid_domain() {
            return None;
        }
        let rel_lon = shortest_dlon(self.bounds.center_lon, pt.lon);
        let wx = rel_lon / 360.0;
        let wy = self.projection.lat_to_norm_y(pt.lat);
        Some([
            (wx - self.x_min) / (self.x_max - self.x_min),
            1.0 - (wy - self.y_min) / (self.y_max - self.y_min),
        ])
    }

    pub(crate) fn pos_from_norm(self, norm: [f64; 2]) -> Pos2 {
        Pos2::new(
            self.viewport.left() + norm[0] as f32 * self.viewport.width(),
            self.viewport.bottom() - norm[1] as f32 * self.viewport.height(),
        )
    }

    /// Projects a valid geographic coordinate into viewport screen pixels (including outside viewport).
    pub fn project_unclipped(self, pt: GeoPoint) -> Option<Pos2> {
        self.norm_xy(pt).map(|n| self.pos_from_norm(n))
    }

    /// Projects a coordinate if finite and within `viewport` (tolerance 2px).
    pub fn project(self, pt: GeoPoint) -> Option<Pos2> {
        let pos = self.project_unclipped(pt)?;
        self.viewport.expand(2.0).contains(pos).then_some(pos)
    }

    /// Inverse transform from screen position inside or near `viewport` to `(lat, lon)`.
    pub fn unproject(self, pos: Pos2) -> Option<GeoPoint> {
        if !pos.is_finite() {
            return None;
        }
        let u = f64::from((pos.x - self.viewport.left()) / self.viewport.width().max(1.0));
        let v = f64::from((pos.y - self.viewport.top()) / self.viewport.height().max(1.0));
        let wx = self.x_min + u * (self.x_max - self.x_min);
        let wy = self.y_min + v * (self.y_max - self.y_min);
        let lon = wrap_lon_180(self.bounds.center_lon + wx * 360.0);
        let lat = self.projection.norm_y_to_lat(wy);
        Some(GeoPoint::new(lat, lon))
    }

    /// Approximate Web-Mercator-equivalent integer/fractional zoom level for current `lon_span`.
    pub fn zoom_level_equivalent(self) -> f64 {
        (360.0 / self.bounds.lon_span.max(1e-4)).log2().max(0.0)
    }
}
