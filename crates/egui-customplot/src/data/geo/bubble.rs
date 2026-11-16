use std::{collections::HashMap, sync::Arc};

use crate::{
    BubbleLegend,
    axis::{GeoBounds, GeoError, GeoPoint},
    layout::bubble_pixel_radius,
};

/// Individual geographic bubble record for [`GeoBubbleData`].
#[derive(Clone, Debug, PartialEq)]
pub struct GeoBubblePoint {
    pub point: GeoPoint,
    pub size: f64,
    pub category: Option<usize>,
    pub label: Option<String>,
}

/// Immutable host-owned geographic bubble dataset (`geobubble` / `ML-044`).
#[derive(Clone, Debug)]
pub struct GeoBubbleData {
    points: Arc<[GeoBubblePoint]>,
    category_names: Vec<String>,
    size_range: [f64; 2],
    radius_range: [f32; 2],
    bounds: Option<GeoBounds>,
    non_finite_points: usize,
    domain_invalid_points: usize,
}

impl GeoBubbleData {
    pub fn new(lat: &[f64], lon: &[f64], sizes: &[f64]) -> Result<Self, GeoError> {
        if lat.is_empty() {
            return Err(GeoError::EmptyData);
        }
        if lat.len() != lon.len() || lat.len() != sizes.len() {
            return Err(GeoError::LengthMismatch {
                expected: lat.len(),
                actual: lon.len().min(sizes.len()),
            });
        }
        let mut points = Vec::with_capacity(lat.len());
        let mut non_finite = 0usize;
        let mut domain_invalid = 0usize;
        let mut s_lo = f64::INFINITY;
        let mut s_hi = f64::NEG_INFINITY;
        for i in 0..lat.len() {
            let pt = GeoPoint::new(lat[i], lon[i]);
            let s = sizes[i];
            if !pt.is_finite() || !s.is_finite() {
                non_finite += 1;
            } else if !pt.is_valid_domain() || s < 0.0 {
                domain_invalid += 1;
            } else {
                s_lo = s_lo.min(s);
                s_hi = s_hi.max(s);
            }
            points.push(GeoBubblePoint {
                point: pt,
                size: s,
                category: None,
                label: None,
            });
        }
        let size_range = if s_lo <= s_hi {
            [s_lo, s_hi]
        } else {
            [0.0, 0.0]
        };
        let bounds = GeoBounds::from_points(
            points
                .iter()
                .filter(|b| b.point.is_valid_domain() && b.size.is_finite() && b.size >= 0.0)
                .map(|b| b.point),
        );
        Ok(Self {
            points: points.into(),
            category_names: Vec::new(),
            size_range,
            radius_range: [4.0, 22.0],
            bounds,
            non_finite_points: non_finite,
            domain_invalid_points: domain_invalid,
        })
    }

    pub fn with_categories<S: AsRef<str>>(mut self, categories: &[S]) -> Result<Self, GeoError> {
        if categories.len() != self.points.len() {
            return Err(GeoError::LengthMismatch {
                expected: self.points.len(),
                actual: categories.len(),
            });
        }
        let mut names = Vec::new();
        let mut map: HashMap<String, usize> = HashMap::new();
        let mut updated: Vec<GeoBubblePoint> = self.points.to_vec();
        for (item, cat) in updated.iter_mut().zip(categories.iter()) {
            let text = cat.as_ref();
            let id = if let Some(&id) = map.get(text) {
                id
            } else {
                let id = names.len();
                let owned = text.to_owned();
                map.insert(owned.clone(), id);
                names.push(owned);
                id
            };
            item.category = Some(id);
        }
        self.category_names = names;
        self.points = updated.into();
        Ok(self)
    }

    pub fn with_labels<S: AsRef<str>>(mut self, labels: &[S]) -> Result<Self, GeoError> {
        if labels.len() != self.points.len() {
            return Err(GeoError::LengthMismatch {
                expected: self.points.len(),
                actual: labels.len(),
            });
        }
        let mut updated: Vec<GeoBubblePoint> = self.points.to_vec();
        for (item, label) in updated.iter_mut().zip(labels.iter()) {
            item.label = Some(label.as_ref().to_owned());
        }
        self.points = updated.into();
        Ok(self)
    }

    pub fn radius_range(mut self, min_px: f32, max_px: f32) -> Result<Self, GeoError> {
        if !min_px.is_finite() || !max_px.is_finite() || min_px < 1.0 || max_px < min_px {
            return Err(GeoError::InvalidParameter(
                "invalid bubble pixel radius range",
            ));
        }
        self.radius_range = [min_px.min(64.0), max_px.min(64.0)];
        Ok(self)
    }

    pub fn pixel_radius(&self, size: f64) -> f32 {
        bubble_pixel_radius(size, self.size_range, self.radius_range)
    }

    pub fn bubble_legend(&self, title: impl Into<String>) -> BubbleLegend {
        BubbleLegend::new(self.size_range, self.radius_range).title(title)
    }

    pub fn points(&self) -> &[GeoBubblePoint] {
        &self.points
    }

    pub fn categories(&self) -> &[String] {
        &self.category_names
    }

    pub const fn size_range(&self) -> [f64; 2] {
        self.size_range
    }

    pub const fn radius_range_px(&self) -> [f32; 2] {
        self.radius_range
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
