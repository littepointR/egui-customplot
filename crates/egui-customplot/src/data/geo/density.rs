use std::sync::Arc;

use crate::axis::{GeoBounds, GeoError, GeoPoint, GeoTransform, haversine_km, shortest_dlon};

/// Kernel bandwidth mode for [`GeoDensityData`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GeoRadiusMode {
    Kilometers(f64),
    ZoomAdaptivePixels(f32),
}

impl Default for GeoRadiusMode {
    fn default() -> Self {
        Self::Kilometers(150.0)
    }
}

/// Evaluated spatial kernel density raster for a given viewport.
#[derive(Clone, Debug)]
pub struct GeoDensityGrid {
    pub cols: usize,
    pub rows: usize,
    pub values: Vec<f64>,
    pub max_density: f64,
    pub visited_samples: usize,
}

/// Immutable host-owned geographic kernel density dataset (`geodensityplot` / `ML-043`).
#[derive(Clone, Debug)]
pub struct GeoDensityData {
    samples: Arc<[(GeoPoint, f64)]>,
    input_len: usize,
    non_finite_points: usize,
    domain_invalid_points: usize,
    bounds: Option<GeoBounds>,
    radius: GeoRadiusMode,
    grid_res: [usize; 2],
}

impl GeoDensityData {
    pub fn from_lat_lon(lat: &[f64], lon: &[f64]) -> Result<Self, GeoError> {
        if lat.is_empty() {
            return Err(GeoError::EmptyData);
        }
        if lat.len() != lon.len() {
            return Err(GeoError::LengthMismatch {
                expected: lat.len(),
                actual: lon.len(),
            });
        }
        let ones = vec![1.0; lat.len()];
        Self::weighted(lat, lon, &ones)
    }

    pub fn weighted(lat: &[f64], lon: &[f64], weights: &[f64]) -> Result<Self, GeoError> {
        if lat.is_empty() {
            return Err(GeoError::EmptyData);
        }
        if lat.len() != lon.len() || lat.len() != weights.len() {
            return Err(GeoError::LengthMismatch {
                expected: lat.len(),
                actual: lon.len().min(weights.len()),
            });
        }
        let mut samples = Vec::with_capacity(lat.len());
        let mut non_finite = 0usize;
        let mut domain_invalid = 0usize;
        for i in 0..lat.len() {
            let pt = GeoPoint::new(lat[i], lon[i]);
            let w = weights[i];
            if !pt.is_finite() || !w.is_finite() {
                non_finite += 1;
            } else if !pt.is_valid_domain() || w < 0.0 {
                domain_invalid += 1;
            } else {
                samples.push((pt, w));
            }
        }
        let bounds = GeoBounds::from_points(samples.iter().map(|(p, _)| *p));
        Ok(Self {
            samples: samples.into(),
            input_len: lat.len(),
            non_finite_points: non_finite,
            domain_invalid_points: domain_invalid,
            bounds,
            radius: GeoRadiusMode::default(),
            grid_res: [56, 40],
        })
    }

    pub fn radius_km(mut self, km: f64) -> Result<Self, GeoError> {
        if !km.is_finite() || km <= 0.0 {
            return Err(GeoError::InvalidParameter(
                "radius_km must be finite and > 0",
            ));
        }
        self.radius = GeoRadiusMode::Kilometers(km);
        Ok(self)
    }

    pub fn radius_pixels(mut self, px: f32) -> Result<Self, GeoError> {
        if !px.is_finite() || px <= 0.0 {
            return Err(GeoError::InvalidParameter(
                "radius_pixels must be finite and > 0",
            ));
        }
        self.radius = GeoRadiusMode::ZoomAdaptivePixels(px);
        Ok(self)
    }

    pub fn grid_resolution(mut self, cols: usize, rows: usize) -> Self {
        self.grid_res = [cols.clamp(8, 128), rows.clamp(8, 128)];
        self
    }

