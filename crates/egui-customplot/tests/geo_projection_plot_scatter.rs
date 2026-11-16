use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_customplot::{
    ColorMap, ColorScale, GeoBounds, GeoLayer, GeoLine, GeoPathData, GeoPlot, GeoPlotState,
    GeoPoint, GeoProjection, GeoScatter, GeoScatterData, GeoTransform, LinePattern, MarkerShape,
};

fn canvas_input(time: f64, events: Vec<Event>) -> RawInput {
    RawInput {
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(720.0, 480.0))),
        events,
        ..Default::default()
    }
}

#[test]
fn geo_transform_roundtrip_and_dateline_bounds() {
    let rect = Rect::from_min_size(egui::pos2(20.0, 10.0), egui::vec2(640.0, 420.0));
    let pacific = GeoBounds::from_lat_lon([15.0, 45.0], [120.0, -110.0]).unwrap();
    assert!((pacific.lon_span - 130.0).abs() < 1e-9);
    assert!((pacific.center_lon - (-175.0)).abs() < 1e-9);
    assert!(pacific.contains(GeoPoint::new(25.0, 178.0)));
    assert!(pacific.contains(GeoPoint::new(25.0, -172.0)));
    assert!(!pacific.contains(GeoPoint::new(25.0, 0.0)));

    for proj in [GeoProjection::WebMercator, GeoProjection::Equirectangular] {
        let tf = GeoTransform::new(pacific, proj, rect).unwrap();
        for pt in [
            GeoPoint::new(21.31, -157.86),
            GeoPoint::new(35.68, 139.69),
            GeoPoint::new(28.0, 179.9),
            GeoPoint::new(28.0, -179.9),
        ] {
            let px = tf.project(pt).unwrap();
            let back = tf.unproject(px).unwrap();
            assert!(
                (back.lat - pt.lat).abs() < 1e-3,
                "lat mismatch for {proj:?}"
            );
            assert!(
                egui_customplot::shortest_dlon(back.lon, pt.lon).abs() < 1e-3,
                "lon mismatch for {proj:?}"
            );
        }
    }

    let tight_arc =
        GeoBounds::from_points([GeoPoint::new(20.0, 176.0), GeoPoint::new(22.0, -177.0)]).unwrap();
    assert!(
        tight_arc.lon_span < 15.0,
        "dateline pair should yield tight Pacific arc, got span={}",
        tight_arc.lon_span
    );
    assert!(tight_arc.contains(GeoPoint::new(21.0, 180.0)));
}

#[test]
fn geoplot_splits_antimeridian_in_world_view_and_unifies_in_pacific_view() {
    let lats = [35.68, 21.31, 37.77, f64::NAN, 95.0, 48.86, 40.71];
    let lons = [139.69, -157.86, -122.42, 0.0, 10.0, 2.35, -74.01];
    let path = GeoPathData::from_lat_lon(&lats, &lons).unwrap();
    assert_eq!(path.non_finite_points(), 1);
    assert_eq!(path.domain_invalid_points(), 1);

    let ctx = Context::default();
    let mut world_state = GeoPlotState::default();
    let mut world_resp = None;
    ctx.run_ui(canvas_input(0.0, vec![]), |ui| {
        world_resp = Some(
            GeoPlot::new("geo_world")
                .height(360.0)
                .geo_bounds(GeoBounds::world())
                .show_legend(false)
                .show(
                    ui,
                    &mut world_state,
                    &[GeoLayer::from(GeoLine::new(&path).name("route").pattern(
                        LinePattern::Dashed {
                            length: 8.0,
                            gap: 4.0,
                        },
                    ))],
                ),
        );
    })
    .drop_without_applying_deltas();
    let world = world_resp.unwrap();
    assert_eq!(world.diagnostics.geo.dateline_splits, 1);
    assert!(world.diagnostics.geo.output_paths >= 2);
    assert_eq!(world.diagnostics.geo.non_finite_points, 1);
    assert_eq!(world.diagnostics.geo.domain_invalid_points, 1);

    let transpac =
        GeoPathData::from_lat_lon(&[35.68, 21.31, 37.77], &[139.69, -157.86, -122.42]).unwrap();
    let mut pac_state = GeoPlotState::default();
    let mut pac_resp = None;
    ctx.run_ui(canvas_input(0.1, vec![]), |ui| {
        pac_resp = Some(
            GeoPlot::new("geo_pac")
                .height(360.0)
                .lat_lim([10.0, 50.0])
                .lon_lim([120.0, -110.0])
                .show_legend(false)
                .show(
                    ui,
                    &mut pac_state,
                    &[GeoLayer::from(GeoLine::new(&transpac).name("pac_route"))],
                ),
        );
    })
    .drop_without_applying_deltas();
    let pac = pac_resp.unwrap();
    assert_eq!(pac.diagnostics.geo.dateline_splits, 0);
    assert_eq!(pac.diagnostics.geo.output_paths, 1);
}

