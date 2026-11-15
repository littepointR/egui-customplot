use egui::{Color32, Context};
use egui_customplot::{
    BarData, BarOrientation, BarSeriesError, Bars, HistogramError, HistogramNormalization,
    LegacyHistData, Line, ParetoChart, ParetoData, PlotData, PlotState, PlotYy,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn grouped_and_stacked_bars_preserve_offsets_and_signs() {
    assert!(matches!(
        BarData::grouped(&[1.0, 2.0], &[&[1.0], &[2.0, 3.0]], 0.8),
        Err(BarSeriesError::LengthMismatch)
    ));
    let centers = [1.0, 2.0, 3.0];
    let s0 = [4.0, -2.0, 5.0];
    let s1 = [2.0, -3.0, 1.0];
    let grouped = BarData::grouped(&centers, &[&s0, &s1], 0.8).unwrap();
    assert_eq!(grouped.len(), 2);
    assert!(grouped[0].bars()[0].center < 1.0);
    assert!(grouped[1].bars()[0].center > 1.0);
    assert_eq!(grouped[0].bars()[0].value, 4.0);

    let stacked = BarData::stacked(&centers, &[&s0, &s1], 0.7).unwrap();
    // Positive stack: 0..4 then 4..6; negative stack: 0..-2 then -2..-5
    assert_eq!(
        (stacked[0].bars()[0].baseline, stacked[0].bars()[0].value),
        (0.0, 4.0)
    );
    assert_eq!(
        (stacked[1].bars()[0].baseline, stacked[1].bars()[0].value),
        (4.0, 6.0)
    );
    assert_eq!(
        (stacked[0].bars()[1].baseline, stacked[0].bars()[1].value),
        (0.0, -2.0)
    );
    assert_eq!(
        (stacked[1].bars()[1].baseline, stacked[1].bars()[1].value),
        (-2.0, -5.0)
    );
}

#[test]
fn legacy_hist_bins_include_extrema_and_respect_explicit_centers() {
    assert_eq!(
        LegacyHistData::new(&[1.0, 2.0], 0).unwrap_err(),
        HistogramError::InvalidEdges
    );
    let hist = LegacyHistData::new(&[0.0, 0.5, 1.0, 2.0, f64::NAN], 2).unwrap();
    assert_eq!(hist.counts().iter().sum::<usize>(), 4);
    assert_eq!(hist.centers().len(), 2);

    let by_centers = LegacyHistData::with_centers(&[0.2, 1.1, 1.2, 2.8], &[0.0, 1.0, 3.0]).unwrap();
    assert_eq!(by_centers.counts(), &[1, 2, 1]);
    let probs = by_centers
        .normalized_bars(HistogramNormalization::Probability)
        .unwrap();
    assert!((probs.bars().iter().map(|b| b.value).sum::<f64>() - 1.0).abs() < 1e-12);
    let _horiz = Bars::new(&by_centers.to_bars()).orientation(BarOrientation::Horizontal);
}

#[test]
fn pareto_and_plotyy_render_dual_y_axes_in_multipass_ui() {
    let pareto = ParetoData::new(
        &["Auth", "IO", "Network", "Parse"],
        &[15.0, 45.0, 30.0, 10.0],
    )
    .unwrap();
    assert_eq!(pareto.labels(), &["IO", "Network", "Auth", "Parse"]);
    assert!((pareto.cumulative_percent()[3] - 100.0).abs() < 1e-9);

    let limited =
        ParetoData::with_limit(&["A", "B", "C", "D"], &[50.0, 30.0, 15.0, 5.0], 0.95, 10).unwrap();
    assert_eq!(limited.labels(), &["A", "B", "C"]);

    let ctx = Context::default();
    let mut p_state = PlotState::default();
    let res = support::run(&ctx, vec![], true, |ui| {
        ParetoChart::new("pareto", &pareto)
            .title("Pareto Latency Causes")
            .show(ui, &mut p_state)
    });
    assert!(res.right_transform.is_some());
    assert!(!res.right_axis_ticks.is_empty());
    assert!(res.diagnostics.output_bars >= 4);

    let d1 = PlotData::new(vec![[0.0, 1.0], [1.0, 3.0], [2.0, 2.0]]);
    let d2 = PlotData::new(vec![[0.0, 100.0], [1.0, 250.0], [2.0, 400.0]]);
    let mut yy_state = PlotState::default();
    let yy_res = support::run(&ctx, vec![], true, |ui| {
        PlotYy::lines(
            "yy",
            Line::new(&d1).name("Left").color(Color32::BLUE),
            Line::new(&d2).name("Right").color(Color32::RED),
        )
        .show_legend(true)
        .show(ui, &mut yy_state)
    });
    assert!(yy_res.right_transform.is_some());
    assert_eq!(yy_res.legend_entries.len(), 2);
}
