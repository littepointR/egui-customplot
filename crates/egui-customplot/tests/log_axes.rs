mod support;

use egui::{Context, Event, Modifiers};
use egui_customplot::{
    Axis, AxisScale, Line, Plot, PlotBounds, PlotData, PlotState, PlotTransform,
};
use support::run;

fn configured() -> Plot {
    Plot::new("log")
        .allow_scroll_zoom(true)
        .title("Log plot")
        .x_axis(
            Axis::new()
                .scale(AxisScale::Log10)
                .reversed(true)
                .label("Frequency / Hz")
                .minor_grid(true),
        )
        .y_axis(Axis::new().scale(AxisScale::Log10).label("Power"))
}

fn wheel(pos: egui::Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 40.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: Modifiers::NONE,
        },
    ]
}

#[test]
fn reversed_log_zoom_preserves_anchor_and_locked_axis_across_layout_passes() {
    let data = PlotData::new(vec![[1.0; 2], [1000.0; 2]]);
    let mut results = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let base = PlotBounds::new([1.2, 2.3], [1200.0, 2300.0]).unwrap();
        state.set_bounds(base);
        let first = run(&ctx, vec![], false, |ui| {
            configured()
                .zoom_axes([true, false])
                .show(ui, &mut state, &[Line::new(&data)])
        });
        let pos = first.transform.rect().center() + egui::vec2(-80.0, 30.0);
        let anchor = first.transform.point_from_position(pos);
        run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            configured()
                .zoom_axes([true, false])
                .show(ui, &mut state, &[Line::new(&data)])
        });
        let out = run(&ctx, wheel(pos), multipass, |ui| {
            configured()
                .zoom_axes([true, false])
                .show(ui, &mut state, &[Line::new(&data)])
        });
        let actual = out.transform.point_from_position(pos);
        for axis in 0..2 {
            assert!((actual[axis] / anchor[axis] - 1.0).abs() < 1e-5);
        }
        assert_eq!(out.transform.bounds().min()[1], base.min()[1]);
        assert_eq!(out.transform.bounds().max()[1], base.max()[1]);
        assert!(out.transform.bounds().span()[0] < base.span()[0]);
        results.push(state.bounds());
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn log_zoom_uses_final_layout_rectangle_not_discarded_geometry() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [1000.0; 2]]);
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        configured().show(ui, &mut state, &[Line::new(&data)])
    });
    let pos = first.transform.rect().center();
    run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        configured().show(ui, &mut state, &[Line::new(&data)])
    });
    let mut pass = 0;
    let out = run(&ctx, wheel(pos), true, |ui| {
        let height = if pass == 0 { 320.0 } else { 440.0 };
        pass += 1;
        configured()
            .height(height)
            .show(ui, &mut state, &[Line::new(&data)])
    });
    let base = PlotTransform::with_axes(
        first.transform.bounds(),
        out.transform.rect(),
        [AxisScale::Log10; 2],
        [true, false],
    )
    .unwrap();
    let a = base.point_from_position(pos);
    let b = out.transform.point_from_position(pos);
    for axis in 0..2 {
        assert!((a[axis] / b[axis] - 1.0).abs() < 1e-5);
    }
}

#[test]
fn log_domain_diagnostics_default_bounds_and_invalid_manual_bounds_are_explicit() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [-1.0; 2], [f64::NAN; 2]]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::default());
    let out = run(&ctx, vec![], false, |ui| {
        configured().show(ui, &mut state, &[Line::new(&data)])
    });
    assert!(out.diagnostics.invalid_manual_bounds);
    assert_eq!(out.diagnostics.domain_invalid_points, 2);
    assert_eq!(out.diagnostics.non_finite_points, 1);
    assert_eq!(out.diagnostics.output_paths, 0);
    assert!(out.transform.bounds().min().iter().all(|x| *x > 0.0));
    assert!(state.bounds().is_none());
    for ticks in out.axis_ticks {
        assert!(!ticks.is_empty());
        assert!(ticks.iter().all(|t| t.value > 0.0 && t.value.is_finite()));
    }
}
