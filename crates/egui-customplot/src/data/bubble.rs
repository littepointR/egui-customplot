use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, PlotBounds, PlotPoint, layout::bubble_pixel_radius};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleError {
    Empty,
    LengthMismatch,
    InvalidRadiusRange,
    InvalidJitterWidth,
}

impl std::fmt::Display for BubbleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bubble/swarm data: {self:?}")
    }
}
impl std::error::Error for BubbleError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BubblePoint {
    pub x: f64,
    pub y: f64,
    pub size: f64,
}

/// Immutable scatter-with-size data for `bubblechart` (`ML-015`).
#[derive(Clone, Debug)]
pub struct BubbleData {
    points: Arc<[BubblePoint]>,
    color_values: Option<Arc<[f64]>>,
    size_range: [f64; 2],
    radius_range: [f32; 2],
    extents: CachedExtents,
    finite_count: usize,
}

impl BubbleData {
    pub fn new(xy: &[PlotPoint], sizes: &[f64]) -> Result<Self, BubbleError> {
        if xy.is_empty() {
            return Err(BubbleError::Empty);
        }
        if xy.len() != sizes.len() {
            return Err(BubbleError::LengthMismatch);
        }
        let mut points = Vec::with_capacity(xy.len());
        let mut extents = CachedExtents::default();
        let mut s_lo = f64::INFINITY;
        let mut s_hi = f64::NEG_INFINITY;
        let mut finite_count = 0usize;
        for (&[x, y], &size) in xy.iter().zip(sizes.iter()) {
            points.push(BubblePoint { x, y, size });
            let valid = x.is_finite() && y.is_finite() && size.is_finite() && size >= 0.0;
            if valid {
                finite_count += 1;
                s_lo = s_lo.min(size);
                s_hi = s_hi.max(size);
                extents.push(Some(([x, y], [x, y])));
            } else {
                extents.push(None);
            }
        }
        let size_range = if finite_count > 0 {
            [s_lo, s_hi]
        } else {
            [0.0, 0.0]
        };
        Ok(Self {
            points: points.into(),
            color_values: None,
            size_range,
            radius_range: [4.0, 22.0],
            extents,
            finite_count,
        })
    }

    pub fn with_colors(mut self, colors: &[f64]) -> Result<Self, BubbleError> {
        if colors.len() != self.points.len() {
            return Err(BubbleError::LengthMismatch);
        }
        self.color_values = Some(colors.into());
        Ok(self)
    }

    pub fn radius_range(mut self, min_px: f32, max_px: f32) -> Result<Self, BubbleError> {
        if !min_px.is_finite() || !max_px.is_finite() || min_px < 1.0 || max_px < min_px {
            return Err(BubbleError::InvalidRadiusRange);
        }
        self.radius_range = [min_px.clamp(1.0, 128.0), max_px.clamp(min_px, 128.0)];
        Ok(self)
    }

    pub fn points(&self) -> &[BubblePoint] {
        &self.points
    }

    pub fn color_values(&self) -> Option<&[f64]> {
        self.color_values.as_deref()
    }

    pub fn size_range(&self) -> [f64; 2] {
        self.size_range
    }

    pub fn radius_limits(&self) -> [f32; 2] {
        self.radius_range
    }

    pub fn pixel_radius(&self, size: f64) -> f32 {
        bubble_pixel_radius(size, self.size_range, self.radius_range)
    }

