mod support;

use egui::Context;
use egui_customplot::{Line, LinePattern, MarkerStyle, Plot, PlotData, PlotState};
use support::run;

#[test]
fn dashed_strokes_are_screen_spaced_and_keep_line_markers() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0, 0.0], [10.0, 0.0]]);
    let mut state = PlotState::default();
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(
            ui,
            &mut state,
            &[Line::new(&data)
                .pattern(LinePattern::Dashed {
                    length: 8.0,
                    gap: 4.0,
                })
                .markers(MarkerStyle::default())],
        )
    });
    assert_eq!(out.diagnostics.visited_segments, 1);
    assert!(out.diagnostics.output_paths > 10);
    assert_eq!(out.diagnostics.output_markers, 2);
    assert!(!out.diagnostics.pattern_truncated);
}

#[test]
fn invalid_dash_patterns_skip_stroke_but_do_not_hide_markers() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    for length in [0.0, 0.1, -1.0, f32::NAN, f32::INFINITY] {
        let mut state = PlotState::default();
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show(
                ui,
                &mut state,
                &[Line::new(&data)
                    .pattern(LinePattern::Dashed { length, gap: 4.0 })
                    .markers(MarkerStyle::default())],
            )
        });
        assert_eq!(out.diagnostics.output_paths, 0);
        assert_eq!(out.diagnostics.output_markers, 2);
        assert_eq!(out.diagnostics.numeric_rejections, 1);
    }
}
