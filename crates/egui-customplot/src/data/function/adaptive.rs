use super::{FunctionCurveData, FunctionPlotError, FunctionPlotOptions};
use crate::{PlotData, PlotPoint};

struct Sampler<'a, F> {
    f: &'a mut F,
    options: FunctionPlotOptions,
    evaluations: usize,
    discontinuities: usize,
    budget_truncated: bool,
    points: Vec<PlotPoint>,
}

pub(super) fn sample_explicit(
    x_range: [f64; 2],
    options: FunctionPlotOptions,
    f: &mut impl FnMut(f64) -> f64,
) -> Result<FunctionCurveData, FunctionPlotError> {
    validate(x_range, options)?;
    let n = options.initial_samples;
    let mut initial = Vec::with_capacity(n);
    let denom = (n - 1) as f64;
    for i in 0..n {
        let t = x_range[0] + (x_range[1] - x_range[0]) * (i as f64 / denom);
        initial.push((t, [t, f(t)]));
    }
    let scale = [
        x_range[1] - x_range[0],
        robust_span(initial.iter().map(|s| s.1[1])),
    ];
    let mut eval = |t: f64| [t, f(t)];
    run_adaptive(&initial, n, options, scale, true, &mut eval)
}

pub(super) fn sample_parametric(
    t_range: [f64; 2],
    options: FunctionPlotOptions,
    f: &mut impl FnMut(f64) -> [f64; 2],
) -> Result<FunctionCurveData, FunctionPlotError> {
    validate(t_range, options)?;
    let n = options.initial_samples;
    let mut initial = Vec::with_capacity(n);
    let denom = (n - 1) as f64;
    for i in 0..n {
        let t = t_range[0] + (t_range[1] - t_range[0]) * (i as f64 / denom);
        initial.push((t, f(t)));
    }
    let scale = [
        robust_span(initial.iter().map(|s| s.1[0])),
        robust_span(initial.iter().map(|s| s.1[1])),
    ];
    run_adaptive(&initial, n, options, scale, false, f)
}

