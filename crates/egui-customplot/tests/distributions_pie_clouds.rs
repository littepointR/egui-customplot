use egui::{Color32, Context};
use egui_customplot::{
    BoxChart, BoxChartData, BubbleCloud, BubbleCloudData, PieChart, PieData, Plot, PlotLayer,
    PlotState, Raincloud, RaincloudData, RegionKind, ViolinData, ViolinHalf, ViolinOptions,
    ViolinPlot, ViolinScale, WordCloud, WordCloudData,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn boxplot_violin_and_raincloud_compute_stats_and_render() {
    let g1 = [1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 12.0];
    let g2 = [3.0, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5];
    let boxes = BoxChartData::from_groups(&[1.0, 2.0], &[&g1, &g2], 0.5).unwrap();
    assert_eq!(boxes.boxes().len(), 2);
    assert_eq!(boxes.boxes()[0].outliers.as_ref(), &[12.0]);

    for scale in [ViolinScale::Width, ViolinScale::Area, ViolinScale::Count] {
        for half in [ViolinHalf::Both, ViolinHalf::Left, ViolinHalf::Right] {
            let v = ViolinData::from_groups(
                &[1.0, 2.0],
                &[&g1, &g2],
                ViolinOptions {
                    scale,
                    half,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(v.entries().len(), 2);
        }
    }

    let violin =
        ViolinData::from_groups(&[1.0, 2.0], &[&g1, &g2], ViolinOptions::default()).unwrap();
    let rain =
        RaincloudData::from_groups(&[1.0, 2.0], &[&g1, &g2], ViolinOptions::default()).unwrap();
    assert_eq!(rain.entries()[0].drop_y.len(), 7);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("dist").show_layers(
            ui,
            &mut state,
            &[
                PlotLayer::from(BoxChart::new(&boxes).notched(true).show_mean(true)),
                PlotLayer::from(ViolinPlot::new(&violin).color(Color32::from_rgb(0, 114, 189))),
                PlotLayer::from(Raincloud::new(&rain).color(Color32::from_rgb(217, 83, 25))),
            ],
        )
    });
    assert_eq!(res.diagnostics.stats.output_boxes, 2);
    assert_eq!(res.diagnostics.stats.output_violins, 4);

    // Pick box region directly at median of first group.
    let mut b_state = PlotState::default();
    let b_res = support::run(&ctx, vec![], false, |ui| {
        Plot::new("only_box").show_layers(
            ui,
            &mut b_state,
            &[PlotLayer::from(BoxChart::new(&boxes))],
        )
    });
    let pos = b_res
        .transform
        .position_from_point([1.0, boxes.boxes()[0].median]);
    let picked = support::run(&ctx, vec![egui::Event::PointerMoved(pos)], false, |ui| {
        Plot::new("only_box").show_layers(
            ui,
            &mut b_state,
            &[PlotLayer::from(BoxChart::new(&boxes))],
        )
    });
    assert_eq!(
        picked.hovered_region.expect("box region").kind,
        RegionKind::BoxChart
    );
}

#[test]
fn pie_donut_wordcloud_and_bubblecloud_render_with_canvas_axes() {
    let pie = PieData::with_options(
        &[30.0, 45.0, 25.0, 0.0],
        Some(&["Compute", "Memory", "IO", "Zero"]),
        Some(&[false, true, false, false]),
        -std::f32::consts::FRAC_PI_2,
    )
    .unwrap();
    assert_eq!(pie.slices().len(), 3);
    assert_eq!(pie.skipped_slices(), 1);
    assert!((pie.slices().iter().map(|s| s.fraction).sum::<f64>() - 1.0).abs() < 1e-6);

    let wc = WordCloudData::new(
        &["egui", "rust", "matlab", "plot", "kde", "mesh"],
        &[90.0, 75.0, 50.0, 40.0, 30.0, 20.0],
    )
    .unwrap();
    // Determinism check: repeated construction yields identical coordinates.
    let wc2 = WordCloudData::new(
        &["egui", "rust", "matlab", "plot", "kde", "mesh"],
        &[90.0, 75.0, 50.0, 40.0, 30.0, 20.0],
    )
    .unwrap();
    assert_eq!(wc.words(), wc2.words());

    let bc = BubbleCloudData::new(
        &["Alpha", "Beta", "Gamma", "Delta"],
        &[100.0, 64.0, 36.0, 16.0],
        Some(&[0, 1, 0, 2]),
    )
    .unwrap();
    for i in 0..bc.bubbles().len() {
        for j in (i + 1)..bc.bubbles().len() {
            let a = &bc.bubbles()[i];
            let b = &bc.bubbles()[j];
            let d =
                ((a.center[0] - b.center[0]).powi(2) + (a.center[1] - b.center[1]).powi(2)).sqrt();
            assert!(d + 1e-3 >= a.radius + b.radius);
        }
    }

    let ctx = Context::default();
    let mut s_pie = PlotState::default();
    let mut s_wc = PlotState::default();
    let mut s_bc = PlotState::default();

    let p_res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("donut").canvas_axes().show_layers(
            ui,
            &mut s_pie,
            &[PlotLayer::from(PieChart::donut(&pie))],
        )
    });
    assert_eq!(p_res.diagnostics.stats.output_slices, 3);
    assert!(p_res.axis_ticks[0].is_empty());

    let wc_res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("wc").canvas_axes().show_layers(
            ui,
            &mut s_wc,
            &[PlotLayer::from(WordCloud::new(&wc))],
        )
    });
    assert_eq!(wc_res.diagnostics.stats.output_clouds, 6);

    let bc_res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("bc").canvas_axes().show_layers(
            ui,
            &mut s_bc,
            &[PlotLayer::from(BubbleCloud::new(&bc))],
        )
    });
    assert_eq!(bc_res.diagnostics.stats.output_clouds, 4);
}
