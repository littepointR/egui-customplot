use std::{collections::VecDeque, f64::consts::TAU};

use egui::Context;

use super::Point3DData;
use crate::{
    AppendReport, CometPlayback, FunctionPlotError, FunctionPlotOptions, Point3D, StreamError,
};

/// Alias reusing the multi-pass frame-idempotent comet playback cursor for [`crate::Comet3D`].
pub type Comet3DPlayback = CometPlayback;

/// Immutable host-owned adaptively sampled 3D parametric curve `[x(t), y(t), z(t)]` (`fplot3` / `ezplot3`).
#[derive(Clone, Debug)]
pub struct ParametricCurve3DData {
    points: Point3DData,
    evaluations: usize,
    discontinuities: usize,
    budget_truncated: bool,
}

impl ParametricCurve3DData {
    pub fn from_parametric<F>(
        t_range: [f64; 2],
        options: FunctionPlotOptions,
        mut f: F,
    ) -> Result<Self, FunctionPlotError>
    where
        F: FnMut(f64) -> Point3D,
    {
        if !t_range[0].is_finite() || !t_range[1].is_finite() || t_range[1] <= t_range[0] {
            return Err(FunctionPlotError::InvalidRange);
        }
        if options.initial_samples < 3
            || options.max_depth > 16
            || !options.tolerance.is_finite()
            || options.tolerance <= 0.0
        {
            return Err(FunctionPlotError::InvalidOptions);
        }
        if options.max_evaluations < options.initial_samples {
            return Err(FunctionPlotError::BudgetExceeded {
                required: options.initial_samples,
                max_evaluations: options.max_evaluations,
            });
        }
        let sampler = Sampler3D {
            f: &mut f,
            options,
            evaluations: 0,
            discontinuities: 0,
            budget_truncated: false,
            out: Vec::new(),
        };
        sampler.sample_range(t_range)
    }

    pub fn ezplot3<F>(max_evaluations: usize, f: F) -> Result<Self, FunctionPlotError>
    where
        F: FnMut(f64) -> Point3D,
    {
        Self::from_parametric(
            [0.0, TAU],
            FunctionPlotOptions {
                max_evaluations: max_evaluations.max(64),
                ..Default::default()
            },
            f,
        )
    }

    pub fn points(&self) -> &Point3DData {
        &self.points
    }
    pub fn evaluations(&self) -> usize {
        self.evaluations
    }
    pub fn discontinuities(&self) -> usize {
        self.discontinuities
    }
    pub fn budget_truncated(&self) -> bool {
        self.budget_truncated
    }
}

struct Sampler3D<'a, F> {
    f: &'a mut F,
    options: FunctionPlotOptions,
    evaluations: usize,
    discontinuities: usize,
    budget_truncated: bool,
    out: Vec<Point3D>,
}

impl<'a, F: FnMut(f64) -> Point3D> Sampler3D<'a, F> {
    fn eval(&mut self, t: f64) -> Option<Point3D> {
        if self.evaluations >= self.options.max_evaluations {
            self.budget_truncated = true;
            return None;
        }
        self.evaluations += 1;
        Some((self.f)(t))
    }

    fn sample_range(mut self, range: [f64; 2]) -> Result<ParametricCurve3DData, FunctionPlotError> {
        let n = self.options.initial_samples;
        let mut coarse = Vec::with_capacity(n);
        for i in 0..n {
            let t = range[0] + (range[1] - range[0]) * (i as f64 / (n - 1) as f64);
            if let Some(p) = self.eval(t) {
                coarse.push((t, p));
            }
        }
        let diag = estimate_scale(&coarse);
        let tol = self.options.tolerance * diag.max(1e-6);
        if let Some((first_t, first_p)) = coarse.first().copied() {
            self.out.push(first_p);
            let mut prev = (first_t, first_p);
            for &cur in &coarse[1..] {
                self.subdivide(prev, cur, 0, tol);
                prev = cur;
            }
        }
        Ok(ParametricCurve3DData {
            points: Point3DData::new(self.out),
            evaluations: self.evaluations,
            discontinuities: self.discontinuities,
            budget_truncated: self.budget_truncated,
        })
    }

