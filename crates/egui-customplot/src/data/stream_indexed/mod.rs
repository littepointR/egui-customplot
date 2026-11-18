//! Bounded ring buffer with an incremental physical-slot extrema tree.
//! Supports $O(\text{batch}/128 + \log N)$ appends, $O(1)$ extents and
//! zero-copy rendering/picking without $O(N)$ snapshot clones.

mod pick;
mod query;
mod search;
#[cfg(test)]
mod tests;
mod tree;

use std::mem::size_of;

use crate::{AppendReport, AxisScale, BoundsError, PlotBounds, PlotPoint, StreamError, StreamLine};

pub(crate) const STREAM_BLOCK_SIZE: usize = 128;

#[derive(Clone, Copy, Debug)]
pub(crate) struct StreamNode {
    pub min_x: [f64; 4],
    pub max_x: [f64; 4],
    pub min_y: [f64; 4],
    pub max_y: [f64; 4],
    pub valid: [u32; 4],
    pub min_slot: u32,
    pub max_slot: u32,
    pub first_x: f64,
    pub last_x: f64,
    pub monotonic_x: bool,
    pub non_empty: bool,
}

impl Default for StreamNode {
    fn default() -> Self {
        Self {
            min_x: [f64::INFINITY; 4],
            max_x: [f64::NEG_INFINITY; 4],
            min_y: [f64::INFINITY; 4],
            max_y: [f64::NEG_INFINITY; 4],
            valid: [0; 4],
            min_slot: u32::MAX,
            max_slot: u32::MAX,
            first_x: f64::NAN,
            last_x: f64::NAN,
            monotonic_x: true,
            non_empty: false,
        }
    }
}

/// Fixed-capacity ring buffer with an incrementally updated segment tree.
#[derive(Debug)]
pub struct IndexedStream {
    points: Box<[PlotPoint]>,
    tree: Box<[StreamNode]>,
    leaf_count: usize,
    start: usize,
    len: usize,
    revision: u64,
    version: u64,
    last_batch: Option<u64>,
    bytes: usize,
}

