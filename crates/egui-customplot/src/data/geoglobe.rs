use std::sync::Arc;

use egui::Color32;

use crate::{GeoError, axis::wrap_lon_180};

#[derive(Clone, Debug)]
pub struct GlobeTrack3D {
    pub name: String,
    pub lat_deg: Arc<[f64]>,
    pub lon_deg: Arc<[f64]>,
    pub alt_m: Arc<[f64]>,
    pub color: Color32,
    pub extruded: bool,
}

#[derive(Clone, Debug)]
pub struct GlobeMarker3D {
    pub label: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: f64,
    pub radius_px: f32,
    pub color: Color32,
}

/// 3D WGS84 Globe dataset with tracks, altitude extrusion, and landmarks (`geoglobe` / `geoplot3`, `EXT-25`).
#[derive(Clone, Debug)]
pub struct GeoGlobeData {
    tracks: Arc<[GlobeTrack3D]>,
    markers: Arc<[GlobeMarker3D]>,
    alt_exaggeration: f64,
}

impl Default for GeoGlobeData {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoGlobeData {
    pub fn new() -> Self {
        Self {
            tracks: Arc::from([]),
            markers: Arc::from([]),
            alt_exaggeration: 30.0,
        }
    }

    pub fn alt_exaggeration(mut self, factor: f64) -> Self {
        if factor.is_finite() && factor >= 0.0 {
            self.alt_exaggeration = factor;
        }
        self
    }

    pub fn with_track(
        self,
        name: impl Into<String>,
        lat_deg: &[f64],
        lon_deg: &[f64],
        alt_m: &[f64],
        color: Color32,
    ) -> Result<Self, GeoError> {
        self.push_track(name, lat_deg, lon_deg, alt_m, color, false)
    }

    pub fn with_extruded_track(
        self,
        name: impl Into<String>,
        lat_deg: &[f64],
        lon_deg: &[f64],
        alt_m: &[f64],
        color: Color32,
    ) -> Result<Self, GeoError> {
        self.push_track(name, lat_deg, lon_deg, alt_m, color, true)
    }

    fn push_track(
        mut self,
        name: impl Into<String>,
        lat_deg: &[f64],
        lon_deg: &[f64],
        alt_m: &[f64],
        color: Color32,
        extruded: bool,
    ) -> Result<Self, GeoError> {
        if lat_deg.is_empty() {
            return Err(GeoError::EmptyData);
        }
        if lon_deg.len() != lat_deg.len() {
            return Err(GeoError::LengthMismatch {
                expected: lat_deg.len(),
                actual: lon_deg.len(),
            });
        }
        if alt_m.len() != lat_deg.len() {
            return Err(GeoError::LengthMismatch {
                expected: lat_deg.len(),
                actual: alt_m.len(),
            });
        }
        let mut norm_lat = Vec::with_capacity(lat_deg.len());
        let mut norm_lon = Vec::with_capacity(lon_deg.len());
        let mut norm_alt = Vec::with_capacity(alt_m.len());
        for i in 0..lat_deg.len() {
            if !lat_deg[i].is_finite() || !lon_deg[i].is_finite() || !alt_m[i].is_finite() {
                return Err(GeoError::InvalidParameter(
                    "non-finite globe coordinate or altitude",
                ));
            }
            norm_lat.push(lat_deg[i].clamp(-90.0, 90.0));
            norm_lon.push(wrap_lon_180(lon_deg[i]));
            norm_alt.push(alt_m[i]);
        }
        let mut list = self.tracks.to_vec();
        list.push(GlobeTrack3D {
            name: name.into(),
            lat_deg: norm_lat.into(),
            lon_deg: norm_lon.into(),
            alt_m: norm_alt.into(),
            color,
            extruded,
        });
        self.tracks = list.into();
        Ok(self)
    }

    pub fn with_markers(mut self, markers: Vec<GlobeMarker3D>) -> Result<Self, GeoError> {
        for m in &markers {
            if !m.lat_deg.is_finite() || !m.lon_deg.is_finite() || !m.alt_m.is_finite() {
                return Err(GeoError::InvalidParameter(
                    "non-finite globe coordinate or altitude",
                ));
            }
        }
        self.markers = markers.into();
        Ok(self)
    }

    pub fn tracks(&self) -> &[GlobeTrack3D] {
        &self.tracks
    }
    pub fn markers(&self) -> &[GlobeMarker3D] {
        &self.markers
    }
    pub fn get_alt_exaggeration(&self) -> f64 {
        self.alt_exaggeration
    }
    pub fn sample_bytes(&self) -> usize {
        self.tracks
            .iter()
            .map(|t| t.lat_deg.len() * 24)
            .sum::<usize>()
            + std::mem::size_of_val(&*self.markers)
    }
}
