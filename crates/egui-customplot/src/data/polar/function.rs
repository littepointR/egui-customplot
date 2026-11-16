use super::PolarCurveData;
use crate::{FunctionPlotError, FunctionPlotOptions};

/// Immutable host-owned adaptively sampled polar curve `r = f(theta)` (`fpolarplot` / `ezpolar`).
#[derive(Clone, Debug)]
pub struct PolarFunctionData {
    curve: PolarCurveData,
    evaluations: usize,
    discontinuities: usize,
    budget_truncated: bool,
}

impl PolarFunctionData {
    pub fn curve(&self) -> &PolarCurveData {
        &self.curve
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

struct Sampler<'a, F> {
    f: &'a mut F,
    options: FunctionPlotOptions,
    evaluations: usize,
    discontinuities: usize,
    budget_truncated: bool,
    points: Vec<[f64; 2]>,
}

pub(super) fn sample_polar<F: FnMut(f64) -> f64>(
    theta_range: [f64; 2],
    options: FunctionPlotOptions,
    f: &mut F,
) -> Result<PolarFunctionData, FunctionPlotError> {
    if !theta_range[0].is_finite()
        || !theta_range[1].is_finite()
        || theta_range[1] <= theta_range[0]
    {
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
    let n = options.initial_samples;
    let denom = (n - 1) as f64;
    let mut initial = Vec::with_capacity(n);
    let mut abs_r = Vec::with_capacity(n);
    for i in 0..n {
        let th = theta_range[0] + (theta_range[1] - theta_range[0]) * (i as f64 / denom);
        let r = f(th);
        if r.is_finite() {
            abs_r.push(r.abs());
        }
        initial.push([th, r]);
    }
    abs_r.sort_by(f64::total_cmp);
    let span = if abs_r.is_empty() {
        1.0
    } else {
        let q = abs_r[(abs_r.len() * 9 / 10).min(abs_r.len() - 1)];
        q.max(1e-3)
    };
    let mut sampler = Sampler {
        f,
        options,
        evaluations: n,
        discontinuities: 0,
        budget_truncated: false,
        points: Vec::with_capacity(n * 2),
    };
    let mut prev = initial[0];
    push_polar_point(&mut sampler.points, prev);
    for &next in initial.iter().skip(1) {
        refine_polar(&mut sampler, prev, next, span, 0);
        prev = next;
    }
    Ok(PolarFunctionData {
        curve: PolarCurveData::new(sampler.points),
        evaluations: sampler.evaluations,
        discontinuities: sampler.discontinuities,
        budget_truncated: sampler.budget_truncated,
    })
}

fn refine_polar<F: FnMut(f64) -> f64>(
    s: &mut Sampler<'_, F>,
    a: [f64; 2],
    b: [f64; 2],
    span: f64,
    depth: u8,
) {
    let mid_t = 0.5 * (a[0] + b[0]);
    if !a[1].is_finite() || !b[1].is_finite() {
        s.discontinuities += 1;
        push_break(&mut s.points);
        push_polar_point(&mut s.points, b);
        return;
    }
    if (a[1] - b[1]).abs() > span * 4.0 && depth >= 3 {
        s.discontinuities += 1;
        push_break(&mut s.points);
        push_polar_point(&mut s.points, b);
        return;
    }
    let pa = [a[1] * a[0].cos(), a[1] * a[0].sin()];
    let pb = [b[1] * b[0].cos(), b[1] * b[0].sin()];
    let chord = (pb[0] - pa[0]).hypot(pb[1] - pa[1]);
    if depth >= s.options.max_depth || chord <= s.options.tolerance * span * 0.25 {
        push_polar_point(&mut s.points, b);
        return;
    }
    if s.evaluations >= s.options.max_evaluations {
        s.budget_truncated = true;
        push_polar_point(&mut s.points, b);
        return;
    }
    s.evaluations += 1;
    let mid_r = (s.f)(mid_t);
    let mid = [mid_t, mid_r];
    if !mid_r.is_finite() {
        s.discontinuities += 1;
        push_break(&mut s.points);
        push_polar_point(&mut s.points, b);
        return;
    }
    let pm = [mid_r * mid_t.cos(), mid_r * mid_t.sin()];
    let dev = point_segment_dist(pm, pa, pb);
    if dev <= s.options.tolerance * span && (b[0] - a[0]).abs() <= 0.12 {
        push_polar_point(&mut s.points, mid);
        push_polar_point(&mut s.points, b);
        return;
    }
    refine_polar(s, a, mid, span, depth + 1);
    refine_polar(s, mid, b, span, depth + 1);
}

fn push_polar_point(points: &mut Vec<[f64; 2]>, p: [f64; 2]) {
    if p[0].is_finite() && p[1].is_finite() {
        points.push(p);
    } else {
        push_break(points);
    }
}

fn push_break(points: &mut Vec<[f64; 2]>) {
    if points
        .last()
        .is_some_and(|p| p[0].is_finite() && p[1].is_finite())
    {
        points.push([f64::NAN, f64::NAN]);
    }
}

fn point_segment_dist(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    if len2 <= 1e-24 {
        return (p[0] - a[0]).hypot(p[1] - a[1]);
    }
    let t = (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / len2).clamp(0.0, 1.0);
    let proj = [a[0] + ab[0] * t, a[1] + ab[1] * t];
    (p[0] - proj[0]).hypot(p[1] - proj[1])
}
