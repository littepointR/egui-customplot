use std::ops::Range;

use egui::Pos2;

use crate::{AxisScale, PlotPoint, PlotTransform};

use super::{BLOCK_SIZE, LineIndex, QueryStats};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScreenAabb {
    pub min_dist: f32,
    pub exact_single_pos: bool,
}

impl ScreenAabb {
    #[inline]
    pub(crate) fn can_improve(
        self,
        radius: f32,
        best: Option<(usize, f32)>,
        min_chrono_idx: usize,
    ) -> bool {
        if self.min_dist > radius {
            return false;
        }
        match best {
            Some((best_idx, best_dist)) => {
                self.min_dist < best_dist
                    || (self.min_dist == best_dist && min_chrono_idx < best_idx)
            }
            None => true,
        }
    }
}

#[inline]
pub(crate) fn project_axis_raw(transform: PlotTransform, axis: usize, v: f64) -> Option<f32> {
    let val = transform.scales()[axis].forward(v)?;
    let mut f = (val - transform.axis_min(axis)) / transform.axis_span(axis);
    if transform.reversed()[axis] {
        f = 1.0 - f;
    }
    let rect = transform.rect();
    let s = if axis == 0 {
        rect.left() + (f as f32) * rect.width()
    } else {
        rect.bottom() - (f as f32) * rect.height()
    };
    (!s.is_nan()).then_some(s)
}

pub(crate) fn screen_aabb(
    transform: PlotTransform,
    x_range: [f64; 2],
    y_range: Option<[f64; 2]>,
    position: Pos2,
) -> Option<ScreenAabb> {
    let ax = project_axis_raw(transform, 0, x_range[0])?;
    let bx = project_axis_raw(transform, 0, x_range[1])?;
    let (mut sx_lo, mut sx_hi) = (ax.min(bx), ax.max(bx));
    let scales = transform.scales();
    if scales[0] == AxisScale::Log10 {
        if sx_lo.is_finite() {
            sx_lo = sx_lo.next_down();
        }
        if sx_hi.is_finite() {
            sx_hi = sx_hi.next_up();
        }
    }
    let rect = transform.rect();
    let (sy_lo, sy_hi, y_bounded) = if let Some([y0, y1]) = y_range {
        let ay = project_axis_raw(transform, 1, y0)?;
        let by = project_axis_raw(transform, 1, y1)?;
        let (mut lo, mut hi) = (ay.min(by), ay.max(by));
        if scales[1] == AxisScale::Log10 {
            if lo.is_finite() {
                lo = lo.next_down();
            }
            if hi.is_finite() {
                hi = hi.next_up();
            }
        }
        (lo, hi, true)
    } else {
        (rect.top(), rect.bottom(), false)
    };
    let cx_lo = sx_lo.max(rect.left());
    let cx_hi = sx_hi.min(rect.right());
    let cy_lo = sy_lo.max(rect.top());
    let cy_hi = sy_hi.min(rect.bottom());
    if cx_lo > cx_hi || cy_lo > cy_hi {
        return None;
    }
    let exact_single_pos = scales == [AxisScale::Linear; 2]
        && y_bounded
        && sx_lo == sx_hi
        && sy_lo == sy_hi
        && rect.contains(Pos2::new(sx_lo, sy_lo));
    let qx = position.x.clamp(cx_lo, cx_hi);
    let qy = position.y.clamp(cy_lo, cy_hi);
    Some(ScreenAabb {
        min_dist: Pos2::new(qx, qy).distance(position),
        exact_single_pos,
    })
}

