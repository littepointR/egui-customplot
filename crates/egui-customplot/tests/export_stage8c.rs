#![cfg(feature = "export")]

use egui::Context;
use egui_customplot::{
    BoxChart, BoxChartData, BubbleChart, BubbleData, CategoricalHeatmap, CategoricalHeatmapData,
    ColorMap, ColorScale, ExportOptions, Figure, PieChart, PieData, Plot, PlotLayer, PlotSnapshot,
    PlotState, WordCloud, WordCloudData,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

#[test]
fn stage8c_charts_export_self_contained_svg_and_png() {
    let bubble =
        BubbleData::new(&[[1.0, 2.0], [2.0, 3.0], [3.0, 1.5]], &[10.0, 25.0, 50.0]).unwrap();
    let box_data = BoxChartData::from_groups(&[1.0], &[&[1.0, 2.0, 2.5, 3.0, 6.0]], 0.5).unwrap();
    let pie = PieData::with_labels(&[30.0, 50.0, 20.0], &["A", "B", "C"]).unwrap();
    let wc = WordCloudData::new(&["Plot", "Stats", "Export"], &[80.0, 60.0, 40.0]).unwrap();
    let cat =
        CategoricalHeatmapData::from_matrix(&["X1", "X2"], &["Y1", "Y2"], &[1.0, 2.0, 3.0, 4.0])
            .unwrap();
    let cscale = ColorScale::new([0.0, 5.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut states = [
        PlotState::default(),
        PlotState::default(),
        PlotState::default(),
        PlotState::default(),
    ];
    let mut snapshot = None;
    ctx.run_ui(support::input(vec![]), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, Default::default(), |ui| {
                let fig = Figure::new("export_8c")
                    .columns(2)
                    .cell_height(200.0)
                    .show(ui, 4, |cell| match cell.index() {
                        0 => cell.show_layers(
                            Plot::new("b"),
                            &mut states[0],
                            &[
                                PlotLayer::from(BubbleChart::new(&bubble)),
                                PlotLayer::from(BoxChart::new(&box_data)),
                            ],
                        ),
                        1 => cell.show_layers(
                            Plot::new("pie").canvas_axes(),
                            &mut states[1],
                            &[PlotLayer::from(PieChart::donut(&pie))],
                        ),
                        2 => cell.show_layers(
                            Plot::new("wc").canvas_axes(),
                            &mut states[2],
                            &[PlotLayer::from(WordCloud::new(&wc))],
                        ),
                        _ => cell.show_layers(
                            Plot::new("cat").categorical_heatmap_axes(&cat),
                            &mut states[3],
                            &[PlotLayer::from(
                                CategoricalHeatmap::new(&cat, cscale).show_cell_labels(true),
                            )],
                        ),
                    })
                    .unwrap();
                fig.response.rect
            })
            .unwrap(),
        );
    })
    .drop_without_applying_deltas();
    drop((bubble, box_data, pie, wc, cat, ctx));

    let opts = ExportOptions {
        pixels_per_point: 0.5,
        ..Default::default()
    };
    let snap = snapshot.unwrap();
    let svg = String::from_utf8(snap.to_svg(opts).unwrap()).unwrap();
    assert!(
        svg.contains("<svg") && svg.contains("clipPath") && svg.contains("data:image/png;base64,")
    );
    assert!(!svg.contains("<text"));
    assert!(snap.to_png(opts).unwrap().starts_with(b"\x89PNG"));
}