    pub fn finite_count(&self) -> usize {
        self.finite_count
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwarmPoint {
    pub x: f64,
    pub y: f64,
    pub group_x: f64,
    pub sample_index: usize,
}

/// Deterministic kernel-packed beeswarm scatter data (`ML-017` `swarmchart`).
#[derive(Clone, Debug)]
pub struct SwarmData {
    points: Arc<[SwarmPoint]>,
    color_values: Option<Arc<[f64]>>,
    extents: CachedExtents,
    finite_count: usize,
}

impl SwarmData {
    pub fn new(x: &[f64], y: &[f64], jitter_width: f64) -> Result<Self, BubbleError> {
        if x.is_empty() {
            return Err(BubbleError::Empty);
        }
        if x.len() != y.len() {
            return Err(BubbleError::LengthMismatch);
        }
        if !jitter_width.is_finite() || jitter_width < 0.0 {
            return Err(BubbleError::InvalidJitterWidth);
        }

        let mut valid_indices: Vec<usize> = x
            .iter()
            .zip(y.iter())
            .enumerate()
            .filter_map(|(i, (&xi, &yi))| (xi.is_finite() && yi.is_finite()).then_some(i))
            .collect();
        valid_indices.sort_by(|&a, &b| x[a].total_cmp(&x[b]).then_with(|| y[a].total_cmp(&y[b])));

        let mut placed = Vec::with_capacity(valid_indices.len());
        let mut extents = CachedExtents::default();
        let mut start = 0usize;
        while start < valid_indices.len() {
            let group_x = x[valid_indices[start]];
            let mut end = start + 1;
            while end < valid_indices.len() && (x[valid_indices[end]] - group_x).abs() <= 1e-12 {
                end += 1;
            }
            let group = &valid_indices[start..end];
            let y_lo = y[group[0]];
            let y_hi = y[group[group.len() - 1]];
            let span = (y_hi - y_lo).max(1e-9);
            let bins = 32usize;
            let mut bin_counts = vec![0usize; bins];
            for &idx in group {
                let b = (((y[idx] - y_lo) / span) * (bins - 1) as f64).round() as usize;
                bin_counts[b.min(bins - 1)] += 1;
            }
            let max_bin = bin_counts.iter().copied().max().unwrap_or(1).max(1);
            let mut seen_in_bin = vec![0usize; bins];
            for &idx in group {
                let b = (((y[idx] - y_lo) / span) * (bins - 1) as f64)
                    .round()
                    .clamp(0.0, (bins - 1) as f64) as usize;
                let k = seen_in_bin[b];
                seen_in_bin[b] += 1;
                let offset = if k == 0 {
                    0.0
                } else {
                    let ring = k.div_ceil(2) as f64;
                    let sign = if k % 2 == 1 { -1.0 } else { 1.0 };
                    let cap = (max_bin.div_ceil(2) as f64).max(1.0);
                    sign * (ring / cap).min(1.0) * jitter_width * 0.48
                };
                let px = group_x + offset;
                let py = y[idx];
                extents.push(Some(([px, py], [px, py])));
                placed.push(SwarmPoint {
                    x: px,
                    y: py,
                    group_x,
                    sample_index: idx,
                });
            }
            start = end;
        }
        placed.sort_by_key(|p| p.sample_index);
        let finite_count = placed.len();
        Ok(Self {
            points: placed.into(),
            color_values: None,
            extents,
            finite_count,
        })
    }

    pub fn from_groups(
        centers: &[f64],
        groups: &[&[f64]],
        jitter_width: f64,
    ) -> Result<Self, BubbleError> {
        if centers.is_empty() || groups.is_empty() {
            return Err(BubbleError::Empty);
        }
        if centers.len() != groups.len() {
            return Err(BubbleError::LengthMismatch);
        }
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for (&cx, &vals) in centers.iter().zip(groups.iter()) {
            for &v in vals {
                xs.push(cx);
                ys.push(v);
            }
        }
        if xs.is_empty() {
            return Err(BubbleError::Empty);
        }
        Self::new(&xs, &ys, jitter_width)
    }

    pub fn with_colors(mut self, colors: &[f64]) -> Result<Self, BubbleError> {
        if colors.len() < self.points.len() {
            return Err(BubbleError::LengthMismatch);
        }
        self.color_values = Some(colors.into());
        Ok(self)
    }

    pub fn points(&self) -> &[SwarmPoint] {
        &self.points
    }

    pub fn color_values(&self) -> Option<&[f64]> {
        self.color_values.as_deref()
    }

    pub fn finite_count(&self) -> usize {
        self.finite_count
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }
}
