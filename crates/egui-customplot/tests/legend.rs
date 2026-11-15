mod support;

use egui::{Context, Event, IdSalt, Modifiers, PointerButton};
use egui_customplot::{Line, Plot, PlotData, PlotState, Scatter, Series};
use support::run;

fn button(pos: egui::Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn legend_click_toggles_once_with_discarded_layout_and_does_not_pan() {
    for multipass in [false, true] {
        let ctx = Context::default();
        let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
        let series = [Line::new(&data).name("signal").id_salt("a").into()];
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_series(ui, &mut state, &series)
        });
        let pos = first.legend_entries[0].rect.center();
        run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_series(ui, &mut state, &series)
        });
        run(&ctx, vec![button(pos, true)], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_series(ui, &mut state, &series)
        });
        let out = run(&ctx, vec![button(pos, false)], multipass, |ui| {
            Plot::new("p")
                .legend(true)
                .show_series(ui, &mut state, &series)
        });
        assert!(!state.series_visible(IdSalt::new("a")), "{multipass}");
        assert!(!out.legend_entries[0].visible);
        assert!(state.bounds().is_none(), "legend click must not navigate");
        assert!(out.hovered_point.is_none());
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_series(ui, &mut state, &series)
        });
        assert_eq!(out.diagnostics.visible_series, 0);
        assert_eq!(out.diagnostics.output_paths, 0);
    }
}

#[test]
fn explicit_ids_survive_reordering_and_removed_visibility_state_is_pruned() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let a: Series<'_> = Line::new(&data).name("same name").id_salt("a").into();
    let b: Series<'_> = Scatter::new(&data).name("same name").id_salt("b").into();
    let mut state = PlotState::default();
    state.set_series_visible(IdSalt::new("a"), false);
    for series in [[a, b], [b, a]] {
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .legend(true)
                .show_series(ui, &mut state, &series)
        });
        assert_eq!(out.diagnostics.duplicate_series_ids, 0);
        assert_eq!(out.diagnostics.visible_series, 1);
        assert!(
            !out.legend_entries
                .iter()
                .find(|e| e.id == IdSalt::new("a"))
                .unwrap()
                .visible
        );
        assert!(
            out.legend_entries
                .iter()
                .find(|e| e.id == IdSalt::new("b"))
                .unwrap()
                .visible
        );
    }
    run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_series(ui, &mut state, &[b])
    });
    assert!(state.series_visible(IdSalt::new("a")));
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_series(ui, &mut state, &[a, b])
    });
    assert_eq!(out.diagnostics.visible_series, 2);
}

#[test]
fn duplicate_implicit_ids_are_reported_instead_of_claiming_independent_visibility() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2]]);
    let mut state = PlotState::default();
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_series(
            ui,
            &mut state,
            &[
                Line::new(&data).name("same").into(),
                Scatter::new(&data).name("same").into(),
            ],
        )
    });
    assert_eq!(out.diagnostics.duplicate_series_ids, 1);
}
