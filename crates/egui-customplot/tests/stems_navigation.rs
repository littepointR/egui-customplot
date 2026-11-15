#[path = "support/line.rs"]
mod line_support;
mod support;

use egui::{Context, Event, IdSalt, Modifiers};
use egui_customplot::{Axis, AxisScale, Plot, PlotData, PlotState, StemOrientation, Stems};
use line_support::{assert_axis_aligned, paths};
use support::{capture, run};

#[test]
fn log_reversed_navigation_and_stem_geometry_are_identical_with_layout_retries() {
    let data = PlotData::new(vec![[1.0, 10.0], [10.0, 100.0], [100.0, 20.0]]);
    for orientation in [StemOrientation::Vertical, StemOrientation::Horizontal] {
        let mut results = Vec::new();
        for multipass in [false, true] {
            let ctx = Context::default();
            let mut state = PlotState::default();
            let stems = Stems::new(&data).baseline(0.5).orientation(orientation);
            let plot = || {
                Plot::new("p")
                    .allow_scroll_zoom(true)
                    .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
                    .y_axis(Axis::new().scale(AxisScale::Log10))
            };
            let first = run(&ctx, vec![], false, |ui| {
                plot().show_layers(ui, &mut state, &[stems.into()])
            });
            let pos = first.transform.rect().center();
            run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
                plot().show_layers(ui, &mut state, &[stems.into()])
            });
            let (out, output) = capture(
                &ctx,
                vec![
                    Event::PointerMoved(pos),
                    Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, 40.0),
                        phase: egui::TouchPhase::Move,
                        source: egui::MouseWheelSource::Unknown,
                        modifiers: Modifiers::NONE,
                    },
                ],
                multipass,
                |ui| plot().show_layers(ui, &mut state, &[stems.into()]),
            );
            let actual = paths(&output);
            output.drop_without_applying_deltas();
            assert!(out.view_changed);
            assert_axis_aligned(&actual);
            results.push((out.transform.bounds(), actual));
        }
        assert_eq!(results[0], results[1]);
    }
}

#[test]
fn legend_click_toggles_all_stem_components_once_without_navigating() {
    let data = PlotData::new(vec![[1.0, 2.0], [3.0, 1.0]]);
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let layer = Stems::new(&data).id_salt("s").name("Stems").into();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_layers(ui, &mut state, &[layer])
        });
        let pos = first.legend_entries[0].rect.center();
        let button = |pressed| Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: Modifiers::NONE,
        };
        for events in [
            vec![Event::PointerMoved(pos)],
            vec![button(true)],
            vec![button(false)],
        ] {
            run(&ctx, events, multipass, |ui| {
                Plot::new("p")
                    .legend(true)
                    .show_layers(ui, &mut state, &[layer])
            });
        }
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_layers(ui, &mut state, &[layer])
        });
        assert!(!state.series_visible(IdSalt::new("s")));
        assert!(state.bounds().is_none());
        assert_eq!(
            (
                out.diagnostics.visited_stems,
                out.diagnostics.output_markers,
                out.diagnostics.output_stem_baselines
            ),
            (0, 0, 0)
        );
    }
}
