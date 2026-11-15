use std::collections::HashMap;

use super::{FunctionPlotError, ImplicitCurveData, ImplicitOptions};
use crate::{
    ContourData, ContourLine, PlotBounds, PlotPoint,
    data::contour::extract::{Node, chain_segments, tri_isoline},
};

struct ImplicitSampler<'a, F> {
    f: &'a mut F,
    bounds: PlotBounds,
    fine_max: [u32; 2],
    max_depth: u8,
    max_evaluations: usize,
    cache: HashMap<(u32, u32), f64>,
    evaluations: usize,
    singular_cells: usize,
    budget_truncated: bool,
    segments: Vec<[PlotPoint; 2]>,
}

impl<'a, F: FnMut(f64, f64) -> f64> ImplicitSampler<'a, F> {
    fn coord(&self, ix: u32, iy: u32) -> PlotPoint {
        let lo = self.bounds.min();
        let span = self.bounds.span();
        [
            lo[0] + span[0] * (f64::from(ix) / f64::from(self.fine_max[0])),
            lo[1] + span[1] * (f64::from(iy) / f64::from(self.fine_max[1])),
        ]
    }

    fn eval(&mut self, ix: u32, iy: u32) -> Option<f64> {
        if let Some(&v) = self.cache.get(&(ix, iy)) {
            return Some(v);
        }
        if self.evaluations >= self.max_evaluations {
            self.budget_truncated = true;
            return None;
        }
        let [x, y] = self.coord(ix, iy);
        let z = (self.f)(x, y);
        self.evaluations += 1;
        self.cache.insert((ix, iy), z);
        Some(z)
    }

    fn eval_center_uncached(&mut self, ix0: u32, iy0: u32, step: u32) -> PlotPoint {
        let a = self.coord(ix0, iy0);
        let b = self.coord(ix0 + step, iy0 + step);
        [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])]
    }
}

pub(super) fn sample_implicit(
    bounds: PlotBounds,
    options: ImplicitOptions,
    f: &mut impl FnMut(f64, f64) -> f64,
) -> Result<ImplicitCurveData, FunctionPlotError> {
    if options.coarse_grid[0] < 2 || options.coarse_grid[1] < 2 || options.max_depth > 8 {
        return Err(FunctionPlotError::InvalidOptions);
    }
    let min_evals = options.coarse_grid[0]
        .checked_mul(options.coarse_grid[1])
        .ok_or(FunctionPlotError::InvalidOptions)?;
    if options.max_evaluations < min_evals {
        return Err(FunctionPlotError::BudgetExceeded {
            required: min_evals,
            max_evaluations: options.max_evaluations,
        });
    }
    let scale = 1u32 << options.max_depth;
    let fine_max = [
        (options.coarse_grid[0] as u32 - 1) * scale,
        (options.coarse_grid[1] as u32 - 1) * scale,
    ];
    let mut sampler = ImplicitSampler {
        f,
        bounds,
        fine_max,
        max_depth: options.max_depth,
        max_evaluations: options.max_evaluations,
        cache: HashMap::with_capacity(min_evals * 2),
        evaluations: 0,
        singular_cells: 0,
        budget_truncated: false,
        segments: Vec::new(),
    };
    for iy in 0..options.coarse_grid[1] as u32 {
        for ix in 0..options.coarse_grid[0] as u32 {
            let _ = sampler.eval(ix * scale, iy * scale);
        }
    }
    for cy in 0..options.coarse_grid[1] as u32 - 1 {
        for cx in 0..options.coarse_grid[0] as u32 - 1 {
            visit_cell(&mut sampler, cx * scale, cy * scale, scale, 0, None);
        }
    }
    let polylines = chain_segments(sampler.segments);
    let lines = polylines
        .into_iter()
        .map(|pts| ContourLine {
            level: 0.0,
            points: pts.into_boxed_slice(),
        })
        .collect();
    let contours = ContourData::from_raw_lines(vec![0.0], lines);
    Ok(ImplicitCurveData {
        contours,
        evaluations: sampler.evaluations,
        singular_cells: sampler.singular_cells,
        budget_truncated: sampler.budget_truncated,
    })
}

