use std::sync::Arc;

use super::{FlowError, VectorField2D};
use crate::{AxisScale, PlotBounds, PlotPoint, data::extents::CachedExtents};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreamOptions {
    pub step_fraction: f64,
    pub max_steps: usize,
    pub bidirectional: bool,
    pub min_speed: f64,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            step_fraction: 0.25,
            max_steps: 2000,
            bidirectional: false,
            min_speed: 1e-12,
        }
    }
}

/// Immutable host-owned integrated 2D streamlines and flow arrows (`streamline` / `streamslice`).
#[derive(Clone, Debug)]
pub struct StreamlineData {
    lines: Arc<[Box<[PlotPoint]>]>,
    arrows: Arc<[(PlotPoint, [f64; 2])]>,
    total_steps: usize,
    budget_stops: usize,
    closed_loops: usize,
    total_bytes: usize,
    extents: CachedExtents,
}

impl StreamlineData {
    pub fn from_seeds(
        field: &VectorField2D,
        seeds: &[PlotPoint],
        options: StreamOptions,
    ) -> Result<Self, FlowError> {
        validate_options(options)?;
        let mut lines = Vec::with_capacity(seeds.len());
        let mut total_steps = 0usize;
        let mut budget_stops = 0usize;
        let mut closed_loops = 0usize;
        let mut extents = CachedExtents::default();
        let mut total_bytes = 0usize;

        for &seed in seeds {
            if field.sample(seed).is_none() {
                continue;
            }
            let traced = trace_one(field, seed, options, None);
            total_steps += traced.steps;
            budget_stops += usize::from(traced.budget_stopped);
            closed_loops += usize::from(traced.closed_loop);
            if traced.points.len() >= 2 {
                push_extent(&mut extents, &traced.points);
                total_bytes += traced.points.len() * std::mem::size_of::<PlotPoint>();
                lines.push(traced.points.into_boxed_slice());
            }
        }
        Ok(Self {
            lines: Arc::from(lines),
            arrows: Arc::from([]),
            total_steps,
            budget_stops,
            closed_loops,
            total_bytes,
            extents,
        })
    }

    pub fn streamslice(field: &VectorField2D, density: f64) -> Result<Self, FlowError> {
        if !density.is_finite() || density <= 0.0 {
            return Err(FlowError::InvalidOptions);
        }
        let d = density.clamp(0.25, 8.0);
        let nx = ((12.0 * d).round() as usize).clamp(4, 96);
        let ny = ((12.0 * d).round() as usize).clamp(4, 96);
        let mut occ = OccupancyGrid::new(field.bounds(), nx, ny);
        let options = StreamOptions {
            bidirectional: true,
            max_steps: 1000,
            ..Default::default()
        };
        let mut lines = Vec::new();
        let mut arrows = Vec::new();
        let mut total_steps = 0usize;
        let mut budget_stops = 0usize;
        let mut closed_loops = 0usize;
        let mut extents = CachedExtents::default();
        let mut total_bytes = 0usize;
        let lo = field.bounds().min();
        let span = field.bounds().span();

        for cy in 0..ny {
            for cx in 0..nx {
                if occ.cells[cy * nx + cx] {
                    continue;
                }
                let seed = [
                    lo[0] + span[0] * ((cx as f64 + 0.5) / nx as f64),
                    lo[1] + span[1] * ((cy as f64 + 0.5) / ny as f64),
                ];
                let Some(v) = field.sample(seed) else {
                    continue;
                };
                if v[0].hypot(v[1]) <= options.min_speed {
                    continue;
                }
                let traced = trace_one(field, seed, options, Some(&occ));
                total_steps += traced.steps;
                budget_stops += usize::from(traced.budget_stopped);
                closed_loops += usize::from(traced.closed_loop);
                if traced.points.len() >= 2 {
                    for &p in &traced.points {
                        occ.mark(p);
                    }
                    collect_slice_arrows(field, &traced.points, &mut arrows);
                    push_extent(&mut extents, &traced.points);
                    total_bytes += traced.points.len() * std::mem::size_of::<PlotPoint>();
                    lines.push(traced.points.into_boxed_slice());
                }
            }
        }
        total_bytes += arrows.len() * std::mem::size_of::<(PlotPoint, [f64; 2])>();
        Ok(Self {
            lines: Arc::from(lines),
            arrows: Arc::from(arrows),
            total_steps,
            budget_stops,
            closed_loops,
            total_bytes,
            extents,
        })
    }

