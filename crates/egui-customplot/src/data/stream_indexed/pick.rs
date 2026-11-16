use std::ops::Range;

use egui::Pos2;

use crate::{
    PlotPoint, PlotTransform,
    data::{
        QueryStats,
        index::{ScreenAabb, screen_aabb},
    },
};

use super::{IndexedStream, STREAM_BLOCK_SIZE, scale_mask};

impl IndexedStream {
    /// Best-first 2D segment-tree pick over chronological ring ranges (works for
    /// both monotonic and non-monotonic streams).
    pub(crate) fn pick(
        &self,
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        limit: f32,
        stats: &mut QueryStats,
    ) -> Option<(usize, PlotPoint, f32)> {
        if self.is_empty() {
            return None;
        }
        let mask = scale_mask(transform.scales());
        let (a, b) = self.as_slices();
        let mut phys_ranges: Vec<(Range<usize>, usize)> = Vec::with_capacity(2);
        if !a.is_empty() {
            phys_ranges.push((self.start..self.start + a.len(), 0));
        }
        if !b.is_empty() {
            phys_ranges.push((0..b.len(), a.len()));
        }
        let root_span = self.leaf_count * STREAM_BLOCK_SIZE;
        let mut best: Option<(usize, PlotPoint, f32)> = None;
        for (range, chrono_offset) in phys_ranges {
            stats.visited_runs += 1;
            self.search_pick_node(
                1,
                0,
                root_span,
                &range,
                chrono_offset,
                mask,
                transform,
                position,
                radius,
                limit,
                &mut best,
                stats,
            );
        }
        best
    }

