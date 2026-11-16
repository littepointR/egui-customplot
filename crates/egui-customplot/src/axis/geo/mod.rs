mod graticule;
mod transform;

use std::fmt;

pub(crate) use graticule::{format_lat, format_lon, geo_graticule_ticks, project_geo_polyline};
pub use transform::GeoTransform;

const EARTH_MEAN_RADIUS_KM: f64 = 6371.0088;
const MERCATOR_MAX_LAT: f64 = 85.051_128_779_806_59;
const MIN_AUTO_SPAN_DEG: f64 = 0.5;

/// Errors raised while validating geographic data, bounds, or tile caches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeoError {
    EmptyData,
    LengthMismatch { expected: usize, actual: usize },
    InvalidBounds(&'static str),
    InvalidParameter(&'static str),
    AttributionRequired,
    CacheBudgetExceeded { required: usize, budget: usize },
}

impl fmt::Display for GeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "geographic dataset is empty"),
            Self::LengthMismatch { expected, actual } => {
                write!(f, "length mismatch: expected {expected}, got {actual}")
            }
            Self::InvalidBounds(msg) => write!(f, "invalid geographic bounds: {msg}"),
            Self::InvalidParameter(msg) => write!(f, "invalid geographic parameter: {msg}"),
            Self::AttributionRequired => {
                write!(
                    f,
                    "non-empty attribution text is required for tile basemaps"
                )
            }
            Self::CacheBudgetExceeded { required, budget } => {
                write!(
                    f,
                    "tile requires {required} bytes which exceeds cache budget of {budget} bytes"
                )
            }
        }
    }
}

impl std::error::Error for GeoError {}

/// Geographic latitude/longitude coordinate in degrees (`WGS84`/`spherical`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

impl GeoPoint {
    pub const fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    pub fn is_finite(self) -> bool {
        self.lat.is_finite() && self.lon.is_finite()
    }

    pub fn is_valid_domain(self) -> bool {
        self.is_finite() && self.lat.abs() <= 90.0
    }
}

/// Wraps a finite longitude in degrees into `[-180.0, 180.0]`.
pub fn wrap_lon_180(lon: f64) -> f64 {
    if !lon.is_finite() {
        return lon;
    }
    if lon == 180.0 {
        return 180.0;
    }
    let wrapped = (lon + 180.0).rem_euclid(360.0) - 180.0;
    if wrapped == -180.0 && lon > 0.0 {
        180.0
    } else {
        wrapped
    }
}

/// Shortest signed longitude delta `lon2 - lon1` in `[-180.0, 180.0]`.
pub fn shortest_dlon(lon1: f64, lon2: f64) -> f64 {
    if !lon1.is_finite() || !lon2.is_finite() {
        return f64::NAN;
    }
    let diff = ((lon2 - lon1 + 180.0).rem_euclid(360.0)) - 180.0;
    if diff == -180.0 && (lon2 - lon1) > 0.0 {
        180.0
    } else {
        diff
    }
}

/// Great-circle distance in kilometers on a sphere of mean Earth radius.
pub fn haversine_km(a: GeoPoint, b: GeoPoint) -> f64 {
    if !a.is_valid_domain() || !b.is_valid_domain() {
        return f64::NAN;
    }
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = shortest_dlon(a.lon, b.lon).to_radians();
    let h = (dlat * 0.5).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon * 0.5).sin().powi(2);
    2.0 * EARTH_MEAN_RADIUS_KM * h.sqrt().clamp(0.0, 1.0).asin()
}

/// 2D geographic map projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GeoProjection {
    #[default]
    WebMercator,
    Equirectangular,
}

impl GeoProjection {
    pub(crate) fn clamp_lat(self, lat: f64) -> f64 {
        match self {
            Self::WebMercator => lat.clamp(-MERCATOR_MAX_LAT, MERCATOR_MAX_LAT),
            Self::Equirectangular => lat.clamp(-90.0, 90.0),
        }
    }

    pub(crate) fn lat_to_norm_y(self, lat: f64) -> f64 {
        let clamped = self.clamp_lat(lat);
        match self {
            Self::WebMercator => {
                let rad = clamped.to_radians();
                let merc = (std::f64::consts::FRAC_PI_4 + 0.5 * rad).tan().ln();
                0.5 - merc / (2.0 * std::f64::consts::PI)
            }
            Self::Equirectangular => (90.0 - clamped) / 180.0,
        }
    }

    pub(crate) fn norm_y_to_lat(self, y: f64) -> f64 {
        match self {
            Self::WebMercator => {
                let merc = (0.5 - y) * 2.0 * std::f64::consts::PI;
                let lat = (2.0 * merc.exp().atan() - std::f64::consts::FRAC_PI_2).to_degrees();
                lat.clamp(-MERCATOR_MAX_LAT, MERCATOR_MAX_LAT)
            }
            Self::Equirectangular => (90.0 - y * 180.0).clamp(-90.0, 90.0),
        }
    }
}