fn validate(range: [f64; 2], options: FunctionPlotOptions) -> Result<(), FunctionPlotError> {
    if !range[0].is_finite() || !range[1].is_finite() || range[1] <= range[0] {
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
    Ok(())
}

fn run_adaptive<F: FnMut(f64) -> PlotPoint>(
    initial: &[(f64, PlotPoint)],
    evals: usize,
    options: FunctionPlotOptions,
    scale: [f64; 2],
    explicit_y: bool,
    f: &mut F,
) -> Result<FunctionCurveData, FunctionPlotError> {
    let mut sampler = Sampler {
        f,
        options,
        evaluations: evals,
        discontinuities: 0,
        budget_truncated: false,
        points: Vec::with_capacity(initial.len() * 2),
    };
    push_point(
        &mut sampler.points,
        initial[0].1,
        &mut sampler.discontinuities,
    );
    for w in initial.windows(2) {
        let jump = interval_jump(w[0].1, w[1].1);
        refine_segment(&mut sampler, w[0], w[1], 0, jump, scale, explicit_y);
    }
    Ok(FunctionCurveData {
        data: PlotData::new(sampler.points),
        evaluations: sampler.evaluations,
        discontinuities: sampler.discontinuities,
        budget_truncated: sampler.budget_truncated,
    })
}

fn refine_segment<F: FnMut(f64) -> PlotPoint>(
    s: &mut Sampler<'_, F>,
    a: (f64, PlotPoint),
    b: (f64, PlotPoint),
    depth: u8,
    parent_jump: f64,
    scale: [f64; 2],
    explicit_y: bool,
) {
    let a_fin = is_finite(a.1);
    let b_fin = is_finite(b.1);
    if depth >= s.options.max_depth {
        let child_jump = interval_jump(a.1, b.1);
        let jump_threshold = if explicit_y {
            16.0 * s.options.tolerance * scale[1]
        } else {
            16.0 * s.options.tolerance * scale[0].hypot(scale[1])
        };
        let non_contracting = a_fin
            && b_fin
            && parent_jump.is_finite()
            && child_jump >= 0.85 * parent_jump
            && child_jump > jump_threshold;
        if non_contracting {
            insert_break(s);
        }
        push_point(&mut s.points, b.1, &mut s.discontinuities);
        return;
    }
    if s.evaluations >= s.options.max_evaluations {
        s.budget_truncated = true;
        push_point(&mut s.points, b.1, &mut s.discontinuities);
        return;
    }
    let tm = 0.5 * (a.0 + b.0);
    if tm == a.0 || tm == b.0 {
        push_point(&mut s.points, b.1, &mut s.discontinuities);
        return;
    }
    let pm = (s.f)(tm);
    s.evaluations += 1;
    let m = (tm, pm);
    let m_fin = is_finite(pm);
    let all_fin = a_fin && m_fin && b_fin;
    let need_split = if all_fin {
        needs_subdivision(a.1, pm, b.1, s.options.tolerance, scale, explicit_y)
    } else {
        a_fin || m_fin || b_fin
    };
    if !need_split {
        if all_fin {
            push_point(&mut s.points, pm, &mut s.discontinuities);
        } else if !s.points.last().is_some_and(|p| !is_finite(*p)) {
            insert_break(s);
        }
        push_point(&mut s.points, b.1, &mut s.discontinuities);
        return;
    }
    let j_left = interval_jump(a.1, pm);
    let j_right = interval_jump(pm, b.1);
    let next_parent = parent_jump.min(j_left.max(j_right));
    refine_segment(s, a, m, depth + 1, next_parent, scale, explicit_y);
    refine_segment(s, m, b, depth + 1, next_parent, scale, explicit_y);
}

fn needs_subdivision(
    a: PlotPoint,
    m: PlotPoint,
    b: PlotPoint,
    tol: f64,
    scale: [f64; 2],
    explicit_y: bool,
) -> bool {
    if explicit_y {
        let err = (m[1] - 0.5 * (a[1] + b[1])).abs() / scale[1];
        return err > tol || !err.is_finite();
    }
    let ax = a[0] / scale[0];
    let ay = a[1] / scale[1];
    let mx = m[0] / scale[0];
    let my = m[1] / scale[1];
    let bx = b[0] / scale[0];
    let by = b[1] / scale[1];
    let ux = bx - ax;
    let uy = by - ay;
    let chord_len = ux.hypot(uy);
    let l1 = (mx - ax).hypot(my - ay);
    let l2 = (bx - mx).hypot(by - my);
    if l1 + l2 < tol * 0.25 {
        return false;
    }
    let perp = if chord_len == 0.0 {
        l1.max(l2)
    } else {
        ((mx - ax) * uy - (my - ay) * ux).abs() / chord_len
    };
    perp > tol || l1 + l2 > 1.5 * chord_len + tol
}

fn robust_span(vals: impl Iterator<Item = f64>) -> f64 {
    let mut finite: Vec<f64> = vals.filter(|v| v.is_finite()).collect();
    if finite.len() < 2 {
        return 1.0;
    }
    finite.sort_by(f64::total_cmp);
    let lo = finite[finite.len() / 10];
    let hi = finite[(finite.len() * 9).saturating_sub(1) / 10];
    let diff = hi - lo;
    if diff == 0.0 || !diff.is_finite() {
        1.0
    } else {
        diff.max(1e-12)
    }
}

fn interval_jump(a: PlotPoint, b: PlotPoint) -> f64 {
    if !is_finite(a) || !is_finite(b) {
        f64::INFINITY
    } else {
        (b[0] - a[0]).hypot(b[1] - a[1])
    }
}

fn is_finite(p: PlotPoint) -> bool {
    p[0].is_finite() && p[1].is_finite()
}

fn insert_break<F>(s: &mut Sampler<'_, F>) {
    if !s.points.last().is_some_and(|p| !is_finite(*p)) {
        s.points.push([f64::NAN, f64::NAN]);
        s.discontinuities += 1;
    }
}

fn push_point(points: &mut Vec<PlotPoint>, p: PlotPoint, discontinuities: &mut usize) {
    if is_finite(p) {
        points.push(p);
    } else if !points.last().is_some_and(|prev| !is_finite(*prev)) {
        points.push([f64::NAN, f64::NAN]);
        *discontinuities += 1;
    }
}
