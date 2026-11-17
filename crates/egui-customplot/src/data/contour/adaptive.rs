use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{FunctionContourError, GridError};

const LATTICE_MAX: u16 = 4096;
const MIN_CELL_STEP: u16 = 8;

#[derive(Clone, Copy)]
struct RefineCell {
    ix0: u16,
    iy0: u16,
    step: u16,
    priority: f64,
    seq: usize,
}

impl PartialEq for RefineCell {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq
    }
}
impl Eq for RefineCell {}
impl PartialOrd for RefineCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RefineCell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .total_cmp(&other.priority)
            .then_with(|| other.step.cmp(&self.step))
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

struct Sampler<F> {
    x0: f64,
    dx: f64,
    y0: f64,
    dy: f64,
    max_evals: usize,
    evals: usize,
    memo: HashMap<(u16, u16), usize>,
    verts: Vec<[f64; 3]>,
    f: F,
}

impl<F: FnMut(f64, f64) -> f64> Sampler<F> {
    fn sample(&mut self, ix: u16, iy: u16) -> Option<usize> {
        if let Some(&idx) = self.memo.get(&(ix, iy)) {
            return Some(idx);
        }
        if self.evals >= self.max_evals {
            return None;
        }
        self.evals += 1;
        let x = self.x0 + (f64::from(ix) / f64::from(LATTICE_MAX)) * self.dx;
        let y = self.y0 + (f64::from(iy) / f64::from(LATTICE_MAX)) * self.dy;
        let z = (self.f)(x, y);
        let idx = self.verts.len();
        self.verts.push([x, y, z]);
        self.memo.insert((ix, iy), idx);
        Some(idx)
    }

    fn cell_priority(&self, ix0: u16, iy0: u16, step: u16) -> Option<f64> {
        if step < MIN_CELL_STEP * 2 {
            return None;
        }
        let half = step / 2;
        let corners = [
            self.memo.get(&(ix0, iy0)).copied()?,
            self.memo.get(&(ix0 + step, iy0)).copied()?,
            self.memo.get(&(ix0 + step, iy0 + step)).copied()?,
            self.memo.get(&(ix0, iy0 + step)).copied()?,
        ];
        let zs = corners.map(|i| self.verts[i][2]);
        let finite_count = zs.iter().filter(|z| z.is_finite()).count();
        let size_weight = f64::from(step) / f64::from(LATTICE_MAX);
        if finite_count > 0 && finite_count < 4 {
            return Some(1e6 * size_weight);
        }
        if finite_count == 0 {
            return None;
        }
        let z_min = zs.iter().copied().fold(f64::INFINITY, f64::min);
        let z_max = zs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let span = z_max - z_min;
        let c_idx = self.memo.get(&(ix0 + half, iy0 + half)).copied();
        let z_center = c_idx.map(|i| self.verts[i][2]).filter(|z| z.is_finite());
        let avg = 0.25 * (zs[0] + zs[1] + zs[2] + zs[3]);
        let curvature = z_center.map_or(0.0, |zc| (zc - avg).abs());
        let err = curvature + 0.35 * span;
        (err > 1e-6).then_some(err * (0.25 + size_weight))
    }
}

pub(crate) type AdaptiveTriangles = (Vec<[f64; 3]>, Vec<[u32; 3]>, Option<[f64; 2]>);

