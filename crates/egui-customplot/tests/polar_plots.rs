use std::f64::consts::{PI, TAU};

use egui::{Color32, Context, Event, PointerButton, Pos2, Vec2};
use egui_customplot::{
    BubbleLegend, ColorMap, ColorScale, CompassData, CompassPlot, FunctionPlotOptions, MarkerShape,
    MarkerStyle, NegativeRadiusMode, PolarBubble, PolarBubbleData, PolarCurveData, PolarLayer,
    PolarLine, PolarPlot, PolarScatter, PolarState, ThetaDirection, ThetaZeroLocation,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

fn run_polar(
    ctx: &Context,
    events: Vec<Event>,
    multipass: bool,
    mut show: impl FnMut(&mut egui::Ui) -> egui_customplot::PolarPlotResponse,
) -> egui_customplot::PolarPlotResponse {
    let mut out = None;
    let mut passes = 0usize;
    ctx.run_ui(support::input(events), |ui| {
        out = Some(show(ui));
        if multipass && passes == 0 {
            ui.ctx().request_discard("test polar multipass");
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
fn polar_line_scatter_bubble_compass_and_functions_render_cleanly() {
    let line_data = PolarCurveData::from_degrees(
        &[0.0, 45.0, 90.0, 135.0, 180.0, 270.0, 360.0],
        &[0.4, 0.8, 1.2, 0.9, 1.5, 1.1, 0.4],
    )
    .unwrap();
    let fpolar = PolarCurveData::from_function([0.0, TAU], FunctionPlotOptions::default(), |t| {
        1.2 + 0.5 * (4.0 * t).cos()
    })
    .unwrap();
    let ezp = PolarCurveData::ezpolar(2000, |t| 0.8 + 0.3 * (3.0 * t).sin()).unwrap();
    assert!(!fpolar.curve().is_empty());
    assert!(!ezp.curve().is_empty());

    let bubble = PolarBubbleData::new(
        &[0.2, 1.4, 2.8, 4.5],
        &[0.5, 1.1, 1.6, 0.9],
        &[10.0, 40.0, 90.0, 25.0],
    )
    .unwrap()
    .with_scalars(&[1.0, 2.0, 3.0, 4.0])
    .unwrap()
    .size_range([4.0, 18.0]);
    let compass =
        CompassData::from_cartesian(&[1.0, -0.7, 0.2, 0.0], &[0.4, 0.9, -1.1, 0.0]).unwrap();
    assert_eq!(compass.len(), 4);
    let compass_complex = CompassData::from_complex(&[[1.0, 1.0], [-1.0, 0.5]]);
    assert_eq!(compass_complex.len(), 2);

    let scale = ColorScale::new([1.0, 4.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut state = PolarState::default();
    let resp = run_polar(&ctx, vec![], true, |ui| {
        PolarPlot::new("polar_combo")
            .height(340.0)
            .title("Stage 8D Polar Gallery")
            .bubble_legend(
                BubbleLegend::new(bubble.size_extent(), bubble.radius_range())
                    .steps(3)
                    .title("Weight"),
            )
            .show(
                ui,
                &mut state,
                &[
                    PolarLayer::from(
                        PolarLine::new(&line_data)
                            .name("polarplot")
                            .color(Color32::from_rgb(0, 114, 189))
                            .width(2.0),
                    ),
                    PolarLayer::from(
                        PolarLine::new(fpolar.curve())
                            .name("fpolarplot")
                            .color(Color32::from_rgb(217, 83, 25)),
                    ),
                    PolarLayer::from(
                        PolarLine::new(ezp.curve())
                            .name("ezpolar")
                            .color(Color32::from_rgb(126, 47, 142)),
                    ),
                    PolarLayer::from(PolarScatter::new(&line_data).name("polarscatter").marker(
                        MarkerStyle {
                            shape: MarkerShape::Diamond,
                            radius: 5.0,
                        },
                    )),
                    PolarLayer::from(
                        PolarBubble::new(&bubble)
                            .name("polarbubblechart")
                            .color_scale(scale),
                    ),
                    PolarLayer::from(
                        CompassPlot::new(&compass)
                            .name("compass")
                            .color(Color32::from_rgb(119, 172, 48)),
                    ),
                ],
            )
    });
    assert!(resp.diagnostics.visible_series == 6);
    assert!(resp.diagnostics.polar.output_paths >= 3);
    assert!(resp.diagnostics.polar.output_markers >= bubble.points().len());
    assert!(resp.diagnostics.polar.output_arrows >= 3);
    assert!(
        resp.legend_entries
            .iter()
            .any(|e| e.name == "polarbubblechart")
    );
}

#[test]
fn negative_radius_modes_and_orientations_behave_distinctly() {
    let neg_curve = PolarCurveData::from_theta_r(&[0.0, PI / 2.0], &[-0.5, 1.5]).unwrap();
    let modes = [
        NegativeRadiusMode::ShiftMinRadius,
        NegativeRadiusMode::ReflectOrigin,
        NegativeRadiusMode::ClipBelowMin,
    ];
    let ctx = Context::default();
    let mut mapped_points = Vec::new();
    for mode in modes {
        let mut st = PolarState::default();
        let res = run_polar(&ctx, vec![], false, |ui| {
            PolarPlot::new("neg_mode")
                .negative_radius_mode(mode)
                .lock_r_zero(false)
                .show(
                    ui,
                    &mut st,
                    &[PolarLayer::from(PolarScatter::new(&neg_curve))],
                )
        });
        let p = res.transform.position_from_polar(0.0, -0.5, mode);
        mapped_points.push(p);
    }
    assert!(mapped_points[0].is_some());
    assert!(mapped_points[1].is_some());
    assert!(mapped_points[2].is_none());

    for zero in [
        ThetaZeroLocation::Right,
        ThetaZeroLocation::Top,
        ThetaZeroLocation::Left,
        ThetaZeroLocation::Bottom,
    ] {
        for dir in [ThetaDirection::CounterClockwise, ThetaDirection::Clockwise] {
            let mut st = PolarState::default();
            let res = run_polar(&ctx, vec![], false, |ui| {
                PolarPlot::new("orient")
                    .theta_zero(zero)
                    .theta_direction(dir)
                    .show(ui, &mut st, &[PolarLayer::from(PolarLine::new(&neg_curve))])
            });
            let pos = res
                .transform
                .position_from_polar(0.4, 1.0, NegativeRadiusMode::ShiftMinRadius)
                .unwrap();
            let back = res.transform.polar_from_position(pos).unwrap();
            assert!((back[1] - 1.0).abs() < 1e-3);
        }
    }
}

#[test]
fn polar_state_navigation_and_legend_toggle_are_idempotent_across_multipass() {
    let data =
        PolarCurveData::from_degrees(&[0.0, 90.0, 180.0, 270.0], &[0.5, 1.0, 1.5, 2.0]).unwrap();
    let mut outcomes = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PolarState::default();
        let show = |ui: &mut egui::Ui, st: &mut PolarState| {
            PolarPlot::new("nav").show(
                ui,
                st,
                &[
                    PolarLayer::from(PolarLine::new(&data).name("first")),
                    PolarLayer::from(PolarScatter::new(&data).name("second")),
                ],
            )
        };
        let first = run_polar(&ctx, vec![], false, |ui| show(ui, &mut state));
        let drag_start = first.plot_area.center();
        let drag_end = drag_start + Vec2::new(60.0, 0.0);
        let btn = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        for events in [
            vec![Event::PointerMoved(drag_start)],
            vec![btn(drag_start, true)],
            vec![Event::PointerMoved(drag_end)],
            vec![btn(drag_end, false)],
        ] {
            let _ = run_polar(&ctx, events, multipass, |ui| show(ui, &mut state));
        }
        assert!(
            state.theta_offset().abs() > 1e-3,
            "dragging in plot area should rotate theta_offset once per frame"
        );
        let drag_theta = state.theta_offset();

        // Scroll wheel zoom inside plot area (multi-pass safe).
        let _ = run_polar(&ctx, vec![Event::PointerMoved(drag_start)], false, |ui| {
            show(ui, &mut state)
        });
        let _ = run_polar(
            &ctx,
            vec![
                Event::PointerMoved(drag_start),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 40.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Unknown,
                    modifiers: Default::default(),
                },
            ],
            multipass,
            |ui| show(ui, &mut state),
        );
        let zoomed_r = state.r_lim().expect("scroll wheel zooms r_lim");

        // Legend click toggles exactly once even in multipass.
        let legend_pos: Pos2 = first.legend_entries[0].rect.center();
        for events in [
            vec![Event::PointerMoved(legend_pos)],
            vec![btn(legend_pos, true)],
            vec![btn(legend_pos, false)],
        ] {
            let _ = run_polar(&ctx, events, multipass, |ui| show(ui, &mut state));
        }
        let after = run_polar(&ctx, vec![], false, |ui| show(ui, &mut state));
        assert!(!after.legend_entries[0].visible);
        assert!(after.legend_entries[1].visible);

        outcomes.push((
            (drag_theta * 1e6).round() as i64,
            (zoomed_r[0] * 1e6).round() as i64,
            (zoomed_r[1] * 1e6).round() as i64,
            after.legend_entries[0].visible,
        ));
        state.reset();
        assert!(state.theta_offset() == 0.0 && state.r_lim().is_none());
    }
    assert_eq!(outcomes[0], outcomes[1]);
}
