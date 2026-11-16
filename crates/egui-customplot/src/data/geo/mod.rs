mod basemap;
mod bubble;
pub(crate) mod coastlines;
mod density;

use std::sync::Arc;

pub use basemap::{
    GeoAttribution, GeoBasemapStyle, GeoOfflinePolicy, GeoRasterTile, GeoTileCache, GeoTileKey,
    visible_tiles_for_bounds,
};
pub use bubble::{GeoBubbleData, GeoBubblePoint};
pub use density::{GeoDensityData, GeoDensityGrid, GeoRadiusMode};

pub use crate::axis::{GeoBounds, GeoError, GeoPoint};

/// Immutable host-owned latitude/longitude polyline (`geoplot` / `ML-041`).
#[derive(Clone, Debug)]
pub struct GeoPathData {
    points: Arc<[GeoPoint]>,
    non_finite_points: usize,
    domain_invalid_points: usize,
    bounds: Option<GeoBounds>,
}

impl GeoPathData {
    pub fn new(points: Vec<GeoPoint>) -> Self {
        let mut non_finite = 0usize;
        let mut domain_invalid = 0usize;
        for &pt in &points {
            if !pt.is_finite() {
                non_finite += 1;
            } else if !pt.is_valid_domain() {
                domain_invalid += 1;
            }
        }
        let bounds = GeoBounds::from_points(points.iter().copied());
        Self {
            points: points.into(),
            non_finite_points: non_finite,
            domain_invalid_points: domain_invalid,
            bounds,
        }
    }

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
        let points: Vec<GeoPoint> = lat
            .iter()
            .zip(lon.iter())
            .map(|(&la, &lo)| GeoPoint::new(la, lo))
            .collect();
        Ok(Self::new(points))
    }

    pub fn points(&self) -> &[GeoPoint] {
        &self.points
    }

    pub const fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }

    pub const fn domain_invalid_points(&self) -> usize {
        self.domain_invalid_points
    }

    pub const fn bounds(&self) -> Option<GeoBounds> {
        self.bounds
    }

    pub(crate) fn allocation(&self) -> (usize, usize) {
        (
            self.points.as_ptr() as usize,
            std::mem::size_of_val(&*self.points),
        )
    }
}

/// Individual geographic scatter record for [`GeoScatterData`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoScatterPoint {
    pub point: GeoPoint,
    pub size: Option<f32>,
    pub color_value: Option<f64>,
    pub rgba: Option<[u8; 4]>,
}

/// Immutable host-owned geographic scatter dataset (`geoscatter` / `ML-042`).
#[derive(Clone, Debug)]
pub struct GeoScatterData {
    points: Arc<[GeoScatterPoint]>,
    color_range: Option<[f64; 2]>,
    bounds: Option<GeoBounds>,
    non_finite_points: usize,
    domain_invalid_points: usize,
}

impl GeoScatterData {
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
        let mut points = Vec::with_capacity(lat.len());
        let mut non_finite = 0usize;
        let mut domain_invalid = 0usize;
        for (&la, &lo) in lat.iter().zip(lon.iter()) {
            let pt = GeoPoint::new(la, lo);
            if !pt.is_finite() {
                non_finite += 1;
            } else if !pt.is_valid_domain() {
                domain_invalid += 1;
            }
            points.push(GeoScatterPoint {
                point: pt,
                size: None,
                color_value: None,
                rgba: None,
            });
        }
        let bounds = GeoBounds::from_points(points.iter().map(|s| s.point));
        Ok(Self {
            points: points.into(),
            color_range: None,
            bounds,
            non_finite_points: non_finite,
            domain_invalid_points: domain_invalid,
        })
    }

    pub fn with_sizes(mut self, sizes: &[f32]) -> Result<Self, GeoError> {
        if sizes.len() != self.points.len() {
            return Err(GeoError::LengthMismatch {
                expected: self.points.len(),
                actual: sizes.len(),
            });
        }
        if sizes.iter().any(|s| !s.is_finite() || *s <= 0.0) {
            return Err(GeoError::InvalidParameter(
                "scatter sizes must be finite and positive",
            ));
        }
        let mut updated = self.points.to_vec();
        for (item, &s) in updated.iter_mut().zip(sizes.iter()) {
            item.size = Some(s);
        }
        self.points = updated.into();
        Ok(self)
    }

    pub fn with_color_values(mut self, values: &[f64]) -> Result<Self, GeoError> {
        if values.len() != self.points.len() {
            return Err(GeoError::LengthMismatch {
                expected: self.points.len(),
                actual: values.len(),
            });
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        let mut updated = self.points.to_vec();
        for (item, &v) in updated.iter_mut().zip(values.iter()) {
            item.color_value = Some(v);
            if v.is_finite() && item.point.is_valid_domain() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        self.color_range = (lo <= hi).then_some([lo, hi]);
        self.points = updated.into();
        Ok(self)
    }

    pub fn with_rgba(mut self, rgba: &[[u8; 4]]) -> Result<Self, GeoError> {
        if rgba.len() != self.points.len() {
            return Err(GeoError::LengthMismatch {
                expected: self.points.len(),
                actual: rgba.len(),
            });
        }
        let mut updated = self.points.to_vec();
        for (item, &col) in updated.iter_mut().zip(rgba.iter()) {
            item.rgba = Some(col);
        }
        self.points = updated.into();
        Ok(self)
    }

    pub fn points(&self) -> &[GeoScatterPoint] {
        &self.points
    }

    pub const fn color_range(&self) -> Option<[f64; 2]> {
        self.color_range
    }

    pub const fn bounds(&self) -> Option<GeoBounds> {
        self.bounds
    }

    pub const fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }

    pub const fn domain_invalid_points(&self) -> usize {
        self.domain_invalid_points
    }

    pub(crate) fn allocation(&self) -> (usize, usize) {
        (
            self.points.as_ptr() as usize,
            std::mem::size_of_val(&*self.points),
        )
    }
}
