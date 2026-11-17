use egui::{Color32, Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_customplot::{
    Axis, AxisScale, BarOrientation, BoxChart, BoxChartData, ColorMap, Line, MarkerSampling,
    MarkerShape, MarkerStyle, Plot, PlotData, PlotLayer, PlotState, Raincloud, RaincloudData,
    Scatter, ViolinData, ViolinOptions, ViolinPlot, YAxis, civil_to_unix_secs, format_duration_hms,
    unix_secs_to_civil,
};

fn input_events(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 620.0))),
        events,
        ..Default::default()
    }
}

#[test]
fn datetime_and_duration_axes_cover_multi_scale_ticks_and_civil_edge_cases() {
    assert!(civil_to_unix_secs(2024, 2, 29, 23, 59, 59.0).is_some());
    assert!(civil_to_unix_secs(2023, 2, 29, 0, 0, 0.0).is_none());
    assert!(civil_to_unix_secs(2026, 13, 1, 0, 0, 0.0).is_none());

    // 999.9 ms rollover carries cleanly to the next second instead of millis == 1000.
    let base = civil_to_unix_secs(2024, 12, 31, 23, 59, 59.0).unwrap() + 0.9999;
    let rolled = unix_secs_to_civil(base).unwrap();
    assert_eq!(
        (
            rolled.year,
            rolled.month,
            rolled.day,
            rolled.hour,
            rolled.millis
        ),
        (2025, 1, 1, 0, 0)
    );

    let t0 = civil_to_unix_secs(2026, 3, 14, 23, 30, 0.0).unwrap();
    for (ax, span) in [
        (Axis::new().datetime(), 5.0),    // seconds
        (Axis::new().datetime(), 7200.0), // hours crossing midnight
        (Axis::new().datetime_with_offset(8 * 3600), 14.0 * 86_400.0), // days with UTC+8
        (Axis::new().datetime(), 180.0 * 86_400.0), // months
        (Axis::new().datetime(), 5.0 * 365.0 * 86_400.0), // years
        (Axis::new().duration(), 0.025),  // sub-second duration
        (Axis::new().duration(), 5400.0), // HH:MM:SS duration
        (Axis::new().duration(), 10.0 * 86_400.0), // multi-day duration
    ] {
        let ticks = ax.ticks([t0, t0 + span], 800.0);
        assert!(ticks.len() >= 2, "span {span}: {ticks:?}");
        for pair in ticks.windows(2) {
            assert!(pair[0].value < pair[1].value, "unordered ticks for {span}");
            assert_ne!(pair[0].label, pair[1].label, "duplicate label for {span}");
        }
    }
    assert!(format_duration_hms(-3661.0, 1.0).starts_with('-'));

    let ctx = Context::default();
    let data = PlotData::new(vec![[t0, 1.0], [t0 + 60.0, 2.5], [t0 + 180.0, 1.8]]);
    let mut state = PlotState::default();
    ctx.run_ui(input_events(vec![]), |ui| {
        let out = Plot::new("dt-plot").x_axis(Axis::new().datetime()).show(
            ui,
            &mut state,
            &[Line::new(&data)],
        );
        assert!(!out.axis_ticks[0].is_empty());
        assert!(out.axis_ticks[0][0].label.contains(':'));
    })
    .drop_without_applying_deltas();
}

#[test]
fn all_eleven_colormaps_preserve_legacy_rgb_and_produce_valid_ramps() {
    assert_eq!(ColorMap::Viridis.sample(0.0), Color32::from_rgb(68, 1, 84));
    assert_eq!(
        ColorMap::Viridis.sample(1.0),
        Color32::from_rgb(253, 231, 37)
    );
    assert_eq!(
        ColorMap::Viridis.sample(f64::NAN),
        ColorMap::Viridis.sample(0.0)
    );
    assert_eq!(ColorMap::Gray.sample(0.5), Color32::from_rgb(128, 128, 128));
    assert_eq!(
        ColorMap::BlueRed.sample(0.5),
        Color32::from_rgb(247, 247, 247)
    );

    assert_eq!(ColorMap::ALL.len(), 11);
    for map in ColorMap::ALL {
        let a = map.sample(0.0);
        let b = map.sample(0.5);
        let c = map.sample(1.0);
        assert!(a != c || a != b, "degenerate colormap {map:?}");
        assert_eq!(a.a(), 255);
        assert_eq!(c.a(), 255);
    }
}