/// Geographic viewport bounds supporting viewports straddling `±180°`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoBounds {
    pub lat_min: f64,
    pub lat_max: f64,
    pub center_lon: f64,
    pub lon_span: f64,
}

impl GeoBounds {
    pub fn world() -> Self {
        Self {
            lat_min: -82.0,
            lat_max: 82.0,
            center_lon: 0.0,
            lon_span: 360.0,
        }
    }

    /// Construct bounds from latitude `[min, max]` and longitude `[west, east]`.
    /// Supports dateline-crossing intervals where `west > east` in `[-180, 180]`
    /// (such as `[120.0, -110.0]`) as well as continuous spans `<= 360°`.
    pub fn from_lat_lon(lat_lim: [f64; 2], lon_lim: [f64; 2]) -> Result<Self, GeoError> {
        if !lat_lim[0].is_finite()
            || !lat_lim[1].is_finite()
            || !lon_lim[0].is_finite()
            || !lon_lim[1].is_finite()
        {
            return Err(GeoError::InvalidBounds("non-finite latitude or longitude"));
        }
        if lat_lim[0] < -90.0 || lat_lim[1] > 90.0 || lat_lim[0] >= lat_lim[1] {
            return Err(GeoError::InvalidBounds(
                "latitude limits must satisfy -90 <= min < max <= 90",
            ));
        }
        let (center_lon, lon_span) = if lon_lim[0] < lon_lim[1] {
            let span = lon_lim[1] - lon_lim[0];
            if span > 360.0 {
                return Err(GeoError::InvalidBounds(
                    "longitude span exceeds 360 degrees",
                ));
            }
            (wrap_lon_180(0.5 * (lon_lim[0] + lon_lim[1])), span)
        } else if lon_lim[0] > lon_lim[1] && lon_lim[0].abs() <= 180.0 && lon_lim[1].abs() <= 180.0
        {
            let span = (180.0 - lon_lim[0]) + (lon_lim[1] + 180.0);
            if !(1e-6..=360.0).contains(&span) {
                return Err(GeoError::InvalidBounds("invalid dateline longitude span"));
            }
            (wrap_lon_180(lon_lim[0] + 0.5 * span), span)
        } else {
            return Err(GeoError::InvalidBounds(
                "longitude limits must define a positive span <= 360 degrees",
            ));
        };
        Ok(Self {
            lat_min: lat_lim[0],
            lat_max: lat_lim[1],
            center_lon,
            lon_span: lon_span.max(1e-5),
        })
    }

    /// Computes minimal S^1 longitude arc and padded latitude extent across valid points.
    pub fn from_points(points: impl IntoIterator<Item = GeoPoint>) -> Option<Self> {
        let mut lat_min = f64::INFINITY;
        let mut lat_max = f64::NEG_INFINITY;
        let mut lons = Vec::new();
        for pt in points {
            if pt.is_valid_domain() {
                lat_min = lat_min.min(pt.lat);
                lat_max = lat_max.max(pt.lat);
                let mut lon = wrap_lon_180(pt.lon);
                if lon == 180.0 {
                    lon = -180.0;
                }
                lons.push(lon);
            }
        }
        if lons.is_empty() {
            return None;
        }
        lons.sort_by(f64::total_cmp);
        let mut best_gap = -1.0;
        let mut best_start = lons[0];
        for pair in lons.windows(2) {
            let gap = pair[1] - pair[0];
            if gap > best_gap {
                best_gap = gap;
                best_start = pair[1];
            }
        }
        let wrap_gap = (lons[0] + 360.0) - *lons.last().unwrap();
        if wrap_gap > best_gap {
            best_gap = wrap_gap;
            best_start = lons[0];
        }
        let raw_span = (360.0 - best_gap).clamp(0.0, 360.0);
        let lon_span = (raw_span * 1.12).clamp(MIN_AUTO_SPAN_DEG, 360.0);
        let center_lon = if lon_span >= 359.9 {
            0.0
        } else {
            wrap_lon_180(best_start + 0.5 * raw_span)
        };
        let lat_span = (lat_max - lat_min).max(MIN_AUTO_SPAN_DEG);
        let lat_pad = lat_span * 0.08;
        let lat_mid = 0.5 * (lat_min + lat_max);
        let out_lat_min = (lat_mid - 0.5 * lat_span - lat_pad).clamp(-85.0, 89.0);
        let out_lat_max = (lat_mid + 0.5 * lat_span + lat_pad)
            .clamp(out_lat_min + 0.05, 85.0)
            .min(89.5);
        Some(Self {
            lat_min: out_lat_min,
            lat_max: out_lat_max,
            center_lon,
            lon_span,
        })
    }

    pub fn contains(self, pt: GeoPoint) -> bool {
        if !pt.is_valid_domain() || pt.lat < self.lat_min || pt.lat > self.lat_max {
            return false;
        }
        shortest_dlon(self.center_lon, pt.lon).abs() <= self.lon_span * 0.5 + 1e-9
    }
}
