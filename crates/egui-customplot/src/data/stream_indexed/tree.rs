use std::ops::Range;

use crate::{
    PlotPoint,
    data::{QueryStats, index::Extrema},
};

use super::{IndexedStream, STREAM_BLOCK_SIZE, StreamNode};

impl StreamNode {
    fn from_slice(start: usize, points: &[PlotPoint]) -> Self {
        let mut node = Self {
            non_empty: !points.is_empty(),
            ..Self::default()
        };
        for (offset, &[x, y]) in points.iter().enumerate() {
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            if node.valid[0] == 0 {
                node.first_x = x;
            } else {
                node.monotonic_x &= node.last_x <= x;
            }
            node.last_x = x;
            let slot = (start + offset) as u32;
            if y < node.min_y[0] {
                node.min_slot = slot;
            }
            if y > node.max_y[0] {
                node.max_slot = slot;
            }
            for mask in 0..4 {
                if (mask & 1 != 0 && x <= 0.0) || (mask & 2 != 0 && y <= 0.0) {
                    continue;
                }
                node.valid[mask] += 1;
                node.min_x[mask] = node.min_x[mask].min(x);
                node.max_x[mask] = node.max_x[mask].max(x);
                node.min_y[mask] = node.min_y[mask].min(y);
                node.max_y[mask] = node.max_y[mask].max(y);
            }
        }
        node
    }

    pub(super) fn merge(self, other: Self) -> Self {
        if !self.non_empty {
            return other;
        }
        if !other.non_empty {
            return self;
        }
        let mut out = Self {
            min_x: [0.0; 4],
            max_x: [0.0; 4],
            min_y: [0.0; 4],
            max_y: [0.0; 4],
            valid: [0; 4],
            min_slot: u32::MAX,
            max_slot: u32::MAX,
            first_x: if self.valid[0] > 0 {
                self.first_x
            } else {
                other.first_x
            },
            last_x: if other.valid[0] > 0 {
                other.last_x
            } else {
                self.last_x
            },
            monotonic_x: self.monotonic_x
                && other.monotonic_x
                && (self.valid[0] == 0 || other.valid[0] == 0 || self.last_x <= other.first_x),
            non_empty: true,
        };
        for mask in 0..4 {
            out.valid[mask] = self.valid[mask] + other.valid[mask];
            out.min_x[mask] = self.min_x[mask].min(other.min_x[mask]);
            out.max_x[mask] = self.max_x[mask].max(other.max_x[mask]);
            out.min_y[mask] = self.min_y[mask].min(other.min_y[mask]);
            out.max_y[mask] = self.max_y[mask].max(other.max_y[mask]);
        }
        match (self.min_slot != u32::MAX, other.min_slot != u32::MAX) {
            (false, false) => {}
            (true, false) => {
                out.min_slot = self.min_slot;
                out.max_slot = self.max_slot;
            }
            (false, true) => {
                out.min_slot = other.min_slot;
                out.max_slot = other.max_slot;
            }
            (true, true) => {
                let take_left_min = self.min_y[0] < other.min_y[0]
                    || (self.min_y[0] == other.min_y[0] && self.min_slot < other.min_slot);
                let take_left_max = self.max_y[0] > other.max_y[0]
                    || (self.max_y[0] == other.max_y[0] && self.max_slot < other.max_slot);
                out.min_slot = if take_left_min {
                    self.min_slot
                } else {
                    other.min_slot
                };
                out.max_slot = if take_left_max {
                    self.max_slot
                } else {
                    other.max_slot
                };
            }
        }
        out
    }
}

impl IndexedStream {
    pub(super) fn rebuild_all_leaves(&mut self) {
        let leaf_total = self.capacity().div_ceil(STREAM_BLOCK_SIZE);
        for leaf_idx in 0..leaf_total {
            self.recompute_leaf(leaf_idx);
        }
        for idx in (1..self.leaf_count).rev() {
            self.tree[idx] = self.tree[idx * 2].merge(self.tree[idx * 2 + 1]);
        }
    }

