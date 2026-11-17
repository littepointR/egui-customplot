mod support;

use egui::{
    Color32, Context, Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, Pos2,
    TouchPhase, Vec2, epaint::Shape, vec2,
};
use egui_customplot::{
    BodePlot, BodePlotState, Bounds3D, Camera3D, ClusterStandardize, Clustergram, ClustergramData,
    ClustergramState, ColorMap, ContourData, ContourLevels, Contours, DepthMode,
    FrequencyResponseData, Layer3D, Line, Line3D, LinkageMethod, Plot, Plot3D, Plot3DState,
    PlotBounds, PlotData, PlotDefaults, PlotLayer, PlotState, PlotTheme, Point3DData, PointCloud3D,
    PointCloud3DData, Projection3D, ScalarField2D, ScalarVolume3D, StackedPlot, Surface3D,
    Surface3DData, TransferFunction, VolumeRender3D, VolumeRenderMode, VolumeTransferFunction,
    plot_defaults, set_plot_defaults,
};

fn drag_horizontal(ctx: &Context, center: Pos2, dx: f32, mut draw: impl FnMut(&mut egui::Ui)) {
    let mut step = |events: Vec<Event>| {
        ctx.run_ui(support::input(events), |ui| draw(ui))
            .drop_without_applying_deltas();
    };
    step(vec![
        Event::PointerMoved(center),
        Event::PointerButton {
            pos: center,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    ]);
    let moved = center + Vec2::new(dx, 0.0);
    step(vec![Event::PointerMoved(moved)]);
    step(vec![
        Event::PointerMoved(moved),
        Event::PointerButton {
            pos: moved,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ]);
}

fn collect_shapes<'a>(shape: &'a Shape, out: &mut Vec<&'a Shape>) {
    match shape {
        Shape::Vec(children) => {
            for c in children {
                collect_shapes(c, out);
            }
        }
        other => out.push(other),
    }
}

fn luma(c: Color32) -> f32 {
    0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
}

#[test]
fn composite_linked_plots_preserve_horizontal_pan_without_snapback_and_reset_on_home() {
    let ctx = Context::default();

    // 1. StackedPlot
    let s1 = PlotData::new(
        (0..60)
            .map(|i| [i as f64 * 0.2, (i as f64 * 0.2).sin()])
            .collect::<Vec<_>>(),
    );
    let s2 = PlotData::new(
        (0..60)
            .map(|i| [i as f64 * 0.2, (i as f64 * 0.2).cos()])
            .collect::<Vec<_>>(),
    );
    let mut stacked_states = vec![PlotState::default(), PlotState::default()];
    let mut top_rect = egui::Rect::NOTHING;
    let render_stacked =
        |states: &mut Vec<PlotState>, rect: &mut egui::Rect, events: Vec<Event>| {
            let mut x_bounds = [0.0, 0.0];
            ctx.run_ui(support::input(events), |ui| {
                let r = StackedPlot::new(
                    "ux_stacked",
                    vec![
                        ("S1", vec![PlotLayer::from(Line::new(&s1))]),
                        ("S2", vec![PlotLayer::from(Line::new(&s2))]),
                    ],
                )
                .row_height(150.0)
                .allow_scroll_zoom(true)
                .show(ui, states)
                .expect("stacked");
                *rect = r.figure.inner[0].transform.rect();
                let b = r.figure.inner[0].transform.bounds();
                x_bounds = [b.min()[0], b.max()[0]];
            })
            .drop_without_applying_deltas();
            x_bounds
        };
    let orig_x = render_stacked(&mut stacked_states, &mut top_rect, vec![]);
    drag_horizontal(&ctx, top_rect.center(), -60.0, |ui| {
        let _ = StackedPlot::new(
            "ux_stacked",
            vec![
                ("S1", vec![PlotLayer::from(Line::new(&s1))]),
                ("S2", vec![PlotLayer::from(Line::new(&s2))]),
            ],
        )
        .row_height(150.0)
        .show(ui, &mut stacked_states)
        .unwrap();
    });
    let panned_x = render_stacked(&mut stacked_states, &mut top_rect, vec![]);
    assert!((panned_x[0] - orig_x[0]).abs() > 0.1, "stacked X must pan");
    for _ in 0..3 {
        let cur = render_stacked(&mut stacked_states, &mut top_rect, vec![]);
        assert!(
            (cur[0] - panned_x[0]).abs() < 1e-9 && (cur[1] - panned_x[1]).abs() < 1e-9,
            "stacked snapped back on idle: orig={orig_x:?}, panned={panned_x:?}, cur={cur:?}"
        );
    }
    let reset_x = render_stacked(
        &mut stacked_states,
        &mut top_rect,
        vec![Event::Key {
            key: Key::Home,
            physical_key: Some(Key::Home),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert!(
        (reset_x[0] - orig_x[0]).abs() < 1e-6,
        "Home must reset StackedPlot: orig={orig_x:?}, reset={reset_x:?}"
    );

    // 2. BodePlot
    let tf = TransferFunction::new(&[1.6], &[1.0, 1.8, 3.05, 2.25]).unwrap();
    let freq = FrequencyResponseData::from_transfer_fn(&tf, 0.1, 20.0, 120).unwrap();
    let mut bode_state = BodePlotState::default();
    let mut mag_rect = egui::Rect::NOTHING;
    let render_bode = |st: &mut BodePlotState, rect: &mut egui::Rect, events: Vec<Event>| {
        let mut x_bounds = [0.0, 0.0];
        ctx.run_ui(support::input(events), |ui| {
            let r = BodePlot::new("ux_bode", &freq)
                .height(300.0)
                .allow_scroll_zoom(true)
                .show(ui, st);
            *rect = r.mag.transform.rect();
            let b = r.mag.transform.bounds();
            x_bounds = [b.min()[0], b.max()[0]];
        })
        .drop_without_applying_deltas();
        x_bounds
    };
    let bode_orig_x = render_bode(&mut bode_state, &mut mag_rect, vec![]);
    drag_horizontal(&ctx, mag_rect.center(), -50.0, |ui| {
        let _ = BodePlot::new("ux_bode", &freq)
            .height(300.0)
            .show(ui, &mut bode_state);
    });
    let bode_panned_x = render_bode(&mut bode_state, &mut mag_rect, vec![]);
    assert!(
        (bode_panned_x[0] - bode_orig_x[0]).abs() > 1e-3,
        "bode X must pan"
    );
    for _ in 0..3 {
        let cur = render_bode(&mut bode_state, &mut mag_rect, vec![]);
        assert!(
            (cur[0] - bode_panned_x[0]).abs() < 1e-9 && (cur[1] - bode_panned_x[1]).abs() < 1e-9,
            "bode X snapped back on idle"
        );
    }

    // 3. Clustergram
    let cg_rows: [&[f64]; 4] = [
        &[2.0, 1.5, -1.0, -1.2],
        &[1.8, 2.1, -0.9, -1.1],
        &[-1.0, -1.2, 2.2, 1.9],
        &[-1.3, -1.1, 1.9, 2.3],
    ];
    let cg_data =
        ClustergramData::new(&cg_rows, LinkageMethod::Average, ClusterStandardize::None).unwrap();
    let mut cg_state = ClustergramState::default();
    let mut cg_rect = egui::Rect::NOTHING;
    let render_cg = |st: &mut ClustergramState, rect: &mut egui::Rect, events: Vec<Event>| {
        let mut x_bounds = [0.0, 0.0];
        ctx.run_ui(support::input(events), |ui| {
            let r = Clustergram::new("ux_cg", &cg_data)
                .height(300.0)
                .allow_scroll_zoom(true)
                .show(ui, st);
            *rect = r.heatmap_rect;
            let b = r.heatmap.transform.bounds();
            x_bounds = [b.min()[0], b.max()[0]];
        })
        .drop_without_applying_deltas();
        x_bounds
    };
    let cg_orig_x = render_cg(&mut cg_state, &mut cg_rect, vec![]);
    drag_horizontal(&ctx, cg_rect.center(), -45.0, |ui| {
        let _ = Clustergram::new("ux_cg", &cg_data)
            .height(300.0)
            .show(ui, &mut cg_state);
    });
    let cg_panned_x = render_cg(&mut cg_state, &mut cg_rect, vec![]);
    assert!(
        (cg_panned_x[0] - cg_orig_x[0]).abs() > 0.05,
        "clustergram must pan"
    );
    for _ in 0..3 {
        let cur = render_cg(&mut cg_state, &mut cg_rect, vec![]);
        assert!(
            (cur[0] - cg_panned_x[0]).abs() < 1e-9 && (cur[1] - cg_panned_x[1]).abs() < 1e-9,
            "clustergram snapped back on idle"
        );
    }
}

#[test]
fn plot_pan_limits_clamp_smoothly_at_boundary_preserving_span() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0, -2.0], [5.0, 0.0], [10.0, 2.0]]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([2.0, -1.0], [6.0, 1.0]).unwrap());

    let render = |st: &mut PlotState| {
        support::run(&ctx, vec![], false, |ui| {
            Plot::new("pan_limit_plot")
                .height(300.0)
                .pan_limits(Some([0.0, 10.0]), Some([-2.0, 2.0]))
                .show(ui, st, &[Line::new(&data)])
        })
    };
    let r0 = render(&mut state);
    let center = r0.transform.rect().center();

    // Repeated large drag right (pulling view past min X = 0.0) should stop cleanly at 0..=4.
    for _ in 0..5 {
        drag_horizontal(&ctx, center, 500.0, |ui| {
            let _ = Plot::new("pan_limit_plot")
                .height(300.0)
                .pan_limits(Some([0.0, 10.0]), Some([-2.0, 2.0]))
                .show(ui, &mut state, &[Line::new(&data)]);
        });
        let b = state.bounds().unwrap();
        assert!(
            (b.min()[0] - 0.0).abs() < 1e-9,
            "clamped min={}",
            b.min()[0]
        );
        assert!(
            (b.max()[0] - 4.0).abs() < 1e-9,
            "span preserved max={}",
            b.max()[0]
        );
    }
}

