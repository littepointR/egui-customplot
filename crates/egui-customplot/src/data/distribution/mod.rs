mod kde;

use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, PlotBounds};

pub use kde::BoxSummary;
pub(super) use kde::{compute_box_stats, gaussian_kde};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatError {
    EmptyInput,
    LengthMismatch,
    InvalidWidth,
    NoFiniteSamples,
}

impl std::fmt::Display for StatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid statistical distribution data: {self:?}")
    }
}
impl std::error::Error for StatError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolinScale {
    Width,
    Area,
    Count,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolinHalf {
    Both,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViolinOptions {
    pub samples: usize,
    pub bandwidth: Option<f64>,
    pub width: f64,
    pub scale: ViolinScale,
    pub half: ViolinHalf,
    pub show_box: bool,
}

impl Default for ViolinOptions {
    fn default() -> Self {
        Self {
            samples: 96,
            bandwidth: None,
            width: 0.72,
            scale: ViolinScale::Width,
            half: ViolinHalf::Both,
            show_box: true,
        }
    }
}

/// Box-and-whisker dataset (`ML-024` `boxchart`).
#[derive(Clone, Debug)]
pub struct BoxChartData {
    boxes: Arc<[BoxSummary]>,
    extents: CachedExtents,
}

impl BoxChartData {
    pub fn from_groups(
        positions: &[f64],
        groups: &[&[f64]],
        width: f64,
    ) -> Result<Self, StatError> {
        if positions.is_empty() || groups.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if positions.len() != groups.len() {
            return Err(StatError::LengthMismatch);
        }
        if !width.is_finite() || width <= 0.0 {
            return Err(StatError::InvalidWidth);
        }
        let mut boxes = Vec::with_capacity(groups.len());
        let mut extents = CachedExtents::default();
        for (&pos, &vals) in positions.iter().zip(groups.iter()) {
            let Some((summary, sorted)) = compute_box_stats(pos, vals, width) else {
                continue;
            };
            let lo_y = sorted[0];
            let hi_y = sorted[sorted.len() - 1];
            let half_w = width * 0.5;
            extents.push(Some(([pos - half_w, lo_y], [pos + half_w, hi_y])));
            boxes.push(summary);
        }
        if boxes.is_empty() {
            return Err(StatError::NoFiniteSamples);
        }
        Ok(Self {
            boxes: boxes.into(),
            extents,
        })
    }

    pub fn from_categories(
        categories: &[usize],
        values: &[f64],
        width: f64,
    ) -> Result<Self, StatError> {
        if categories.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if categories.len() != values.len() {
            return Err(StatError::LengthMismatch);
        }
        let max_cat = categories.iter().copied().max().unwrap_or(0);
        let mut buckets = vec![Vec::new(); max_cat + 1];
        for (&cat, &v) in categories.iter().zip(values.iter()) {
            buckets[cat].push(v);
        }
        let mut positions = Vec::new();
        let mut slices = Vec::new();
        for (cat, bucket) in buckets.iter().enumerate() {
            if !bucket.is_empty() {
                positions.push(cat as f64 + 1.0);
                slices.push(bucket.as_slice());
            }
        }
        Self::from_groups(&positions, &slices, width)
    }

    pub fn boxes(&self) -> &[BoxSummary] {
        &self.boxes
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }

    pub fn bounds_oriented(
        &self,
        scales: [AxisScale; 2],
        orientation: crate::BarOrientation,
    ) -> Option<PlotBounds> {
        match orientation {
            crate::BarOrientation::Vertical => self.bounds(scales),
            crate::BarOrientation::Horizontal => {
                let (lo, hi) = self.extents.extent([scales[1], scales[0]])?;
                PlotBounds::new([lo[1], lo[0]], [hi[1], hi[0]]).ok()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ViolinEntry {
    pub y: Arc<[f64]>,
    pub half_widths: Arc<[f64]>,
    pub summary: BoxSummary,
}

/// Kernel density violin dataset (`ML-025` `violinplot`).
#[derive(Clone, Debug)]
pub struct ViolinData {
    entries: Arc<[ViolinEntry]>,
    options: ViolinOptions,
    extents: CachedExtents,
}

impl ViolinData {
    pub fn from_groups(
        positions: &[f64],
        groups: &[&[f64]],
        options: ViolinOptions,
    ) -> Result<Self, StatError> {
        if positions.is_empty() || groups.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if positions.len() != groups.len() {
            return Err(StatError::LengthMismatch);
        }
        if !options.width.is_finite() || options.width <= 0.0 {
            return Err(StatError::InvalidWidth);
        }
        let mut raw = Vec::new();
        let mut global_peak = 0.0_f64;
        let mut max_count = 1usize;
        for (&pos, &vals) in positions.iter().zip(groups.iter()) {
            let Some((summary, sorted)) = compute_box_stats(pos, vals, options.width * 0.22) else {
                continue;
            };
            max_count = max_count.max(sorted.len());
            let (y, density) = gaussian_kde(&sorted, options.samples, options.bandwidth);
            let peak = density.iter().copied().fold(0.0_f64, f64::max);
            global_peak = global_peak.max(peak);
            raw.push((summary, y, density, peak));
        }
        if raw.is_empty() {
            return Err(StatError::NoFiniteSamples);
        }
        let mut entries = Vec::with_capacity(raw.len());
        let mut extents = CachedExtents::default();
        let half_max = options.width * 0.5;
        for (summary, y, density, peak) in raw {
            let denom = match options.scale {
                ViolinScale::Width => peak.max(1e-12),
                ViolinScale::Area => global_peak.max(1e-12),
                ViolinScale::Count => {
                    (global_peak * max_count as f64 / summary.count.max(1) as f64).max(1e-12)
                }
            };
            let half_widths: Arc<[f64]> = density
                .iter()
                .map(|&d| (d / denom).clamp(0.0, 1.0) * half_max)
                .collect();
            let y_lo = y[0];
            let y_hi = y[y.len() - 1];
            extents.push(Some((
                [summary.position - half_max, y_lo],
                [summary.position + half_max, y_hi],
            )));
            entries.push(ViolinEntry {
                y,
                half_widths,
                summary,
            });
        }
        Ok(Self {
            entries: entries.into(),
            options,
            extents,
        })
    }

    pub fn entries(&self) -> &[ViolinEntry] {
        &self.entries
    }

    pub fn options(&self) -> ViolinOptions {
        self.options
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }

    pub fn bounds_oriented(
        &self,
        scales: [AxisScale; 2],
        orientation: crate::BarOrientation,
    ) -> Option<PlotBounds> {
        match orientation {
            crate::BarOrientation::Vertical => self.bounds(scales),
            crate::BarOrientation::Horizontal => {
                let (lo, hi) = self.extents.extent([scales[1], scales[0]])?;
                PlotBounds::new([lo[1], lo[0]], [hi[1], hi[0]]).ok()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RaincloudEntry {
    pub y: Arc<[f64]>,
    pub half_widths: Arc<[f64]>,
    pub summary: BoxSummary,
    pub drop_x: Arc<[f64]>,
    pub drop_y: Arc<[f64]>,
}

/// Combined half-violin cloud + box summary + deterministic raindrop scatter (`ML-026` `raincloudplot`).
#[derive(Clone, Debug)]
pub struct RaincloudData {
    entries: Arc<[RaincloudEntry]>,
    width: f64,
    extents: CachedExtents,
}

impl RaincloudData {
    pub fn from_groups(
        positions: &[f64],
        groups: &[&[f64]],
        options: ViolinOptions,
    ) -> Result<Self, StatError> {
        let violin = ViolinData::from_groups(positions, groups, options)?;
        let half_w = options.width * 0.5;
        let mut entries = Vec::with_capacity(violin.entries().len());
        let mut extents = CachedExtents::default();
        for (entry, &vals) in violin.entries().iter().zip(groups.iter()) {
            let mut drop_y: Vec<f64> = vals.iter().copied().filter(|v| v.is_finite()).collect();
            drop_y.sort_by(f64::total_cmp);
            let mut drop_x = Vec::with_capacity(drop_y.len());
            for (i, _) in drop_y.iter().enumerate() {
                // Deterministic Van der Corput low-discrepancy scatter in [-0.46*w, -0.10*w].
                let frac = halton_base2(i);
                let x = entry.summary.position - (0.10 + 0.36 * frac) * options.width;
                drop_x.push(x);
            }
            let y_lo = entry.y[0].min(drop_y.first().copied().unwrap_or(entry.y[0]));
            let y_hi = entry.y[entry.y.len() - 1]
                .max(drop_y.last().copied().unwrap_or(entry.y[entry.y.len() - 1]));
            extents.push(Some((
                [entry.summary.position - half_w, y_lo],
                [entry.summary.position + half_w, y_hi],
            )));
            entries.push(RaincloudEntry {
                y: entry.y.clone(),
                half_widths: entry.half_widths.clone(),
                summary: entry.summary.clone(),
                drop_x: drop_x.into(),
                drop_y: drop_y.into(),
            });
        }
        Ok(Self {
            entries: entries.into(),
            width: options.width,
            extents,
        })
    }

    pub fn entries(&self) -> &[RaincloudEntry] {
        &self.entries
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }

    pub fn bounds_oriented(
        &self,
        scales: [AxisScale; 2],
        orientation: crate::BarOrientation,
    ) -> Option<PlotBounds> {
        match orientation {
            crate::BarOrientation::Vertical => self.bounds(scales),
            crate::BarOrientation::Horizontal => {
                let (lo, hi) = self.extents.extent([scales[1], scales[0]])?;
                PlotBounds::new([lo[1], lo[0]], [hi[1], hi[0]]).ok()
            }
        }
    }
}

fn halton_base2(mut index: usize) -> f64 {
    index += 1;
    let mut f = 1.0;
    let mut r = 0.0;
    while index > 0 {
        f *= 0.5;
        r += f * (index & 1) as f64;
        index >>= 1;
    }
    r
}
