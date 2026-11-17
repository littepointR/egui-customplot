use std::{f64::consts::TAU, sync::Arc};

use super::radial_flow::ModernError;
use crate::{AxisScale, PlotBounds};

#[derive(Clone, Debug)]
pub struct TreemapRect {
    pub index: usize,
    pub group: usize,
    pub label: String,
    pub value: f64,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// Squarified Treemap (`TreemapChart`, `EXT-26`) packing area-proportional cells into `[0, 1]^2`.
#[derive(Clone, Debug)]
pub struct TreemapData {
    rects: Arc<[TreemapRect]>,
    total: f64,
}

impl TreemapData {
    pub fn squarified(items: &[(&str, usize, f64)]) -> Result<Self, ModernError> {
        if items.is_empty() {
            return Err(ModernError::Empty);
        }
        let mut total = 0.0_f64;
        for &(_, _, v) in items {
            if !v.is_finite() {
                return Err(ModernError::NonFinite);
            }
            if v <= 0.0 {
                return Err(ModernError::NegativeWeight);
            }
            total += v;
        }
        let mut ordered: Vec<(usize, usize, String, f64)> = items
            .iter()
            .enumerate()
            .map(|(i, &(lbl, g, v))| (i, g, lbl.to_owned(), v / total))
            .collect();
        ordered.sort_by(|a, b| b.3.total_cmp(&a.3));

        let mut rects = Vec::with_capacity(ordered.len());
        squarify_into(&ordered, [0.0, 0.0, 1.0, 1.0], total, &mut rects);
        Ok(Self {
            rects: rects.into(),
            total,
        })
    }

    pub fn rects(&self) -> &[TreemapRect] {
        &self.rects
    }
    pub fn total(&self) -> f64 {
        self.total
    }
    pub fn sample_bytes(&self) -> usize {
        self.rects.len() * std::mem::size_of::<TreemapRect>()
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales[0] != AxisScale::Linear || scales[1] != AxisScale::Linear {
            return None;
        }
        PlotBounds::new([0.0, 0.0], [1.0, 1.0]).ok()
    }
}

fn squarify_into(
    items: &[(usize, usize, String, f64)],
    mut box_r: [f64; 4],
    total: f64,
    out: &mut Vec<TreemapRect>,
) {
    let mut rem_idx = 0usize;
    while rem_idx < items.len() {
        let w = (box_r[2] - box_r[0]).max(1e-9);
        let h = (box_r[3] - box_r[1]).max(1e-9);
        let short = w.min(h);
        let rem_area: f64 = items[rem_idx..].iter().map(|x| x.3).sum();
        let norm_scale = (w * h) / rem_area.max(1e-12);

        let mut row_end = rem_idx + 1;
        let mut best_worst = worst_aspect(items, rem_idx, row_end, short, norm_scale);
        while row_end < items.len() {
            let cand = worst_aspect(items, rem_idx, row_end + 1, short, norm_scale);
            if cand <= best_worst {
                best_worst = cand;
                row_end += 1;
            } else {
                break;
            }
        }

        let row_area: f64 = items[rem_idx..row_end]
            .iter()
            .map(|x| x.3 * norm_scale)
            .sum();
        if w >= h {
            let col_w = row_area / h;
            let mut y = box_r[1];
            for item in &items[rem_idx..row_end] {
                let rh = (item.3 * norm_scale) / col_w;
                out.push(TreemapRect {
                    index: item.0,
                    group: item.1,
                    label: item.2.clone(),
                    value: item.3 * total,
                    x0: box_r[0],
                    y0: y,
                    x1: box_r[0] + col_w,
                    y1: y + rh,
                });
                y += rh;
            }
            box_r[0] += col_w;
        } else {
            let row_h = row_area / w;
            let mut x = box_r[0];
            for item in &items[rem_idx..row_end] {
                let rw = (item.3 * norm_scale) / row_h;
                out.push(TreemapRect {
                    index: item.0,
                    group: item.1,
                    label: item.2.clone(),
                    value: item.3 * total,
                    x0: x,
                    y0: box_r[1],
                    x1: x + rw,
                    y1: box_r[1] + row_h,
                });
                x += rw;
            }
            box_r[1] += row_h;
        }
        rem_idx = row_end;
    }
}

fn worst_aspect(
    items: &[(usize, usize, String, f64)],
    start: usize,
    end: usize,
    short: f64,
    norm_scale: f64,
) -> f64 {
    let mut sum = 0.0_f64;
    let mut min_a = f64::INFINITY;
    let mut max_a = 0.0_f64;
    for item in &items[start..end] {
        let a = item.3 * norm_scale;
        sum += a;
        min_a = min_a.min(a);
        max_a = max_a.max(a);
    }
    let s2 = sum * sum;
    let l2 = short * short;
    (l2 * max_a / s2.max(1e-18)).max(s2 / (l2 * min_a).max(1e-18))
}

#[derive(Clone, Debug)]
pub struct SunburstInput {
    pub label: String,
    pub parent: Option<usize>,
    pub value: f64,
    pub group: usize,
}

#[derive(Clone, Debug)]
pub struct SunburstSector {
    pub index: usize,
    pub depth: usize,
    pub group: usize,
    pub label: String,
    pub value: f64,
    pub r_in: f64,
    pub r_out: f64,
    pub start_rad: f64,
    pub end_rad: f64,
}

/// Concentric hierarchy ring chart (`SunburstChart`, `EXT-26`).
#[derive(Clone, Debug)]
pub struct SunburstData {
    sectors: Arc<[SunburstSector]>,
    max_depth: usize,
}

impl SunburstData {
    pub fn new(nodes: &[SunburstInput]) -> Result<Self, ModernError> {
        let n = nodes.len();
        if n == 0 {
            return Err(ModernError::Empty);
        }
        let mut children = vec![Vec::<usize>::new(); n];
        let mut roots = Vec::new();
        for (i, nd) in nodes.iter().enumerate() {
            if !nd.value.is_finite() || nd.value < 0.0 {
                return Err(ModernError::NegativeWeight);
            }
            match nd.parent {
                None => roots.push(i),
                Some(p) if p < n && p != i => children[p].push(i),
                Some(_) => return Err(ModernError::InvalidHierarchy),
            }
        }
        if roots.is_empty() {
            return Err(ModernError::InvalidHierarchy);
        }

        let mut weights = vec![0.0_f64; n];
        let mut depths = vec![0usize; n];
        let mut visiting = vec![false; n];
        for &r in &roots {
            compute_sunburst_weights(
                r,
                0,
                nodes,
                &children,
                &mut visiting,
                &mut weights,
                &mut depths,
            )?;
        }
        let total: f64 = roots.iter().map(|&r| weights[r]).sum();
        if total <= 0.0 {
            return Err(ModernError::NegativeWeight);
        }
        let max_depth = depths.iter().copied().max().unwrap_or(0) + 1;
        let dr = 0.85 / max_depth as f64;
        let r0 = 0.22_f64;

        let mut sectors = Vec::with_capacity(n);
        let mut a_cursor = 0.0_f64;
        for &r in &roots {
            let span = TAU * (weights[r] / total);
            layout_sunburst(
                r,
                0,
                a_cursor,
                a_cursor + span,
                r0,
                dr,
                nodes,
                &children,
                &weights,
                &mut sectors,
            );
            a_cursor += span;
        }
        Ok(Self {
            sectors: sectors.into(),
            max_depth,
        })
    }