    fn subdivide(&mut self, a: (f64, Point3D), b: (f64, Point3D), depth: usize, tol: f64) {
        let a_fin = is_finite3(a.1);
        let b_fin = is_finite3(b.1);
        if !a_fin || !b_fin {
            if depth < usize::from(self.options.max_depth.min(6))
                && self.evaluations < self.options.max_evaluations
            {
                let tm = 0.5 * (a.0 + b.0);
                if let Some(pm) = self.eval(tm) {
                    self.subdivide(a, (tm, pm), depth + 1, tol);
                    self.subdivide((tm, pm), b, depth + 1, tol);
                    return;
                }
            }
            self.discontinuities += 1;
            self.out.push([f64::NAN; 3]);
            if b_fin {
                self.out.push(b.1);
            }
            return;
        }
        if depth >= usize::from(self.options.max_depth)
            || self.evaluations >= self.options.max_evaluations
        {
            if self.evaluations >= self.options.max_evaluations {
                self.budget_truncated = true;
            }
            self.out.push(b.1);
            return;
        }
        let tm = 0.5 * (a.0 + b.0);
        let Some(pm) = self.eval(tm) else {
            self.out.push(b.1);
            return;
        };
        if !is_finite3(pm) {
            self.subdivide(a, (tm, pm), depth + 1, tol);
            self.subdivide((tm, pm), b, depth + 1, tol);
            return;
        }
        let chord_mid = [
            0.5 * (a.1[0] + b.1[0]),
            0.5 * (a.1[1] + b.1[1]),
            0.5 * (a.1[2] + b.1[2]),
        ];
        let err = dist3(pm, chord_mid);
        if err > tol {
            self.subdivide(a, (tm, pm), depth + 1, tol);
            self.subdivide((tm, pm), b, depth + 1, tol);
        } else {
            self.out.push(pm);
            self.out.push(b.1);
        }
    }
}

fn estimate_scale(samples: &[(f64, Point3D)]) -> f64 {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for &(_, p) in samples {
        if is_finite3(p) {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
    }
    if lo[0].is_finite() {
        ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2) + (hi[2] - lo[2]).powi(2)).sqrt()
    } else {
        1.0
    }
}

fn is_finite3(p: Point3D) -> bool {
    p[0].is_finite() && p[1].is_finite() && p[2].is_finite()
}

fn dist3(a: Point3D, b: Point3D) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Bounded host-owned streaming 3D trajectory for `animatedline` in 3D (`ML-078` / `ML-080`).
pub struct AnimatedLine3D {
    capacity: usize,
    ring: VecDeque<Point3D>,
    published: Point3DData,
    last_batch_id: Option<u64>,
    revision: u64,
    paused: bool,
}

impl AnimatedLine3D {
    pub fn new(capacity_points: usize) -> Result<Self, StreamError> {
        if capacity_points == 0 {
            return Err(StreamError::ZeroCapacity);
        }
        if capacity_points > 4_000_000 {
            return Err(StreamError::SizeOverflow);
        }
        Ok(Self {
            capacity: capacity_points,
            ring: VecDeque::with_capacity(capacity_points.min(4096)),
            published: Point3DData::new(Vec::new()),
            last_batch_id: None,
            revision: 0,
            paused: false,
        })
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn add_points_once(
        &mut self,
        batch_id: u64,
        pts: &[Point3D],
    ) -> Result<AppendReport, StreamError> {
        if self.paused || self.last_batch_id == Some(batch_id) || pts.is_empty() {
            return Ok(AppendReport {
                applied: false,
                revision: self.revision,
                retained_input_points: 0,
                evicted_points: 0,
                dropped_input_points: 0,
            });
        }
        self.last_batch_id = Some(batch_id);
        let skip = pts.len().saturating_sub(self.capacity);
        let keep = &pts[skip..];
        let mut evicted = 0usize;
        for &p in keep {
            if self.ring.len() == self.capacity {
                self.ring.pop_front();
                evicted += 1;
            }
            self.ring.push_back(p);
        }
        self.revision += 1;
        let vec: Vec<Point3D> = self.ring.iter().copied().collect();
        self.published = Point3DData::new(vec);
        Ok(AppendReport {
            applied: true,
            revision: self.revision,
            retained_input_points: keep.len(),
            evicted_points: evicted,
            dropped_input_points: skip,
        })
    }

    pub fn clear(&mut self) {
        if !self.ring.is_empty() {
            self.ring.clear();
            self.revision += 1;
            self.published = Point3DData::new(Vec::new());
        }
    }

    pub fn data(&self) -> &Point3DData {
        &self.published
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn len(&self) -> usize {
        self.ring.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    pub fn sync_repaint(&self, ctx: &Context, last_seen_revision: &mut u64) -> bool {
        if !self.paused && self.revision != *last_seen_revision {
            *last_seen_revision = self.revision;
            ctx.request_repaint();
            true
        } else {
            false
        }
    }
}
