#![cfg(feature = "export")]

use std::f64::consts::TAU;

use egui::Color32;
use egui_customplot::{
    CompassData, CompassPlot, EliminationTree, ExportOptions, Figure, GraphPlot, Plot, PlotLayer,
    PlotState, PolarBubble, PolarBubbleData, PolarCurveData, PolarHistogram, PolarHistogramData,
    PolarLayer, PolarLine, PolarPlot, PolarRegionAnnotation, PolarState,
};

#[path = "support/export.rs"]
mod export_support;

#[test]
fn stage8d_polar_and_network_charts_export_self_contained_svg_and_png_after_host_drop() {
    let p_curve = PolarCurveData::from_degrees(
        &[0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0],
        &[0.6, 1.4, 1.1, 1.8, 0.9, 1.3, 0.6],
    )
    .unwrap();
    let p_hist = PolarHistogramData::new(&[0.2, 0.3, 1.1, 2.5, 4.0, 5.2], 6).unwrap();
    let p_bubble =
        PolarBubbleData::new(&[0.5, 2.2, 4.4], &[0.8, 1.5, 1.1], &[12.0, 36.0, 20.0]).unwrap();
    let compass = CompassData::from_cartesian(&[0.8, -0.6], &[0.5, 1.0]).unwrap();

    let tri_edges: Vec<[usize; 2]> = (0..7).map(|i| [i, i + 1]).collect();
    let etree = EliminationTree::from_symmetric_edges(8, &tri_edges).unwrap();
    let t_graph = etree.to_tree_data().to_graph();

    let region = PolarRegionAnnotation::polar_region([0.0, TAU / 4.0], [0.5, 1.5]).label("Q1");

    let mut polar_state = PolarState::default();
    let mut graph_state = PlotState::default();
    let snap = export_support::snapshot(|ui| {
        let fig = Figure::new("export_8d")
            .columns(2)
            .cell_height(240.0)
            .show(ui, 2, |cell| match cell.index() {
                0 => cell
                    .show_polar(
                        PolarPlot::new("polar_cell")
                            .title("Polar Export")
                            .regions([region.clone()]),
                        &mut polar_state,
                        &[
                            PolarLayer::from(PolarHistogram::new(&p_hist).name("hist")),
                            PolarLayer::from(
                                PolarLine::new(&p_curve)
                                    .name("curve")
                                    .color(Color32::from_rgb(0, 114, 189)),
                            ),
                            PolarLayer::from(PolarBubble::new(&p_bubble).name("bubble")),
                            PolarLayer::from(CompassPlot::new(&compass).name("compass")),
                        ],
                    )
                    .into_plot_response(),
                _ => cell.show_layers(
                    Plot::new("etree_cell")
                        .title("Elimination Tree")
                        .graph_axes(),
                    &mut graph_state,
                    &[PlotLayer::from(GraphPlot::new(&t_graph).name("etree"))],
                ),
            })
            .unwrap();
        fig.response.rect
    });
    drop((p_curve, p_hist, p_bubble, compass, etree, t_graph));

    let opts = ExportOptions {
        pixels_per_point: 0.5,
        ..Default::default()
    };
    let svg = String::from_utf8(snap.to_svg(opts).unwrap()).unwrap();
    assert!(svg.contains("<svg") && svg.contains("clipPath"));
    assert!(!svg.contains("<text"));

    let png = snap.to_png(opts).unwrap();
    assert!(png.starts_with(b"\x89PNG"));
    let (info, pixels) = export_support::decode_png(&png);
    assert!(info.width > 100 && info.height > 80);
    // Ensure non-uniform pixel content (more than 1 unique RGBA color rendered).
    let (chunks, _) = pixels.as_chunks::<4>();
    let first = chunks[0];
    assert!(chunks.iter().any(|&px| px != first));
}