/// Watertight adaptive 2D sampler with crack-free boundary split triangulation (`EXT-12`).
pub(crate) fn sample_adaptive_triangles(
    x_range: [f64; 2],
    y_range: [f64; 2],
    max_evaluations: usize,
    f: impl FnMut(f64, f64) -> f64,
) -> Result<AdaptiveTriangles, FunctionContourError> {
    if !x_range[0].is_finite()
        || !x_range[1].is_finite()
        || x_range[1] <= x_range[0]
        || !y_range[0].is_finite()
        || !y_range[1].is_finite()
        || y_range[1] <= y_range[0]
    {
        return Err(FunctionContourError::InvalidGrid(GridError::InvalidEdges));
    }
    if max_evaluations < 25 {
        return Err(FunctionContourError::BudgetExceeded {
            required: 25,
            max_evaluations,
        });
    }
    if max_evaluations > 500_000 {
        return Err(FunctionContourError::InvalidResolution);
    }

    let root_n: u16 = if max_evaluations >= 200 { 8 } else { 4 };
    let root_step = LATTICE_MAX / root_n;
    let mut sampler = Sampler {
        x0: x_range[0],
        dx: x_range[1] - x_range[0],
        y0: y_range[0],
        dy: y_range[1] - y_range[0],
        max_evals: max_evaluations,
        evals: 0,
        memo: HashMap::with_capacity(max_evaluations.min(8192)),
        verts: Vec::with_capacity(max_evaluations.min(8192)),
        f,
    };

    for gy in 0..=root_n {
        for gx in 0..=root_n {
            sampler.sample(gx * root_step, gy * root_step);
        }
    }

    let mut leaves = Vec::new();
    let mut heap = BinaryHeap::new();
    let mut seq = 0usize;
    for gy in 0..root_n {
        for gx in 0..root_n {
            let (ix0, iy0) = (gx * root_step, gy * root_step);
            let half = root_step / 2;
            sampler.sample(ix0 + half, iy0 + half);
            if let Some(p) = sampler.cell_priority(ix0, iy0, root_step) {
                heap.push(RefineCell {
                    ix0,
                    iy0,
                    step: root_step,
                    priority: p,
                    seq,
                });
                seq += 1;
            } else {
                leaves.push((ix0, iy0, root_step));
            }
        }
    }

    while let Some(cell) = heap.pop() {
        let half = cell.step / 2;
        let qtr = half / 2;
        if sampler.evals + 8 > max_evaluations || qtr == 0 {
            leaves.push((cell.ix0, cell.iy0, cell.step));
            continue;
        }
        let (x0, y0, s) = (cell.ix0, cell.iy0, cell.step);
        let ok = sampler.sample(x0 + half, y0).is_some()
            && sampler.sample(x0 + s, y0 + half).is_some()
            && sampler.sample(x0 + half, y0 + s).is_some()
            && sampler.sample(x0, y0 + half).is_some()
            && sampler.sample(x0 + half, y0 + half).is_some();
        if !ok {
            leaves.push((x0, y0, s));
            continue;
        }
        for (cx, cy) in [
            (x0, y0),
            (x0 + half, y0),
            (x0 + half, y0 + half),
            (x0, y0 + half),
        ] {
            if half >= MIN_CELL_STEP * 2 && sampler.evals < max_evaluations {
                sampler.sample(cx + qtr, cy + qtr);
            }
            if let Some(p) = sampler.cell_priority(cx, cy, half) {
                heap.push(RefineCell {
                    ix0: cx,
                    iy0: cy,
                    step: half,
                    priority: p,
                    seq,
                });
                seq += 1;
            } else {
                leaves.push((cx, cy, half));
            }
        }
    }

    let mut tris = Vec::with_capacity(leaves.len() * 3);
    for (ix0, iy0, step) in leaves {
        let corners = [
            (ix0, iy0),
            (ix0 + step, iy0),
            (ix0 + step, iy0 + step),
            (ix0, iy0 + step),
        ];
        let mut ring = Vec::with_capacity(8);
        for edge in 0..4 {
            let a = corners[edge];
            let b = corners[(edge + 1) % 4];
            if let Some(&ia) = sampler.memo.get(&a) {
                ring.push(ia);
            }
            collect_edge_pts(&sampler.memo, a, b, &mut ring);
        }
        triangulate_leaf(&sampler.verts, &ring, &mut tris);
    }

    let mut zmin = f64::INFINITY;
    let mut zmax = f64::NEG_INFINITY;
    for &p in &sampler.verts {
        if p[2].is_finite() {
            zmin = zmin.min(p[2]);
            zmax = zmax.max(p[2]);
        }
    }
    let range = (zmin <= zmax).then_some([zmin, zmax]);
    let cliff_limit = range.map_or(f64::INFINITY, |[lo, hi]| (hi - lo).max(1.0) * 0.75);
    tris.retain(|t| {
        let zs = t.map(|i| sampler.verts[i as usize][2]);
        if zs.iter().any(|z| !z.is_finite()) {
            return false;
        }
        let has_pos = zs.iter().any(|&z| z > 0.0);
        let has_neg = zs.iter().any(|&z| z < 0.0);
        if has_pos && has_neg {
            let span = zs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                - zs.iter().copied().fold(f64::INFINITY, f64::min);
            if span > cliff_limit * 50.0 {
                return false;
            }
        }
        true
    });

    Ok((sampler.verts, tris, range))
}