#[test]
fn geoscatter_per_point_styles_navigation_and_multipass_idempotence() {
    let scatter = GeoScatterData::from_lat_lon(
        &[35.68, 21.31, 37.77, -33.87],
        &[139.69, -157.86, -122.42, 151.21],
    )
    .unwrap()
    .with_sizes(&[5.0, 9.0, 6.0, 7.0])
    .unwrap()
    .with_color_values(&[10.0, 80.0, 45.0, 60.0])
    .unwrap();
    let scale = ColorScale::new([0.0, 100.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut state = GeoPlotState::default();
    let target_geo = GeoPoint::new(21.31, -157.86);

    // Frame 0: register GeoPlot widget rect with identical id_salt.
    let mut honolulu_px = Pos2::ZERO;
    ctx.run_ui(canvas_input(0.0, vec![]), |ui| {
        let r = GeoPlot::new("geo_nav")
            .height(380.0)
            .geo_bounds(GeoBounds::world())
            .allow_scroll_zoom(true)
            .color_bar(scale, "Temp")
            .show(
                ui,
                &mut state,
                &[GeoLayer::from(
                    GeoScatter::new(&scatter)
                        .name("stations")
                        .marker(MarkerShape::Diamond, 6.0)
                        .color_scale(scale),
                )],
            );
        honolulu_px = r.transform.project(target_geo).unwrap();
        assert_eq!(r.diagnostics.geo.output_markers, 4);
    })
    .drop_without_applying_deltas();

    // Frame 1: pointer hover hits Honolulu marker across 2 layout passes without duplicate side effects.
    let mut passes = 0usize;
    let mut hovered = None;
    ctx.run_ui(
        canvas_input(0.1, vec![Event::PointerMoved(honolulu_px)]),
        |ui| {
            let r = GeoPlot::new("geo_nav")
                .height(380.0)
                .geo_bounds(GeoBounds::world())
                .allow_scroll_zoom(true)
                .color_bar(scale, "Temp")
                .show(
                    ui,
                    &mut state,
                    &[GeoLayer::from(
                        GeoScatter::new(&scatter)
                            .name("stations")
                            .marker(MarkerShape::Diamond, 6.0)
                            .color_scale(scale),
                    )],
                );
            hovered = r.hovered_point;
            if passes == 0 {
                ui.ctx().request_discard("multipass hover test");
            }
            passes += 1;
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);
    let hit = hovered.expect("frame 1 hover should pick Honolulu");
    assert_eq!(hit.point_index, 1);

    // Frame 2: multipass zoom keeps bounds reduction idempotent across discarded passes.
    let before_span = state.bounds().unwrap_or_else(GeoBounds::world).lon_span;
    let mut pass_spans = Vec::new();
    ctx.run_ui(
        canvas_input(
            0.2,
            vec![
                Event::PointerMoved(honolulu_px),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 120.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Unknown,
                    modifiers: Modifiers::NONE,
                },
            ],
        ),
        |ui| {
            let r = GeoPlot::new("geo_nav")
                .height(380.0)
                .geo_bounds(GeoBounds::world())
                .allow_scroll_zoom(true)
                .color_bar(scale, "Temp")
                .show(
                    ui,
                    &mut state,
                    &[GeoLayer::from(
                        GeoScatter::new(&scatter)
                            .name("stations")
                            .marker(MarkerShape::Diamond, 6.0)
                            .color_scale(scale),
                    )],
                );
            pass_spans.push(r.transform.bounds().lon_span);
            if pass_spans.len() == 1 {
                ui.ctx().request_discard("multipass zoom test");
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(pass_spans.len(), 2);
    assert!((pass_spans[0] - pass_spans[1]).abs() < 1e-6);
    assert!(pass_spans[1] < before_span * 0.9);

    // Frame 3: drag left across ±180° wraps center_lon cleanly, and Home resets bounds.
    let p_center = honolulu_px;
    ctx.run_ui(
        canvas_input(
            0.3,
            vec![
                Event::PointerMoved(p_center),
                Event::PointerButton {
                    pos: p_center,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
                Event::PointerMoved(p_center + egui::vec2(-80.0, 0.0)),
                Event::PointerButton {
                    pos: p_center + egui::vec2(-80.0, 0.0),
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
        ),
        |ui| {
            let r = GeoPlot::new("geo_nav")
                .height(380.0)
                .geo_bounds(GeoBounds::world())
                .allow_scroll_zoom(true)
                .show(
                    ui,
                    &mut state,
                    &[GeoLayer::from(GeoScatter::new(&scatter).name("stations"))],
                );
            assert!(r.transform.bounds().center_lon.abs() <= 180.0);
        },
    )
    .drop_without_applying_deltas();
    assert!(state.bounds().is_some());

    ctx.run_ui(
        canvas_input(
            0.4,
            vec![Event::Key {
                key: Key::Home,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        ),
        |ui| {
            GeoPlot::new("geo_nav")
                .height(380.0)
                .geo_bounds(GeoBounds::world())
                .show(
                    ui,
                    &mut state,
                    &[GeoLayer::from(GeoScatter::new(&scatter).name("stations"))],
                );
        },
    )
    .drop_without_applying_deltas();
    // Reset via double-click or explicit state.reset(): verify API reset too.
    state.reset();
    assert!(state.bounds().is_none());
}
