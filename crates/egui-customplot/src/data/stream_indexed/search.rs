use std::ops::Range;

use crate::PlotPoint;

use super::{IndexedStream, STREAM_BLOCK_SIZE};

impl IndexedStream {
    pub(super) fn slot_valid(p: PlotPoint, mask: usize) -> bool {
        p[0].is_finite()
            && p[1].is_finite()
            && (mask & 1 == 0 || p[0] > 0.0)
            && (mask & 2 == 0 || p[1] > 0.0)
    }

    pub(super) fn edge_valid_slot(
        &self,
        mut node_idx: usize,
        mut node_lo: usize,
        mut node_hi: usize,
        mask: usize,
        rightmost: bool,
    ) -> Option<u32> {
        if self.tree[node_idx].valid[mask] == 0 {
            return None;
        }
        while node_idx < self.leaf_count {
            let mid = (node_lo + node_hi) / 2;
            let left = node_idx * 2;
            let right = node_idx * 2 + 1;
            let go_right = if rightmost {
                self.tree[right].valid[mask] > 0
            } else {
                self.tree[left].valid[mask] == 0
            };
            if go_right {
                node_idx = right;
                node_lo = mid;
            } else {
                node_idx = left;
                node_hi = mid;
            }
        }
        let lo = node_lo;
        let hi = node_hi.min(self.capacity());
        if rightmost {
            (lo..hi)
                .rev()
                .find(|&i| Self::slot_valid(self.points[i], mask))
                .map(|i| i as u32)
        } else {
            (lo..hi)
                .find(|&i| Self::slot_valid(self.points[i], mask))
                .map(|i| i as u32)
        }
    }

    pub(super) fn extreme_slot_for_mask(
        &self,
        mut node_idx: usize,
        mut node_lo: usize,
        mut node_hi: usize,
        mask: usize,
        find_max: bool,
    ) -> Option<u32> {
        let root = self.tree[node_idx];
        if root.valid[mask] == 0 {
            return None;
        }
        let target_y = if find_max {
            root.max_y[mask]
        } else {
            root.min_y[mask]
        };
        while node_idx < self.leaf_count {
            let mid = (node_lo + node_hi) / 2;
            let left = self.tree[node_idx * 2];
            let go_left = left.valid[mask] > 0
                && if find_max {
                    left.max_y[mask] == target_y
                } else {
                    left.min_y[mask] == target_y
                };
            if go_left {
                node_idx *= 2;
                node_hi = mid;
            } else {
                node_idx = node_idx * 2 + 1;
                node_lo = mid;
            }
        }
        let lo = node_lo;
        let hi = node_hi.min(self.capacity());
        (lo..hi)
            .find(|&i| Self::slot_valid(self.points[i], mask) && self.points[i][1] == target_y)
            .map(|i| i as u32)
    }

    pub(super) fn find_first_ge(
        &self,
        range: Range<usize>,
        boundary: f64,
        comparisons: &mut usize,
    ) -> usize {
        self.find_in_tree(
            1,
            0,
            self.leaf_count * STREAM_BLOCK_SIZE,
            range,
            boundary,
            false,
            comparisons,
        )
    }

    pub(super) fn find_first_gt(
        &self,
        range: Range<usize>,
        boundary: f64,
        comparisons: &mut usize,
    ) -> usize {
        self.find_in_tree(
            1,
            0,
            self.leaf_count * STREAM_BLOCK_SIZE,
            range,
            boundary,
            true,
            comparisons,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn find_in_tree(
        &self,
        node_idx: usize,
        node_start: usize,
        node_end: usize,
        range: Range<usize>,
        boundary: f64,
        strict_gt: bool,
        comparisons: &mut usize,
    ) -> usize {
        if node_end <= range.start || node_start >= range.end {
            return range.end;
        }
        let node = self.tree[node_idx];
        if node.valid[0] == 0 {
            return range.end;
        }
        if node_start >= range.start && node_end <= range.end {
            *comparisons += 1;
            let matches = if strict_gt {
                node.last_x > boundary
            } else {
                node.last_x >= boundary
            };
            if !matches {
                return range.end;
            }
        }
        if node_idx >= self.leaf_count {
            let lo = node_start.max(range.start);
            let hi = node_end.min(range.end);
            for i in lo..hi {
                let x = self.points[i][0];
                if x.is_finite() {
                    *comparisons += 1;
                    let ok = if strict_gt {
                        x > boundary
                    } else {
                        x >= boundary
                    };
                    if ok {
                        return i;
                    }
                }
            }
            return range.end;
        }
        let mid = (node_start + node_end) / 2;
        let left_ans = self.find_in_tree(
            node_idx * 2,
            node_start,
            mid,
            range.clone(),
            boundary,
            strict_gt,
            comparisons,
        );
        if left_ans < range.end {
            left_ans
        } else {
            self.find_in_tree(
                node_idx * 2 + 1,
                mid,
                node_end,
                range,
                boundary,
                strict_gt,
                comparisons,
            )
        }
    }
}