    pub fn lines(&self) -> &[Box<[PlotPoint]>] {
        &self.lines
    }
    pub fn arrows(&self) -> &[(PlotPoint, [f64; 2])] {
        &self.arrows
    }
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }
    pub fn budget_stops(&self) -> usize {
        self.budget_stops
    }
    pub fn closed_loops(&self) -> usize {
        self.closed_loops
    }
    pub fn sample_bytes(&self) -> usize {
        self.total_bytes
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.lines.as_ptr() as usize, self.total_bytes)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }
}

struct TraceResult {
    points: Vec<PlotPoint>,
    steps: usize,
    budget_stopped: bool,
    closed_loop: bool,
}

struct OccupancyGrid {
    bounds: PlotBounds,
    nx: usize,
    ny: usize,
    cells: Vec<bool>,
}

impl OccupancyGrid {
    fn new(bounds: PlotBounds, nx: usize, ny: usize) -> Self {
        Self {
            bounds,
            nx,
            ny,
            cells: vec![false; nx * ny],
        }
    }

    fn cell_of(&self, p: PlotPoint) -> Option<usize> {
        let lo = self.bounds.min();
        let span = self.bounds.span();
        let tx = ((p[0] - lo[0]) / span[0]).clamp(0.0, 1.0);
        let ty = ((p[1] - lo[1]) / span[1]).clamp(0.0, 1.0);
        let ix = ((tx * self.nx as f64) as usize).min(self.nx - 1);
        let iy = ((ty * self.ny as f64) as usize).min(self.ny - 1);
        Some(iy * self.nx + ix)
    }

    fn is_occupied(&self, p: PlotPoint) -> bool {
        self.cell_of(p).is_some_and(|idx| self.cells[idx])
    }

    fn mark(&mut self, p: PlotPoint) {
        if let Some(idx) = self.cell_of(p) {
            self.cells[idx] = true;
        }
    }
}

fn validate_options(options: StreamOptions) -> Result<(), FlowError> {
    if !options.step_fraction.is_finite()
        || options.step_fraction <= 0.0
        || options.step_fraction > 2.0
        || options.max_steps == 0
        || !options.min_speed.is_finite()
        || options.min_speed < 0.0
    {
        return Err(FlowError::InvalidOptions);
    }
    Ok(())
}

fn trace_one(
    field: &VectorField2D,
    seed: PlotPoint,
    options: StreamOptions,
    occ: Option<&OccupancyGrid>,
) -> TraceResult {
    let fwd = integrate_dir(field, seed, 1.0, options, occ);
    if !options.bidirectional || fwd.closed_loop {
        return fwd;
    }
    let bwd = integrate_dir(field, seed, -1.0, options, occ);
    let mut points = Vec::with_capacity(bwd.points.len() + fwd.points.len());
    points.extend(bwd.points.iter().rev().copied());
    points.extend(fwd.points.iter().skip(1).copied());
    TraceResult {
        points,
        steps: fwd.steps + bwd.steps,
        budget_stopped: fwd.budget_stopped || bwd.budget_stopped,
        closed_loop: false,
    }
}

