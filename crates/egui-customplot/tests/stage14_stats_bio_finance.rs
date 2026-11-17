use egui::{Context, Pos2, RawInput, Rect};
use egui_customplot::{
    Axis, AxisScale, BarOrientation, Biplot, BiplotData, Candlestick, ClusterStandardize,
    Clustergram, ClustergramData, ClustergramState, ControlChart, ControlChartData, DendrogramData,
    DendrogramPlot, EcdfData, EcdfKind, EcdfPlot, LinkageMethod, OhlcBar, OhlcBars, OhlcData, Plot,
    PlotLayer, PlotState, ProbabilityDist, ProbabilityPlot, ProbabilityPlotData, RegionKind,
    SpcChartKind, SpcViolationKind, Waterfall2D, Waterfall2DData, civil_to_unix_secs, norm_cdf,
    norm_inv,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(720.0, 520.0))),
        ..Default::default()
    }
}

fn hovered_raw(pos: Pos2) -> RawInput {
    let mut input = raw();
    input.events.push(egui::Event::PointerMoved(pos));
    input
}

#[test]
fn ext13_probability_scales_and_probplot_qq_roundtrip() {
    for z in [-4.0, -2.5, -1.0, 0.0, 0.674_489_75, 1.959_963_98, 3.8] {
        let p = norm_cdf(z);
        let z_back = norm_inv(p).expect("finite inverse");
        assert!((z - z_back).abs() < 1e-6, "z={z} p={p} z_back={z_back}");
    }

    for scale in [AxisScale::Probit, AxisScale::Weibull] {
        let ax = Axis::new().scale(scale);
        let ticks = ax.ticks([0.01, 0.99], 400.0);
        assert!(ticks.len() >= 5);
        for w in ticks.windows(2) {
            assert!(w[0].value < w[1].value);
        }
        assert!(ax.ticks([-0.2, 0.5], 400.0).is_empty());
        assert!(ax.ticks([0.2, 1.2], 400.0).is_empty());
    }

    let x = [1.1, 1.5, 1.9, 2.2, 2.4, 2.8, 3.1, 3.7];
    let norm_p = ProbabilityPlotData::normal(&x).unwrap();
    assert_eq!(norm_p.dist(), ProbabilityDist::Normal);
    assert_eq!(
        norm_p.recommended_scales(),
        [AxisScale::Linear, AxisScale::Probit]
    );

    let wbl_p = ProbabilityPlotData::weibull(&x).unwrap();
    assert_eq!(wbl_p.dist(), ProbabilityDist::Weibull);
    assert_eq!(
        wbl_p.recommended_scales(),
        [AxisScale::Log10, AxisScale::Weibull]
    );
    assert!(ProbabilityPlotData::weibull(&[-1.0, 2.0]).is_err());

    let qq = ProbabilityPlotData::qq_two_sample(&x, &[2.0, 4.0, 6.0, 8.0]).unwrap();
    assert_eq!(qq.len(), 4);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut sample_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = Plot::new("prob_pick").probplot_axes(&norm_p).show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(ProbabilityPlot::new(&norm_p))],
        );
        sample_pos = resp.transform.position_from_point(norm_p.points()[3]);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(sample_pos), |ui| {
        let resp = Plot::new("prob_pick").probplot_axes(&norm_p).show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(ProbabilityPlot::new(&norm_p))],
        );
        let hit = resp.hovered_point.expect("probability point hover");
        assert_eq!(hit.point_index, 3);
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext14_kaplan_meier_ecdf_survival_ties_and_greenwood_ci() {
    let times = [2.0, 4.0, 4.0, 6.0, 8.0, 10.0];
    let censored = [false, false, true, false, true, false];
    let surv = EcdfData::kaplan_meier(&times, &censored, EcdfKind::Survival, 0.95).unwrap();
    let cdf = EcdfData::kaplan_meier(&times, &censored, EcdfKind::Cdf, 0.95).unwrap();

    for ((s, lo), hi) in surv
        .step_points()
        .iter()
        .zip(surv.ci_lower())
        .zip(surv.ci_upper())
    {
        assert!((0.0..=1.0).contains(&s[1]));
        assert!(lo[1] <= s[1] + 1e-12 && s[1] <= hi[1] + 1e-12);
    }
    assert_eq!(surv.censored_marks().len(), 2);
    assert!(cdf.step_points().last().unwrap()[1] > 0.7);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut event_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp =
            Plot::new("km").show_layers(ui, &mut state, &[PlotLayer::from(EcdfPlot::new(&surv))]);
        event_pos = resp.transform.position_from_point(surv.event_points()[0]);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(event_pos), |ui| {
        let resp =
            Plot::new("km").show_layers(ui, &mut state, &[PlotLayer::from(EcdfPlot::new(&surv))]);
        let hit = resp.hovered_point.expect("ecdf event pick");
        assert_eq!(hit.point_index, 0);
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext15_dendrogram_all_linkages_and_clustergram_axis_sync() {
    let rows: [&[f64]; 5] = [
        &[0.0, 0.0],
        &[0.4, 0.1],
        &[5.0, 5.0],
        &[5.2, 5.3],
        &[2.5, 2.5],
    ];
    for method in [
        LinkageMethod::Single,
        LinkageMethod::Complete,
        LinkageMethod::Average,
        LinkageMethod::Ward,
    ] {
        let d = DendrogramData::from_observations(&rows, method).unwrap();
        assert_eq!(d.steps().len(), 4);
        let mut sorted = d.leaf_order().to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4]);
        for w in d.steps().windows(2) {
            assert!(w[1].distance + 1e-12 >= w[0].distance);
        }
    }

    let d = DendrogramData::from_observations(&rows, LinkageMethod::Ward).unwrap();
    let top_branch = d.branches().last().unwrap();
    let ctx = Context::default();
    let mut d_state = PlotState::default();
    let mut branch_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = Plot::new("dendro_pick").show_layers(
            ui,
            &mut d_state,
            &[PlotLayer::from(
                DendrogramPlot::new(&d).orientation(BarOrientation::Vertical),
            )],
        );
        branch_pos = resp
            .transform
            .position_from_point([top_branch.x_mid, top_branch.y_top]);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(branch_pos), |ui| {
        let resp = Plot::new("dendro_pick").show_layers(
            ui,
            &mut d_state,
            &[PlotLayer::from(
                DendrogramPlot::new(&d).orientation(BarOrientation::Vertical),
            )],
        );
        let region = resp.hovered_region.expect("dendrogram branch pick");
        assert_eq!(region.kind, RegionKind::DendrogramBranch);
        assert_eq!(region.item_index, top_branch.index);
    })
    .drop_without_applying_deltas();

    let mat: [&[f64]; 4] = [
        &[3.0, 2.8, -1.0],
        &[-1.2, -1.0, 2.5],
        &[2.7, 3.1, -0.9],
        &[-1.1, -1.3, 2.7],
    ];
    let cg = ClustergramData::new(&mat, LinkageMethod::Average, ClusterStandardize::RowZScore)
        .unwrap()
        .with_row_labels(["G1", "G2", "G3", "G4"].map(str::to_owned))
        .with_col_labels(["T1", "T2", "T3"].map(str::to_owned));
    let mut cg_state = ClustergramState::default();
    let mut top_x = [0.0_f64; 2];
    let mut heat_x = [0.0_f64; 2];
    let mut left_y = [0.0_f64; 2];
    let mut heat_y = [0.0_f64; 2];
    for _ in 0..2 {
        ctx.run_ui(raw(), |ui| {
            let res = Clustergram::new("cg", &cg).show(ui, &mut cg_state);
            assert!(res.heatmap_rect.is_positive());
            let tb = res.top_dendrogram.transform.bounds();
            let hb = res.heatmap.transform.bounds();
            let lb = res.left_dendrogram.transform.bounds();
            top_x = [tb.min()[0], tb.max()[0]];
            heat_x = [hb.min()[0], hb.max()[0]];
            left_y = [lb.min()[1], lb.max()[1]];
            heat_y = [hb.min()[1], hb.max()[1]];
        })
        .drop_without_applying_deltas();
    }
    assert!((top_x[0] - heat_x[0]).abs() < 1e-6 && (top_x[1] - heat_x[1]).abs() < 1e-6);
    assert!((left_y[0] - heat_y[0]).abs() < 1e-6 && (left_y[1] - heat_y[1]).abs() < 1e-6);
}