#[test]
fn all_fifteen_marker_shapes_render_exact_and_lod_without_hollow_fill() {
    let ctx = Context::default();
    let pts: Vec<[f64; 2]> = (0..64)
        .map(|i| [i as f64, (i as f64 * 0.2).sin()])
        .collect();
    let data = PlotData::new(pts);

    for sampling in [MarkerSampling::Exact, MarkerSampling::PixelGrid] {
        for shape in MarkerShape::ALL {
            let mut state = PlotState::default();
            ctx.run_ui(input_events(vec![]), |ui| {
                let out = Plot::new(("shape", shape as u8, sampling as u8)).show_series(
                    ui,
                    &mut state,
                    &[Scatter::new(&data)
                        .shape(shape)
                        .radius(7.0)
                        .sampling(sampling)
                        .color(Color32::LIGHT_BLUE)
                        .into()],
                );
                assert!(out.diagnostics.output_markers > 0, "{shape:?} {sampling:?}");
            })
            .drop_without_applying_deltas();
        }
    }
    assert!(MarkerShape::Circle.to_hollow().is_hollow());
    assert!(!MarkerShape::CircleHollow.to_filled().is_hollow());
    assert_eq!(
        MarkerStyle::hollow(MarkerShape::Diamond, 5.0).shape,
        MarkerShape::DiamondHollow
    );
}

