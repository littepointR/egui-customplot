use std::ops::Range;

use crate::data::QueryStats;

use super::{UniformWaveform, WAVE_BLOCK_SIZE, WaveNode};

impl WaveNode {
    fn from_slice(start: usize, samples: &[f32]) -> Self {
        let mut node = Self::default();
        for (offset, &y) in samples.iter().enumerate() {
            if !y.is_finite() {
                continue;
            }
            let idx = (start + offset) as u32;
            node.valid_linear += 1;
            node.first_valid = node.first_valid.min(idx);
            node.last_valid = idx;
            if y < node.min_y {
                node.min_y = y;
                node.min_idx = idx;
            }
            if y > node.max_y {
                node.max_y = y;
                node.max_idx = idx;
            }
            if y > 0.0 {
                node.valid_log_y += 1;
                node.first_pos_valid = node.first_pos_valid.min(idx);
                node.last_pos_valid = idx;
                if y < node.min_pos_y {
                    node.min_pos_y = y;
                    node.min_pos_idx = idx;
                }
                if y > node.max_pos_y {
                    node.max_pos_y = y;
                    node.max_pos_idx = idx;
                }
            }
        }
        node
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
        self.tree[self.leaf_count + leaf_idx] =
            WaveNode::from_slice(start, &self.samples[start..end]);
    }

    /// Partial sample ranges around the half-open range of complete tree leaves.
    pub(super) fn split_range(
        &self,
        range: Range<usize>,
    ) -> (Range<usize>, Range<usize>, Range<usize>) {
        let start = range.start.next_multiple_of(WAVE_BLOCK_SIZE).min(range.end);
        let end = (range.end / WAVE_BLOCK_SIZE * WAVE_BLOCK_SIZE).max(start);
        let leaves =
            (self.leaf_count + start / WAVE_BLOCK_SIZE)..(self.leaf_count + end / WAVE_BLOCK_SIZE);
        (range.start..start, leaves, end..range.end)
    }

    pub(super) fn range_node(
        &self,
        range: Range<usize>,
        stats: Option<&mut QueryStats>,
    ) -> WaveNode {
        let (prefix, leaves, suffix) = self.split_range(range);
        let scanned = prefix.len() + suffix.len();
        let mut nodes = 0;
        let mut left_acc = WaveNode::from_slice(prefix.start, &self.samples[prefix]);
        let mut right_acc = WaveNode::from_slice(suffix.start, &self.samples[suffix]);
        let (mut left, mut right) = (leaves.start, leaves.end);
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
