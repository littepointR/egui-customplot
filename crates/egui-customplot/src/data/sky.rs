use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyError {
    EmptyInput,
    LengthMismatch,
    InvalidElevation,
}

impl std::fmt::Display for SkyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("skyplot input must not be empty"),
            Self::LengthMismatch => f.write_str("skyplot slices must have equal length"),
            Self::InvalidElevation => {
                f.write_str("skyplot angles must be finite with elevation in [-90, 90]")
            }
        }
    }
}

impl std::error::Error for SkyError {}

#[derive(Clone, Debug)]
pub struct SkySatellite {
    pub label: String,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub group: usize,
}

#[derive(Clone, Debug)]
pub struct SkyTrack {
    pub label: String,
    pub azimuth_deg: Box<[f64]>,
    pub elevation_deg: Box<[f64]>,
    pub scalars: Option<Box<[f64]>>,
    pub group: usize,
}

impl SkyTrack {
    pub fn new(
        label: impl Into<String>,
        azimuth_deg: &[f64],
        elevation_deg: &[f64],
    ) -> Result<Self, SkyError> {
        if azimuth_deg.is_empty() {
            return Err(SkyError::EmptyInput);
        }
        if azimuth_deg.len() != elevation_deg.len() {
            return Err(SkyError::LengthMismatch);
        }
        for (&az, &el) in azimuth_deg.iter().zip(elevation_deg.iter()) {
            if !az.is_finite() || !el.is_finite() || !(-90.0..=90.0).contains(&el) {
                return Err(SkyError::InvalidElevation);
            }
        }
        Ok(Self {
            label: label.into(),
            azimuth_deg: azimuth_deg.into(),
            elevation_deg: elevation_deg.into(),
            scalars: None,
            group: 0,
        })
    }

    pub fn with_scalars(mut self, scalars: &[f64]) -> Result<Self, SkyError> {
        if scalars.len() != self.azimuth_deg.len() {
            return Err(SkyError::LengthMismatch);
        }
        self.scalars = Some(scalars.into());
        Ok(self)
    }

    pub fn group(mut self, group: usize) -> Self {
        self.group = group;
        self
    }
}

/// Host-owned azimuth/elevation satellite constellations and trajectories (`skyplot`, `EXT-07`).
#[derive(Clone, Debug)]
pub struct SkyPlotData {
    satellites: Arc<[SkySatellite]>,
    tracks: Arc<[SkyTrack]>,
    group_names: Arc<[String]>,
}

impl SkyPlotData {
    pub fn from_az_el(azimuth_deg: &[f64], elevation_deg: &[f64]) -> Result<Self, SkyError> {
        if azimuth_deg.is_empty() {
            return Err(SkyError::EmptyInput);
        }
        if azimuth_deg.len() != elevation_deg.len() {
            return Err(SkyError::LengthMismatch);
        }
        let mut sats = Vec::with_capacity(azimuth_deg.len());
        for (i, (&az, &el)) in azimuth_deg.iter().zip(elevation_deg.iter()).enumerate() {
            if !az.is_finite() || !el.is_finite() || !(-90.0..=90.0).contains(&el) {
                return Err(SkyError::InvalidElevation);
            }
            sats.push(SkySatellite {
                label: format!("S{}", i + 1),
                azimuth_deg: az,
                elevation_deg: el,
                group: 0,
            });
        }
        Ok(Self {
            satellites: sats.into(),
            tracks: Arc::from([]),
            group_names: Arc::from([]),
        })
    }

    pub fn with_labels(mut self, labels: &[impl AsRef<str>]) -> Result<Self, SkyError> {
        if labels.len() != self.satellites.len() {
            return Err(SkyError::LengthMismatch);
        }
        let mut sats: Vec<SkySatellite> = self.satellites.to_vec();
        for (sat, label) in sats.iter_mut().zip(labels.iter()) {
            sat.label = label.as_ref().to_owned();
        }
        self.satellites = sats.into();
        Ok(self)
    }

    pub fn with_groups(mut self, groups: &[usize]) -> Result<Self, SkyError> {
        if groups.len() != self.satellites.len() {
            return Err(SkyError::LengthMismatch);
        }
        let mut sats: Vec<SkySatellite> = self.satellites.to_vec();
        for (sat, &g) in sats.iter_mut().zip(groups.iter()) {
            sat.group = g;
        }
        self.satellites = sats.into();
        Ok(self)
    }

    pub fn with_group_names(mut self, names: &[impl AsRef<str>]) -> Self {
        self.group_names = names
            .iter()
            .map(|n| n.as_ref().to_owned())
            .collect::<Vec<_>>()
            .into();
        self
    }

    pub fn with_tracks(mut self, tracks: Vec<SkyTrack>) -> Self {
        self.tracks = tracks.into();
        self
    }

    pub fn satellites(&self) -> &[SkySatellite] {
        &self.satellites
    }
    pub fn tracks(&self) -> &[SkyTrack] {
        &self.tracks
    }
    pub fn group_names(&self) -> &[String] {
        &self.group_names
    }
}