#[test]
fn box_zoom_and_brush_are_multipass_idempotent_sync_right_y_and_clear_on_escape() {
    let ctx = Context::default();
    let pts: Vec<[f64; 2]> = (0..100).map(|i| [i as f64, i as f64 * 2.0]).collect();
    let data_idx = PlotData::new(pts.clone());
    let data_unidx = PlotData::new_unindexed(pts);
    let right_data = PlotData::new(vec![[0.0, 100.0], [99.0, 500.0]]);

    // Single-pass vs 2-pass request_discard Secondary box zoom must produce identical bounds + right Y.
    let mut final_bounds = Vec::new();
    let mut final_right = Vec::new();
    for multipass in [false, true] {
        let mut state = PlotState::default();
        let run_step = |state: &mut PlotState, events: Vec<Event>, do_discard: bool| {
            let mut resp = None;
            let mut pass = 0;
            let out = ctx.run_ui(input_events(events), |ui| {
                let r = Plot::new("bz")
                    .allow_box_zoom(true)
                    .allow_brush(true)
                    .right_y_axis(Axis::new().label("R"))
                    .show_layers(
                        ui,
                        state,
                        &[
                            PlotLayer::from(Line::new(&data_idx)),
                            PlotLayer::from(Line::new(&right_data).y_axis(YAxis::Right)),
                        ],
                    );
                resp = Some(r);
                if do_discard && pass == 0 {
                    ui.ctx().request_discard("box zoom multipass");
                }
                pass += 1;
            });
            out.drop_without_applying_deltas();
            resp.unwrap()
        };

        let rect = run_step(&mut state, vec![], false).transform.rect();
        let p0 = Pos2::new(
            rect.left() + rect.width() * 0.25,
            rect.top() + rect.height() * 0.25,
        );
        let p1 = Pos2::new(
            rect.left() + rect.width() * 0.75,
            rect.top() + rect.height() * 0.75,
        );
        run_step(
            &mut state,
            vec![
                Event::PointerMoved(p0),
                Event::PointerButton {
                    pos: p0,
                    button: PointerButton::Secondary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
            multipass,
        );
        run_step(&mut state, vec![Event::PointerMoved(p1)], multipass);
        let zoomed = run_step(
            &mut state,
            vec![
                Event::PointerMoved(p1),
                Event::PointerButton {
                    pos: p1,
                    button: PointerButton::Secondary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
            multipass,
        );
        assert!(zoomed.view_changed);
        let b = state.bounds().unwrap();
        assert!(b.min()[0] > 15.0 && b.max()[0] < 85.0, "zoomed X: {b:?}");
        let rr = state.right_y_range().unwrap();
        assert!(rr[0] > 150.0 && rr[1] < 450.0, "synced right Y: {rr:?}");
        final_bounds.push(b);
        final_right.push(rr);

        // Alt + Primary drag for brushing, then Escape clears.
        let b0 = Pos2::new(
            rect.left() + rect.width() * 0.2,
            rect.top() + rect.height() * 0.2,
        );
        let b1 = Pos2::new(
            rect.left() + rect.width() * 0.8,
            rect.top() + rect.height() * 0.8,
        );
        run_step(
            &mut state,
            vec![
                Event::PointerMoved(b0),
                Event::PointerButton {
                    pos: b0,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::ALT,
                },
            ],
            false,
        );
        let finished = run_step(
            &mut state,
            vec![
                Event::PointerMoved(b1),
                Event::PointerButton {
                    pos: b1,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::ALT,
                },
            ],
            multipass,
        );
        assert!(finished.brush_finished);
        let brushed = finished.brushed_bounds.unwrap();
        let idx_hit = data_idx.brushed_indices(brushed);
        let unidx_hit = data_unidx.brushed_indices(brushed);
        assert_eq!(idx_hit, unidx_hit);
        assert!(!idx_hit.is_empty() && idx_hit.len() < 100);

        // Escape clears brushed bounds even under 2-pass layout.
        let cleared = run_step(
            &mut state,
            vec![Event::Key {
                key: Key::Escape,
                physical_key: Some(Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            multipass,
        );
        assert!(cleared.brushed_bounds.is_none());
        assert!(state.brushed_bounds().is_none());
    }
    assert_eq!(final_bounds[0], final_bounds[1]);
    assert_eq!(final_right[0], final_right[1]);
}

#[test]
fn horizontal_box_violin_raincloud_support_bounds_log_render_and_exact_picking() {
    let g1: Vec<f64> = (0..40).map(|i| 10.0 + (i as f64) * 2.5).collect();
    let g2: Vec<f64> = (0..40).map(|i| 50.0 + (i as f64) * 3.0).collect();
    let mut g_with_out = g1.clone();
    g_with_out.push(300.0); // explicit far outlier at X = 300.0, Y = 1.0
    let groups: [&[f64]; 2] = [&g_with_out, &g2];
    let pos = [1.0, 2.0];

    let box_data = BoxChartData::from_groups(&pos, &groups, 0.5).unwrap();
    let violin_data = ViolinData::from_groups(&pos, &groups, ViolinOptions::default()).unwrap();
    let rain_data = RaincloudData::from_groups(&pos, &groups, ViolinOptions::default()).unwrap();

    let h_box = box_data
        .bounds_oriented(
            [AxisScale::Log10, AxisScale::Linear],
            BarOrientation::Horizontal,
        )
        .unwrap();
    assert!(h_box.min()[0] > 0.0 && h_box.max()[0] >= 300.0);
    assert!(h_box.min()[1] < 1.0 && h_box.max()[1] > 2.0);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let out = ctx.run_ui(input_events(vec![]), |ui| {
        let _r = Plot::new("h-dist").show_layers(
            ui,
            &mut state,
            &[
                PlotLayer::from(BoxChart::new(&box_data).orientation(BarOrientation::Horizontal)),
                PlotLayer::from(
                    ViolinPlot::new(&violin_data).orientation(BarOrientation::Horizontal),
                ),
                PlotLayer::from(Raincloud::new(&rain_data).orientation(BarOrientation::Horizontal)),
            ],
        );
    });
    assert!(!out.shapes.is_empty());
    out.drop_without_applying_deltas();

    // Capture transform of BoxChart alone, then hover over box 2 median and outlier [300.0, 1.0].
    let mut box_tr = None;
    let mut box_state = PlotState::default();
    ctx.run_ui(input_events(vec![]), |ui| {
        let r = Plot::new("h-dist-box").show_layers(
            ui,
            &mut box_state,
            &[PlotLayer::from(
                BoxChart::new(&box_data).orientation(BarOrientation::Horizontal),
            )],
        );
        box_tr = Some(r.transform);
    })
    .drop_without_applying_deltas();

    let tr = box_tr.unwrap();
    let med2 = box_data.boxes()[1].median;
    let med_pos = tr.position_from_point([med2, 2.0]);
    let mut hovered_region = None;
    ctx.run_ui(input_events(vec![Event::PointerMoved(med_pos)]), |ui| {
        let r = Plot::new("h-dist-box").show_layers(
            ui,
            &mut box_state,
            &[PlotLayer::from(
                BoxChart::new(&box_data).orientation(BarOrientation::Horizontal),
            )],
        );
        hovered_region = r.hovered_region;
    })
    .drop_without_applying_deltas();
    assert!(
        hovered_region.is_some(),
        "expected horizontal box region hit"
    );

    let out_pos = tr.position_from_point([300.0, 1.0]);
    let mut hovered_point = None;
    ctx.run_ui(input_events(vec![Event::PointerMoved(out_pos)]), |ui| {
        let r = Plot::new("h-dist-box").show_layers(
            ui,
            &mut box_state,
            &[PlotLayer::from(
                BoxChart::new(&box_data).orientation(BarOrientation::Horizontal),
            )],
        );
        hovered_point = r.hovered_point;
    })
    .drop_without_applying_deltas();
    let hp = hovered_point.expect("expected horizontal outlier pick");
    assert!((hp.point[0] - 300.0).abs() < 1e-6);
    assert!((hp.point[1] - 1.0).abs() < 1e-6);
}
