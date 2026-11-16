use std::f64::consts::TAU;

use egui::{Color32, Context, Event, IdSalt, Key, Modifiers, PointerButton, Vec2};
use egui_customplot::{
    AnimatedLine3D, Axis3D, AxisScale, Bounds3D, BoxStyle3D, Camera3D, Comet3D, Comet3DPlayback,
    FunctionPlotError, FunctionPlotOptions, Layer3D, Line3D, MarkerShape, MarkerStyle,
    ParametricCurve3DData, Plot3D, Plot3DError, Plot3DResponse, Plot3DState, Point3DData,
    Projection3D, StreamError, Text3D,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

fn run_plot3d(
    ctx: &Context,
    events: Vec<Event>,
    multipass: bool,
    mut show: impl FnMut(&mut egui::Ui) -> Plot3DResponse,
) -> Plot3DResponse {
    let mut out = None;
    let mut passes = 0usize;
    ctx.run_ui(support::input(events), |ui| {
        out = Some(show(ui));
        if multipass && passes == 0 {
            ui.ctx().request_discard("test plot3d multipass");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    if multipass {
        assert_eq!(passes, 2);
    }
    out.unwrap()
}

#[test]
fn plot3_validation_nan_breaks_log_reversed_axes_primitive_and_text3d() {
    assert_eq!(
        Point3DData::from_xyz(&[], &[], &[]).unwrap_err(),
        Plot3DError::EmptyInput
    );
    assert_eq!(
        Point3DData::from_xyz(&[1.0, 2.0], &[1.0], &[1.0, 2.0]).unwrap_err(),
        Plot3DError::LengthMismatch
    );

    // Includes NaN break + a non-positive z (domain-invalid under Log10 Z axis).
    let pts = Point3DData::from_xyz(
        &[0.0, 1.0, f64::NAN, 2.0, 3.0, 4.0],
        &[0.0, 0.5, f64::NAN, 1.0, 1.5, 2.0],
        &[1.0, 10.0, f64::NAN, -5.0, 100.0, 1000.0],
    )
    .unwrap();
    let prim = Point3DData::from_xyz(&[0.5, 3.5], &[0.5, 1.8], &[2.0, 200.0]).unwrap();
    assert_eq!(pts.non_finite_points(), 1);

    let ctx = Context::default();
    for box_style in [
        BoxStyle3D::BackPlanes,
        BoxStyle3D::FullBox,
        BoxStyle3D::AxesOnly,
    ] {
        let mut state = Plot3DState::default();
        let out = run_plot3d(&ctx, vec![], true, |ui| {
            Plot3D::new(("plot3_log", box_style as u8))
                .height(280.0)
                .box_style(box_style)
                .x_axis(Axis3D::new().reversed(true).label("X rev"))
                .y_label("Y")
                .z_axis(Axis3D::new().scale(AxisScale::Log10).label("Z log10"))
                .texts([Text3D::new([3.0, 1.5, 100.0], "Peak").color(Color32::RED)])
                .show(
                    ui,
                    &mut state,
                    &[
                        Layer3D::from(Line3D::new(&pts).name("signal3").width(2.0).marker(
                            MarkerStyle {
                                shape: MarkerShape::Circle,
                                radius: 4.0,
                            },
                        )),
                        Layer3D::from(Line3D::primitive(&prim).color(Color32::GRAY)),
                    ],
                )
        });
        assert_eq!(out.diagnostics.plot3d.non_finite_items, 1);
        assert_eq!(out.diagnostics.plot3d.domain_invalid_items, 1);
        // Only 2 segments exist in valid domain across (0..1) and (3..4) + 1 primitive segment = 3.
        // Crucially, z=-5 at idx 3 must NOT bridge across indices 1->3->4.
        assert_eq!(out.diagnostics.plot3d.output_segments, 3);
        // Primitive has empty name so legend omits it, keeping only "signal3".
        assert_eq!(out.legend_entries.len(), 1);
        assert_eq!(out.legend_entries[0].name, "signal3");

        // Verify hover skips Line3D::primitive even right at its projected vertex!
        let (prim_pos, _) = out.transform.project_data([0.5, 0.5, 2.0]).unwrap();
        let hover_out = run_plot3d(&ctx, vec![Event::PointerMoved(prim_pos)], false, |ui| {
            Plot3D::new(("plot3_log", box_style as u8))
                .height(280.0)
                .x_axis(Axis3D::new().reversed(true))
                .z_axis(Axis3D::new().scale(AxisScale::Log10))
                .show(
                    ui,
                    &mut state,
                    &[Layer3D::from(Line3D::primitive(&prim).color(Color32::GRAY))],
                )
        });
        assert!(hover_out.hovered_point.is_none());
    }
}

#[test]
fn fplot3_and_ezplot3_sample_parametrics_and_report_errors() {
    assert_eq!(
        ParametricCurve3DData::from_parametric([2.0, 1.0], FunctionPlotOptions::default(), |t| {
            [t, t, t]
        })
        .unwrap_err(),
        FunctionPlotError::InvalidRange
    );
    assert_eq!(
        ParametricCurve3DData::from_parametric(
            [0.0, 1.0],
            FunctionPlotOptions {
                initial_samples: 2,
                ..Default::default()
            },
            |t| [t, t, t]
        )
        .unwrap_err(),
        FunctionPlotError::InvalidOptions
    );
    assert!(matches!(
        ParametricCurve3DData::from_parametric(
            [0.0, 1.0],
            FunctionPlotOptions {
                initial_samples: 32,
                max_evaluations: 8,
                ..Default::default()
            },
            |t| [t, t, t]
        )
        .unwrap_err(),
        FunctionPlotError::BudgetExceeded { .. }
    ));

    let helix = ParametricCurve3DData::from_parametric(
        [0.0, 2.0 * TAU],
        FunctionPlotOptions::default(),
        |t| [t.cos(), t.sin(), t],
    )
    .unwrap();
    assert!(helix.points().len() >= 65);
    assert!(helix.evaluations() >= helix.points().len());

    let singular =
        ParametricCurve3DData::from_parametric([0.0, 2.0], FunctionPlotOptions::default(), |t| {
            [t, 1.0 / (t - 1.0), (t * 2.0).sin()]
        })
        .unwrap();
    assert!(singular.discontinuities() >= 1);

    let ez = ParametricCurve3DData::ezplot3(1200, |t| [t.cos(), (2.0 * t).sin(), t / TAU]).unwrap();
    assert!(ez.points().len() >= 65);
}

#[test]
fn comet3_and_animated_line3d_stay_idempotent_across_multipass_frames() {
    assert!(matches!(
        AnimatedLine3D::new(0),
        Err(StreamError::ZeroCapacity)
    ));
    assert!(matches!(
        AnimatedLine3D::new(5_000_000),
        Err(StreamError::SizeOverflow)
    ));

    let mut stream = AnimatedLine3D::new(4).unwrap();
    let batch1 = [[0.0, 0.0, 0.0], [1.0, 0.5, 0.2], [2.0, 1.0, 0.4]];
    let r1 = stream.add_points_once(1, &batch1).unwrap();
    assert!(r1.applied && stream.len() == 3);
    // Duplicate batch_id (simulating egui second layout pass) is a no-op.
    let r1_dup = stream.add_points_once(1, &batch1).unwrap();
    assert!(!r1_dup.applied && stream.len() == 3);

    // Exceeding capacity=4 evicts oldest elements while preserving newest.
    let batch2 = [[3.0, 1.5, 0.6], [4.0, 2.0, 0.8], [5.0, 2.5, 1.0]];
    let r2 = stream.add_points_once(2, &batch2).unwrap();
    assert!(r2.applied);
    assert_eq!(stream.len(), 4);
    assert_eq!(stream.data().points()[0], [2.0, 1.0, 0.4]);

    stream.set_paused(true);
    assert!(
        !stream
            .add_points_once(3, &[[6.0, 3.0, 1.2]])
            .unwrap()
            .applied
    );
    stream.set_paused(false);
    stream.clear();
    assert!(stream.is_empty());

    // Verify Comet3D + Comet3DPlayback::advance_once across multipass UI.
    let traj: Vec<[f64; 3]> = (0..40)
        .map(|i| {
            let t = i as f64 * 0.2;
            [t.cos(), t.sin(), t]
        })
        .collect();
    let data = Point3DData::new(traj);
    let ctx = Context::default();
    let mut pb_single = Comet3DPlayback::new(100.0).tail_fraction(0.25);
    let mut pb_multi = pb_single.clone();
    let mut st_single = Plot3DState::default();
    let mut st_multi = Plot3DState::default();

    for frame in 1..=3u64 {
        let mut passes = 0;
        ctx.run_ui(support::input(vec![]), |ui| {
            pb_multi.advance_once(frame, 0.05, data.len());
            let _ = Plot3D::new("comet_multi").show(
                ui,
                &mut st_multi,
                &[Layer3D::from(Comet3D::new(&data, &pb_multi).name("comet"))],
            );
            if passes == 0 {
                ui.ctx().request_discard("comet multipass");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);

        pb_single.advance_once(frame, 0.05, data.len());
        let _ = run_plot3d(&ctx, vec![], false, |ui| {
            Plot3D::new("comet_single").show(
                ui,
                &mut st_single,
                &[Layer3D::from(Comet3D::new(&data, &pb_single).name("comet"))],
            )
        });
    }
    assert!((pb_multi.progress_points() - pb_single.progress_points()).abs() < 1e-12);
    assert!(pb_multi.progress_points() > 10.0);
}

#[test]
fn perspective_projection_and_multipass_navigation_and_legend_work_identically() {
    let pts =
        Point3DData::from_xyz(&[-0.5, 0.5, -0.5, 0.5], &[0.0; 4], &[-0.8, -0.8, 0.8, 0.8]).unwrap();
    let ctx = Context::default();
    let mut st_ortho = Plot3DState::default();
    let mut st_persp = Plot3DState::default();
    let res_ortho = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("ortho")
            .camera(Camera3D::view_2d_xy().projection(Projection3D::Orthographic))
            .x_axis(Axis3D::new().bounds(-1.0, 1.0))
            .y_axis(Axis3D::new().bounds(-1.0, 1.0))
            .z_axis(Axis3D::new().bounds(-1.0, 1.0))
            .show(ui, &mut st_ortho, &[Layer3D::from(Line3D::new(&pts))])
    });
    let res_persp = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("persp")
            .camera(Camera3D::view_2d_xy().projection(Projection3D::perspective_default()))
            .x_axis(Axis3D::new().bounds(-1.0, 1.0))
            .y_axis(Axis3D::new().bounds(-1.0, 1.0))
            .z_axis(Axis3D::new().bounds(-1.0, 1.0))
            .show(ui, &mut st_persp, &[Layer3D::from(Line3D::new(&pts))])
    });

    let o_far = res_ortho
        .transform
        .project_data([0.5, 0.0, -0.8])
        .unwrap()
        .0;
    let o_near = res_ortho.transform.project_data([0.5, 0.0, 0.8]).unwrap().0;
    assert!((o_far.x - o_near.x).abs() < 1e-3);

    let p_far = res_persp
        .transform
        .project_data([0.5, 0.0, -0.8])
        .unwrap()
        .0;
    let p_near = res_persp.transform.project_data([0.5, 0.0, 0.8]).unwrap().0;
    let cx = res_persp.plot_area.center().x;
    assert!(
        (p_near.x - cx).abs() > (p_far.x - cx).abs() + 5.0,
        "perspective should enlarge nearer positive-Z coordinates"
    );

    // Verify orbit, pan, scroll zoom, legend toggle, and Home reset match across single vs multipass.
    let mut outcomes = Vec::new();
    for multipass in [false, true] {
        let nav_ctx = Context::default();
        let mut st = Plot3DState::default();
        let draw = |ui: &mut egui::Ui, state: &mut Plot3DState| {
            Plot3D::new("nav3d")
                .height(300.0)
                .allow_scroll_zoom(true)
                .show(
                    ui,
                    state,
                    &[
                        Layer3D::from(Line3D::new(&pts).id_salt("first").name("first")),
                        Layer3D::from(Line3D::new(&pts).id_salt("second").name("second")),
                    ],
                )
        };
        let first = run_plot3d(&nav_ctx, vec![], false, |ui| draw(ui, &mut st));
        let start = first.plot_area.center();
        let end = start + Vec2::new(40.0, -20.0);
        let btn = |pos, button, pressed| Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        };
        // Primary drag orbit (separate frames for press, move, release).
        for events in [
            vec![Event::PointerMoved(start)],
            vec![btn(start, PointerButton::Primary, true)],
            vec![Event::PointerMoved(end)],
            vec![btn(end, PointerButton::Primary, false)],
        ] {
            let _ = run_plot3d(&nav_ctx, events, multipass, |ui| draw(ui, &mut st));
        }
        let cam_orbit = st.camera().expect("orbit updates camera");

        // Secondary drag pan.
        let pan_end = start + Vec2::new(-24.0, 16.0);
        for events in [
            vec![Event::PointerMoved(start)],
            vec![btn(start, PointerButton::Secondary, true)],
            vec![Event::PointerMoved(pan_end)],
            vec![btn(pan_end, PointerButton::Secondary, false)],
        ] {
            let _ = run_plot3d(&nav_ctx, events, multipass, |ui| draw(ui, &mut st));
        }
        let cam_pan = st.camera().expect("pan updates camera");
        assert!(cam_pan.pan != [0.0, 0.0]);

        // Scroll zoom inside plot area.
        let _ = run_plot3d(&nav_ctx, vec![Event::PointerMoved(start)], false, |ui| {
            draw(ui, &mut st)
        });
        let _ = run_plot3d(
            &nav_ctx,
            vec![
                Event::PointerMoved(start),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 50.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Unknown,
                    modifiers: Modifiers::NONE,
                },
            ],
            multipass,
            |ui| draw(ui, &mut st),
        );
        let cam_zoom = st.camera().expect("scroll wheel zooms camera");
        assert!(cam_zoom.zoom > 1.0);

        // Legend click toggles exactly once across multipass.
        let legend_pos = first.legend_entries[0].rect.center();
        for events in [
            vec![Event::PointerMoved(legend_pos)],
            vec![btn(legend_pos, PointerButton::Primary, true)],
            vec![btn(legend_pos, PointerButton::Primary, false)],
        ] {
            let _ = run_plot3d(&nav_ctx, events, multipass, |ui| draw(ui, &mut st));
        }
        assert!(!st.layer_visible(IdSalt::new("first")));
        assert!(st.layer_visible(IdSalt::new("second")));

        outcomes.push((
            (cam_orbit.azimuth_deg * 1e4).round() as i64,
            (cam_orbit.elevation_deg * 1e4).round() as i64,
            (cam_pan.pan[0] * 1e4).round() as i64,
            (cam_zoom.zoom * 1e4).round() as i64,
            st.layer_visible(IdSalt::new("first")),
        ));

        // Click inside plot area to focus, then Home key resets camera & bounds.
        for events in [
            vec![Event::PointerMoved(start)],
            vec![btn(start, PointerButton::Primary, true)],
            vec![btn(start, PointerButton::Primary, false)],
        ] {
            let _ = run_plot3d(&nav_ctx, events, false, |ui| draw(ui, &mut st));
        }
        let _ = run_plot3d(
            &nav_ctx,
            vec![Event::Key {
                key: Key::Home,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            multipass,
            |ui| draw(ui, &mut st),
        );
        assert!(st.camera().is_none());
        let _ = Bounds3D::default();
    }
    assert_eq!(outcomes[0], outcomes[1]);
}
