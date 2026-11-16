use std::ops::Range;

use egui::{Color32, Pos2};

use crate::{
    AxisScale, PlotPoint, PlotTransform,
    data::{BLOCK_SIZE, LineIndex, project_axis_raw},
};

pub(super) struct GridTreeResult {
    pub chosen: Vec<(Pos2, f32, Color32)>,
    pub visited_points: usize,
    pub numeric_rejections: usize,
    pub reduced: bool,
}

struct GridBuildState {
    occupied: Vec<u64>,
    chosen: Vec<(Pos2, f32, Color32)>,
    visited_points: usize,
    numeric_rejections: usize,
    reduced: bool,
    cols: usize,
    max_gx: usize,
    max_gy: usize,
    scale_x: f32,
    scale_y: f32,
    rect_l: f32,
    rect_t: f32,
    min_px: f32,
    max_px: f32,
    min_py: f32,
    max_py: f32,
}

impl GridBuildState {
    #[inline]
    fn cell_x(&self, sx: f32) -> usize {
        (((sx - self.rect_l) * self.scale_x).clamp(0.0, self.max_gx as f32) as usize)
            .min(self.max_gx)
    }

    #[inline]
    fn cell_y(&self, sy: f32) -> usize {
        (((sy - self.rect_t) * self.scale_y).clamp(0.0, self.max_gy as f32) as usize)
            .min(self.max_gy)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_indexed_grid_markers(
    index: &LineIndex,
    points: &[PlotPoint],
    ranges: &[Range<usize>],
    transform: PlotTransform,
    radius: f32,
    color: Color32,
    ppp: f32,
    fringe: f32,
) -> GridTreeResult {
    let rect = transform.rect();
    let cell = (1.0 / ppp).max(0.5);
    let cols = (rect.width() / cell).ceil().clamp(1.0, 2048.0) as usize;
    let rows = (rect.height() / cell).ceil().clamp(1.0, 2048.0) as usize;
    let pad = radius + fringe;
    let mut state = GridBuildState {
        occupied: vec![0u64; (cols * rows).div_ceil(64)],
        chosen: Vec::new(),
        visited_points: 0,
        numeric_rejections: 0,
        reduced: false,
        cols,
        max_gx: cols - 1,
        max_gy: rows - 1,
        scale_x: cols as f32 / rect.width().max(1e-3),
        scale_y: rows as f32 / rect.height().max(1e-3),
        rect_l: rect.left(),
        rect_t: rect.top(),
        min_px: rect.left() - pad,
        max_px: rect.right() + pad,
        min_py: rect.top() - pad,
        max_py: rect.bottom() + pad,
    };
    let root_span = index.leaf_count() * BLOCK_SIZE;
    for range in ranges.iter().rev() {
        if range.is_empty() {
            continue;
        }
        visit_node(
            index, points, 1, 0, root_span, range, transform, radius, color, &mut state,
        );
    }
    state.chosen.reverse();
    GridTreeResult {
        chosen: state.chosen,
        visited_points: state.visited_points,
        numeric_rejections: state.numeric_rejections,
        reduced: state.reduced,
    }
}

#[allow(clippy::too_many_arguments)]
fn visit_node(
    index: &LineIndex,
    points: &[PlotPoint],
    node_idx: usize,
    node_lo: usize,
    node_hi: usize,
    range: &Range<usize>,
    transform: PlotTransform,
    radius: f32,
    color: Color32,
    state: &mut GridBuildState,
) {
    let sub_lo = node_lo.max(range.start);
    let sub_hi = node_hi.min(range.end);
    if sub_lo >= sub_hi {
        return;
    }
    let ext = index.node_extrema(node_idx);
    if ext.min == u32::MAX {
        return;
    }
    let scales = transform.scales();
    let Some(ax) = project_axis_raw(transform, 0, points[sub_lo][0]) else {
        return;
    };
    let Some(bx) = project_axis_raw(transform, 0, points[sub_hi - 1][0]) else {
        return;
    };
    let (mut sx_lo, mut sx_hi) = (ax.min(bx), ax.max(bx));
    if scales[0] == AxisScale::Log10 {
        if sx_lo.is_finite() {
            sx_lo = sx_lo.next_down();
        }
        if sx_hi.is_finite() {
            sx_hi = sx_hi.next_up();
        }
    }
    if sx_hi < state.min_px || sx_lo > state.max_px {
        return;
    }

    let (sy_lo, sy_hi, y_bounded) = if scales[1] == AxisScale::Log10 && ext.min_y <= 0.0 {
        (state.min_py, state.max_py, false)
    } else if let (Some(ay), Some(by)) = (
        project_axis_raw(transform, 1, ext.min_y),
        project_axis_raw(transform, 1, ext.max_y),
    ) {
        let (mut lo, mut hi) = (ay.min(by), ay.max(by));
        if scales[1] == AxisScale::Log10 {
            if lo.is_finite() {
                lo = lo.next_down();
            }
            if hi.is_finite() {
                hi = hi.next_up();
            }
        }
        if hi < state.min_py || lo > state.max_py {
            return;
        }
        (lo, hi, true)
    } else {
        (state.min_py, state.max_py, false)
    };

    let gx_lo = state.cell_x(sx_lo);
    let gx_hi = state.cell_x(sx_hi);
    let gy_lo = state.cell_y(sy_lo);
    let gy_hi = state.cell_y(sy_hi);
    let fully_in_pad = sx_lo >= state.min_px
        && sx_hi <= state.max_px
        && sy_lo >= state.min_py
        && sy_hi <= state.max_py;

    if scales == [AxisScale::Linear; 2]
        && y_bounded
        && fully_in_pad
        && gx_lo == gx_hi
        && gy_lo == gy_hi
    {
        if sub_hi - sub_lo > 1 {
            state.reduced = true;
        }
        let bit = gy_lo * state.cols + gx_lo;
        let word = &mut state.occupied[bit >> 6];
        let mask = 1u64 << (bit & 63);
        if (*word & mask) == 0 {
            *word |= mask;
            let pos = transform.position_from_point(points[sub_hi - 1]);
            state.visited_points += 1;
            state.chosen.push((pos, radius, color));
        } else {
            state.reduced = true;
        }
        return;
    }

    let cell_w = gx_hi - gx_lo + 1;
    let cell_h = gy_hi - gy_lo + 1;
    if cell_w.saturating_mul(cell_h) <= 64
        && all_cells_occupied(&state.occupied, state.cols, gx_lo, gx_hi, gy_lo, gy_hi)
    {
        if fully_in_pad {
            state.reduced = true;
        }
        return;
    }

    if node_idx >= index.leaf_count() {
        for &point in points[sub_lo..sub_hi].iter().rev() {
            state.visited_points += 1;
            if !transform.is_valid_point(point) {
                continue;
            }
            let center = transform.position_from_point(point);
            if !center.is_finite() {
                state.numeric_rejections += 1;
                continue;
            }
            if !(center.x >= state.min_px
                && center.x <= state.max_px
                && center.y >= state.min_py
                && center.y <= state.max_py)
            {
                continue;
            }
            let gx = state.cell_x(center.x);
            let gy = state.cell_y(center.y);
            let bit = gy * state.cols + gx;
            let word = &mut state.occupied[bit >> 6];
            let mask = 1u64 << (bit & 63);
            if (*word & mask) == 0 {
                *word |= mask;
                state.chosen.push((center, radius, color));
            } else {
                state.reduced = true;
            }
        }
        return;
    }

    let mid = (node_lo + node_hi) / 2;
    visit_node(
        index,
        points,
        node_idx * 2 + 1,
        mid,
        node_hi,
        range,
        transform,
        radius,
        color,
        state,
    );
    visit_node(
        index,
        points,
        node_idx * 2,
        node_lo,
        mid,
        range,
        transform,
        radius,
        color,
        state,
    );
}

fn all_cells_occupied(
    occupied: &[u64],
    cols: usize,
    gx_lo: usize,
    gx_hi: usize,
    gy_lo: usize,
    gy_hi: usize,
) -> bool {
    for gy in gy_lo..=gy_hi {
        let mut bit = gy * cols + gx_lo;
        let end = gy * cols + gx_hi;
        while bit <= end {
            let word_idx = bit >> 6;
            let offset = bit & 63;
            let remaining = end - bit + 1;
            let count = remaining.min(64 - offset);
            let mask = if count == 64 {
                u64::MAX
            } else {
                ((1u64 << count) - 1) << offset
            };
            if (occupied[word_idx] & mask) != mask {
                return false;
            }
            bit += count;
        }
    }
    true
}
