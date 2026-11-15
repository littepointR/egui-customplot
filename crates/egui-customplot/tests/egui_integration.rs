use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use egui_customplot::{Line, Plot, PlotBounds, PlotData, PlotResponse, PlotState};

fn input(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
        events,
        ..Default::default()
    }
}

fn wheel(pos: Pos2) -> Vec<Event> {
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

fn show(
    ctx: &Context,
    state: &mut PlotState,
    data: &PlotData,
    events: Vec<Event>,
    multipass: bool,
) -> PlotResponse {
    let mut result = None;
    let mut pass = 0;
    let out = ctx.run_ui(input(events), |ui| {
        result = Some(Plot::new("plot").allow_scroll_zoom(true).show(
            ui,
            state,
            &[Line::new(data)],
        ));
        if multipass && pass == 0 {
            ui.ctx().request_discard("integration test");
        }
        pass += 1;
    });
    if multipass {
        assert_eq!(pass, 2);
    }
    out.drop_without_applying_deltas();
    result.unwrap()
}

fn data() -> PlotData {
    PlotData::new(vec![[0.0, 0.0], [10.0, 10.0]])
}

#[test]
fn wheel_anchor_and_multipass_match_single_pass() {
    let data = data();
    let mut results = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let initial = show(&ctx, &mut state, &data, vec![], false);
        let pos = initial.transform.rect().center() + egui::vec2(50.0, -30.0);
        let anchor = initial.transform.point_from_position(pos);
        show(
            &ctx,
            &mut state,
            &data,
            vec![Event::PointerMoved(pos)],
            false,
        );
        let out = show(&ctx, &mut state, &data, wheel(pos), multipass);
        assert!(out.view_changed);
        for (a, b) in out.transform.point_from_position(pos).iter().zip(anchor) {
            assert!((a - b).abs() < 1e-5);
        }
        assert!(state.bounds().unwrap().span()[0] < initial.transform.bounds().span()[0]);
        assert!(out.diagnostics.output_paths > 0, "final pass must paint");
        results.push(state.bounds());
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn drag_multipass_matches_single_pass() {
    let data = data();
    let mut results = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let pos = show(&ctx, &mut state, &data, vec![], false)
            .transform
            .rect()
            .center();
        show(
            &ctx,
            &mut state,
            &data,
            vec![Event::PointerMoved(pos)],
            false,
        );
        show(
            &ctx,
            &mut state,
            &data,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }],
            false,
        );
        let out = show(
            &ctx,
            &mut state,
            &data,
            vec![Event::PointerMoved(pos + egui::vec2(40.0, 20.0))],
            multipass,
        );
        assert!(out.view_changed);
        assert!(out.transform.bounds().min()[0] < -0.5);
        results.push(state.bounds());
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn final_layout_changes_recompute_the_zoom_anchor() {
    let ctx = Context::default();
    let data = data();
    let mut state = PlotState::default();
    let initial = show(&ctx, &mut state, &data, vec![], false);
    let pos = initial.transform.rect().center();
    show(
        &ctx,
        &mut state,
        &data,
        vec![Event::PointerMoved(pos)],
        false,
    );
    let mut pass = 0;
    let mut final_result = None;
    let out = ctx.run_ui(input(wheel(pos)), |ui| {
        let height = if pass == 0 { 320.0 } else { 440.0 };
        let result = Plot::new("plot")
            .height(height)
            .allow_scroll_zoom(true)
            .show(ui, &mut state, &[Line::new(&data)]);
        final_result = Some(result);
        if pass == 0 {
            ui.ctx().request_discard("size changed");
        }
        pass += 1;
    });
    out.drop_without_applying_deltas();
    assert_eq!(pass, 2);
    let final_result = final_result.unwrap();
    let base_transform = egui_customplot::PlotTransform::new(
        initial.transform.bounds(),
        final_result.transform.rect(),
    )
    .unwrap();
    let expected_anchor = base_transform.point_from_position(pos);
    let actual_anchor = final_result.transform.point_from_position(pos);
    for axis in 0..2 {
        assert!((expected_anchor[axis] - actual_anchor[axis]).abs() < 1e-5);
    }
    assert_eq!(final_result.transform.rect().height(), 396.0);
}

#[test]
fn stable_ids_and_separate_states_share_only_samples() {
    let ctx = Context::default();
    let data = data();
    let mut a = PlotState::default();
    let mut b = PlotState::default();
    a.set_bounds(PlotBounds::new([1.0; 2], [3.0; 2]).unwrap());
    let mut ids = Vec::new();
    for insert_label in [false, true] {
        ctx.run_ui(input(vec![]), |ui| {
            if insert_label {
                ui.label("Inserted before plots");
            }
            let first = Plot::new("a").height(100.0).show(
                ui,
                &mut a,
                &[Line::new(&data), Line::new(&data)],
            );
            let second = Plot::new("b")
                .height(100.0)
                .show(ui, &mut b, &[Line::new(&data)]);
            assert_ne!(first.response.id, second.response.id);
            assert_eq!(first.diagnostics.unique_sample_bytes, data.sample_bytes());
            ids.push(first.response.id);
        })
        .drop_without_applying_deltas();
    }
    assert_eq!(ids[0], ids[1]);
    assert!(a.bounds().is_some());
    assert!(b.bounds().is_none());
}

#[test]
fn disabled_outside_and_default_wheel_do_not_navigate() {
    let data = data();
    for mode in ["disabled", "outside", "default"] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        for frame in 0..3 {
            ctx.run_ui(
                input(if frame == 0 {
                    vec![]
                } else {
                    wheel(if mode == "outside" {
                        egui::pos2(799.0, 599.0)
                    } else {
                        egui::pos2(300.0, 150.0)
                    })
                }),
                |ui| {
                    ui.add_enabled_ui(mode != "disabled", |ui| {
                        Plot::new("p").allow_scroll_zoom(mode != "default").show(
                            ui,
                            &mut state,
                            &[Line::new(&data)],
                        );
                    });
                },
            )
            .drop_without_applying_deltas();
        }
        assert!(state.bounds().is_none(), "{mode}");
    }
}

