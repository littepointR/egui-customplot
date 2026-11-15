use egui::Context;
use egui_customplot::{
    CategoricalHeatmap, CategoricalHeatmapData, ColorMap, ColorScale, Line, MarginalKind,
    ParallelData, ParallelNormalization, ParallelPlot, Plot, PlotData, PlotLayer, PlotMatrix,
    PlotMatrixData, PlotState, ScatterHistogram, ScatterHistogramData, ScatterHistogramState,
    SortOrder, StackedPlot, TableAggregation,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

fn x_range(b: egui_customplot::PlotBounds) -> [f64; 2] {
    [b.min()[0], b.max()[0]]
}

fn y_range(b: egui_customplot::PlotBounds) -> [f64; 2] {
    [b.min()[1], b.max()[1]]
}

#[test]
fn categorical_heatmap_and_parallel_plot_support_table_ops_and_norms() {
    let cat = CategoricalHeatmapData::from_table(
        &["US", "EU", "US", "APAC", "EU"],
        &["Mon", "Mon", "Tue", "Tue", "Tue"],
        Some(&[10.0, 20.0, 30.0, 40.0, 50.0]),
        TableAggregation::Mean,
        SortOrder::Ascending,
    )
    .unwrap();
    assert_eq!(cat.x_categories(), &["APAC", "EU", "US"]);
    assert_eq!(cat.y_categories(), &["Mon", "Tue"]);
    let scale = ColorScale::new([0.0, 60.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("cat_hm")
            .categorical_heatmap_axes(&cat)
            .show_layers(
                ui,
                &mut state,
                &[PlotLayer::from(
                    CategoricalHeatmap::new(&cat, scale).show_cell_labels(true),
                )],
            )
    });
    assert_eq!(res.axis_ticks[0].len(), 3);
    assert_eq!(res.axis_ticks[1].len(), 2);

    // Test HoveredCell on APACS + Tue cell at [0.5, 1.5].
    let hover_pos = res.transform.position_from_point([0.5, 1.5]);
    let picked = support::run(
        &ctx,
        vec![egui::Event::PointerMoved(hover_pos)],
        false,
        |ui| {
            Plot::new("cat_hm")
                .categorical_heatmap_axes(&cat)
                .show_layers(
                    ui,
                    &mut state,
                    &[PlotLayer::from(CategoricalHeatmap::new(&cat, scale))],
                )
        },
    );
    assert_eq!(picked.hovered_cell.expect("categorical cell").value, 40.0);

    for norm in [
        ParallelNormalization::None,
        ParallelNormalization::ZScore,
        ParallelNormalization::Norm,
        ParallelNormalization::Range,
        ParallelNormalization::MedianIqr,
    ] {
        let par = ParallelData::new(
            &["V1", "V2", "V3"],
            &[
                &[1.0, 10.0, 100.0],
                &[2.0, 20.0, 200.0],
                &[4.0, 40.0, 400.0],
            ],
            norm,
        )
        .unwrap()
        .with_groups(&[0, 1, 0], &["Low", "High"])
        .unwrap();
        let mut pstate = PlotState::default();
        let pres = support::run(&ctx, vec![], false, |ui| {
            Plot::new("par")
                .x_axis(egui_customplot::Axis::new().explicit_ticks(par.x_ticks()))
                .show_layers(
                    ui,
                    &mut pstate,
                    &[PlotLayer::from(ParallelPlot::new(&par).show_markers(true))],
                )
        });
        assert_eq!(pres.diagnostics.stats.output_parallel_segments, 6);
    }
}

#[test]
fn stacked_scatterhistogram_and_plotmatrix_synchronize_views() {
    let ctx = Context::default();
    let d1 = PlotData::new(vec![[0.0, 1.0], [1.0, 2.0], [2.0, 0.5]]);
    let d2 = PlotData::new(vec![[0.0, 10.0], [1.5, 20.0], [3.0, 15.0]]);
    let mut stacked_states = vec![PlotState::default(), PlotState::default()];
    let mut stacked_x = [[0.0; 2]; 2];
    for _ in 0..2 {
        ctx.run_ui(support::input(vec![]), |ui| {
            let out = StackedPlot::new(
                "stacked",
                vec![
                    ("Signal A", vec![PlotLayer::from(Line::new(&d1))]),
                    ("Signal B", vec![PlotLayer::from(Line::new(&d2))]),
                ],
            )
            .x_label("Time (s)")
            .show(ui, &mut stacked_states)
            .unwrap();
            assert!(
                out.figure.inner[0].axis_ticks[0]
                    .iter()
                    .all(|t| t.label.is_empty())
            );
            assert!(
                out.figure.inner[1].axis_ticks[0]
                    .iter()
                    .any(|t| !t.label.is_empty())
            );
            stacked_x = [
                x_range(out.figure.inner[0].transform.bounds()),
                x_range(out.figure.inner[1].transform.bounds()),
            ];
        })
        .drop_without_applying_deltas();
    }
    assert_eq!(stacked_x[0], stacked_x[1]);

    let sh_data = ScatterHistogramData::new(
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        &[2.0, 3.5, 3.0, 5.0, 6.0, 5.5],
        Some(&[0, 0, 1, 1, 0, 1]),
        4,
        MarginalKind::Kernel,
    )
    .unwrap();
    let mut sh_state = ScatterHistogramState::default();
    let mut sh_x = [[0.0; 2]; 2];
    let mut sh_y = [[0.0; 2]; 2];
    for _ in 0..2 {
        ctx.run_ui(support::input(vec![]), |ui| {
            let res = ScatterHistogram::new("sh", &sh_data)
                .show(ui, &mut sh_state)
                .unwrap();
            assert_eq!(res.inner.len(), 4);
            sh_x = [
                x_range(res.inner[0].transform.bounds()),
                x_range(res.inner[2].transform.bounds()),
            ];
            sh_y = [
                y_range(res.inner[2].transform.bounds()),
                y_range(res.inner[3].transform.bounds()),
            ];
        })
        .drop_without_applying_deltas();
    }
    assert_eq!(sh_x[0], sh_x[1]);
    assert_eq!(sh_y[0], sh_y[1]);

    let pm_data = PlotMatrixData::new(
        &["A", "B", "C"],
        &[
            &[1.0, 2.0, 3.0, 4.0, 5.0],
            &[2.0, 4.0, 3.0, 6.0, 5.0],
            &[5.0, 4.0, 3.0, 2.0, 1.0],
        ],
        Some(&[0, 0, 1, 1, 1]),
        3,
    )
    .unwrap();
    let mut pm_states = Vec::new();
    ctx.run_ui(support::input(vec![]), |ui| {
        let fig = PlotMatrix::new("pm", &pm_data)
            .show(ui, &mut pm_states)
            .unwrap();
        assert_eq!(fig.inner.len(), 9);
    })
    .drop_without_applying_deltas();
    assert_eq!(pm_states.len(), 9);
}
