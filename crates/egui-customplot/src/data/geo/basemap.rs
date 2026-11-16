use std::{
    collections::{HashMap, VecDeque},
    f64::consts::PI,
    sync::Arc,
};

use crate::axis::{GeoBounds, GeoError, wrap_lon_180};

const MAX_TILE_DIM: u16 = 256;
const MAX_TILE_ZOOM: u8 = 18;

/// Offline/cached geographic basemap policy.
/// Never reads MathWorks basemap services and never initiates network I/O.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GeoBasemapStyle {
    None,
    #[default]
    OfflineVectorLight,
    OfflineVectorDark,
    HostTiles,
}

/// Deterministic fallback when a tile is absent in `GeoTileCache`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GeoOfflinePolicy {
    #[default]
    FallbackToVector,
    BlankWithNotice,
}

/// Explicit source & license metadata rendered with geographic plots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeoAttribution {
    text: String,
    license: String,
}

impl GeoAttribution {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            license: String::new(),
        }
    }

    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = license.into();
        self
    }

    pub fn builtin_offline() -> Self {
        Self {
            text: "Offline schematic coastlines (egui-customplot built-in)".into(),
            license: "Public-domain schematic vector geometry".into(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn license(&self) -> &str {
        &self.license
    }
}

/// Slippy-map XYZ tile coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeoTileKey {
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

impl GeoTileKey {
    pub fn new(z: u8, x: u32, y: u32) -> Result<Self, GeoError> {
        if z > MAX_TILE_ZOOM {
            return Err(GeoError::InvalidParameter("tile zoom must be <= 18"));
        }
        let n = 1_u32 << z;
        if x >= n || y >= n {
            return Err(GeoError::InvalidParameter("tile x/y out of range for zoom"));
        }
        Ok(Self { z, x, y })
    }

    pub fn tile_bounds(self) -> GeoBounds {
        let n = (1_u32 << self.z) as f64;
        let lon_w = (f64::from(self.x) / n) * 360.0 - 180.0;
        let lon_e = (f64::from(self.x + 1) / n) * 360.0 - 180.0;
        let lat_n = (PI * (1.0 - 2.0 * f64::from(self.y) / n))
            .sinh()
            .atan()
            .to_degrees()
            .clamp(-85.051_128_78, 85.051_128_78);
        let lat_s = (PI * (1.0 - 2.0 * f64::from(self.y + 1) / n))
            .sinh()
            .atan()
            .to_degrees()
            .clamp(-85.051_128_78, 85.051_128_78);
        GeoBounds::from_lat_lon([lat_s, lat_n], [lon_w, lon_e])
            .unwrap_or_else(|_| GeoBounds::world())
    }
}

/// Enumerates XYZ tiles covering `bounds` at zoom `z`, wrapping across `±180°`.
pub fn visible_tiles_for_bounds(bounds: GeoBounds, z: u8, max_tiles: usize) -> Vec<GeoTileKey> {
    let z = z.min(MAX_TILE_ZOOM);
    let n_u32 = 1_u32 << z;
    let n = f64::from(n_u32);
    let n_i64 = i64::from(n_u32);
    let lat_top = bounds
        .lat_max
        .clamp(-85.051_128_78, 85.051_128_78)
        .to_radians();
    let lat_bot = bounds
        .lat_min
        .clamp(-85.051_128_78, 85.051_128_78)
        .to_radians();
    let ty_from_lat = |lat_r: f64| -> i64 {
        let m = (0.5 * (1.0 - (lat_r.tan() + 1.0 / lat_r.cos()).ln() / PI)) * n;
        (m.floor() as i64).clamp(0, n_i64 - 1)
    };
    let y0 = ty_from_lat(lat_top);
    let y1 = ty_from_lat(lat_bot).max(y0);
    let tx_min = ((bounds.center_lon - 0.5 * bounds.lon_span + 180.0) / 360.0) * n;
    let tx_max = ((bounds.center_lon + 0.5 * bounds.lon_span + 180.0) / 360.0) * n;
    let x0 = tx_min.floor() as i64;
    let x1 = (tx_max.ceil() as i64 - 1).max(x0);
    let mut keys = Vec::new();
    let mut seen = Vec::new();
    'outer: for y in y0..=y1 {
        for x in x0..=x1 {
            let wx = x.rem_euclid(n_i64) as u32;
            let key = GeoTileKey {
                z,
                x: wx,
                y: y as u32,
            };
            if !seen.contains(&key) {
                seen.push(key);
                keys.push(key);
                if keys.len() >= max_tiles {
                    break 'outer;
                }
            }
        }
    }
    let _ = wrap_lon_180;
    keys
}

/// Host-supplied uncompressed RGBA raster basemap tile (`<= 256x256`).
#[derive(Clone, Debug)]
pub struct GeoRasterTile {
    width: u16,
    height: u16,
    rgba: Arc<[u8]>,
}

impl GeoRasterTile {
    pub fn from_rgba(width: u16, height: u16, rgba: Vec<u8>) -> Result<Self, GeoError> {
        if width == 0 || height == 0 || width > MAX_TILE_DIM || height > MAX_TILE_DIM {
            return Err(GeoError::InvalidParameter(
                "tile width/height must be 1..=256",
            ));
        }
        let expected = usize::from(width) * usize::from(height) * 4;
        if rgba.len() != expected {
            return Err(GeoError::LengthMismatch {
                expected,
                actual: rgba.len(),
            });
        }
        Ok(Self {
            width,
            height,
            rgba: rgba.into(),
        })
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub fn byte_len(&self) -> usize {
        self.rgba.len()
    }

    pub fn pixel(&self, col: usize, row: usize) -> [u8; 4] {
        let idx = (row * usize::from(self.width) + col) * 4;
        [
            self.rgba[idx],
            self.rgba[idx + 1],
            self.rgba[idx + 2],
            self.rgba[idx + 3],
        ]
    }
}

/// Host-owned byte-budgeted LRU cache requiring explicit attribution.
#[derive(Clone, Debug)]
pub struct GeoTileCache {
    budget_bytes: usize,
    retained_bytes: usize,
    attribution: GeoAttribution,
    offline_policy: GeoOfflinePolicy,
    entries: HashMap<GeoTileKey, GeoRasterTile>,
    lru: VecDeque<GeoTileKey>,
    evictions: usize,
    hits: usize,
    misses: usize,
}

impl GeoTileCache {
    pub fn new(budget_bytes: usize, attribution: GeoAttribution) -> Result<Self, GeoError> {
        if attribution.text().trim().is_empty() {
            return Err(GeoError::AttributionRequired);
        }
        Ok(Self {
            budget_bytes,
            retained_bytes: 0,
            attribution,
            offline_policy: GeoOfflinePolicy::default(),
            entries: HashMap::new(),
            lru: VecDeque::new(),
            evictions: 0,
            hits: 0,
            misses: 0,
        })
    }

    pub fn with_offline_policy(mut self, policy: GeoOfflinePolicy) -> Self {
        self.offline_policy = policy;
        self
    }

    pub const fn offline_policy(&self) -> GeoOfflinePolicy {
        self.offline_policy
    }

    pub fn attribution(&self) -> &GeoAttribution {
        &self.attribution
    }

    pub const fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub const fn evictions(&self) -> usize {
        self.evictions
    }

    pub const fn hits(&self) -> usize {
        self.hits
    }

    pub const fn misses(&self) -> usize {
        self.misses
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru.clear();
        self.retained_bytes = 0;
    }

    pub fn insert(&mut self, key: GeoTileKey, tile: GeoRasterTile) -> Result<(), GeoError> {
        let required = tile.byte_len();
        if required > self.budget_bytes {
            return Err(GeoError::CacheBudgetExceeded {
                required,
                budget: self.budget_bytes,
            });
        }
        if let Some(old) = self.entries.remove(&key) {
            self.retained_bytes = self.retained_bytes.saturating_sub(old.byte_len());
            self.lru.retain(|k| *k != key);
        }
        while self.retained_bytes + required > self.budget_bytes {
            let Some(victim) = self.lru.pop_front() else {
                break;
            };
            if let Some(removed) = self.entries.remove(&victim) {
                self.retained_bytes = self.retained_bytes.saturating_sub(removed.byte_len());
                self.evictions += 1;
            }
        }
        self.retained_bytes += required;
        self.entries.insert(key, tile);
        self.lru.push_back(key);
        Ok(())
    }

    pub fn get(&mut self, key: GeoTileKey) -> Option<GeoRasterTile> {
        if let Some(tile) = self.entries.get(&key).cloned() {
            self.hits += 1;
            self.lru.retain(|k| *k != key);
            self.lru.push_back(key);
            Some(tile)
        } else {
            self.misses += 1;
            None
        }
    }

    pub(crate) fn best_zoom_for(&self, desired_z: u8) -> u8 {
        if self.entries.is_empty() {
            return desired_z;
        }
        if self.entries.keys().any(|k| k.z == desired_z) {
            return desired_z;
        }
        self.entries
            .keys()
            .map(|k| k.z)
            .min_by_key(|z| z.abs_diff(desired_z))
            .unwrap_or(desired_z)
    }
}
