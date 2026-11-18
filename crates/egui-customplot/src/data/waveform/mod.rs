//! Compact uniformly-spaced `f32` waveform source (`x = x0 + i * dx`) with
//! $O(1)$ coordinate slicing, an incremental min/max tree, and partial GPU
//! upload tracking.

mod extrema;
mod pick;
mod query;
#[cfg(test)]
mod tests;
mod tree;

use std::{mem::size_of, ops::Range};

use crate::{AxisScale, BoundsError, PlotBounds, PlotPoint, WaveformLine};

pub(crate) const WAVE_BLOCK_SIZE: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaveformError {
    InvalidSpacing,
    BudgetExceeded {
        required_bytes: usize,
        budget_bytes: usize,
    },
    SizeOverflow,
    AllocationFailed,
    OutOfBounds {
        offset: usize,
        count: usize,
        len: usize,
    },
    StaleBatch {
        batch_id: u64,
        last_batch_id: u64,
    },
    RevisionExhausted,
}

impl std::fmt::Display for WaveformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uniform waveform: {self:?}")
    }
}

impl std::error::Error for WaveformError {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WaveNode {
    pub min_y: f32,
    pub max_y: f32,
    pub min_pos_y: f32,
    pub max_pos_y: f32,
    pub min_idx: u32,
    pub max_idx: u32,
    pub min_pos_idx: u32,
    pub max_pos_idx: u32,
    pub valid_linear: u32,
    pub valid_log_y: u32,
    pub first_valid: u32,
    pub last_valid: u32,
    pub first_pos_valid: u32,
    pub last_pos_valid: u32,
}

impl Default for WaveNode {
    fn default() -> Self {
        Self {
            min_y: f32::INFINITY,
            max_y: f32::NEG_INFINITY,
            min_pos_y: f32::INFINITY,
            max_pos_y: f32::NEG_INFINITY,
            min_idx: u32::MAX,
            max_idx: u32::MAX,
            min_pos_idx: u32::MAX,
            max_pos_idx: u32::MAX,
            valid_linear: 0,
            valid_log_y: 0,
            first_valid: u32::MAX,
            last_valid: u32::MAX,
            first_pos_valid: u32::MAX,
            last_pos_valid: u32::MAX,
        }
    }
}

/// Compact uniformly-spaced `f32` waveform (`x = x0 + i * dx`).
/// 10,000,000 samples occupy 40 MB + ~29 MB incremental extrema tree.
#[derive(Debug)]
pub struct UniformWaveform {
    x0: f64,
    dx: f64,
    samples: Box<[f32]>,
    tree: Box<[WaveNode]>,
    leaf_count: usize,
    #[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
    source_id: u64,
    version: u64,
    samples_version: u64,
    prev_samples_version: u64,
    dirty_sample_range: Option<Range<usize>>,
    revision: u64,
    last_batch: Option<u64>,
    bytes: usize,
}

impl UniformWaveform {
    pub fn from_f32(
        x0: f64,
        dx: f64,
        samples: impl Into<Box<[f32]>>,
        budget_bytes: usize,
    ) -> Result<Self, WaveformError> {
        let samples = samples.into();
        Self::build_from_box(x0, dx, samples, budget_bytes)
    }

    pub fn from_f64(
        x0: f64,
        dx: f64,
        samples: &[f64],
        budget_bytes: usize,
    ) -> Result<Self, WaveformError> {
        if !x0.is_finite() || !dx.is_finite() || dx <= 0.0 {
            return Err(WaveformError::InvalidSpacing);
        }
        let (leaf_count, tree_len, total_bytes) = compute_budget(samples.len(), budget_bytes)?;
        let mut buf = Vec::new();
        buf.try_reserve_exact(samples.len())
            .map_err(|_| WaveformError::AllocationFailed)?;
        buf.extend(samples.iter().map(|&v| v as f32));
        Self::finish_build(
            x0,
            dx,
            buf.into_boxed_slice(),
            leaf_count,
            tree_len,
            total_bytes,
        )
    }

    pub fn new_zeroed(
        x0: f64,
        dx: f64,
        len: usize,
        budget_bytes: usize,
    ) -> Result<Self, WaveformError> {
        if !x0.is_finite() || !dx.is_finite() || dx <= 0.0 {
            return Err(WaveformError::InvalidSpacing);
        }
        let (leaf_count, tree_len, total_bytes) = compute_budget(len, budget_bytes)?;
        let mut buf = Vec::new();
        buf.try_reserve_exact(len)
            .map_err(|_| WaveformError::AllocationFailed)?;
        buf.resize(len, 0.0);
        Self::finish_build(
            x0,
            dx,
            buf.into_boxed_slice(),
            leaf_count,
            tree_len,
            total_bytes,
        )
    }

    fn build_from_box(
        x0: f64,
        dx: f64,
        samples: Box<[f32]>,
        budget_bytes: usize,
    ) -> Result<Self, WaveformError> {
        if !x0.is_finite() || !dx.is_finite() || dx <= 0.0 {
            return Err(WaveformError::InvalidSpacing);
        }
        let (leaf_count, tree_len, total_bytes) = compute_budget(samples.len(), budget_bytes)?;
        Self::finish_build(x0, dx, samples, leaf_count, tree_len, total_bytes)
    }

