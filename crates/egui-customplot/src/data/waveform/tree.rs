use std::ops::Range;

use crate::data::QueryStats;

use super::{UniformWaveform, WAVE_BLOCK_SIZE, WaveNode};

impl WaveNode {
    pub(super) fn from_sample(idx: u32, y: f32) -> Self {
        if !y.is_finite() {
            return Self::default();
        }
        let log_ok = y > 0.0;
        Self {
            min_y: y,
            max_y: y,
            min_pos_y: if log_ok { y } else { f32::INFINITY },
            max_pos_y: if log_ok { y } else { f32::NEG_INFINITY },
            min_idx: idx,
            max_idx: idx,
            min_pos_idx: if log_ok { idx } else { u32::MAX },
            max_pos_idx: if log_ok { idx } else { u32::MAX },
            valid_linear: 1,
            valid_log_y: u32::from(log_ok),
            first_valid: idx,
            last_valid: idx,
            first_pos_valid: if log_ok { idx } else { u32::MAX },
            last_pos_valid: if log_ok { idx } else { u32::MAX },
        }
    }

    pub(super) fn merge(self, other: Self) -> Self {
        if self.valid_linear == 0 {
            return other;
        }
        if other.valid_linear == 0 {
            return self;
        }
        let left_min =
            self.min_y < other.min_y || (self.min_y == other.min_y && self.min_idx < other.min_idx);
        let left_max =
            self.max_y > other.max_y || (self.max_y == other.max_y && self.max_idx < other.max_idx);
        let (min_pos_y, min_pos_idx, max_pos_y, max_pos_idx, first_pos_valid, last_pos_valid) =
            match (self.valid_log_y > 0, other.valid_log_y > 0) {
                (false, false) => (
                    f32::INFINITY,
                    u32::MAX,
                    f32::NEG_INFINITY,
                    u32::MAX,
                    u32::MAX,
                    u32::MAX,
                ),
                (true, false) => (
                    self.min_pos_y,
                    self.min_pos_idx,
                    self.max_pos_y,
                    self.max_pos_idx,
                    self.first_pos_valid,
                    self.last_pos_valid,
                ),
                (false, true) => (
                    other.min_pos_y,
                    other.min_pos_idx,
                    other.max_pos_y,
                    other.max_pos_idx,
                    other.first_pos_valid,
                    other.last_pos_valid,
                ),
                (true, true) => {
                    let l_min = self.min_pos_y < other.min_pos_y
                        || (self.min_pos_y == other.min_pos_y
                            && self.min_pos_idx < other.min_pos_idx);
                    let l_max = self.max_pos_y > other.max_pos_y
                        || (self.max_pos_y == other.max_pos_y
                            && self.max_pos_idx < other.max_pos_idx);
                    (
                        if l_min {
                            self.min_pos_y
                        } else {
                            other.min_pos_y
                        },
                        if l_min {
                            self.min_pos_idx
                        } else {
                            other.min_pos_idx
                        },
                        if l_max {
                            self.max_pos_y
                        } else {
                            other.max_pos_y
                        },
                        if l_max {
                            self.max_pos_idx
                        } else {
                            other.max_pos_idx
                        },
                        self.first_pos_valid,
                        other.last_pos_valid,
                    )
                }
            };
        Self {
            min_y: if left_min { self.min_y } else { other.min_y },
            max_y: if left_max { self.max_y } else { other.max_y },
            min_pos_y,
            max_pos_y,
            min_idx: if left_min {
                self.min_idx
            } else {
                other.min_idx
            },
            max_idx: if left_max {
                self.max_idx
            } else {
                other.max_idx
            },
            min_pos_idx,
            max_pos_idx,
            valid_linear: self.valid_linear + other.valid_linear,
            valid_log_y: self.valid_log_y + other.valid_log_y,
            first_valid: self.first_valid,
            last_valid: other.last_valid,
            first_pos_valid,
            last_pos_valid,
        }
    }
}

impl UniformWaveform {
    pub(super) fn rebuild_all_leaves(&mut self) {
        let leaf_total = self.len().div_ceil(WAVE_BLOCK_SIZE);
        for leaf_idx in 0..leaf_total {
            self.recompute_leaf(leaf_idx);
        }
        for idx in (1..self.leaf_count).rev() {
            self.tree[idx] = self.tree[idx * 2].merge(self.tree[idx * 2 + 1]);
        }
    }

    pub(super) fn update_leaf_range(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let leaf_lo = start / WAVE_BLOCK_SIZE;
        let leaf_hi = (end - 1) / WAVE_BLOCK_SIZE;
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
        let start = leaf_idx * WAVE_BLOCK_SIZE;
        let end = (start + WAVE_BLOCK_SIZE).min(self.samples.len());
        let mut acc = WaveNode::default();
        for idx in start..end {
            acc = acc.merge(WaveNode::from_sample(idx as u32, self.samples[idx]));
        }
        self.tree[self.leaf_count + leaf_idx] = acc;
    }

    pub(super) fn range_node(
        &self,
        range: Range<usize>,
        stats: Option<&mut QueryStats>,
    ) -> WaveNode {
        let mut left_acc = WaveNode::default();
        let mut right_acc = WaveNode::default();
        let mut start = range.start;
        let mut end = range.end;
        let mut scanned = 0usize;
        let mut nodes = 0usize;
        while start < end && !start.is_multiple_of(WAVE_BLOCK_SIZE) {
            left_acc = left_acc.merge(WaveNode::from_sample(start as u32, self.samples[start]));
            scanned += 1;
            start += 1;
        }
        while start < end && !end.is_multiple_of(WAVE_BLOCK_SIZE) {
            end -= 1;
            right_acc = WaveNode::from_sample(end as u32, self.samples[end]).merge(right_acc);
            scanned += 1;
        }
        let mut left = start / WAVE_BLOCK_SIZE + self.leaf_count;
        let mut right = end / WAVE_BLOCK_SIZE + self.leaf_count;
        while left < right {
            if left % 2 == 1 {
                left_acc = left_acc.merge(self.tree[left]);
                nodes += 1;
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                right_acc = self.tree[right].merge(right_acc);
                nodes += 1;
            }
            left /= 2;
            right /= 2;
        }
        if let Some(s) = stats {
            s.scanned_points += scanned;
            s.visited_nodes += nodes;
        }
        left_acc.merge(right_acc)
    }
}