impl LineIndex {
    /// Best-first 2D bounding-box search over `candidates` returning the exact
    /// smallest-distance raw point (ties resolved by smallest point index) and
    /// candidate buffer capacity bytes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn pick_runs(
        &self,
        points: &[PlotPoint],
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        initial_best: Option<(usize, f32)>,
        stats: &mut QueryStats,
        mut visit: impl FnMut(usize) -> (bool, Option<(usize, f32)>),
    ) -> usize {
        let mut best = initial_best;
        let root_span = self.leaf_count * BLOCK_SIZE;
        let scales = transform.scales();
        let rect = transform.rect();
        let a = transform
            .point_from_position(egui::pos2(position.x - radius - 1.0, rect.center().y))[0];
        let b = transform
            .point_from_position(egui::pos2(position.x + radius + 1.0, rect.center().y))[0];
        let x_window = if a.is_finite() && b.is_finite() {
            [a.min(b), a.max(b)]
        } else {
            [f64::NEG_INFINITY, f64::INFINITY]
        };
        let ranges = self.candidates(points, scales, x_window, stats);
        let capacity_bytes = ranges.capacity() * std::mem::size_of::<Range<usize>>();
        for range in ranges {
            if range.is_empty() {
                continue;
            }
            if range.len() <= BLOCK_SIZE {
                for idx in range {
                    let (_, next_best) = visit(idx);
                    best = next_best;
                }
                continue;
            }
            self.search_pick_node(
                1, 0, root_span, &range, points, transform, position, radius, &mut best, stats,
                &mut visit,
            );
        }
        capacity_bytes
    }

    #[allow(clippy::too_many_arguments)]
    fn node_aabb(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        run: &Range<usize>,
        points: &[PlotPoint],
        transform: PlotTransform,
        position: Pos2,
        stats: &mut QueryStats,
    ) -> Option<(ScreenAabb, usize, usize)> {
        let sub_lo = node_lo.max(run.start);
        let sub_hi = node_hi.min(run.end);
        if sub_lo >= sub_hi {
            return None;
        }
        stats.visited_nodes += 1;
        let ext = self.tree[node_idx];
        let y_range = if ext.min == u32::MAX {
            if node_lo >= run.start && node_hi <= run.end {
                return None;
            }
            None
        } else if transform.scales()[1] == AxisScale::Log10 && ext.min_y <= 0.0 {
            None
        } else {
            Some([ext.min_y, ext.max_y])
        };
        let x_range = [points[sub_lo][0], points[sub_hi - 1][0]];
        screen_aabb(transform, x_range, y_range, position).map(|aabb| (aabb, sub_lo, sub_hi))
    }

    #[allow(clippy::too_many_arguments)]
    fn search_pick_node(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        run: &Range<usize>,
        points: &[PlotPoint],
        transform: PlotTransform,
        position: Pos2,
        radius: f32,
        best: &mut Option<(usize, f32)>,
        stats: &mut QueryStats,
        visit: &mut impl FnMut(usize) -> (bool, Option<(usize, f32)>),
    ) {
        let Some((aabb, sub_lo, sub_hi)) = self.node_aabb(
            node_idx, node_lo, node_hi, run, points, transform, position, stats,
        ) else {
            return;
        };
        if !aabb.can_improve(radius, *best, sub_lo) {
            return;
        }
        if aabb.exact_single_pos {
            for idx in sub_lo..sub_hi {
                let (eligible, next_best) = visit(idx);
                *best = next_best;
                if eligible {
                    return;
                }
            }
            return;
        }
        if node_idx >= self.leaf_count {
            for idx in sub_lo..sub_hi {
                let (_, next_best) = visit(idx);
                *best = next_best;
                if best.is_some_and(|(bi, bd)| {
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
            run,
            points,
            transform,
            position,
            stats,
        );
        let right = self.node_aabb(
            node_idx * 2 + 1,
            mid,
            node_hi,
            run,
            points,
            transform,
            position,
            stats,
        );
        match (left, right) {
            (Some((la, l_lo, _)), Some((ra, r_lo, _))) => {
                let left_first =
                    la.min_dist < ra.min_dist || (la.min_dist == ra.min_dist && l_lo <= r_lo);
                if left_first {
                    if la.can_improve(radius, *best, l_lo) {
                        self.search_pick_node(
                            node_idx * 2,
                            node_lo,
                            mid,
                            run,
                            points,
                            transform,
                            position,
                            radius,
                            best,
                            stats,
                            visit,
                        );
                    }
                    if ra.can_improve(radius, *best, r_lo) {
                        self.search_pick_node(
                            node_idx * 2 + 1,
                            mid,
                            node_hi,
                            run,
                            points,
                            transform,
                            position,
                            radius,
                            best,
                            stats,
                            visit,
                        );
                    }
                } else {
                    if ra.can_improve(radius, *best, r_lo) {
                        self.search_pick_node(
                            node_idx * 2 + 1,
                            mid,
                            node_hi,
                            run,
                            points,
                            transform,
                            position,
                            radius,
                            best,
                            stats,
                            visit,
                        );
                    }
                    if la.can_improve(radius, *best, l_lo) {
                        self.search_pick_node(
                            node_idx * 2,
                            node_lo,
                            mid,
                            run,
                            points,
                            transform,
                            position,
                            radius,
                            best,
                            stats,
                            visit,
                        );
                    }
                }
            }
            (Some((la, l_lo, _)), None) if la.can_improve(radius, *best, l_lo) => {
                self.search_pick_node(
                    node_idx * 2,
                    node_lo,
                    mid,
                    run,
                    points,
                    transform,
                    position,
                    radius,
                    best,
                    stats,
                    visit,
                );
            }
            (None, Some((ra, r_lo, _))) if ra.can_improve(radius, *best, r_lo) => {
                self.search_pick_node(
                    node_idx * 2 + 1,
                    mid,
                    node_hi,
                    run,
                    points,
                    transform,
                    position,
                    radius,
                    best,
                    stats,
                    visit,
                );
            }
            _ => {}
        }
    }
}