#[test]
fn plot3d_projection_switches_immediately_after_camera_orbit() {
    let ctx = Context::default();
    let data = Point3DData::from_xyz(&[0.0, 1.0, 2.0], &[0.0, 1.0, 0.0], &[0.0, 1.0, 2.0]).unwrap();
    let mut state = Plot3DState::default();

    let run = |st: &mut Plot3DState, proj: Projection3D, events: Vec<Event>| {
        let mut resp = None;
        ctx.run_ui(support::input(events), |ui| {
            resp = Some(
                Plot3D::new("ux_p3d_proj")
                    .height(320.0)
                    .projection(proj)
                    .show(
                        ui,
                        st,
                        &[Layer3D::from(egui_customplot::Line3D::new(&data))],
                    ),
            );
        })
        .drop_without_applying_deltas();
        resp.unwrap()
    };

    let init = run(&mut state, Projection3D::Orthographic, vec![]);
    let c = init.plot_area.center();
    let init_az = init.transform.camera().azimuth_deg;

    // Orbit the 3D camera so `state.camera()` is Some(...) override.
    drag_horizontal(&ctx, c, 45.0, |ui| {
        let _ = Plot3D::new("ux_p3d_proj")
            .height(320.0)
            .projection(Projection3D::Orthographic)
            .show(
                ui,
                &mut state,
                &[Layer3D::from(egui_customplot::Line3D::new(&data))],
            );
    });
    assert!(state.camera().is_some());
    let orbited_az = state.camera().unwrap().azimuth_deg;
    assert!((orbited_az - init_az).abs() > 1.0);

    // Toggle to Perspective while keeping the orbited azimuth!
    let r_persp = run(&mut state, Projection3D::perspective_default(), vec![]);
    assert!(matches!(
        r_persp.transform.camera().projection,
        Projection3D::Perspective { .. }
    ));
    assert!((r_persp.transform.camera().azimuth_deg - orbited_az).abs() < 1e-9);

    // Toggle back to Orthographic immediately without resetting azimuth.
    let r_ortho = run(&mut state, Projection3D::Orthographic, vec![]);
    assert_eq!(
        r_ortho.transform.camera().projection,
        Projection3D::Orthographic
    );
    assert!((r_ortho.transform.camera().azimuth_deg - orbited_az).abs() < 1e-9);
}