    fn finish_build(
        x0: f64,
        dx: f64,
        samples: Box<[f32]>,
        leaf_count: usize,
        tree_len: usize,
        total_bytes: usize,
    ) -> Result<Self, WaveformError> {
        let mut tree = Vec::new();
        tree.try_reserve_exact(tree_len)
            .map_err(|_| WaveformError::AllocationFailed)?;
        tree.resize(tree_len, WaveNode::default());
        let ver = super::points::next_version();
        let mut wave = Self {
            x0,
            dx,
            samples,
            tree: tree.into_boxed_slice(),
            leaf_count,
            source_id: super::points::next_version(),
            version: ver,
            samples_version: ver,
            prev_samples_version: 0,
            dirty_sample_range: None,
            revision: 0,
            last_batch: None,
            bytes: total_bytes,
        };
        wave.rebuild_all_leaves();
        Ok(wave)
    }

    pub fn x0(&self) -> f64 {
        self.x0
    }
    pub fn dx(&self) -> f64 {
        self.dx
    }
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.samples.as_ref())
    }
    pub fn index_bytes(&self) -> usize {
        std::mem::size_of_val(self.tree.as_ref())
    }
    pub fn allocated_bytes(&self) -> usize {
        self.bytes
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    #[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
    pub(crate) fn source_id(&self) -> u64 {
        self.source_id
    }
    #[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
    pub(crate) fn samples_version(&self) -> u64 {
        self.samples_version
    }
    #[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
    pub(crate) fn prev_samples_version(&self) -> u64 {
        self.prev_samples_version
    }
    #[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
    pub(crate) fn dirty_sample_range(&self) -> Option<&Range<usize>> {
        self.dirty_sample_range.as_ref()
    }
    pub(crate) fn identity(&self) -> (*const f32, usize) {
        (self.samples.as_ptr(), self.samples.len())
    }

    pub fn x_at(&self, index: usize) -> f64 {
        self.x0 + index as f64 * self.dx
    }

    pub fn point_at(&self, index: usize) -> PlotPoint {
        [self.x_at(index), f64::from(self.samples[index])]
    }

    pub fn line(&self) -> WaveformLine<'_> {
        WaveformLine::new(self)
    }

    pub fn write_range_once(
        &mut self,
        batch_id: u64,
        offset: usize,
        values: &[f32],
    ) -> Result<bool, WaveformError> {
        self.update_once(batch_id, None, offset, values)
    }

    pub fn set_x_origin_once(&mut self, batch_id: u64, x0: f64) -> Result<bool, WaveformError> {
        self.update_once(batch_id, Some(x0), 0, &[])
    }

    /// Updates `x0` and/or sample slice `[offset..offset + values.len()]` once per
    /// strictly increasing `batch_id`, updating only touched leaf blocks.
    pub fn update_once(
        &mut self,
        batch_id: u64,
        new_x0: Option<f64>,
        offset: usize,
        values: &[f32],
    ) -> Result<bool, WaveformError> {
        if let Some(last) = self.last_batch {
            if batch_id == last {
                return Ok(false);
            }
            if batch_id < last {
                return Err(WaveformError::StaleBatch {
                    batch_id,
                    last_batch_id: last,
                });
            }
        }
        if let Some(x0) = new_x0
            && !x0.is_finite()
        {
            return Err(WaveformError::InvalidSpacing);
        }
        if offset
            .checked_add(values.len())
            .is_none_or(|end| end > self.samples.len())
        {
            return Err(WaveformError::OutOfBounds {
                offset,
                count: values.len(),
                len: self.samples.len(),
            });
        }
        let rev = self
            .revision
            .checked_add(1)
            .ok_or(WaveformError::RevisionExhausted)?;
        let mut changed = false;
        if let Some(x0) = new_x0 {
            self.x0 = x0;
            changed = true;
        }
        if !values.is_empty() {
            let end = offset + values.len();
            self.samples[offset..end].copy_from_slice(values);
            self.update_leaf_range(offset, end);
            self.prev_samples_version = self.samples_version;
            self.samples_version = super::points::next_version();
            self.dirty_sample_range = Some(offset..end);
            changed = true;
        }
        if changed {
            self.version = super::points::next_version();
            self.revision = rev;
        }
        self.last_batch = Some(batch_id);
        Ok(true)
    }

    pub fn fitted_bounds(&self) -> Result<Option<PlotBounds>, BoundsError> {
        self.fitted_bounds_with_scales([AxisScale::Linear; 2])
    }

    pub fn fitted_bounds_with_scales(
        &self,
        scales: [AxisScale; 2],
    ) -> Result<Option<PlotBounds>, BoundsError> {
        self.extent(scales)
            .map(|(min, max)| PlotBounds::fit_scaled(min, max, scales))
            .transpose()
    }
}

fn compute_budget(len: usize, budget_bytes: usize) -> Result<(usize, usize, usize), WaveformError> {
    if len >= u32::MAX as usize {
        return Err(WaveformError::SizeOverflow);
    }
    let sample_bytes = len
        .checked_mul(size_of::<f32>())
        .ok_or(WaveformError::SizeOverflow)?;
    let leaf_count = len
        .div_ceil(WAVE_BLOCK_SIZE)
        .max(1)
        .checked_next_power_of_two()
        .ok_or(WaveformError::SizeOverflow)?;
    let tree_len = leaf_count
        .checked_mul(2)
        .ok_or(WaveformError::SizeOverflow)?;
    let tree_bytes = tree_len
        .checked_mul(size_of::<WaveNode>())
        .ok_or(WaveformError::SizeOverflow)?;
    let total = sample_bytes
        .checked_add(tree_bytes)
        .ok_or(WaveformError::SizeOverflow)?;
    if total > budget_bytes {
        return Err(WaveformError::BudgetExceeded {
            required_bytes: total,
            budget_bytes,
        });
    }
    Ok((leaf_count, tree_len, total))
}
