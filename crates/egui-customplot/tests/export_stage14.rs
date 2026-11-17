#![cfg(feature = "export")]

use std::collections::HashSet;

use egui::{Context, Pos2, RawInput, Rect};
use egui_customplot::{
    Axis, Biplot, BiplotData, Candlestick, ClusterStandardize, Clustergram, ClustergramData,
    ClustergramState, ControlChart, ControlChartData, EcdfData, EcdfPlot, ExportOptions, Figure,
    LinkageMethod, OhlcData, Plot, PlotLayer, PlotSnapshot, PlotState, ProbabilityPlot,
    ProbabilityPlotData, Waterfall2D, Waterfall2DData, civil_to_unix_secs,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1080.0, 760.0))),
        ..Default::default()
    }
}

#[test]
fn stage14_figure_exports_self_contained_svg_and_non_uniform_png() {
    let prob = ProbabilityPlotData::normal(&[1.2, 1.8, 2.1, 2.3, 2.7, 3.0, 3.4, 3.9]).unwrap();
    let ecdf = EcdfData::from_samples(&[1.0, 2.0, 2.5, 4.0, 5.5, 6.2, 8.0]).unwrap();
    let mat: [&[f64]; 4] = [
        &[2.2, 2.0, -1.1],
        &[1.9, 2.3, -1.3],
        &[-1.0, -1.2, 2.4],
        &[-1.4, -1.0, 2.1],
    ];
    let cg =
        ClustergramData::new(&mat, LinkageMethod::Average, ClusterStandardize::RowZScore).unwrap();
    let pca = BiplotData::from_pca(&[
        &[2.5, 2.4, 1.0],
        &[0.5, 0.7, 0.2],
        &[2.2, 2.9, 1.3],
        &[1.1, 0.9, 0.3],
    ])
    .unwrap();
    let spc = ControlChartData::imr(&[10.1, 10.2, 9.9, 10.3, 13.5, 10.0, 9.8]).unwrap();
    let t0 = civil_to_unix_secs(2026, 3, 2, 0, 0, 0.0).expect("date");
    let ohlc = OhlcData::from_slices(
        &[t0, t0 + 86400.0, t0 + 172800.0],
        &[100.0, 103.0, 101.0],
        &[105.0, 104.5, 106.5],
        &[99.0, 100.5, 100.0],
        &[103.0, 101.0, 106.0],
    )
    .unwrap();
    let wf = Waterfall2DData::from_deltas(
        &[40.0, 15.0, -10.0, 0.0],
        Some(&[false, false, false, true]),
        Some(&["Init", "Up", "Down", "End"]),
    )
    .unwrap();

    let ctx = Context::default();
    let mut ps: [PlotState; 5] = std::array::from_fn(|_| PlotState::default());
    let mut cg_state = ClustergramState::default();
    let mut snapshot = None;

    ctx.run_ui(raw(), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, Default::default(), |ui| {
                let fig = Figure::new("export_stage14")
                    .columns(3)
                    .cell_height(240.0)
                    .show(ui, 6, |cell| match cell.index() {
                        0 => cell.show_layers(
                            Plot::new("prob").probplot_axes(&prob),
                            &mut ps[0],
                            &[PlotLayer::from(ProbabilityPlot::new(&prob))],
                        ),
                        1 => cell.show_layers(
                            Plot::new("ecdf"),
                            &mut ps[1],
                            &[PlotLayer::from(EcdfPlot::new(&ecdf))],
                        ),
                        2 => cell
                            .show_clustergram(Clustergram::new("cg", &cg), &mut cg_state)
                            .into_plot_response(),
                        3 => cell.show_layers(
                            Plot::new("bp").data_aspect(1.0),
                            &mut ps[2],
                            &[PlotLayer::from(Biplot::new(&pca))],
                        ),
                        4 => cell.show_layers(
                            Plot::new("spc"),
                            &mut ps[3],
                            &[PlotLayer::from(ControlChart::new(&spc))],
                        ),
                        _ => cell.show_layers(
                            Plot::new("fin").x_axis(Axis::new().datetime()),
                            &mut ps[4],
                            &[
                                PlotLayer::from(Candlestick::new(&ohlc)),
                                PlotLayer::from(Waterfall2D::new(&wf)),
                            ],
                        ),
                    })
                    .unwrap();
                fig.response.rect
            })
            .unwrap(),
        );
    })
    .drop_without_applying_deltas();

    let opts = ExportOptions {
        pixels_per_point: 0.75,
        ..Default::default()
    };
    let snap = snapshot.unwrap();
    let svg = String::from_utf8(snap.to_svg(opts).unwrap()).unwrap();
    assert!(roxmltree::Document::parse(&svg).is_ok());
    assert!(svg.contains("<svg") && svg.contains("clipPath"));
    assert!(!svg.contains("<text"));

    let png_bytes = snap.to_png(opts).unwrap();
    assert!(png_bytes.starts_with(b"\x89PNG"));
    let decoder = png::Decoder::new(std::io::Cursor::new(&png_bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let unique: HashSet<[u8; 3]> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .take(40_000)
        .map(|px| [px[0], px[1], px[2]])
        .collect();
    assert!(unique.len() >= 4, "expected rich Stage 14 PNG colors");
}
