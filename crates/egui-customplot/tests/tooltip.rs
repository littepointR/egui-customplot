mod support;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use egui::{Context, Event};
use egui_customplot::{Line, Plot, PlotData, PlotState};
use support::run;

#[test]
fn custom_formatter_receives_raw_pick_and_can_read_context_outside_locks() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0, 2.0], [3.0, 4.0]]);
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let pos = first.transform.position_from_point([1.0, 2.0]);
    let calls = Arc::new(AtomicUsize::new(0));
    let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        let calls = calls.clone();
        let context = ctx.clone();
        Plot::new("p")
            .hover_formatter(move |info| {
                context.input(|i| assert!(i.pointer.hover_pos().is_some()));
                calls.fetch_add(1, Ordering::SeqCst);
                let point = info.hovered_point.unwrap();
                assert_eq!(point.point, [1.0, 2.0]);
                assert_eq!(point.point_index, 0);
                assert_eq!(info.series_name, "voltage");
                format!("{}: {} V", info.series_name, point.point[1])
            })
            .show(ui, &mut state, &[Line::new(&data).name("voltage")])
    });
    assert_eq!(out.hover_text.as_deref(), Some("voltage: 2 V"));
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "formatter is per pass, not a once-per-frame event"
    );
}

#[test]
fn cursor_formatter_runs_without_a_hit_but_not_when_hover_is_disabled_or_clipped() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[])
    });
    let position = first.transform.rect().center();
    let out = run(&ctx, vec![Event::PointerMoved(position)], false, |ui| {
        Plot::new("p")
            .hover_formatter(|info| {
                assert!(info.hovered_point.is_none());
                assert!(info.series_name.is_empty());
                assert!(info.cursor.iter().all(|v| v.abs() < 1e-5));
                "cursor".to_owned()
            })
            .show(ui, &mut state, &[])
    });
    assert_eq!(out.hover_text.as_deref(), Some("cursor"));
    for clipped in [false, true] {
        let out = run(&ctx, vec![], false, |ui| {
            if clipped {
                ui.set_clip_rect(egui::Rect::NOTHING);
            }
            Plot::new("p")
                .show_hover(clipped)
                .hover_formatter(|_| panic!("not hovered"))
                .show(ui, &mut state, &[])
        });
        assert!(out.hover_text.is_none());
    }
}

#[test]
fn default_text_includes_series_and_original_index() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let position = first.transform.position_from_point([2.0; 2]);
    let out = run(&ctx, vec![Event::PointerMoved(position)], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data).name("sample")])
    });
    assert!(out.hover_text.unwrap().starts_with("sample\n[1]"));
}
