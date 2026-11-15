mod support;

use std::f64::consts::{PI, TAU};

use egui::{Context, Shape};
use egui_customplot::{
    FunctionCurveData, FunctionPlot, FunctionPlotError, FunctionPlotOptions, ImplicitCurve,
    ImplicitCurveData, ImplicitOptions, Plot, PlotBounds, PlotState,
};
use support::{capture, run};

#[test]
fn fplot_adaptive_chirp_and_parametric_lissajous() {
    let chirp = FunctionCurveData::explicit([0.1, 4.0], FunctionPlotOptions::default(), |x| {
        (1.0 / (x * 0.25)).sin()
    })
    .unwrap();
    let pts = chirp.data().points();
    assert!(pts.len() > FunctionPlotOptions::default().initial_samples);
    assert!(chirp.evaluations() <= FunctionPlotOptions::default().max_evaluations);
    assert!(!chirp.budget_truncated());

    // Dense refinement clusters near x < 0.8 where frequency 1/(0.25*x) is highest.
    let near_left = pts
        .iter()
        .filter(|p| p[0].is_finite() && p[0] <= 0.8)
        .count();
    let near_right = pts
        .iter()
        .filter(|p| p[0].is_finite() && p[0] >= 3.3)
        .count();
    assert!(
        near_left > near_right * 2,
        "expected adaptive clustering on steep region: left={near_left}, right={near_right}"
    );

    // Parametric Lissajous (sin 3t, cos 2t)
    let liss = FunctionCurveData::parametric([0.0, TAU], FunctionPlotOptions::default(), |t| {
        [(3.0 * t).sin(), (2.0 * t).cos()]
    })
    .unwrap();
    assert!(liss.data().len() > FunctionPlotOptions::default().initial_samples);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let resp = run(&ctx, vec![], true, |ui| {
        Plot::new("fplot").show_layers(
            ui,
            &mut state,
            &[
                FunctionPlot::new(&chirp).into(),
                FunctionPlot::new(&liss).into(),
            ],
        )
    });
    assert!(resp.diagnostics.output_paths >= 2);
}

#[test]
fn fplot_poles_and_step_discontinuities_split_at_nan_without_vertical_chord() {
    let tan_curve =
        FunctionCurveData::explicit([0.0, PI], FunctionPlotOptions::default(), |x| x.tan())
            .unwrap();
    assert!(tan_curve.discontinuities() >= 1);

    let step = FunctionCurveData::explicit([-2.0, 2.0], FunctionPlotOptions::default(), |x| {
        if x < 0.0 { -1.0 } else { 1.0 }
    })
    .unwrap();
    assert!(step.discontinuities() >= 1);

    let recip =
        FunctionCurveData::explicit([-1.5, 1.5], FunctionPlotOptions::default(), |x| 1.0 / x)
            .unwrap();
    assert!(recip.discontinuities() >= 1);

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([-2.0, -4.0], [2.0, 4.0]).unwrap());
    let (resp, out) = capture(&ctx, vec![], false, |ui| {
        Plot::new("step").show_layers(ui, &mut state, &[FunctionPlot::new(&step).into()])
    });
    let paths: Vec<_> = out
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(p) => Some(p.points.clone()),
            _ => None,
        })
        .collect();
    out.drop_without_applying_deltas();
    // Step discontinuity produces two separate branches and never connects across y=-1..1.
    assert!(
        paths.len() >= 2,
        "expected split branches across step jump, got {}",
        paths.len()
    );
    for path in &paths {
        let ys: Vec<f64> = path
            .iter()
            .map(|p| resp.transform.point_from_position(*p)[1])
            .collect();
        let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max_y - min_y < 0.5,
            "single branch must not bridge -1 to +1 jump: min={min_y}, max={max_y}"
        );
    }
}

#[test]
fn fplot_budget_limits_and_ezplot_defaults() {
    let err = FunctionCurveData::explicit(
        [0.0, 1.0],
        FunctionPlotOptions {
            initial_samples: 65,
            max_evaluations: 32,
            ..Default::default()
        },
        f64::sin,
    )
    .unwrap_err();
    assert!(matches!(err, FunctionPlotError::BudgetExceeded { .. }));

    let tight = FunctionCurveData::explicit(
        [0.05, 5.0],
        FunctionPlotOptions {
            initial_samples: 33,
            max_depth: 10,
            max_evaluations: 80,
            tolerance: 1e-6,
        },
        |x| (1.0 / x).sin(),
    )
    .unwrap();
    assert!(tight.evaluations() <= 80);
    assert!(tight.budget_truncated());

    let ez_exp = FunctionCurveData::ezplot_explicit(2048, f64::cos).unwrap();
    let xs: Vec<f64> = ez_exp.data().points().iter().map(|p| p[0]).collect();
    assert!((xs.first().copied().unwrap() + TAU).abs() < 1e-12);
    assert!((xs.last().copied().unwrap() - TAU).abs() < 1e-12);

    let ez_par = FunctionCurveData::ezplot_parametric(2048, |t| [t.cos(), t.sin()]).unwrap();
    assert!(ez_par.data().len() >= 65);
}

#[test]
fn fimplicit_extracts_cassini_branches_and_guards_singularities() {
    // Cassini oval with a = 1.0, b = 0.96 -> two disjoint lobes symmetric around (+-1, 0).
    let a: f64 = 1.0;
    let b: f64 = 0.96;
    let bounds = PlotBounds::new([-2.2, -1.5], [2.2, 1.5]).unwrap();
    let cassini = ImplicitCurveData::new(bounds, ImplicitOptions::default(), |x, y| {
        let r2 = x * x + y * y;
        r2 * r2 - 2.0 * a * a * (x * x - y * y) - (b.powi(4) - a.powi(4))
    })
    .unwrap();
    assert!(cassini.contours().lines().len() >= 2);
    let has_left_branch = cassini
        .contours()
        .lines()
        .iter()
        .any(|line| line.points.iter().all(|p| p[0] < 0.0));
    let has_right_branch = cassini
        .contours()
        .lines()
        .iter()
        .any(|line| line.points.iter().all(|p| p[0] > 0.0));
    assert!(has_left_branch && has_right_branch);

    // Flat 0 == 0 must not emit spurious contour polygons.
    let flat_zero = ImplicitCurveData::new(bounds, ImplicitOptions::default(), |_, _| 0.0).unwrap();
    assert!(flat_zero.contours().lines().is_empty());

    // Pole singularity 1/(x - y) changes sign without true zero crossing.
    let pole =
        ImplicitCurveData::new(bounds, ImplicitOptions::default(), |x, y| 1.0 / (x - y)).unwrap();
    assert!(pole.singular_cells() > 0);
    for line in pole.contours().lines() {
        for p in &line.points {
            let val = 1.0 / (p[0] - p[1]);
            assert!(val.abs() < 10.0, "spurious contour near pole: {val}");
        }
    }

    let ez_imp = ImplicitCurveData::ezplot_implicit(16_384, |x, y| x * x + y * y - 4.0).unwrap();
    assert!(!ez_imp.contours().lines().is_empty());

    let ctx = Context::default();
    let mut state = PlotState::default();
    let resp = run(&ctx, vec![], true, |ui| {
        Plot::new("fimplicit").show_layers(ui, &mut state, &[ImplicitCurve::new(&cassini).into()])
    });
    assert!(resp.diagnostics.contours.output_paths >= 2);
}