    fn effective_best(
        best: Option<(usize, PlotPoint, f32)>,
        radius: f32,
        limit: f32,
    ) -> Option<(usize, f32)> {
        match best {
            Some((bi, _, bd)) => Some((bi, bd)),
            None if limit < radius => Some((0, limit)),
            None => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn node_aabb(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        phys_range: &Range<usize>,
        chrono_offset: usize,
        mask: usize,
        transform: PlotTransform,
        position: Pos2,
        stats: &mut QueryStats,
    ) -> Option<(ScreenAabb, usize, usize, usize)> {
        let sub_lo = node_lo.max(phys_range.start);
        let sub_hi = node_hi.min(phys_range.end);
        if sub_lo >= sub_hi {
            return None;
        }
        stats.visited_nodes += 1;
        let node = self.tree[node_idx];
        if node.valid[mask] == 0 {
            return None;
        }
        let min_chrono = chrono_offset + (sub_lo - phys_range.start);
        screen_aabb(
            transform,
            [node.min_x[mask], node.max_x[mask]],
            Some([node.min_y[mask], node.max_y[mask]]),
            position,
        )
        .map(|aabb| (aabb, sub_lo, sub_hi, min_chrono))
    }

    #[allow(clippy::too_many_arguments)]
    fn visit_slot(
        &self,
        slot: usize,
        chrono: usize,
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        best: &mut Option<(usize, PlotPoint, f32)>,
        stats: &mut QueryStats,
    ) -> bool {
        let p = self.points[slot];
        stats.scanned_points += 1;
        if !transform.is_valid_point(p) {
            return false;
        }
        let screen = transform.position_from_point(p);
        if !transform.rect().contains(screen) {
            return false;
        }
        let dist = screen.distance(position);
        if dist <= radius
            && best
                .as_ref()
                .is_none_or(|b| dist < b.2 || (dist == b.2 && chrono < b.0))
        {
            *best = Some((chrono, p, dist));
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn search_pick_node(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        phys_range: &Range<usize>,
        chrono_offset: usize,
        mask: usize,
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        limit: f32,
        best: &mut Option<(usize, PlotPoint, f32)>,
        stats: &mut QueryStats,
    ) {
        let Some((aabb, sub_lo, sub_hi, min_chrono)) = self.node_aabb(
            node_idx,
            node_lo,
            node_hi,
            phys_range,
            chrono_offset,
            mask,
            transform,
            position,
            stats,
        ) else {
            return;
        };
        if !aabb.can_improve(
            radius,
            Self::effective_best(*best, radius, limit),
            min_chrono,
        ) {
            return;
        }
        if aabb.exact_single_pos {
            if node_lo >= phys_range.start
                && node_hi <= phys_range.end
                && let Some(first_slot) =
                    self.edge_valid_slot(node_idx, node_lo, node_hi, mask, false)
            {
                let slot = first_slot as usize;
                let chrono = chrono_offset + (slot - phys_range.start);
                self.visit_slot(slot, chrono, transform, position, radius, best, stats);
                return;
            }
            if node_idx >= self.leaf_count {
                for slot in sub_lo..sub_hi {
                    let chrono = chrono_offset + (slot - phys_range.start);
                    if self.visit_slot(slot, chrono, transform, position, radius, best, stats) {
                        return;
                    }
                }
                return;
            }
        }
        if node_idx >= self.leaf_count {
            for slot in sub_lo..sub_hi {
                let chrono = chrono_offset + (slot - phys_range.start);
                self.visit_slot(slot, chrono, transform, position, radius, best, stats);
                if Self::effective_best(*best, radius, limit).is_some_and(|(bi, bd)| {
                    aabb.min_dist > bd || (aabb.min_dist == bd && chrono + 1 >= bi)
                }) {
                    break;
                }
            }
            return;
        }
        let mid = (node_lo + node_hi) / 2;
        let left = self.node_aabb(
            node_idx * 2,
            node_lo,
            mid,
            phys_range,
            chrono_offset,
            mask,
            transform,
            position,
            stats,
        );
        let right = self.node_aabb(
            node_idx * 2 + 1,
            mid,
            node_hi,
            phys_range,
            chrono_offset,
            mask,
            transform,
            position,
            stats,
        );
        match (left, right) {
            (Some((la, _, _, l_ch)), Some((ra, _, _, r_ch))) => {
                let left_first =
                    la.min_dist < ra.min_dist || (la.min_dist == ra.min_dist && l_ch <= r_ch);
                if left_first {
                    if la.can_improve(radius, Self::effective_best(*best, radius, limit), l_ch) {
                        self.search_pick_node(
                            node_idx * 2,
                            node_lo,
                            mid,
                            phys_range,
                            chrono_offset,
                            mask,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                    if ra.can_improve(radius, Self::effective_best(*best, radius, limit), r_ch) {
                        self.search_pick_node(
                            node_idx * 2 + 1,
                            mid,
                            node_hi,
                            phys_range,
                            chrono_offset,
                            mask,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                } else {
                    if ra.can_improve(radius, Self::effective_best(*best, radius, limit), r_ch) {
                        self.search_pick_node(
                            node_idx * 2 + 1,
                            mid,
                            node_hi,
                            phys_range,
                            chrono_offset,
                            mask,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                    if la.can_improve(radius, Self::effective_best(*best, radius, limit), l_ch) {
                        self.search_pick_node(
                            node_idx * 2,
                            node_lo,
                            mid,
                            phys_range,
                            chrono_offset,
                            mask,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                }
            }
            (Some((la, _, _, l_ch)), None)
                if la.can_improve(radius, Self::effective_best(*best, radius, limit), l_ch) =>
            {
                self.search_pick_node(
                    node_idx * 2,
                    node_lo,
                    mid,
                    phys_range,
                    chrono_offset,
                    mask,
                    transform,
                    position,
                    radius,
                    limit,
                    best,
                    stats,
                );
            }
            (None, Some((ra, _, _, r_ch)))
                if ra.can_improve(radius, Self::effective_best(*best, radius, limit), r_ch) =>
            {
                self.search_pick_node(
                    node_idx * 2 + 1,
                    mid,
                    node_hi,
                    phys_range,
                    chrono_offset,
                    mask,
                    transform,
                    position,
                    radius,
                    limit,
                    best,
                    stats,
                );
            }
            _ => {}
        }
    }
}