#[test]
fn contour_labels_have_high_luminance_contrast_in_light_and_dark_themes() {
    let bounds = PlotBounds::new([-2.0, -2.0], [2.0, 2.0]).unwrap();
    let field =
        ScalarField2D::sample_function(bounds, [24, 24], 4_000, |x, y| x * x + y * y).unwrap();
    let cdata = ContourData::from_field(&field, ContourLevels::Count(5)).unwrap();

    for (theme, stroke_col) in [
        (PlotTheme::scientific_light(), Color32::WHITE),
        (PlotTheme::monitoring_dark(), Color32::from_gray(25)),
    ] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let (_, output) = support::capture(&ctx, vec![], false, |ui| {
            Plot::new("ux_contour_contrast")
                .height(320.0)
                .theme(theme.clone())
                .show_layers(
                    ui,
                    &mut state,
                    &[PlotLayer::from(
                        Contours::new(&cdata).color(stroke_col).show_labels(true),
                    )],
                )
        });
        let checked_labels = {
            let mut flat = Vec::new();
            for clipped in &output.shapes {
                collect_shapes(&clipped.shape, &mut flat);
            }
            let pills: Vec<_> = flat
                .iter()
                .filter_map(|s| match s {
                    Shape::Rect(r)
                        if r.fill != Color32::TRANSPARENT
                            && r.rect.width() < 80.0
                            && r.rect.height() < 30.0 =>
                    {
                        Some(r.clone())
                    }
                    _ => None,
                })
                .collect();
            let mut count = 0usize;
            for s in &flat {
                if let Shape::Text(ts) = s {
                    let tc = ts.galley.rect.center() + ts.pos.to_vec2();
                    if let Some(pill) = pills.iter().find(|r| r.rect.contains(tc)) {
                        let fg = ts
                            .galley
                            .job
                            .sections
                            .first()
                            .map_or(ts.fallback_color, |sec| sec.format.color);
                        let diff = (luma(fg) - luma(pill.fill)).abs();
                        assert!(
                            diff >= 90.0,
                            "insufficient contour label contrast diff={diff:.1}, fg={fg:?}, bg={:?}",
                            pill.fill
                        );
                        count += 1;
                    }
                }
            }
            count
        };
        output.drop_without_applying_deltas();
        assert!(checked_labels > 0, "expected labeled contour pills");
    }
}

