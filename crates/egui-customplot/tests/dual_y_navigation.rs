mod support;

use egui::{Context, Event, Modifiers, PointerButton};
use egui_customplot::{Axis, AxisScale, Line, Plot, PlotBounds, PlotData, PlotState, YAxis};
use support::run;

#[test]
fn drag_replays_in_both_y_coordinates_once_and_y_lock_applies_to_both() {
    for lock_y in [false, true] {
        let mut outcomes = Vec::new();
        for multipass in [false, true] {
            let ctx = Context::default();
            let mut state = PlotState::default();
            let left = PlotBounds::new([0.0, -10.0], [10.0, 10.0]).unwrap();
            state.set_bounds(left);
            state.set_right_y_range([1.0, 1000.0]).unwrap();
            let plot = || {
                Plot::new("p")
                    .drag_axes([true, !lock_y])
                    .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            };
            let initial = run(&ctx, vec![], false, |ui| plot().show(ui, &mut state, &[]));
            let pos = initial.transform.rect().center();
            run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
                plot().show(ui, &mut state, &[])
            });
            run(
                &ctx,
                vec![Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                }],
                false,
                |ui| plot().show(ui, &mut state, &[]),
            );
            let delta = egui::vec2(40.0, 20.0);
            let out = run(
                &ctx,
                vec![Event::PointerMoved(pos + delta)],
                multipass,
                |ui| plot().show(ui, &mut state, &[]),
            );
            let applied = egui::vec2(delta.x, if lock_y { 0.0 } else { delta.y });
            for (before, after, point) in [
                (initial.transform, out.transform, [5.0, 0.0]),
                (
                    initial.right_transform.unwrap(),
                    out.right_transform.unwrap(),
                    [5.0, 1000.0_f64.sqrt()],
                ),
            ] {
                let expected = before.position_from_point(point) + applied;
                assert!(after.position_from_point(point).distance(expected) < 0.001);
            }
            if lock_y {
                assert_eq!(state.right_y_range(), Some([1.0, 1000.0]));
            }
            outcomes.push((state.bounds(), state.right_y_range()));
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }
}

#[test]
fn equal_screen_distance_across_y_axes_keeps_submission_order() {
    let ctx = Context::default();
    let a = PlotData::new(vec![[5.0, 5.0]]);
    let b = PlotData::new(vec![[5.0, 500.0]]);
    for right_first in [false, true] {
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([0.0; 2], [10.0; 2]).unwrap());
        state.set_right_y_range([0.0, 1000.0]).unwrap();
        let mut lines = [Line::new(&a), Line::new(&b).y_axis(YAxis::Right)];
        if right_first {
            lines.reverse();
        }
        let initial = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .right_y_axis(Axis::new())
                .show(ui, &mut state, &lines)
        });
        let pos = initial.transform.position_from_point(a.points()[0]);
        assert_eq!(
            pos,
            initial
                .right_transform
                .unwrap()
                .position_from_point(b.points()[0])
        );
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p")
                .right_y_axis(Axis::new())
                .show(ui, &mut state, &lines)
        });
        let hovered = out.hovered_point.unwrap();
        assert_eq!(hovered.series_index, 0);
        assert_eq!(
            hovered.y_axis,
            if right_first {
                YAxis::Right
            } else {
                YAxis::Left
            }
        );
    }
}
