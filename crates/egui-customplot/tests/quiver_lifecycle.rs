mod support;

use std::sync::Arc;

use egui::{Context, Event, IdSalt, Modifiers, Shape};
use egui_customplot::{
    Axis, AxisScale, Plot, PlotCache, PlotState, Quiver, QuiverArrow, QuiverData,
};
use support::{capture, run};

#[test]
fn payloads_are_shared_deduplicated_and_released_while_state_context_and_line_cache_survive() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(4096);
    for _ in 0..32 {
        let arrows: Arc<[QuiverArrow]> = vec![QuiverArrow::new([1.0; 2], [2.0; 2])].into();
        let data = QuiverData::new(arrows.clone());
        let clone = data.clone();
        let layers = [
            Quiver::new(&data).id_salt("a").into(),
            Quiver::new(&clone).id_salt("b").into(),
        ];
        let out = run(&ctx, vec![], true, |ui| {
            Plot::new("p").show_layers_cached(ui, &mut state, &layers, &mut cache)
        });
        assert_eq!(out.diagnostics.input_quiver_arrows, 2);
        assert_eq!(out.diagnostics.unique_quiver_bytes, data.sample_bytes());
        assert_eq!(out.diagnostics.line_cache_bytes, 0);
        assert_eq!(out.diagnostics.line_cache_misses, 0);
        drop((data, clone));
        assert_eq!(Arc::strong_count(&arrows), 1);
    }
    assert_eq!(cache.retained_bytes(), 0);
}

#[test]
fn log_reversed_zoom_and_vector_geometry_match_with_layout_retries() {
    let data = QuiverData::new(vec![
        QuiverArrow::new([1.0; 2], [99.0, 9.0]),
        QuiverArrow::new([50.0; 2], [-40.0, 50.0]),
    ]);
    let mut results = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let layer = Quiver::new(&data).into();
        let plot = || {
            Plot::new("p")
                .allow_scroll_zoom(true)
                .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
                .y_axis(Axis::new().scale(AxisScale::Log10))
        };
        let first = run(&ctx, vec![], false, |ui| {
            plot().show_layers(ui, &mut state, &[layer])
        });
        let pos = first.transform.rect().center();
        run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            plot().show_layers(ui, &mut state, &[layer])
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
            |ui| plot().show_layers(ui, &mut state, &[layer]),
        );
        let paths: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                Shape::Path(p) => Some(p.points.clone()),
                _ => None,
            })
            .collect();
        output.drop_without_applying_deltas();
        assert!(out.view_changed);
        results.push((out.transform.bounds(), paths));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn parent_clip_visibility_and_legend_once_per_frame_cover_all_vector_parts() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![QuiverArrow::new([1.0; 2], [2.0; 2])]);
    let mut state = PlotState::default();
    let layer = Quiver::new(&data).name("field").id_salt("q").into();
    let clipped = run(&ctx, vec![], true, |ui| {
        ui.set_clip_rect(egui::Rect::NOTHING);
        Plot::new("p").show_layers(ui, &mut state, &[layer])
    });
    assert!(clipped.diagnostics.painting_skipped);
    assert_eq!(clipped.diagnostics.visited_quiver_arrows, 0);
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
        run(&ctx, events, true, |ui| {
            Plot::new("p")
                .legend(true)
                .show_layers(ui, &mut state, &[layer])
        });
    }
    assert!(!state.series_visible(IdSalt::new("q")));
    let hidden = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[layer])
    });
    assert_eq!(hidden.diagnostics.visited_quiver_arrows, 0);
    assert_eq!(hidden.diagnostics.output_quiver_heads, 0);
    assert!(state.bounds().is_none());
    run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[])
    });
    assert!(state.series_visible(IdSalt::new("q")));
}
