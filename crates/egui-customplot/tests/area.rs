mod support;

use std::sync::Arc;

use egui::{Color32, Context, Event, Shape};
use egui_customplot::{
    Area, AreaBandData, AreaError, AreaStackMode, Axis, AxisScale, Fill, Plot, PlotData, PlotState,
    PolygonData, RegionKind,
};
use support::{capture, run};

#[test]
fn single_curve_crosses_baseline_respects_gaps_and_rejects_invalid_log_baseline() {
    let ctx = Context::default();
    let data = PlotData::new(vec![
        [0.0, -2.0],
        [2.0, 2.0],
        [f64::NAN; 2],
        [3.0, 1.0],
        [4.0, -1.0],
    ]);
    let mut state = PlotState::default();
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("area").show_layers(
            ui,
            &mut state,
            &[Area::new(&data)
                .baseline(0.0)
                .fill_color(Color32::from_rgba_unmultiplied(40, 120, 220, 140))
                .color(Color32::BLUE)
                .into()],
        )
    });
    let meshes: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Mesh(m) => Some(m),
            _ => None,
        })
        .collect();
    assert_eq!(meshes.len(), 1);
    assert!(meshes[0].is_valid());
    assert_eq!(out.diagnostics.patches.output_meshes, 1);
    assert_eq!(out.diagnostics.patches.output_triangles, 4);
    assert_eq!(out.diagnostics.patches.output_strokes, 2);
    output.drop_without_applying_deltas();

    let log_data = PlotData::new(vec![[1.0, 2.0], [10.0, 20.0]]);
    let mut log_state = PlotState::default();
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("log-area")
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show_layers(
                ui,
                &mut log_state,
                &[Area::new(&log_data).baseline(0.0).into()],
            )
    });
    assert_eq!(out.diagnostics.patches.invalid_area_baselines, 1);
    assert_eq!(out.diagnostics.patches.output_triangles, 0);

    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("log-area-ok")
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show_layers(
                ui,
                &mut log_state,
                &[Area::new(&log_data).baseline(0.5).into()],
            )
    });
    assert_eq!(out.diagnostics.patches.invalid_area_baselines, 0);
    assert!(out.diagnostics.patches.output_triangles >= 2);
}

#[test]
fn stacked_bands_share_x_allocation_and_support_pos_neg_split() {
    assert_eq!(
        AreaBandData::stack(
            [1.0, 2.0].as_slice(),
            &[&[1.0]],
            0.0,
            AreaStackMode::Cumulative
        )
        .unwrap_err(),
        AreaError::LengthMismatch
    );
    assert_eq!(
        AreaBandData::stack(
            [1.0, 2.0].as_slice(),
            &[&[1.0, 2.0]],
            f64::NAN,
            AreaStackMode::Cumulative
        )
        .unwrap_err(),
        AreaError::InvalidBaseline
    );

    let x: Arc<[f64]> = Arc::from([0.0, 1.0, 2.0, 3.0].as_slice());
    let s1 = [1.0, -2.0, 3.0, 1.0];
    let s2 = [2.0, -1.0, 1.0, 2.0];
    let bands =
        AreaBandData::stack(Arc::clone(&x), &[&s1, &s2], 0.0, AreaStackMode::PosNegSplit).unwrap();
    assert_eq!(bands[0].lower(), &[0.0, -2.0, 0.0, 0.0]);
    assert_eq!(bands[0].upper(), &[1.0, 0.0, 3.0, 1.0]);
    assert_eq!(bands[1].lower(), &[1.0, -3.0, 3.0, 1.0]);
    assert_eq!(bands[1].upper(), &[3.0, -2.0, 4.0, 3.0]);

    let cum = AreaBandData::stack(
        Arc::clone(&x),
        &[&[1.0, 1.0, 1.0, 1.0], &[2.0, 2.0, 2.0, 2.0]],
        0.0,
        AreaStackMode::Cumulative,
    )
    .unwrap();
    assert_eq!(cum[1].lower(), &[1.0; 4]);
    assert_eq!(cum[1].upper(), &[3.0; 4]);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let layers = [
        Area::from_band(&bands[0]).name("A").into(),
        Area::from_band(&bands[1]).name("B").into(),
    ];
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("stacked").show_layers(ui, &mut state, &layers)
    });
    // Shared x (4 * 8 = 32 bytes) is counted once + 4 unique y slices (4 * 32 = 128 bytes) = 160.
    assert_eq!(out.diagnostics.patches.unique_bytes, 32 + 4 * 32);

    let _ = layers;
    drop(bands);
    drop(cum);
    let weak = Arc::downgrade(&x);
    drop(x);
    assert!(weak.upgrade().is_none());
}

