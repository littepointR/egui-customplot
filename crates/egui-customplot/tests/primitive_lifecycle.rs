mod support;

use egui::{Color32, Context, IdSalt, Shape};
use egui_customplot::{
    Bar, BarData, Bars, ErrorBar, ErrorBarData, ErrorBars, Plot, PlotCache, PlotState,
};
use std::sync::Arc;
use support::{capture, run};

#[test]
fn primitive_payloads_are_shared_deduplicated_and_not_retained_by_plot_state_or_cache() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(1024);
    for _ in 0..32 {
        let bars: Arc<[Bar]> = vec![Bar::new(1.0, 2.0)].into();
        let errors: Arc<[ErrorBar]> = vec![ErrorBar::new([1.0; 2]).y_error([0.5; 2])].into();
        let data = BarData::new(bars.clone());
        let clone = data.clone();
        let errors_data = ErrorBarData::new(errors.clone());
        let errors_clone = errors_data.clone();
        let layers = [
            Bars::new(&data).id_salt("a").into(),
            Bars::new(&clone).id_salt("b").into(),
            ErrorBars::new(&errors_data).id_salt("c").into(),
            ErrorBars::new(&errors_clone).id_salt("d").into(),
        ];
        let out = run(&ctx, vec![], true, |ui| {
            Plot::new("p").show_layers_cached(ui, &mut state, &layers, &mut cache)
        });
        assert_eq!(out.diagnostics.unique_bar_bytes, std::mem::size_of::<Bar>());
        assert_eq!(
            out.diagnostics.unique_error_bar_bytes,
            std::mem::size_of::<ErrorBar>()
        );
        assert_eq!(
            (out.diagnostics.input_bars, out.diagnostics.input_error_bars),
            (2, 2)
        );
        drop((data, clone, errors_data, errors_clone));
        assert_eq!(Arc::strong_count(&bars), 1);
        assert_eq!(Arc::strong_count(&errors), 1);
        assert_eq!(cache.entry_count(), 0);
    }
}

#[test]
fn hidden_and_parent_clipped_primitives_do_no_geometry_work() {
    let ctx = Context::default();
    let bars = BarData::new(vec![Bar::new(1.0, 2.0)]);
    let errors = ErrorBarData::new(vec![ErrorBar::new([1.0; 2]).y_error([1.0; 2])]);
    let mut state = PlotState::default();
    let layers = [
        Bars::new(&bars).id_salt("a").into(),
        ErrorBars::new(&errors).id_salt("b").into(),
    ];
    let out = run(&ctx, vec![], false, |ui| {
        ui.set_clip_rect(egui::Rect::NOTHING);
        Plot::new("p").show_layers(ui, &mut state, &layers)
    });
    assert!(out.diagnostics.painting_skipped);
    assert_eq!(
        (
            out.diagnostics.visited_bars,
            out.diagnostics.visited_error_bars
        ),
        (0, 0)
    );
    state.set_series_visible(IdSalt::new("a"), false);
    state.set_series_visible(IdSalt::new("b"), false);
    let out = run(&ctx, vec![], true, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &layers)
    });
    assert_eq!(
        (
            out.diagnostics.visited_bars,
            out.diagnostics.visited_error_bars
        ),
        (0, 0)
    );
}

#[test]
fn transparent_bar_composition_preserves_fill_outline_then_next_record_order() {
    let ctx = Context::default();
    let data = BarData::new(vec![Bar::new(1.0, 2.0), Bar::new(1.0, 3.0)]);
    let mut state = PlotState::default();
    let fill = Color32::from_rgba_unmultiplied(255, 0, 0, 80);
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[Bars::new(&data)
                .color(fill)
                .border(egui::Stroke::new(1.0, Color32::GREEN))
                .into()],
        )
    });
    assert_eq!(out.diagnostics.output_bars, 2);
    let order = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Rect(r) if r.fill == fill => Some('f'),
            Shape::LineSegment { stroke, .. } if stroke.color == Color32::GREEN => Some('b'),
            _ => None,
        })
        .collect::<String>();
    assert_eq!(order, "fbbbbfbbbb");
    output.drop_without_applying_deltas();
}