#[test]
fn respects_parent_clipping_and_skips_hidden_geometry() {
    let ctx = Context::default();
    let data = data();
    let mut state = PlotState::default();
    let clip = Rect::from_min_size(egui::pos2(100.0, 80.0), egui::vec2(100.0, 80.0));
    let output = ctx.run_ui(input(vec![]), |ui| {
        ui.set_clip_rect(clip);
        Plot::new("clipped").show(ui, &mut state, &[Line::new(&data)]);
    });
    assert!(!output.shapes.is_empty());
    for shape in &output.shapes {
        assert!(clip.contains_rect(shape.clip_rect), "{:?}", shape.clip_rect);
    }
    output.drop_without_applying_deltas();
    ctx.run_ui(input(vec![]), |ui| {
        ui.set_clip_rect(Rect::from_min_size(
            egui::pos2(900.0, 900.0),
            Vec2::splat(10.0),
        ));
        let result = Plot::new("hidden").show(ui, &mut state, &[Line::new(&data)]);
        assert!(result.diagnostics.painting_skipped);
        assert_eq!(result.diagnostics.visited_segments, 0);
        assert!(result.transform.rect().is_positive());
    })
    .drop_without_applying_deltas();
}

#[test]
fn focused_home_resets_without_reapplying_pan_or_zoom() {
    let ctx = Context::default();
    let data = data();
    let mut state = PlotState::default();
    let result = show(&ctx, &mut state, &data, vec![], false);
    result.response.request_focus();
    state.set_bounds(PlotBounds::new([2.0; 2], [4.0; 2]).unwrap());
    let result = show(
        &ctx,
        &mut state,
        &data,
        vec![Event::Key {
            key: egui::Key::Home,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        true,
    );
    assert!(state.bounds().is_none());
    assert!(result.view_changed);
}

#[test]
fn empty_invalid_and_tiny_plots_remain_finite() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    ctx.run_ui(input(vec![]), |ui| {
        let invalid = PlotData::new(vec![[f64::NAN; 2], [f64::INFINITY; 2]]);
        let out = Plot::new("tiny")
            .height(1.0)
            .show(ui, &mut state, &[Line::new(&invalid)]);
        assert!(out.transform.rect().is_finite());
        assert_eq!(out.diagnostics.output_paths, 0);
        assert_eq!(out.diagnostics.non_finite_points, 2);
    })
    .drop_without_applying_deltas();
}

#[test]
fn settled_static_plot_does_not_schedule_continuous_repaints() {
    let ctx = Context::default();
    let data = data();
    let mut state = PlotState::default();
    for frame in 0..10 {
        let output = ctx.run_ui(input(vec![]), |ui| {
            Plot::new("idle").show(ui, &mut state, &[Line::new(&data)]);
        });
        if frame > 5 {
            let delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            assert!(
                delay > std::time::Duration::ZERO,
                "static plot is requesting immediate repaint"
            );
        }
        output.drop_without_applying_deltas();
    }
}

#[test]
fn default_plot_leaves_wheel_for_parent_scroll_area() {
    let ctx = Context::default();
    let data = data();
    let mut state = PlotState::default();
    let mut offset = 0.0;
    for frame in 0..5 {
        let mut events = if frame >= 2 {
            wheel(egui::pos2(300.0, 150.0))
        } else {
            vec![Event::PointerMoved(egui::pos2(300.0, 150.0))]
        };
        for event in &mut events {
            if let Event::MouseWheel { delta, .. } = event {
                delta.y = -40.0;
            }
        }
        ctx.run_ui(input(events), |ui| {
            offset = egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    Plot::new("nested")
                        .height(1000.0)
                        .show(ui, &mut state, &[Line::new(&data)]);
                })
                .state
                .clamped_offset()
                .y;
        })
        .drop_without_applying_deltas();
    }
    assert!(offset > 0.0, "parent should scroll");
    assert!(state.bounds().is_none(), "plot should remain auto-fit");
}

#[test]
fn repeated_data_destruction_leaves_no_sample_references_in_egui_or_state() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    for _ in 0..100 {
        let points: std::sync::Arc<[[f64; 2]]> =
            (0..1000).map(|i| [i as f64, (i as f64).sin()]).collect();
        let weak = std::sync::Arc::downgrade(&points);
        let data = PlotData::new(points);
        ctx.run_ui(input(vec![]), |ui| {
            Plot::new("lifecycle").show(ui, &mut state, &[Line::new(&data)]);
        })
        .drop_without_applying_deltas();
        drop(data);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn horizontally_unbounded_parent_still_gets_finite_plot_geometry() {
    let ctx = Context::default();
    let data = data();
    let mut state = PlotState::default();
    ctx.run_ui(input(vec![]), |ui| {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let result = Plot::new("horizontal-scroll").show(ui, &mut state, &[Line::new(&data)]);
            assert!(result.transform.rect().is_finite());
            assert!(result.transform.rect().width() < 1000.0);
        });
    })
    .drop_without_applying_deltas();
}