#[test]
fn area_hover_picks_body_or_stroke_and_top_fill_occludes() {
    let ctx = Context::default();
    let band = AreaBandData::from_band(
        [0.0, 2.0, 10.0].as_slice(),
        [1.0, 1.0, 1.0].as_slice(),
        [5.0, 5.0, 5.0].as_slice(),
    )
    .unwrap();
    let top_poly = PolygonData::new(vec![[4.0, 0.0], [6.0, 0.0], [6.0, 6.0], [4.0, 6.0]]);
    let layers = [
        Area::from_band(&band)
            .name("band")
            .fill_color(Color32::LIGHT_BLUE)
            .into(),
        Fill::new(&top_poly)
            .name("cover")
            .color(Color32::RED)
            .into(),
    ];
    let mut state = PlotState::default();
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("hover").show_layers(ui, &mut state, &layers)
    });
    let body_pos = initial.transform.position_from_point([2.0, 3.0]);
    let stroke_pos = initial.transform.position_from_point([2.0, 5.0]);
    let covered_pos = initial.transform.position_from_point([5.0, 5.0]);

    let body = run(&ctx, vec![Event::PointerMoved(body_pos)], false, |ui| {
        Plot::new("hover").show_layers(ui, &mut state, &layers)
    });
    assert_eq!(body.hovered_region.unwrap().kind, RegionKind::Area);
    assert!(body.hovered_point.is_none());

    let edge = run(&ctx, vec![Event::PointerMoved(stroke_pos)], false, |ui| {
        Plot::new("hover").show_layers(ui, &mut state, &layers)
    });
    assert!(edge.hovered_point.is_some());
    assert!(edge.hovered_region.is_none());

    let occ = run(&ctx, vec![Event::PointerMoved(covered_pos)], false, |ui| {
        Plot::new("hover").show_layers(ui, &mut state, &layers)
    });
    assert_eq!(occ.hovered_region.unwrap().kind, RegionKind::Fill);
    assert!(occ.hovered_point.is_none());
}

#[test]
fn non_monotonic_area_band_renders_and_hits_safely_and_is_multipass_idempotent() {
    let ctx = Context::default();
    let non_mono = AreaBandData::from_band(
        [0.0, 2.0, 1.0].as_slice(),
        [0.0, 0.0, 0.0].as_slice(),
        [2.0, 2.0, 2.0].as_slice(),
    )
    .unwrap();
    let mut nm_state = PlotState::default();
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("nm").show_layers(ui, &mut nm_state, &[Area::from_band(&non_mono).into()])
    });
    assert!(initial.diagnostics.patches.output_triangles >= 4);
    let inside = initial.transform.position_from_point([1.5, 1.0]);
    let hit = run(&ctx, vec![Event::PointerMoved(inside)], false, |ui| {
        Plot::new("nm").show_layers(ui, &mut nm_state, &[Area::from_band(&non_mono).into()])
    });
    assert_eq!(hit.hovered_region.unwrap().kind, RegionKind::Area);

    let band = AreaBandData::from_band(
        [0.0, 1.0, 2.0].as_slice(),
        [0.0, 0.5, 0.0].as_slice(),
        [1.0, 2.0, 1.0].as_slice(),
    )
    .unwrap();
    let mut state = PlotState::default();
    let mut pass = 0;
    let output = ctx.run_ui(support::input(vec![]), |ui| {
        let out = Plot::new("mp").show_layers(ui, &mut state, &[Area::from_band(&band).into()]);
        assert_eq!(out.diagnostics.patches.output_triangles, 4);
        if pass == 0 {
            ui.ctx().request_discard("layout second pass");
        }
        pass += 1;
    });
    output.drop_without_applying_deltas();
    assert_eq!(pass, 2);
}