fn visit_cell<F: FnMut(f64, f64) -> f64>(
    s: &mut ImplicitSampler<'_, F>,
    ix0: u32,
    iy0: u32,
    step: u32,
    depth: u8,
    parent_span: Option<f64>,
) {
    let ix1 = ix0 + step;
    let iy1 = iy0 + step;
    let Some(z00) = s.eval(ix0, iy0) else { return };
    let Some(z10) = s.eval(ix1, iy0) else { return };
    let Some(z11) = s.eval(ix1, iy1) else { return };
    let Some(z01) = s.eval(ix0, iy1) else { return };
    let zs = [z00, z10, z11, z01];
    let any_non_finite = zs.iter().any(|z| !z.is_finite());
    if any_non_finite {
        if depth >= s.max_depth {
            s.singular_cells += 1;
            return;
        }
        subdivide_children(s, ix0, iy0, step, depth, parent_span);
        return;
    }
    let zmin = zs.into_iter().fold(f64::INFINITY, f64::min);
    let zmax = zs.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let span = zmax - zmin;
    if zmax <= 1e-12 && zmin >= -1e-12 {
        s.singular_cells += 1;
        return;
    }
    let straddles = zmin <= 0.0 && zmax >= 0.0;
    let near_root = zmin.abs().min(zmax.abs()) <= span * 0.5;
    if depth < s.max_depth && (straddles || near_root) && s.evaluations + 5 <= s.max_evaluations {
        subdivide_children(s, ix0, iy0, step, depth, Some(span));
        return;
    }
    if !straddles {
        return;
    }
    let growing_pole = parent_span.is_some_and(|p| depth >= s.max_depth && span > p * 1.1);
    let (center_p, zc) = if depth >= s.max_depth && s.evaluations < s.max_evaluations {
        let p = s.eval_center_uncached(ix0, iy0, step);
        let z = (s.f)(p[0], p[1]);
        s.evaluations += 1;
        (p, z)
    } else {
        let p = s.eval_center_uncached(ix0, iy0, step);
        (p, 0.25 * (z00 + z10 + z11 + z01))
    };
    if !zc.is_finite() || growing_pole {
        s.singular_cells += 1;
        return;
    }
    let nodes = [
        Node {
            p: s.coord(ix0, iy0),
            z: z00,
        },
        Node {
            p: s.coord(ix1, iy0),
            z: z10,
        },
        Node {
            p: s.coord(ix1, iy1),
            z: z11,
        },
        Node {
            p: s.coord(ix0, iy1),
            z: z01,
        },
    ];
    let center = Node { p: center_p, z: zc };
    for k in 0..4 {
        let tri = [nodes[k], nodes[(k + 1) % 4], center];
        if let Some(seg) = tri_isoline(tri, 0.0) {
            let tri_min = tri[0].z.abs().min(tri[1].z.abs()).min(tri[2].z.abs());
            let tri_span =
                tri[0].z.max(tri[1].z).max(tri[2].z) - tri[0].z.min(tri[1].z).min(tri[2].z);
            let limit = if depth >= s.max_depth {
                (0.45 * tri_min).max(1e-6)
            } else {
                (0.95 * tri_min).max(0.5 * tri_span).max(1e-6)
            };
            let mid = [0.5 * (seg[0][0] + seg[1][0]), 0.5 * (seg[0][1] + seg[1][1])];
            let valid_root = [seg[0], seg[1], mid].into_iter().all(|pt| {
                if s.evaluations >= s.max_evaluations {
                    return depth < s.max_depth;
                }
                s.evaluations += 1;
                let zv = (s.f)(pt[0], pt[1]);
                zv.is_finite() && zv.abs() <= limit
            });
            if valid_root {
                s.segments.push(seg);
            } else {
                s.singular_cells += 1;
            }
        }
    }
}

fn subdivide_children<F: FnMut(f64, f64) -> f64>(
    s: &mut ImplicitSampler<'_, F>,
    ix0: u32,
    iy0: u32,
    step: u32,
    depth: u8,
    parent_span: Option<f64>,
) {
    let half = step / 2;
    if half == 0 {
        return;
    }
    for dy in [0, half] {
        for dx in [0, half] {
            if s.evaluations >= s.max_evaluations {
                s.budget_truncated = true;
                return;
            }
            visit_cell(s, ix0 + dx, iy0 + dy, half, depth + 1, parent_span);
        }
    }
}
