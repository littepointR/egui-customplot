use std::ops::Range;

use super::{UniformWaveform, WAVE_BLOCK_SIZE, WaveNode};
use crate::data::{QueryStats, index::Extrema};

pub(super) enum Column {
    Extrema(u32, u32),
    Gap,
    Mixed,
}

impl WaveNode {
    pub(super) fn valid_count(&self, log_y: bool) -> u32 {
        if log_y {
            self.valid_log_y
        } else {
            self.valid_linear
        }
    }

    fn extrema(&self) -> Extrema {
        Extrema {
            min_y: f64::from(self.min_y),
            min: self.min_idx,
            max_y: f64::from(self.max_y),
            max: self.max_idx,
        }
    }
}

impl UniformWaveform {
    pub(super) fn column(
        &self,
        range: Range<usize>,
        log_y: bool,
        all_valid: bool,
        stats: &mut QueryStats,
    ) -> Column {
        if all_valid {
            let ext = self.finite_extrema(range, stats);
            return Column::Extrema(ext.min, ext.max);
        }
        let node = self.range_node(range.clone(), Some(stats));
        match node.valid_count(log_y) as usize {
            0 => Column::Gap,
            n if n == range.len() => Column::Extrema(node.min_idx, node.max_idx),
            _ => Column::Mixed,
        }
    }

    /// Merge complete blocks first. Their extrema can rule out either partial edge.
    fn finite_extrema(&self, range: Range<usize>, stats: &mut QueryStats) -> Extrema {
        let (prefix, leaves, suffix) = self.split_range(range);
        let (mut left, mut right) = (leaves.start, leaves.end);
        let mut best = Extrema::default();
        while left < right {
            if left % 2 == 1 {
                best = best.merge(self.tree[left].extrema());
                stats.visited_nodes += 1;
                left += 1;
            }
            if right % 2 == 1 {
                right -= 1;
                best = best.merge(self.tree[right].extrema());
                stats.visited_nodes += 1;
            }
            left /= 2;
            right /= 2;
        }
        for edge in [prefix, suffix] {
            if !edge.is_empty() {
                best = self.merge_edge(best, edge, stats);
            }
        }
        best
    }

    fn merge_edge(&self, mut best: Extrema, edge: Range<usize>, stats: &mut QueryStats) -> Extrema {
        let leaf = self.tree[self.leaf_count + edge.start / WAVE_BLOCK_SIZE].extrema();
        stats.visited_nodes += 1;
        let slots = edge.start as u32..edge.end as u32;
        if slots.contains(&leaf.min) {
            best = best.merge(Extrema::point(leaf.min, leaf.min_y));
        }
        if slots.contains(&leaf.max) {
            best = best.merge(Extrema::point(leaf.max, leaf.max_y));
        }
        // Extrema store the earliest equal value. Even an out-of-range tie
        // cannot win before both the edge start and the leaf's first occurrence.
        let min_can_win = leaf.min_y < best.min_y
            || (leaf.min_y == best.min_y
                && leaf.min.max(slots.start) < best.min
                && leaf.min < slots.end);
        let max_can_win = leaf.max_y > best.max_y
            || (leaf.max_y == best.max_y
                && leaf.max.max(slots.start) < best.max
                && leaf.max < slots.end);
        if min_can_win || max_can_win {
            stats.scanned_points += edge.len();
            for (offset, &y) in self.samples[edge].iter().enumerate() {
                best = best.merge(Extrema::point(slots.start + offset as u32, f64::from(y)));
            }
        }
        best
    }
}
