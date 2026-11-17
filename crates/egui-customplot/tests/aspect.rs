mod support;

use egui::{Context, Event, Modifiers};
use egui_customplot::{Axis, AxisScale, Line, Plot, PlotBounds, PlotData, PlotState};
use support::run;

#[test]
fn equal_aspect_uses_data_units_or_log_decades_and_preserves_explicit_state() {
    for scale in [AxisScale::Linear, AxisScale::Log10] {
        let ctx = Context::default();
        let data = PlotData::new(vec![[1.0; 2], [100.0; 2]]);
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([1.0; 2], [100.0; 2]).unwrap());
        let plot = || {
            Plot::new("p")
                .data_aspect(1.0)
                .x_axis(Axis::new().scale(scale).reversed(true))
                .y_axis(Axis::new().scale(scale))
        };
        let out = run(&ctx, vec![], false, |ui| {
            plot().show(ui, &mut state, &[Line::new(&data)])
        });
        let p = out.transform.position_from_point([10.0; 2]);
        let q = out.transform.position_from_point(match scale {
            AxisScale::Linear => [11.0; 2],
            AxisScale::Log10 => [100.0; 2],
            AxisScale::Probit | AxisScale::Weibull => unreachable!(),
        });
        assert!(((p.x - q.x).abs() - (p.y - q.y).abs()).abs() < 0.001);
        let fixed = state.bounds().unwrap();
        assert_eq!(fixed, out.transform.bounds());
        assert!(fixed.min()[0] <= 1.0 && fixed.max()[0] >= 100.0);
        for _ in 0..5 {
            let out = run(&ctx, vec![], true, |ui| {
                plot().show(ui, &mut state, &[Line::new(&data)])
            });
            assert_eq!(state.bounds(), Some(fixed));
            assert!(!out.view_changed, "aspect fit must not drift each frame");
        }
    }
}

#[test]
fn equal_aspect_wheel_preserves_anchor_and_respects_zoom_axis_locks() {
    let data = PlotData::new(vec![[0.0; 2], [10.0; 2]]);
    for lock_y in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let plot = || {
            Plot::new("p")
                .data_aspect(1.0)
                .allow_scroll_zoom(true)
                .zoom_axes([true, !lock_y])
        };
        let first = run(&ctx, vec![], false, |ui| {
            plot().show(ui, &mut state, &[Line::new(&data)])
        });
        let pos = first.transform.rect().center() + egui::vec2(40.0, 30.0);
        let anchor = first.transform.point_from_position(pos);
        run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            plot().show(ui, &mut state, &[Line::new(&data)])
        });
        let out = run(
            &ctx,
            vec![Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 40.0),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
                modifiers: Modifiers::NONE,
            }],
            true,
            |ui| plot().show(ui, &mut state, &[Line::new(&data)]),
        );
        let actual = out.transform.point_from_position(pos);
        for axis in 0..2 {
            assert!((actual[axis] - anchor[axis]).abs() < 1e-5);
        }
        assert_eq!(out.view_changed, !lock_y);
        if lock_y {
            assert!(state.bounds().is_none());
        }
    }
}

#[test]
fn unrepresentable_aspect_falls_back_with_diagnostic_not_a_panic() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p")
            .data_aspect(f64::from_bits(1))
            .show(ui, &mut state, &[])
    });
    assert!(out.diagnostics.aspect_constraint_failed);
    assert_eq!(out.transform.data_aspect(), None);
}