    pub(super) fn update_leaf_range(&mut self, slot_start: usize, slot_end: usize) {
        if slot_start >= slot_end {
            return;
        }
        let leaf_lo = slot_start / STREAM_BLOCK_SIZE;
        let leaf_hi = (slot_end - 1) / STREAM_BLOCK_SIZE;
        for leaf_idx in leaf_lo..=leaf_hi {
            self.recompute_leaf(leaf_idx);
        }
        let mut lo = self.leaf_count + leaf_lo;
        let mut hi = self.leaf_count + leaf_hi;
        while lo > 1 {
            lo /= 2;
            hi /= 2;
            for idx in lo..=hi {
                self.tree[idx] = self.tree[idx * 2].merge(self.tree[idx * 2 + 1]);
            }
        }
    }

    fn recompute_leaf(&mut self, leaf_idx: usize) {
        let start = leaf_idx * STREAM_BLOCK_SIZE;
        let end = (start + STREAM_BLOCK_SIZE).min(self.capacity());
        self.tree[self.leaf_count + leaf_idx] =
            StreamNode::from_slice(start, &self.points[start..end]);
    }

    /// Partial leaves surround the half-open range of complete tree leaves.
    fn split_range(&self, range: Range<usize>) -> (Range<usize>, Range<usize>, Range<usize>) {
        let start = range
            .start
            .next_multiple_of(STREAM_BLOCK_SIZE)
            .min(range.end);
        let end = (range.end / STREAM_BLOCK_SIZE * STREAM_BLOCK_SIZE).max(start);
        let blocks = (self.leaf_count + start / STREAM_BLOCK_SIZE)
            ..(self.leaf_count + end / STREAM_BLOCK_SIZE);
        (range.start..start, blocks, end..range.end)
    }

    /// The caller already proved every point in the column is in-domain.
    /// Only the two Y extrema (with earliest-slot ties) are needed here.
    pub(super) fn finite_extrema(&self, range: Range<usize>, stats: &mut QueryStats) -> Extrema {
        let (prefix, blocks, suffix) = self.split_range(range);
        let mut extrema = Extrema::default();
        for edge in [prefix, suffix] {
            stats.scanned_points += edge.len();
            for (i, p) in self.points[edge.clone()].iter().enumerate() {
                extrema = extrema.merge(Extrema::point((edge.start + i) as u32, p[1]));
            }
        }
        let mut merge = |i: usize| {
            stats.visited_nodes += 1;
            let node = &self.tree[i];
            extrema = extrema.merge(Extrema {
                min: node.min_slot,
                max: node.max_slot,
                min_y: node.min_y[0],
                max_y: node.max_y[0],
            });
        };
        let (mut left, mut right) = (blocks.start, blocks.end);
        while left < right {
            if left % 2 == 1 {
                merge(left);
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                merge(right);
            }
            left /= 2;
            right /= 2;
        }
        extrema
    }

    pub(super) fn range_node(&self, range: Range<usize>) -> StreamNode {
        let (prefix, blocks, suffix) = self.split_range(range);
        let mut left_acc = StreamNode::from_slice(prefix.start, &self.points[prefix]);
        let mut right_acc = StreamNode::from_slice(suffix.start, &self.points[suffix]);
        let (mut left, mut right) = (blocks.start, blocks.end);
        while left < right {
            if left % 2 == 1 {
                left_acc = left_acc.merge(self.tree[left]);
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                right_acc = self.tree[right].merge(right_acc);
            }
            left /= 2;
            right /= 2;
        }
        left_acc.merge(right_acc)
    }

    pub(crate) fn is_monotonic_x(&self) -> bool {
        if self.len <= 1 {
            return true;
        }
        let (a, b) = self.as_slices();
        if b.is_empty() {
            self.range_node(self.start..self.start + a.len())
                .monotonic_x
        } else {
            let left = self.range_node(self.start..self.capacity());
            let right = self.range_node(0..b.len());
            left.merge(right).monotonic_x
        }
    }
}