fn integrate_dir(
    field: &VectorField2D,
    seed: PlotPoint,
    sign: f64,
    options: StreamOptions,
    occ: Option<&OccupancyGrid>,
) -> TraceResult {
    let ds_target = (options.step_fraction * field.min_cell_span()).max(1e-9);
    let seed_cell = occ.and_then(|g| g.cell_of(seed));
    let mut points = vec![seed];
    let mut p = seed;
    let mut steps = 0usize;
    let mut budget_stopped = false;
    let mut closed_loop = false;

    for _ in 0..options.max_steps {
        let Some(k1_raw) = field.sample(p) else { break };
        let k1 = [k1_raw[0] * sign, k1_raw[1] * sign];
        let speed = k1[0].hypot(k1[1]);
        if !speed.is_finite() || speed <= options.min_speed {
            break;
        }
        let h = ds_target / speed;
        let p2 = [p[0] + 0.5 * h * k1[0], p[1] + 0.5 * h * k1[1]];
        let Some(k2_raw) = field.sample(p2) else {
            push_clamped_exit(field.bounds(), p, p2, &mut points);
            break;
        };
        let k2 = [k2_raw[0] * sign, k2_raw[1] * sign];
        let p3 = [p[0] + 0.5 * h * k2[0], p[1] + 0.5 * h * k2[1]];
        let Some(k3_raw) = field.sample(p3) else {
            push_clamped_exit(field.bounds(), p, p3, &mut points);
            break;
        };
        let k3 = [k3_raw[0] * sign, k3_raw[1] * sign];
        let p4 = [p[0] + h * k3[0], p[1] + h * k3[1]];
        let Some(k4_raw) = field.sample(p4) else {
            push_clamped_exit(field.bounds(), p, p4, &mut points);
            break;
        };
        let k4 = [k4_raw[0] * sign, k4_raw[1] * sign];
        let next = [
            p[0] + (h / 6.0) * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
            p[1] + (h / 6.0) * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
        ];
        if !next[0].is_finite() || !next[1].is_finite() {
            break;
        }
        steps += 1;
        if field.sample(next).is_none() {
            push_clamped_exit(field.bounds(), p, next, &mut points);
            break;
        }
        if let Some(grid) = occ
            && grid.cell_of(next) != seed_cell
            && grid.is_occupied(next)
        {
            points.push(next);
            break;
        }
        points.push(next);
        p = next;
        if steps >= 8 && (p[0] - seed[0]).hypot(p[1] - seed[1]) <= 0.75 * ds_target {
            points.push(seed);
            closed_loop = true;
            break;
        }
        if steps >= 16 {
            let anchor = points[points.len() - 17];
            if (p[0] - anchor[0]).hypot(p[1] - anchor[1]) < 0.1 * ds_target {
                break;
            }
        }
        if steps == options.max_steps {
            budget_stopped = true;
        }
    }
    TraceResult {
        points,
        steps,
        budget_stopped,
        closed_loop,
    }
}

fn push_clamped_exit(
    bounds: PlotBounds,
    inside: PlotPoint,
    outside: PlotPoint,
    out: &mut Vec<PlotPoint>,
) {
    let lo = bounds.min();
    let hi = bounds.max();
    let mut t = 1.0_f64;
    for axis in 0..2 {
        let d = outside[axis] - inside[axis];
        if outside[axis] < lo[axis] && d != 0.0 {
            t = t.min((lo[axis] - inside[axis]) / d);
        } else if outside[axis] > hi[axis] && d != 0.0 {
            t = t.min((hi[axis] - inside[axis]) / d);
        }
    }
    let t = t.clamp(0.0, 1.0);
    if t > 0.0 {
        out.push([
            inside[0] + t * (outside[0] - inside[0]),
            inside[1] + t * (outside[1] - inside[1]),
        ]);
    }
}

fn collect_slice_arrows(
    field: &VectorField2D,
    pts: &[PlotPoint],
    out: &mut Vec<(PlotPoint, [f64; 2])>,
) {
    if pts.len() < 4 {
        return;
    }
    let stride = (pts.len() / 3).max(8);
    for idx in (stride / 2..pts.len() - 1).step_by(stride) {
        let p = pts[idx];
        if let Some(v) = field.sample(p) {
            let len = v[0].hypot(v[1]);
            if len > 0.0 && len.is_finite() {
                out.push((p, [v[0] / len, v[1] / len]));
            }
        }
    }
}

fn push_extent(extents: &mut CachedExtents, pts: &[PlotPoint]) {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for &p in pts {
        lo[0] = lo[0].min(p[0]);
        lo[1] = lo[1].min(p[1]);
        hi[0] = hi[0].max(p[0]);
        hi[1] = hi[1].max(p[1]);
    }
    extents.push(Some((lo, hi)));
}
