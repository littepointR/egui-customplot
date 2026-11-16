use std::ops::Range;

use crate::{AxisScale, PlotPoint, PlotTransform, data::QueryStats};

use super::{UniformWaveform, WAVE_BLOCK_SIZE};

pub(crate) struct WaveQuery {
    pub indices: Vec<u32>,
    pub stats: QueryStats,
    pub capacity_bytes: usize,
}

impl UniformWaveform {
    pub(super) fn first_pos_x_index(&self) -> usize {
        if self.x0 > 0.0 || self.samples.is_empty() {
            return 0;
        }
        let approx = ((-self.x0) / self.dx).floor();
        if !approx.is_finite() || approx >= self.samples.len() as f64 {
            return self.samples.len();
        }
        let mut idx = (approx.max(0.0) as usize).min(self.samples.len());
        while idx < self.samples.len() && self.x_at(idx) <= 0.0 {
            idx += 1;
        }
        idx
    }

    pub(crate) fn non_finite_points(&self) -> usize {
        self.len() - self.tree[1].valid_linear as usize
    }

    pub(crate) fn invalid_points(&self, scales: [AxisScale; 2]) -> usize {
        let start = if scales[0] == AxisScale::Log10 {
            self.first_pos_x_index()
        } else {
            0
        };
        if start >= self.len() {
            return self.len();
        }
        let node = if start == 0 {
            self.tree[1]
        } else {
            self.range_node(start..self.len(), None)
        };
        let valid = if scales[1] == AxisScale::Log10 {
            node.valid_log_y
        } else {
            node.valid_linear
        };
        self.len() - valid as usize
    }

    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        if self.samples.is_empty() {
            return None;
        }
        let start = if scales[0] == AxisScale::Log10 {
            self.first_pos_x_index()
        } else {
            0
        };
        if start >= self.len() {
            return None;
        }
        let node = if start == 0 {
            self.tree[1]
        } else {
            self.range_node(start..self.len(), None)
        };
        if scales[1] == AxisScale::Log10 {
            (node.valid_log_y > 0).then(|| {
                (
                    [
                        self.x_at(node.first_pos_valid as usize),
                        f64::from(node.min_pos_y),
                    ],
                    [
                        self.x_at(node.last_pos_valid as usize),
                        f64::from(node.max_pos_y),
                    ],
                )
            })
        } else {
            (node.valid_linear > 0).then(|| {
                (
                    [self.x_at(node.first_valid as usize), f64::from(node.min_y)],
                    [self.x_at(node.last_valid as usize), f64::from(node.max_y)],
                )
            })
        }
    }

    fn visible_range(&self, x_lo: f64, x_hi: f64, scales: [AxisScale; 2]) -> Range<usize> {
        if self.samples.is_empty() {
            return 0..0;
        }
        let x_max_data = self.x_at(self.samples.len() - 1);
        if x_hi < self.x0 || x_lo > x_max_data {
            return 0..0;
        }
        let raw_lo = ((x_lo - self.x0) / self.dx).floor();
        let raw_hi = ((x_hi - self.x0) / self.dx).ceil();
        let min_i = if scales[0] == AxisScale::Log10 {
            self.first_pos_x_index()
        } else {
            0
        };
        let lo = if raw_lo <= 0.0 {
            min_i
        } else {
            (raw_lo as usize).saturating_sub(1).max(min_i)
        };
        let hi = if !raw_hi.is_finite() || raw_hi >= self.samples.len() as f64 {
            self.samples.len()
        } else {
            ((raw_hi.max(0.0) as usize) + 2).min(self.samples.len())
        };
        if lo >= hi { 0..0 } else { lo..hi }
    }

    /// $O(1)$-indexed visible & column-reduced MinMax query (0 binary searches).
    pub(crate) fn query(
        &self,
        transform: PlotTransform,
        pixels_per_point: f32,
        reduce: bool,
    ) -> WaveQuery {
        let mut stats = QueryStats::default();
        let x_bounds = [
            transform.bounds().min()[0].min(transform.bounds().max()[0]),
            transform.bounds().min()[0].max(transform.bounds().max()[0]),
        ];
        let range = self.visible_range(x_bounds[0], x_bounds[1], transform.scales());
        if range.len() < 2 {
            return WaveQuery {
                indices: Vec::new(),
                stats,
                capacity_bytes: 0,
            };
        }
        stats.visited_runs = 1;
        let columns = (transform.rect().width() * pixels_per_point.max(0.01))
            .ceil()
            .clamp(1.0, 16_384.0) as usize;
        if !reduce || range.len() <= columns * 4 {
            let indices: Vec<u32> = range.map(|i| i as u32).collect();
            stats.selected_points = indices.len();
            let capacity_bytes = indices.capacity() * std::mem::size_of::<u32>();
            return WaveQuery {
                indices,
                stats,
                capacity_bytes,
            };
        }
        stats.reduced = true;
        let x_scale = transform.scales()[0];
        let lo = x_scale.forward(x_bounds[0]).expect("valid transform");
        let hi = x_scale.forward(x_bounds[1]).expect("valid transform");
        let log_y = transform.scales()[1] == AxisScale::Log10;
        let x0 = self.x_at(range.start).clamp(x_bounds[0], x_bounds[1]);
        let x1 = self.x_at(range.end - 1).clamp(x_bounds[0], x_bounds[1]);
        let v0 = x_scale.forward(x0).expect("valid transform");
        let v1 = x_scale.forward(x1).expect("valid transform");
        let (first_col, last_col) = if (hi - lo).abs() > 0.0 {
            let u0 = ((v0 - lo) / (hi - lo)).clamp(0.0, 1.0);
            let u1 = ((v1 - lo) / (hi - lo)).clamp(0.0, 1.0);
            let f = ((u0.min(u1) * columns as f64).floor() as usize).clamp(1, columns);
            let l = ((u0.max(u1) * columns as f64).ceil() as usize + 1).clamp(f, columns);
            (f, l)
        } else {
            (1, columns)
        };
        let mut indices = Vec::with_capacity(((last_col - first_col + 1) * 4).min(range.len()));
        let mut col_start = range.start;
        let mut in_gap = false;
        let root_span = self.leaf_count * WAVE_BLOCK_SIZE;
        for column in first_col..=last_col {
            let col_end = if column == last_col {
                range.end
            } else {
                let boundary = x_scale
                    .inverse(lo + (hi - lo) * column as f64 / columns as f64)
                    .unwrap_or(x_bounds[1]);
                let raw = ((boundary - self.x0) / self.dx).ceil();
                if raw <= col_start as f64 {
                    col_start
                } else if raw >= range.end as f64 {
                    range.end
                } else {
                    (raw as usize).clamp(col_start, range.end)
                }
            };
            if col_end > col_start {
                let node = self.range_node(col_start..col_end, Some(&mut stats));
                let valid = if log_y {
                    node.valid_log_y
                } else {
                    node.valid_linear
                };
                if valid == (col_end - col_start) as u32 {
                    in_gap = false;
                    let (min_i, max_i) = if log_y {
                        (node.min_pos_idx, node.max_pos_idx)
                    } else {
                        (node.min_idx, node.max_idx)
                    };
                    push_sorted4(
                        &mut indices,
                        [col_start as u32, min_i, max_i, (col_end - 1) as u32],
                    );
                } else if valid == 0 {
                    if !in_gap && !indices.is_empty() {
                        indices.push(col_start as u32);
                        in_gap = true;
                    }
                } else {
                    let mut mixed_splits = 16usize;
                    self.emit_mixed_column(
                        1,
                        0,
                        root_span,
                        col_start..col_end,
                        log_y,
                        &mut indices,
                        &mut in_gap,
                        &mut stats,
                        &mut mixed_splits,
                    );
                }
            }
            col_start = col_end;
            if col_start >= range.end {
                break;
            }
        }
        stats.selected_points = indices.len();
        let capacity_bytes = indices.capacity() * std::mem::size_of::<u32>();
        WaveQuery {
            indices,
            stats,
            capacity_bytes,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_mixed_column(
        &self,
        node_idx: usize,
        node_lo: usize,
        node_hi: usize,
        col_range: Range<usize>,
        log_y: bool,
        indices: &mut Vec<u32>,
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
            let valid = if log_y {
                node.valid_log_y
            } else {
                node.valid_linear
            };
            if valid == 0 {
                if !*in_gap && !indices.is_empty() {
                    indices.push(node_lo as u32);
                    *in_gap = true;
                }
                return;
            }
            if valid == (node_hi - node_lo) as u32 {
                *in_gap = false;
                let (min_i, max_i) = if log_y {
                    (node.min_pos_idx, node.max_pos_idx)
                } else {
                    (node.min_idx, node.max_idx)
                };
                push_sorted4(
                    indices,
                    [node_lo as u32, min_i, max_i, (node_hi - 1) as u32],
                );
                return;
            }
            if *mixed_splits == 0 {
                let (first_v, last_v, min_i, max_i) = if log_y {
                    (
                        node.first_pos_valid,
                        node.last_pos_valid,
                        node.min_pos_idx,
                        node.max_pos_idx,
                    )
                } else {
                    (
                        node.first_valid,
                        node.last_valid,
                        node.min_idx,
                        node.max_idx,
                    )
                };
                if (first_v as usize) > node_lo && !*in_gap && !indices.is_empty() {
                    indices.push(node_lo as u32);
                }
                *in_gap = false;
                push_sorted4(indices, [first_v, min_i, max_i, last_v]);
                if (last_v as usize) + 1 < node_hi && !indices.is_empty() {
                    indices.push(last_v + 1);
                    *in_gap = true;
                }
                return;
            }
        }
        if node_idx >= self.leaf_count {
            let lo = node_lo.max(col_range.start);
            let hi = node_hi.min(col_range.end);
            for i in lo..hi {
                let y = self.samples[i];
                stats.scanned_points += 1;
                if y.is_finite() && (!log_y || y > 0.0) {
                    *in_gap = false;
                    let idx = i as u32;
                    if indices.last() != Some(&idx) {
                        indices.push(idx);
                    }
                } else if !*in_gap && !indices.is_empty() {
                    *in_gap = true;
                    indices.push(i as u32);
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
                let v = if log_y {
                    self.tree[idx].valid_log_y
                } else {
                    self.tree[idx].valid_linear
                };
                v == 0 || v == (c_hi - c_lo) as u32
            };
            if !pure(node_idx * 2, node_lo, mid) && !pure(node_idx * 2 + 1, mid, node_hi) {
                *mixed_splits = mixed_splits.saturating_sub(1);
            }
        }
        self.emit_mixed_column(
            node_idx * 2,
            node_lo,
            mid,
            col_range.clone(),
            log_y,
            indices,
            in_gap,
            stats,
            mixed_splits,
        );
        self.emit_mixed_column(
            node_idx * 2 + 1,
            mid,
            node_hi,
            col_range,
            log_y,
            indices,
            in_gap,
            stats,
            mixed_splits,
        );
    }
}

fn push_sorted4(indices: &mut Vec<u32>, mut picks: [u32; 4]) {
    picks.sort_unstable();
    for idx in picks {
        if idx != u32::MAX && indices.last() != Some(&idx) {
            indices.push(idx);
        }
    }
}