    pub const fn radius_mode(&self) -> GeoRadiusMode {
        self.radius
    }

    pub const fn bounds(&self) -> Option<GeoBounds> {
        self.bounds
    }

    pub const fn input_len(&self) -> usize {
        self.input_len
    }

    pub const fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }

    pub const fn domain_invalid_points(&self) -> usize {
        self.domain_invalid_points
    }

    pub(crate) fn allocation(&self) -> (usize, usize) {
        (
            self.samples.as_ptr() as usize,
            std::mem::size_of_val(&*self.samples),
        )
    }

    pub(crate) fn sample_points(&self) -> impl Iterator<Item = GeoPoint> + '_ {
        self.samples.iter().map(|(p, _)| *p)
    }

    pub(crate) fn effective_radius_km(&self, transform: GeoTransform) -> f64 {
        match self.radius {
            GeoRadiusMode::Kilometers(km) => km,
            GeoRadiusMode::ZoomAdaptivePixels(px) => {
                let bounds = transform.bounds();
                let mid_lat = 0.5 * (bounds.lat_min + bounds.lat_max);
                let span_km = (2.0 * std::f64::consts::PI * 6371.0088)
                    * (bounds.lon_span / 360.0)
                    * mid_lat.to_radians().cos().abs().max(0.15);
                let km_per_px = span_km / f64::from(transform.viewport().width().max(1.0));
                (km_per_px * f64::from(px)).max(0.5)
            }
        }
    }

    /// Evaluates spherical quartic biweight density over `transform.viewport()`.
    pub fn evaluate_viewport(&self, transform: GeoTransform) -> GeoDensityGrid {
        let [cols, rows] = self.grid_res;
        let vp = transform.viewport();
        let radius_km = self.effective_radius_km(transform);
        let bounds = transform.bounds();
        let dlat_pad = radius_km / 111.19 + 0.5;
        let lat_lo = bounds.lat_min - dlat_pad;
        let lat_hi = bounds.lat_max + dlat_pad;
        let half_dlon = (0.5 * bounds.lon_span + dlat_pad * 2.0).min(180.0);

        let mut nearby: Vec<(GeoPoint, f64)> = Vec::new();
        for &(pt, w) in self.samples.iter() {
            if w <= 0.0 || pt.lat < lat_lo || pt.lat > lat_hi {
                continue;
            }
            if shortest_dlon(bounds.center_lon, pt.lon).abs() <= half_dlon {
                nearby.push((pt, w));
            }
        }

        let mut values = vec![0.0; cols * rows];
        let mut max_density = 0.0_f64;
        for row in 0..rows {
            let cy = vp.top() + (row as f32 + 0.5) / rows as f32 * vp.height();
            for col in 0..cols {
                let cx = vp.left() + (col as f32 + 0.5) / cols as f32 * vp.width();
                let Some(cell_pt) = transform.unproject(egui::pos2(cx, cy)) else {
                    continue;
                };
                let max_dlon = (radius_km
                    / (111.19 * cell_pt.lat.to_radians().cos().abs().max(0.05)))
                .min(180.0)
                    + 0.25;
                let mut acc = 0.0_f64;
                for &(sp, w) in &nearby {
                    if (sp.lat - cell_pt.lat).abs() > dlat_pad {
                        continue;
                    }
                    if max_dlon < 179.9 && shortest_dlon(cell_pt.lon, sp.lon).abs() > max_dlon {
                        continue;
                    }
                    let d = haversine_km(cell_pt, sp);
                    if d < radius_km {
                        let u = d / radius_km;
                        let k = 1.0 - u * u;
                        acc += w * k * k;
                    }
                }
                values[row * cols + col] = acc;
                max_density = max_density.max(acc);
            }
        }
        GeoDensityGrid {
            cols,
            rows,
            values,
            max_density,
            visited_samples: nearby.len(),
        }
    }
}
