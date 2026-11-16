use std::ops::Range;

use crate::PlotPoint;

use super::{BLOCK_SIZE, LineIndex, QueryStats};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Extrema {
    pub min_y: f64,
    pub max_y: f64,
    pub min: u32,
    pub max: u32,
}

impl Default for Extrema {
    fn default() -> Self {
        Self {
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
            min: u32::MAX,
            max: u32::MAX,
        }
    }
}

impl Extrema {
    pub fn point(index: u32, y: f64) -> Self {
        Self {
            min_y: y,
            max_y: y,
            min: index,
            max: index,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        if self.min == u32::MAX {
            return other;
        }
        if other.min == u32::MAX {
            return self;
        }
        let earlier_min =
            self.min_y < other.min_y || (self.min_y == other.min_y && self.min < other.min);
        let earlier_max =
            self.max_y > other.max_y || (self.max_y == other.max_y && self.max < other.max);
        Self {
            min_y: if earlier_min { self.min_y } else { other.min_y },
            max_y: if earlier_max { self.max_y } else { other.max_y },
            min: if earlier_min { self.min } else { other.min },
            max: if earlier_max { self.max } else { other.max },
        }
    }
}

impl LineIndex {
    /// Scan at most two partial blocks, then use the cache-local range-min/max tree.
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
                result = result.merge(Extrema::point(start as u32, points[start][1]));
            }
            stats.scanned_points += 1;
            start += 1;
        }
        while start < end && !end.is_multiple_of(BLOCK_SIZE) {
            end -= 1;
            if super::valid(points[end], 0) {
                result = result.merge(Extrema::point(end as u32, points[end][1]));
            }
            stats.scanned_points += 1;
        }
        let mut left = start / BLOCK_SIZE + self.leaf_count;
        let mut right = end / BLOCK_SIZE + self.leaf_count;
        while left < right {
            if left % 2 == 1 {
                result = result.merge(self.tree[left]);
                stats.visited_nodes += 1;
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                result = result.merge(self.tree[right]);
                stats.visited_nodes += 1;
            }
            left /= 2;
            right /= 2;
        }
        result
    }
}