#[test]
fn volshow_zbuffer_caches_idle_frames_and_plot_defaults_propagate_cleanly() {
    let ctx = Context::default();
    let bounds = Bounds3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [18, 18, 18], 10_000, |x, y, z| {
        (-(x * x + y * y + z * z) * 2.0).exp()
    })
    .unwrap();
    let tf = VolumeTransferFunction::ramp(ColorMap::Viridis, 0.0, 0.85);
    let mut state = Plot3DState::default();

    let run_vol = |st: &mut Plot3DState| {
        let mut res = None;
        ctx.run_ui(support::input(vec![]), |ui| {
            res = Some(
                Plot3D::new("ux_vol")
                    .height(260.0)
                    .depth_mode(DepthMode::ZBuffer)
                    .show(
                        ui,
                        st,
                        &[Layer3D::from(
                            VolumeRender3D::new(&vol)
                                .transfer(tf)
                                .mode(VolumeRenderMode::DirectAlpha)
                                .max_resolution(64)
                                .sample_steps(24),
                        )],
                    ),
            );
        })
        .drop_without_applying_deltas();
        res.unwrap()
    };

    let r0 = run_vol(&mut state);
    assert!(r0.diagnostics.plot3d.zbuffer_pixels > 0);
    assert!(r0.diagnostics.plot3d.zbuffer_spans > 0);
    let r1 = run_vol(&mut state);
    assert_eq!(
        r1.diagnostics.plot3d.zbuffer_spans,
        r0.diagnostics.plot3d.zbuffer_spans
    );

    // PlotDefaults sets global overrides and clears cleanly.
    set_plot_defaults(
        &ctx,
        Some(PlotDefaults {
            allow_scroll_zoom: Some(true),
            projection_3d: Some(Projection3D::perspective_default()),
            ..Default::default()
        }),
    );
    assert!(plot_defaults(&ctx).is_some());
    let r_defs = run_vol(&mut state);
    assert!(matches!(
        r_defs.transform.camera().projection,
        Projection3D::Perspective { .. }
    ));

    set_plot_defaults(&ctx, None);
    assert!(plot_defaults(&ctx).is_none());
}

