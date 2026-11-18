use std::ops::Range;

use crate::{
    PlotPoint, PlotTransform,
    data::{QueryStats, index::gallop_partition_point},
};

use super::{IndexedStream, STREAM_BLOCK_SIZE, scale_mask};

pub(crate) struct StreamQuery {
    pub slots: Vec<u32>,
    pub monotonic: bool,
    pub stats: QueryStats,
    pub capacity_bytes: usize,
}

impl IndexedStream {
    pub(crate) fn point_at_slot(&self, slot: u32) -> PlotPoint {
        self.points[slot as usize]
    }

    /// Queries visible physical slot indices in chronological order.
    pub(crate) fn query(
        &self,
        transform: PlotTransform,
        pixels_per_point: f32,
        reduce: bool,
    ) -> StreamQuery {
        let mut stats = QueryStats::default();
        if self.len < 2 {
            return StreamQuery {
                slots: Vec::new(),
                monotonic: true,
                stats,
                capacity_bytes: 0,
            };
        }
        let monotonic = self.is_monotonic_x();
        let (a, b) = self.as_slices();
        let mut phys_ranges = Vec::with_capacity(2);
        if !a.is_empty() {
            phys_ranges.push(self.start..self.start + a.len());
        }
        if !b.is_empty() {
            phys_ranges.push(0..b.len());
        }
        if !monotonic {
            let mut slots = Vec::with_capacity(self.len);
            for range in phys_ranges {
                slots.extend(range.map(|i| i as u32));
            }
            stats.selected_points = slots.len();
            let capacity_bytes = slots.capacity() * std::mem::size_of::<u32>();
            return StreamQuery {
                slots,
                monotonic: false,
                stats,
                capacity_bytes,
            };
        }

        let x_bounds = [
            transform.bounds().min()[0].min(transform.bounds().max()[0]),
            transform.bounds().min()[0].max(transform.bounds().max()[0]),
        ];
        let columns = (transform.rect().width() * pixels_per_point.max(0.01))
            .ceil()
            .clamp(1.0, 16_384.0) as usize;
        let x_scale = transform.scales()[0];
        let lo = x_scale.forward(x_bounds[0]).expect("valid transform");
        let hi = x_scale.forward(x_bounds[1]).expect("valid transform");
        let mask = scale_mask(transform.scales());
        let mut slots = Vec::with_capacity((columns * 4).min(self.len));
        let mut in_gap = false;

        for range in phys_ranges {
            stats.visited_runs += 1;
            let node = self.range_node(range.clone());
            if node.valid[mask] == 0 {
                if !in_gap && !slots.is_empty() {
                    slots.push(range.start as u32);
                    in_gap = true;
                }
                continue;
            }
            if node.max_x[mask] < x_bounds[0] {
                in_gap = false;
                slots.push((range.end - 1) as u32);
                continue;
            }
            if node.min_x[mask] > x_bounds[1] {
                in_gap = false;
                slots.push(range.start as u32);
                continue;
            }
            let cand_start = self
                .find_first_ge(range.clone(), x_bounds[0], &mut stats.comparisons)
                .saturating_sub(1)
                .max(range.start);
            let cand_end = (self.find_first_gt(range.clone(), x_bounds[1], &mut stats.comparisons)
                + 1)
            .min(range.end);
            if cand_start >= cand_end {
                continue;
            }
            if !reduce || (cand_end - cand_start) <= columns * 4 {
                slots.extend((cand_start..cand_end).map(|i| i as u32));
                continue;
            }
            stats.reduced = true;
            let cand_node = self.range_node(cand_start..cand_end);
            if cand_node.valid[mask] == 0 {
                if !in_gap && !slots.is_empty() {
                    slots.push(cand_start as u32);
                    in_gap = true;
                }
                continue;
            }
            let (first_col, last_col) = if (hi - lo).abs() > 0.0 {
                let cx0 = cand_node.min_x[mask].clamp(x_bounds[0], x_bounds[1]);
                let cx1 = cand_node.max_x[mask].clamp(x_bounds[0], x_bounds[1]);
                let v0 = x_scale.forward(cx0).expect("valid transform");
                let v1 = x_scale.forward(cx1).expect("valid transform");
                let u0 = ((v0 - lo) / (hi - lo)).clamp(0.0, 1.0);
                let u1 = ((v1 - lo) / (hi - lo)).clamp(0.0, 1.0);
                let f = ((u0.min(u1) * columns as f64).floor() as usize).clamp(1, columns);
                let l = ((u0.max(u1) * columns as f64).ceil() as usize + 1).clamp(f, columns);
                (f, l)
            } else {
                (1, columns)
            };
            let no_gaps = cand_node.valid[mask] == (cand_end - cand_start) as u32;
            let mut col_start = cand_start;
            for column in first_col..=last_col {
                let col_end = if column == last_col {
                    cand_end
                } else {
                    let boundary = x_scale
                        .inverse(lo + (hi - lo) * column as f64 / columns as f64)
                        .unwrap_or(x_bounds[1]);
                    if no_gaps {
                        let hint = ((cand_end - col_start) / (last_col - column + 1).max(1)).max(1);
                        gallop_partition_point(
                            &self.points,
                            col_start,
                            cand_end,
                            hint,
                            boundary,
                            &mut stats.comparisons,
                        )
                    } else {
                        self.find_first_ge(col_start..cand_end, boundary, &mut stats.comparisons)
                    }
                };
                if no_gaps && col_end > col_start {
                    let range = col_start..col_end;
                    let extrema = self.finite_extrema(range.clone(), &mut stats);
                    emit_extrema(range, extrema.min, extrema.max, &mut slots);
                    in_gap = false;
                } else if col_end > col_start {
                    self.emit_column(
                        col_start..col_end,
                        mask,
                        &mut slots,
                        &mut in_gap,
                        &mut stats,
                    );
                }
                col_start = col_end;
                if col_start >= cand_end {
                    break;
                }
            }
        }
        stats.selected_points = slots.len();
        let capacity_bytes = slots.capacity() * std::mem::size_of::<u32>();
        StreamQuery {
            slots,
            monotonic: true,
            stats,
            capacity_bytes,
        }
    }

