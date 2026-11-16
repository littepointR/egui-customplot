use egui::Pos2;

use crate::{
    AxisScale, PlotPoint, PlotTransform,
    data::{
        QueryStats,
        index::{ScreenAabb, screen_aabb},
    },
};

use super::{UniformWaveform, WAVE_BLOCK_SIZE};

impl UniformWaveform {
    /// Best-first 2D segment-tree raw-sample pick. Stays sublinear even when
    /// millions of points collapse to a single pixel or screen coordinate.
    pub(crate) fn pick(
        &self,
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        limit: f32,
        stats: &mut QueryStats,
    ) -> Option<(usize, PlotPoint, f32)> {
        if self.samples.is_empty() {
            return None;
        }
        let scales = transform.scales();
        let min_valid_i = if scales[0] == AxisScale::Log10 {
            self.first_pos_x_index()
        } else {
            0
        };
        if min_valid_i >= self.len() {
            return None;
        }
        stats.visited_runs = 1;
        let log_y = scales[1] == AxisScale::Log10;
        let mut best: Option<(usize, PlotPoint, f32)> = None;
        self.search_pick_node(
            1,
            0,
            self.leaf_count * WAVE_BLOCK_SIZE,
            min_valid_i,
            log_y,
            transform,
            position,
            radius,
            limit,
            &mut best,
            stats,
        );
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
        min_valid_i: usize,
        log_y: bool,
        transform: PlotTransform,
        position: Pos2,
        stats: &mut QueryStats,
    ) -> Option<(ScreenAabb, usize, usize, usize)> {
        let sub_lo = node_lo.max(min_valid_i);
        let sub_hi = node_hi.min(self.len());
        if sub_lo >= sub_hi {
            return None;
        }
        stats.visited_nodes += 1;
        let node = self.tree[node_idx];
        let valid = if log_y {
            node.valid_log_y
        } else {
            node.valid_linear
        };
        if valid == 0 {
            return None;
        }
        let full_node = node_lo >= min_valid_i && node_hi <= self.len();
        let (first_i, last_i) = if full_node {
            let (f, l) = if log_y {
                (node.first_pos_valid, node.last_pos_valid)
            } else {
                (node.first_valid, node.last_valid)
            };
            (
                (f as usize).clamp(sub_lo, sub_hi - 1),
                (l as usize).clamp(sub_lo, sub_hi - 1),
            )
        } else {
            (sub_lo, sub_hi - 1)
        };
        let y_range = if log_y {
            [f64::from(node.min_pos_y), f64::from(node.max_pos_y)]
        } else {
            [f64::from(node.min_y), f64::from(node.max_y)]
        };
        let x_range = [self.x_at(first_i), self.x_at(last_i)];
        screen_aabb(transform, x_range, Some(y_range), position)
            .map(|aabb| (aabb, sub_lo, sub_hi, first_i))
    }

    #[allow(clippy::too_many_arguments)]
    fn visit_idx(
        &self,
        idx: usize,
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        best: &mut Option<(usize, PlotPoint, f32)>,
        stats: &mut QueryStats,
    ) -> bool {
        stats.scanned_points += 1;
        let pt = self.point_at(idx);
        if !transform.is_valid_point(pt) {
            return false;
        }
        let screen = transform.position_from_point(pt);
        if !transform.rect().contains(screen) {
            return false;
        }
        let dist = screen.distance(position);
        if dist <= radius
            && best
                .as_ref()
                .is_none_or(|b| dist < b.2 || (dist == b.2 && idx < b.0))
        {
            *best = Some((idx, pt, dist));
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn search_pick_node(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        min_valid_i: usize,
        log_y: bool,
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        limit: f32,
        best: &mut Option<(usize, PlotPoint, f32)>,
        stats: &mut QueryStats,
    ) {
        let Some((aabb, sub_lo, sub_hi, first_i)) = self.node_aabb(
            node_idx,
            node_lo,
            node_hi,
            min_valid_i,
            log_y,
            transform,
            position,
            stats,
        ) else {
            return;
        };
        if !aabb.can_improve(radius, Self::effective_best(*best, radius, limit), first_i) {
            return;
        }
        if aabb.exact_single_pos {
            if node_lo >= min_valid_i && node_hi <= self.len() {
                self.visit_idx(first_i, transform, position, radius, best, stats);
                return;
            }
            if node_idx >= self.leaf_count {
                for idx in sub_lo..sub_hi {
                    if self.visit_idx(idx, transform, position, radius, best, stats) {
                        return;
                    }
                }
                return;
            }
        }
        if node_idx >= self.leaf_count {
            for idx in sub_lo..sub_hi {
                self.visit_idx(idx, transform, position, radius, best, stats);
                if Self::effective_best(*best, radius, limit).is_some_and(|(bi, bd)| {
                    aabb.min_dist > bd || (aabb.min_dist == bd && idx + 1 >= bi)
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
            min_valid_i,
            log_y,
            transform,
            position,
            stats,
        );
        let right = self.node_aabb(
            node_idx * 2 + 1,
            mid,
            node_hi,
            min_valid_i,
            log_y,
            transform,
            position,
            stats,
        );
        match (left, right) {
            (Some((la, _, _, lf)), Some((ra, _, _, rf))) => {
                let left_first =
                    la.min_dist < ra.min_dist || (la.min_dist == ra.min_dist && lf <= rf);
                if left_first {
                    if la.can_improve(radius, Self::effective_best(*best, radius, limit), lf) {
                        self.search_pick_node(
                            node_idx * 2,
                            node_lo,
                            mid,
                            min_valid_i,
                            log_y,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                    if ra.can_improve(radius, Self::effective_best(*best, radius, limit), rf) {
                        self.search_pick_node(
                            node_idx * 2 + 1,
                            mid,
                            node_hi,
                            min_valid_i,
                            log_y,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                } else {
                    if ra.can_improve(radius, Self::effective_best(*best, radius, limit), rf) {
                        self.search_pick_node(
                            node_idx * 2 + 1,
                            mid,
                            node_hi,
                            min_valid_i,
                            log_y,
                            transform,
                            position,
                            radius,
                            limit,
                            best,
                            stats,
                        );
                    }
                    if la.can_improve(radius, Self::effective_best(*best, radius, limit), lf) {
                        self.search_pick_node(
                            node_idx * 2,
                            node_lo,
                            mid,
                            min_valid_i,
                            log_y,
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
            (Some((la, _, _, lf)), None)
                if la.can_improve(radius, Self::effective_best(*best, radius, limit), lf) =>
            {
                self.search_pick_node(
                    node_idx * 2,
                    node_lo,
                    mid,
                    min_valid_i,
                    log_y,
                    transform,
                    position,
                    radius,
                    limit,
                    best,
                    stats,
                );
            }
            (None, Some((ra, _, _, rf)))
                if ra.can_improve(radius, Self::effective_best(*best, radius, limit), rf) =>
            {
                self.search_pick_node(
                    node_idx * 2 + 1,
                    mid,
                    node_hi,
                    min_valid_i,
                    log_y,
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