#[test]
fn pointcloud_40k_and_surface_zbuffer_deep_zoom_never_hangs_on_negative_coords() {
    use std::{f64::consts::TAU, time::Instant};

    let n_pts = 40_000usize;
    let mut pts = Vec::with_capacity(n_pts);
    let mut rgb = Vec::with_capacity(n_pts);
    for i in 0..n_pts {
        let t = i as f64 / n_pts as f64;
        let ang = 18.0 * TAU * t;
        let rad = 0.25 + 0.9 * t;
        let z = 1.6 * (t - 0.5) + 0.14 * (7.0 * ang).sin();
        pts.push([rad * ang.cos(), rad * ang.sin(), z]);
        let c = ColorMap::Turbo.sample(t);
        rgb.push([c.r(), c.g(), c.b()]);
    }
    let cloud = PointCloud3DData::from_xyz(pts)
        .unwrap()
        .with_rgb(rgb)
        .unwrap();

    let xs: Vec<f64> = (-6..=6).map(|i| i as f64 * 0.2).collect();
    let ys: Vec<f64> = (-6..=6).map(|i| i as f64 * 0.2).collect();
    let mut zs = Vec::with_capacity(xs.len() * ys.len());
    for &y in &ys {
        for &x in &xs {
            zs.push(-0.65 + 0.12 * (2.0 * x).sin() * (2.0 * y).cos());
        }
    }
    let surf = Surface3DData::from_grid(&xs, &ys, &zs).unwrap();
    let line =
        Point3DData::from_xyz(&[-1.2, 0.0, 1.2], &[-1.2, 0.8, -1.0], &[-0.8, 0.9, 0.8]).unwrap();

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let start = Instant::now();
    let mut plot_area = egui::Rect::NOTHING;

    for proj in [
        Projection3D::Orthographic,
        Projection3D::perspective_default(),
    ] {
        for &zoom in &[0.5_f64, 2.5, 8.0, 20.0] {
            for &pan in &[[-1.5_f64, -1.5], [1.5, 1.5], [0.0, 0.0]] {
                let cam = Camera3D {
                    projection: proj,
                    zoom,
                    pan,
                    ..Default::default()
                };
                state.set_camera(cam);
                let out = ctx.run_ui(support::input(vec![]), |ui| {
                    let r = Plot3D::new("ux_pc40k_deep_zoom")
                        .height(300.0)
                        .depth_mode(DepthMode::ZBuffer)
                        .allow_scroll_zoom(true)
                        .show(
                            ui,
                            &mut state,
                            &[
                                Layer3D::from(PointCloud3D::new(&cloud).max_points(12_000)),
                                Layer3D::from(Surface3D::new(&surf)),
                                Layer3D::from(Line3D::new(&line)),
                            ],
                        );
                    plot_area = r.plot_area;
                });
                assert!(!out.shapes.is_empty());
                out.drop_without_applying_deltas();
            }
        }
    }

    // Drive real MouseWheel zoom-in steps inside plot_area from high zoom where points cross negative x/y.
    let wheel_cam = Camera3D {
        zoom: 2.0,
        ..Default::default()
    };
    state.set_camera(wheel_cam);
    let hover = plot_area.center();
    for _ in 0..8 {
        let prev_zoom = state.camera().unwrap().zoom;
        let events = vec![
            Event::PointerMoved(hover),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: vec2(0.0, 120.0),
                phase: TouchPhase::Move,
                source: MouseWheelSource::Unknown,
                modifiers: Modifiers::NONE,
            },
        ];
        let out = ctx.run_ui(support::input(events), |ui| {
            let _ = Plot3D::new("ux_pc40k_deep_zoom")
                .height(300.0)
                .depth_mode(DepthMode::ZBuffer)
                .allow_scroll_zoom(true)
                .show(
                    ui,
                    &mut state,
                    &[
                        Layer3D::from(PointCloud3D::new(&cloud).max_points(12_000)),
                        Layer3D::from(Surface3D::new(&surf)),
                        Layer3D::from(Line3D::new(&line)),
                    ],
                );
        });
        assert!(!out.shapes.is_empty());
        out.drop_without_applying_deltas();
        assert!(state.camera().unwrap().zoom >= prev_zoom);
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f32() < 3.0,
        "40k point cloud + surface deep zoom took too long: {elapsed:?}"
    );
}
