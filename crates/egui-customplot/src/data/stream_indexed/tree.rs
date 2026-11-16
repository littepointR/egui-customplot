use std::ops::Range;

use crate::PlotPoint;

use super::{IndexedStream, STREAM_BLOCK_SIZE, StreamNode};

impl StreamNode {
    pub(super) fn from_point(slot: u32, p: PlotPoint) -> Self {
        if !p[0].is_finite() || !p[1].is_finite() {
            return Self {
                non_empty: true,
                ..Self::default()
            };
        }
        let mut node = Self {
            min_x: [f64::INFINITY; 4],
            max_x: [f64::NEG_INFINITY; 4],
            min_y: [f64::INFINITY; 4],
            max_y: [f64::NEG_INFINITY; 4],
            valid: [0; 4],
            min_slot: slot,
            max_slot: slot,
            first_x: p[0],
            last_x: p[0],
            starts_finite: true,
            ends_finite: true,
            monotonic_x: true,
            non_empty: true,
        };
        for mask in 0..4 {
            if (mask & 1 != 0 && p[0] <= 0.0) || (mask & 2 != 0 && p[1] <= 0.0) {
                continue;
            }
            node.valid[mask] = 1;
            node.min_x[mask] = p[0];
            node.max_x[mask] = p[0];
            node.min_y[mask] = p[1];
            node.max_y[mask] = p[1];
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
            starts_finite: self.starts_finite,
            ends_finite: other.ends_finite,
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
        let mut acc = StreamNode::default();
        for slot in start..end {
            acc = acc.merge(StreamNode::from_point(slot as u32, self.points[slot]));
        }
        self.tree[self.leaf_count + leaf_idx] = acc;
    }

    pub(super) fn range_node(&self, range: Range<usize>) -> StreamNode {
        let mut left_acc = StreamNode::default();
        let mut right_acc = StreamNode::default();
        let mut start = range.start;
        let mut end = range.end;
        while start < end && !start.is_multiple_of(STREAM_BLOCK_SIZE) {
            left_acc = left_acc.merge(StreamNode::from_point(start as u32, self.points[start]));
            start += 1;
        }
        while start < end && !end.is_multiple_of(STREAM_BLOCK_SIZE) {
            end -= 1;
            right_acc = StreamNode::from_point(end as u32, self.points[end]).merge(right_acc);
        }
        let mut left = start / STREAM_BLOCK_SIZE + self.leaf_count;
        let mut right = end / STREAM_BLOCK_SIZE + self.leaf_count;
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