#[test]
fn ext16_biplot_pca_varimax_communalities_and_vector_picking() {
    let obs: [&[f64]; 6] = [
        &[2.5, 2.4, 0.9],
        &[0.5, 0.7, 0.2],
        &[2.2, 2.9, 1.1],
        &[1.9, 2.2, 0.8],
        &[3.1, 3.0, 1.5],
        &[1.1, 0.9, 0.4],
    ];
    let pca = BiplotData::from_pca(&obs)
        .unwrap()
        .with_var_labels(["A", "B", "C"].map(str::to_owned));
    for &[c0, c1] in pca.loadings_raw() {
        assert!(c0.is_finite() && c1.is_finite());
    }
    let varimax = pca.clone().with_varimax();
    for (a, b) in pca.loadings_raw().iter().zip(varimax.loadings_raw()) {
        let h_a = a[0] * a[0] + a[1] * a[1];
        let h_b = b[0] * b[0] + b[1] * b[1];
        assert!((h_a - h_b).abs() < 1e-10, "Varimax altered communality");
    }

    let ctx = Context::default();
    let mut state = PlotState::default();
    let arrow = pca.loadings_quiver().arrows()[0];
    let tip_data = arrow.endpoint().unwrap();
    let mut tip_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = Plot::new("bp").data_aspect(1.0).show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(Biplot::new(&pca))],
        );
        tip_pos = resp.transform.position_from_point(tip_data);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(tip_pos), |ui| {
        let resp = Plot::new("bp").data_aspect(1.0).show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(Biplot::new(&pca))],
        );
        let vec_hit = resp.hovered_vector.expect("biplot loading vector pick");
        assert_eq!(vec_hit.arrow_index, 0);
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext17_spc_control_charts_and_nelson_rules() {
    // 10 rising/high values + clear outlier at index 10
    let mut vals = vec![
        10.0, 10.2, 10.1, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8, 10.9, 15.8,
    ];
    vals.extend([9.9, 9.8, 9.7, 9.6, 9.5, 9.4]);
    let imr = ControlChartData::imr(&vals).unwrap();
    assert_eq!(imr.kind(), SpcChartKind::IMR);
    assert!(
        imr.points()
            .iter()
            .any(|p| p.violation == Some(SpcViolationKind::BeyondThreeSigma))
    );
    assert!(
        imr.points()
            .iter()
            .any(|p| p.violation == Some(SpcViolationKind::SixTrend))
    );

    let subgroups: [&[f64]; 4] = [
        &[10.0, 10.2, 10.1],
        &[10.3, 10.1, 10.2],
        &[9.8, 10.0, 9.9],
        &[10.1, 10.4, 10.2],
    ];
    let xbar = ControlChartData::xbar_r(&subgroups).unwrap();
    assert_eq!(xbar.kind(), SpcChartKind::XbarR);

    let p = ControlChartData::p_chart(&[3.0, 5.0, 18.0], &[100.0, 120.0, 100.0]).unwrap();
    assert_ne!(p.points()[0].ucl, p.points()[1].ucl);
    let u = ControlChartData::u_chart(&[2.0, 4.0, 15.0], &[10.0, 20.0, 10.0]).unwrap();
    assert_eq!(u.kind(), SpcChartKind::UChart);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut p0_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = Plot::new("spc").show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(ControlChart::new(&imr))],
        );
        p0_pos = resp
            .transform
            .position_from_point([imr.points()[10].x, imr.points()[10].value]);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(p0_pos), |ui| {
        let resp = Plot::new("spc").show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(ControlChart::new(&imr))],
        );
        let hit = resp.hovered_point.expect("spc point hover");
        assert_eq!(hit.point_index, 10);
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext18_candlestick_ohlc_waterfall_validation_picking_and_multipass() {
    assert!(OhlcData::new(vec![]).is_err());
    assert!(
        OhlcData::new(vec![OhlcBar {
            x: 1.0,
            width: 0.5,
            open: 10.0,
            high: 11.0,
            low: 10.5, // invalid low > open
            close: 10.8,
        }])
        .is_err()
    );

    let t0 = civil_to_unix_secs(2026, 4, 1, 0, 0, 0.0).expect("date");
    let ohlc = OhlcData::from_slices(
        &[t0, t0 + 86400.0, t0 + 2.0 * 86400.0],
        &[100.0, 104.0, 102.0],
        &[106.0, 105.0, 108.0],
        &[99.0, 101.0, 101.5],
        &[104.0, 102.0, 107.0],
    )
    .unwrap();
    let wf = Waterfall2DData::from_deltas(
        &[50.0, 20.0, -15.0, 0.0],
        Some(&[false, false, false, true]),
        Some(&["Start", "Gain", "Loss", "Total"]),
    )
    .unwrap();
    assert!((wf.columns()[3].start - 0.0).abs() < 1e-12);
    assert!((wf.columns()[3].end - 55.0).abs() < 1e-12);

    let ctx = Context::default();
    let mut cs = PlotState::default();
    let mut c_pos = Pos2::ZERO;
    let mut passes = 0;
    ctx.run_ui(raw(), |ui| {
        let r1 = Plot::new("c").x_axis(Axis::new().datetime()).show_layers(
            ui,
            &mut cs,
            &[PlotLayer::from(Candlestick::new(&ohlc))],
        );
        c_pos = r1
            .transform
            .position_from_point([ohlc.bars()[1].x, ohlc.bars()[1].close]);
        if passes == 0 {
            ui.ctx().request_discard("stage14 multipass");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    ctx.run_ui(hovered_raw(c_pos), |ui| {
        let r = Plot::new("c").x_axis(Axis::new().datetime()).show_layers(
            ui,
            &mut cs,
            &[PlotLayer::from(Candlestick::new(&ohlc))],
        );
        let hit = r.hovered_region.expect("candlestick region hover");
        assert_eq!(hit.kind, RegionKind::OhlcCandle);
        assert_eq!(hit.item_index, 1);
    })
    .drop_without_applying_deltas();

    let mut os = PlotState::default();
    let mut o_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let r2 = Plot::new("o").x_axis(Axis::new().datetime()).show_layers(
            ui,
            &mut os,
            &[PlotLayer::from(OhlcBars::new(&ohlc))],
        );
        o_pos = r2
            .transform
            .position_from_point([ohlc.bars()[2].x, ohlc.bars()[2].close]);
    })
    .drop_without_applying_deltas();
    ctx.run_ui(hovered_raw(o_pos), |ui| {
        let r = Plot::new("o").x_axis(Axis::new().datetime()).show_layers(
            ui,
            &mut os,
            &[PlotLayer::from(OhlcBars::new(&ohlc))],
        );
        let hit = r.hovered_point.expect("ohlc bar point hover");
        assert_eq!(hit.point_index, 2);
    })
    .drop_without_applying_deltas();

    let mut ws = PlotState::default();
    let mut w_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let r3 = Plot::new("w").waterfall_axes(&wf).show_layers(
            ui,
            &mut ws,
            &[PlotLayer::from(Waterfall2D::new(&wf))],
        );
        w_pos = r3.transform.position_from_point([
            wf.columns()[1].x,
            0.5 * (wf.columns()[1].start + wf.columns()[1].end),
        ]);
    })
    .drop_without_applying_deltas();
    ctx.run_ui(hovered_raw(w_pos), |ui| {
        let r = Plot::new("w").waterfall_axes(&wf).show_layers(
            ui,
            &mut ws,
            &[PlotLayer::from(Waterfall2D::new(&wf))],
        );
        let hit = r.hovered_region.expect("waterfall bar hover");
        assert_eq!(hit.kind, RegionKind::WaterfallBar);
        assert_eq!(hit.item_index, 1);
    })
    .drop_without_applying_deltas();
}
