use std::ops::Range;

use crate::PlotPoint;

use super::{BLOCK_SIZE, LineIndex, QueryStats};

#[derive(Clone, Copy, Debug)]
pub(super) struct Extrema {
    pub min: usize,
    pub max: usize,
}

impl Default for Extrema {
    fn default() -> Self {
        Self {
            min: usize::MAX,
            max: usize::MAX,
        }
    }
}

impl Extrema {
    pub fn point(index: usize) -> Self {
        Self {
            min: index,
            max: index,
        }
    }

    pub fn merge(self, other: Self, points: &[PlotPoint]) -> Self {
        if self.min == usize::MAX {
            return other;
        }
        if other.min == usize::MAX {
            return self;
        }
        let earlier_min = points[self.min][1] < points[other.min][1]
            || (points[self.min][1] == points[other.min][1] && self.min < other.min);
        let earlier_max = points[self.max][1] > points[other.max][1]
            || (points[self.max][1] == points[other.max][1] && self.max < other.max);
        Self {
            min: if earlier_min { self.min } else { other.min },
            max: if earlier_max { self.max } else { other.max },
        }
    }
}

impl LineIndex {
    /// Scan at most two partial blocks, then use the range-min/max tree.
    pub(super) fn extrema(
        &self,
        points: &[PlotPoint],
        range: Range<usize>,
        stats: &mut QueryStats,
    ) -> Extrema {
        let mut result = Extrema::default();
        let mut start = range.start;
        let mut end = range.end;
        while start < end && !start.is_multiple_of(BLOCK_SIZE) {
            if super::valid(points[start], 0) {
                result = result.merge(Extrema::point(start), points);
            }
            stats.scanned_points += 1;
            start += 1;
        }
        while start < end && !end.is_multiple_of(BLOCK_SIZE) {
            end -= 1;
            if super::valid(points[end], 0) {
                result = result.merge(Extrema::point(end), points);
            }
            stats.scanned_points += 1;
        }
        let mut left = start / BLOCK_SIZE + self.leaf_count;
        let mut right = end / BLOCK_SIZE + self.leaf_count;
        while left < right {
            if left % 2 == 1 {
                result = result.merge(self.tree[left], points);
                stats.visited_nodes += 1;
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                result = result.merge(self.tree[right], points);
                stats.visited_nodes += 1;
            }
            left /= 2;
            right /= 2;
        }
        result
    }
}