impl IndexedStream {
    /// Allocates both sample ring and incremental segment tree upfront within
    /// `budget_bytes`. Never reallocates on append.
    pub fn new(capacity_points: usize, budget_bytes: usize) -> Result<Self, StreamError> {
        if capacity_points == 0 {
            return Err(StreamError::ZeroCapacity);
        }
        if capacity_points >= u32::MAX as usize {
            return Err(StreamError::SizeOverflow);
        }
        let sample_bytes = capacity_points
            .checked_mul(size_of::<PlotPoint>())
            .ok_or(StreamError::SizeOverflow)?;
        let leaf_count = capacity_points
            .div_ceil(STREAM_BLOCK_SIZE)
            .max(1)
            .checked_next_power_of_two()
            .ok_or(StreamError::SizeOverflow)?;
        let tree_len = leaf_count.checked_mul(2).ok_or(StreamError::SizeOverflow)?;
        let tree_bytes = tree_len
            .checked_mul(size_of::<StreamNode>())
            .ok_or(StreamError::SizeOverflow)?;
        let required_bytes = sample_bytes
            .checked_add(tree_bytes)
            .ok_or(StreamError::SizeOverflow)?;
        if required_bytes > budget_bytes {
            return Err(StreamError::BudgetExceeded {
                required_bytes,
                budget_bytes,
            });
        }
        let mut points = Vec::new();
        points
            .try_reserve_exact(capacity_points)
            .map_err(|_| StreamError::AllocationFailed)?;
        let mut tree = Vec::new();
        tree.try_reserve_exact(tree_len)
            .map_err(|_| StreamError::AllocationFailed)?;
        let actual_bytes =
            points.capacity() * size_of::<PlotPoint>() + tree.capacity() * size_of::<StreamNode>();
        if actual_bytes > budget_bytes {
            return Err(StreamError::BudgetExceeded {
                required_bytes: actual_bytes,
                budget_bytes,
            });
        }
        points.resize(capacity_points, [f64::NAN; 2]);
        tree.resize(tree_len, StreamNode::default());
        Ok(Self {
            points: points.into_boxed_slice(),
            tree: tree.into_boxed_slice(),
            leaf_count,
            start: 0,
            len: 0,
            revision: 0,
            version: super::points::next_version(),
            last_batch: None,
            bytes: sample_bytes + tree_bytes,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn capacity(&self) -> usize {
        self.points.len()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.points.as_ref())
    }
    pub fn index_bytes(&self) -> usize {
        std::mem::size_of_val(self.tree.as_ref())
    }
    pub fn allocated_bytes(&self) -> usize {
        self.bytes
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub(crate) fn identity(&self) -> (*const PlotPoint, usize) {
        (self.points.as_ptr(), self.points.len())
    }

    /// Returns chronological slices without allocation.
    pub fn as_slices(&self) -> (&[PlotPoint], &[PlotPoint]) {
        let first_len = self.len.min(self.capacity() - self.start);
        (
            &self.points[self.start..self.start + first_len],
            &self.points[..self.len - first_len],
        )
    }

    pub fn line(&self) -> StreamLine<'_> {
        StreamLine::new(self)
    }

    /// Appends a batch once per strictly increasing `batch_id`, incrementally
    /// updating only the touched 128-point leaf blocks and their ancestors.
    pub fn append_once(
        &mut self,
        batch_id: u64,
        points: &[PlotPoint],
    ) -> Result<AppendReport, StreamError> {
        let mut report = AppendReport {
            applied: false,
            revision: self.revision,
            retained_input_points: 0,
            evicted_points: 0,
            dropped_input_points: 0,
        };
        if let Some(last_batch_id) = self.last_batch {
            if batch_id == last_batch_id {
                return Ok(report);
            }
            if batch_id < last_batch_id {
                return Err(StreamError::StaleBatch {
                    batch_id,
                    last_batch_id,
                });
            }
        }
        if !points.is_empty() {
            let revision = self
                .revision
                .checked_add(1)
                .ok_or(StreamError::RevisionExhausted)?;
            let capacity = self.capacity();
            report.retained_input_points = points.len().min(capacity);
            report.dropped_input_points = points.len().saturating_sub(capacity);
            report.evicted_points = self
                .len
                .saturating_sub(capacity.saturating_sub(points.len()));
            if points.len() >= capacity {
                self.points
                    .copy_from_slice(&points[points.len() - capacity..]);
                self.start = 0;
                self.len = capacity;
                self.rebuild_all_leaves();
            } else {
                let write_start = (self.start + self.len) % capacity;
                let first_chunk = points.len().min(capacity - write_start);
                self.points[write_start..write_start + first_chunk]
                    .copy_from_slice(&points[..first_chunk]);
                let second_chunk = points.len() - first_chunk;
                if second_chunk > 0 {
                    self.points[..second_chunk].copy_from_slice(&points[first_chunk..]);
                }
                if self.len + points.len() > capacity {
                    self.start = (self.start + (self.len + points.len() - capacity)) % capacity;
                    self.len = capacity;
                } else {
                    self.len += points.len();
                }
                self.update_leaf_range(write_start, write_start + first_chunk);
                if second_chunk > 0 {
                    self.update_leaf_range(0, second_chunk);
                }
            }
            self.revision = revision;
            self.version = super::points::next_version();
        }
        self.last_batch = Some(batch_id);
        report.applied = true;
        report.revision = self.revision;
        Ok(report)
    }

    pub fn clear(&mut self) -> Result<(), StreamError> {
        if !self.is_empty() {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or(StreamError::RevisionExhausted)?;
            self.version = super::points::next_version();
            self.points.fill([f64::NAN; 2]);
            self.tree.fill(StreamNode::default());
            self.start = 0;
            self.len = 0;
        }
        Ok(())
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

    pub(crate) fn non_finite_points(&self) -> usize {
        self.len.saturating_sub(self.tree[1].valid[0] as usize)
    }

    pub(crate) fn invalid_points(&self, scales: [AxisScale; 2]) -> usize {
        let mask = scale_mask(scales);
        self.len.saturating_sub(self.tree[1].valid[mask] as usize)
    }

    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        if self.len == 0 {
            return None;
        }
        let mask = scale_mask(scales);
        let root = &self.tree[1];
        (root.valid[mask] > 0).then_some((
            [root.min_x[mask], root.min_y[mask]],
            [root.max_x[mask], root.max_y[mask]],
        ))
    }
}

pub(super) fn scale_mask(scales: [AxisScale; 2]) -> usize {
    usize::from(scales[0] == AxisScale::Log10) | (usize::from(scales[1] == AxisScale::Log10) << 1)
}