    pub fn sectors(&self) -> &[SunburstSector] {
        &self.sectors
    }
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
    pub fn sample_bytes(&self) -> usize {
        self.sectors.len() * std::mem::size_of::<SunburstSector>()
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales[0] != AxisScale::Linear || scales[1] != AxisScale::Linear {
            return None;
        }
        PlotBounds::new([-1.15, -1.15], [1.15, 1.15]).ok()
    }
}

fn compute_sunburst_weights(
    u: usize,
    depth: usize,
    nodes: &[SunburstInput],
    children: &[Vec<usize>],
    visiting: &mut [bool],
    weights: &mut [f64],
    depths: &mut [usize],
) -> Result<f64, ModernError> {
    if visiting[u] {
        return Err(ModernError::InvalidHierarchy);
    }
    visiting[u] = true;
    depths[u] = depth;
    let w = if children[u].is_empty() {
        nodes[u].value
    } else {
        let mut sum = 0.0;
        for &c in &children[u] {
            sum +=
                compute_sunburst_weights(c, depth + 1, nodes, children, visiting, weights, depths)?;
        }
        sum.max(nodes[u].value)
    };
    visiting[u] = false;
    weights[u] = w;
    Ok(w)
}

#[allow(clippy::too_many_arguments)]
fn layout_sunburst(
    u: usize,
    depth: usize,
    a0: f64,
    a1: f64,
    r0: f64,
    dr: f64,
    nodes: &[SunburstInput],
    children: &[Vec<usize>],
    weights: &[f64],
    out: &mut Vec<SunburstSector>,
) {
    out.push(SunburstSector {
        index: u,
        depth,
        group: nodes[u].group,
        label: nodes[u].label.clone(),
        value: weights[u],
        r_in: r0 + depth as f64 * dr,
        r_out: r0 + (depth + 1) as f64 * dr,
        start_rad: a0,
        end_rad: a1,
    });
    if children[u].is_empty() || weights[u] <= 0.0 {
        return;
    }
    let mut cur = a0;
    for &c in &children[u] {
        let span = (a1 - a0) * (weights[c] / weights[u]);
        layout_sunburst(
            c,
            depth + 1,
            cur,
            cur + span,
            r0,
            dr,
            nodes,
            children,
            weights,
            out,
        );
        cur += span;
    }
}
