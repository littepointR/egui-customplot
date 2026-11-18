#[path = "support/input.rs"]
mod support;

use egui::{Context, Event, Modifiers, PointerButton, pos2, vec2};
use egui_customplot::{
    GeoGlobeData, GeoGlobePlot, GeoGlobeState, GeoGlobeTransform, SkyPlot, SkyPlotData,
    SkyPlotState, SkyTrack, SmithPlot, SmithPlotData, SmithPlotState, SmithTransform,
};

#[test]
fn sparse_sky_legend_preserves_named_and_occupied_group_ids() {
    let data = SkyPlotData::from_az_el(&[0.0, 45.0], &[30.0, 50.0])
        .unwrap()
        .with_tracks(vec![
            SkyTrack::new("arc", &[0.0], &[40.0])
                .unwrap()
                .group(100_000),
        ])
        .with_groups(&[0, usize::MAX])
        .unwrap()
        .with_group_names(&["GPS", "planned"]);
    let mut state = SkyPlotState::default();
    state.set_group_visible(usize::MAX, false);
    Context::default()
        .run_ui(support::input(vec![]), |ui| {
            let response = SkyPlot::new("sky", &data).show(ui, &mut state);
            let entries = response.legend_entries;
            assert_eq!(
                entries.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
                [
                    "GPS",
                    "planned",
                    "Group 100001",
                    &format!("Group {}", usize::MAX as u128 + 1)
                ]
            );
            assert!(!entries[3].visible);
        })
        .drop_without_applying_deltas();
}

#[test]
fn smith_scroll_replays_about_the_anchor_at_final_layout_size() {
    let ctx = Context::default();
    let data = SmithPlotData::from_impedance(&[1.0], &[[50.0, 0.0]], 50.0).unwrap();
    let mut state = SmithPlotState::default();
    let anchor = pos2(325.0, 120.0);
    let plot = |height| {
        SmithPlot::new("smith", &data)
            .height(height)
            .allow_scroll_zoom(true)
    };
    ctx.run_ui(support::input(vec![Event::PointerMoved(anchor)]), |ui| {
        plot(240.0).show(ui, &mut state);
    })
    .drop_without_applying_deltas();
    let wheel = Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: vec2(0.0, 40.0),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        modifiers: Modifiers::NONE,
    };
    let mut spans = Vec::new();
    ctx.run_ui(support::input(vec![wheel]), |ui| {
        let first = spans.is_empty();
        let r = plot(if first { 240.0 } else { 420.0 }).show(ui, &mut state);
        assert!(r.view_changed);
        spans.push(state.half_span());
        let base = SmithTransform::new(r.plot_area, [0.0; 2], 1.08);
        assert!(
            r.transform
                .gamma_to_pos(base.pos_to_gamma(anchor))
                .distance(anchor)
                < 0.001
        );
        if first {
            ui.ctx().request_discard("larger chart");
        }
    })
    .drop_without_applying_deltas();
    assert_eq!(spans.len(), 2);
    assert!(spans[0] < 1.08);
    assert_eq!(spans[0], spans[1]);
}

#[test]
fn globe_drag_replays_at_final_radius_from_the_initial_orbit() {
    let ctx = Context::default();
    let data = GeoGlobeData::new();
    let mut state = GeoGlobeState::default();
    let start = pos2(350.0, 100.0);
    let delta = vec2(36.0, 18.0);
    for events in [
        vec![Event::PointerMoved(start)],
        vec![Event::PointerButton {
            pos: start,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    ] {
        ctx.run_ui(support::input(events), |ui| {
            GeoGlobePlot::new("globe", &data)
                .height(240.0)
                .show(ui, &mut state);
        })
        .drop_without_applying_deltas();
    }
    let mut passes = 0;
    ctx.run_ui(
        support::input(vec![Event::PointerMoved(start + delta)]),
        |ui| {
            let height = if passes == 0 { 240.0 } else { 420.0 };
            let r = GeoGlobePlot::new("globe", &data)
                .height(height)
                .show(ui, &mut state);
            assert!(r.view_changed);
            let base = GeoGlobeTransform::new(r.outer_rect, 25.0, 105.0, 1.0, 1.0);
            assert_eq!(state.center_lat_lon(), base.orbit(delta));
            if passes == 0 {
                ui.ctx().request_discard("larger globe");
            }
            passes += 1;
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);
}