    fn emit_column(
        &self,
        range: Range<usize>,
        mask: usize,
        slots: &mut Vec<u32>,
        in_gap: &mut bool,
        stats: &mut QueryStats,
    ) {
        let summary = self.range_node(range.clone());
        stats.visited_nodes += 1;
        if summary.valid[mask] == 0 {
            if !*in_gap && !slots.is_empty() {
                slots.push(range.start as u32);
                *in_gap = true;
            }
            return;
        }
        if summary.valid[0] == range.len() as u32
            && (mask == 0 || summary.valid[mask] == summary.valid[0])
        {
            *in_gap = false;
            emit_extrema(range, summary.min_slot, summary.max_slot, slots);
            return;
        }
        let mut mixed_splits = 16usize;
        self.emit_mixed_tree(
            1,
            0,
            self.leaf_count * STREAM_BLOCK_SIZE,
            range,
            mask,
            slots,
            in_gap,
            stats,
            &mut mixed_splits,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_mixed_tree(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        col_range: Range<usize>,
        mask: usize,
        slots: &mut Vec<u32>,
        in_gap: &mut bool,
        stats: &mut QueryStats,
        mixed_splits: &mut usize,
    ) {
        if node_hi <= col_range.start || node_lo >= col_range.end {
            return;
        }
        if node_lo >= col_range.start && node_hi <= col_range.end {
            let node = self.tree[node_idx];
            stats.visited_nodes += 1;
            let valid = node.valid[mask];
            if valid == 0 {
                if !*in_gap && !slots.is_empty() {
                    slots.push(node_lo as u32);
                    *in_gap = true;
                }
                return;
            }
            if node.valid[0] == (node_hi - node_lo) as u32 && (mask == 0 || valid == node.valid[0])
            {
                *in_gap = false;
                emit_extrema(node_lo..node_hi, node.min_slot, node.max_slot, slots);
                return;
            }
            if *mixed_splits == 0 {
                let first_v = self.edge_valid_slot(node_idx, node_lo, node_hi, mask, false);
                let last_v = self.edge_valid_slot(node_idx, node_lo, node_hi, mask, true);
                let (min_s, max_s) = if mask == 0 {
                    (Some(node.min_slot), Some(node.max_slot))
                } else {
                    (
                        self.extreme_slot_for_mask(node_idx, node_lo, node_hi, mask, false),
                        self.extreme_slot_for_mask(node_idx, node_lo, node_hi, mask, true),
                    )
                };
                if let Some(fv) = first_v {
                    if (fv as usize) > node_lo && !*in_gap && !slots.is_empty() {
                        slots.push(node_lo as u32);
                    }
                    *in_gap = false;
                    let mut picks = [
                        fv,
                        min_s.unwrap_or(fv),
                        max_s.unwrap_or(fv),
                        last_v.unwrap_or(fv),
                    ];
                    picks.sort_unstable();
                    for s in picks {
                        if s != u32::MAX && slots.last() != Some(&s) {
                            slots.push(s);
                        }
                    }
                    if let Some(lv) = last_v
                        && (lv as usize) + 1 < node_hi
                        && !slots.is_empty()
                    {
                        slots.push(lv + 1);
                        *in_gap = true;
                    }
                } else if !*in_gap && !slots.is_empty() {
                    slots.push(node_lo as u32);
                    *in_gap = true;
                }
                return;
            }
        }
        if node_idx >= self.leaf_count {
            let lo = node_lo.max(col_range.start);
            let hi = node_hi.min(col_range.end);
            for i in lo..hi {
                let p = self.points[i];
                stats.scanned_points += 1;
                if Self::slot_valid(p, mask) {
                    *in_gap = false;
                    let s = i as u32;
                    if slots.last() != Some(&s) {
                        slots.push(s);
                    }
                } else if !*in_gap && !slots.is_empty() {
                    *in_gap = true;
                    slots.push(i as u32);
                }
            }
            return;
        }
        let mid = (node_lo + node_hi) / 2;
        let l_over = col_range.start < mid && node_lo < col_range.end;
        let r_over = node_hi > col_range.start && mid < col_range.end;
        if l_over && r_over {
            let pure = |idx: usize, c_lo: usize, c_hi: usize| -> bool {
                if c_lo < col_range.start || c_hi > col_range.end {
                    return false;
                }
                let node = self.tree[idx];
                node.valid[mask] == 0
                    || (node.valid[0] == (c_hi - c_lo) as u32
                        && (mask == 0 || node.valid[mask] == node.valid[0]))
            };
            if !pure(node_idx * 2, node_lo, mid) && !pure(node_idx * 2 + 1, mid, node_hi) {
                *mixed_splits = mixed_splits.saturating_sub(1);
            }
        }
        self.emit_mixed_tree(
            node_idx * 2,
            node_lo,
            mid,
            col_range.clone(),
            mask,
            slots,
            in_gap,
            stats,
            mixed_splits,
        );
        self.emit_mixed_tree(
            node_idx * 2 + 1,
            mid,
            node_hi,
            col_range,
            mask,
            slots,
            in_gap,
            stats,
            mixed_splits,
        );
    }
}

fn emit_extrema(range: Range<usize>, min: u32, max: u32, slots: &mut Vec<u32>) {
    let mut picks = [range.start as u32, min, max, (range.end - 1) as u32];
    picks.sort_unstable();
    for slot in picks {
        if slots.last() != Some(&slot) {
            slots.push(slot);
        }
    }
}