fn collect_edge_pts(
    memo: &HashMap<(u16, u16), usize>,
    a: (u16, u16),
    b: (u16, u16),
    out: &mut Vec<usize>,
) {
    let mid = (a.0.midpoint(b.0), a.1.midpoint(b.1));
    if mid != a
        && mid != b
        && let Some(&idx) = memo.get(&mid)
    {
        collect_edge_pts(memo, a, mid, out);
        out.push(idx);
        collect_edge_pts(memo, mid, b, out);
    }
}

fn triangulate_leaf(verts: &[[f64; 3]], ring: &[usize], out: &mut Vec<[u32; 3]>) {
    if ring.len() < 3 {
        return;
    }
    if ring.len() == 4 {
        out.push([ring[0] as u32, ring[1] as u32, ring[2] as u32]);
        out.push([ring[0] as u32, ring[2] as u32, ring[3] as u32]);
        return;
    }
    let mut bx0 = f64::INFINITY;
    let mut bx1 = f64::NEG_INFINITY;
    let mut by0 = f64::INFINITY;
    let mut by1 = f64::NEG_INFINITY;
    for &idx in ring {
        let p = verts[idx];
        bx0 = bx0.min(p[0]);
        bx1 = bx1.max(p[0]);
        by0 = by0.min(p[1]);
        by1 = by1.max(p[1]);
    }
    let tol_x = (bx1 - bx0).max(1e-12) * 1e-11;
    let tol_y = (by1 - by0).max(1e-12) * 1e-11;
    let same_side = |pa: [f64; 3], pc: [f64; 3]| {
        ((pa[0] - bx0).abs() <= tol_x && (pc[0] - bx0).abs() <= tol_x)
            || ((pa[0] - bx1).abs() <= tol_x && (pc[0] - bx1).abs() <= tol_x)
            || ((pa[1] - by0).abs() <= tol_y && (pc[1] - by0).abs() <= tol_y)
            || ((pa[1] - by1).abs() <= tol_y && (pc[1] - by1).abs() <= tol_y)
    };
    let mut rem: Vec<usize> = ring.to_vec();
    while rem.len() > 3 {
        let n = rem.len();
        let mut clipped = false;
        for i in 0..n {
            let a = rem[(i + n - 1) % n];
            let b = rem[i];
            let c = rem[(i + 1) % n];
            let pa = verts[a];
            let pb = verts[b];
            let pc = verts[c];
            if same_side(pa, pc) {
                continue;
            }
            let area = (pb[0] - pa[0]) * (pc[1] - pa[1]) - (pb[1] - pa[1]) * (pc[0] - pa[0]);
            if area > 1e-18 {
                out.push([a as u32, b as u32, c as u32]);
                rem.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            break;
        }
    }
    if rem.len() == 3 {
        out.push([rem[0] as u32, rem[1] as u32, rem[2] as u32]);
    }
}
