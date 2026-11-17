#![cfg(feature = "export")]

use std::{collections::HashSet, f64::consts::TAU};

use egui::{Context, Pos2, RawInput, Rect};
use egui_customplot::{
    BodePlot, BodePlotState, ColorMap, ColorScale, CwtData, CwtPlot, ExportOptions, Figure,
    FrequencyResponseData, NicholsPlot, NyquistPlot, Plot, PlotLayer, PlotSnapshot, PlotState,
    PoleZeroData, RootLocus, RootLocusData, SmithPlot, SmithPlotData, SmithPlotState,
    TransferFunction, ZPlanePlot,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1080.0, 760.0))),
        ..Default::default()
    }
}

#[test]
fn stage15_figure_exports_self_contained_svg_and_non_uniform_png() {
    let freqs = [1e9, 1.5e9, 2.0e9, 2.5e9, 3.0e9];
    let z = [
        [25.0, -30.0],
        [42.0, -10.0],
        [50.0, 0.0],
        [68.0, 22.0],
        [95.0, 48.0],
    ];
    let smith = SmithPlotData::from_impedance(&freqs, &z, 50.0)
        .unwrap()
        .with_vswr_circles(&[2.0]);
    let zp =
        PoleZeroData::from_zeros_poles(&[[-1.0, 0.0], [0.0, 0.7]], &[[0.55, 0.5], [0.55, -0.5]])
            .unwrap();
    let plant = TransferFunction::new(&[1.6], &[1.0, 1.8, 3.05, 2.25]).unwrap();
    let fr = FrequencyResponseData::from_transfer_fn(&plant, 0.1, 20.0, 120).unwrap();
    let rl = RootLocusData::auto_gains(&plant, 12.0, 48).unwrap();

    let fs = 128.0;
    let sig: Vec<f64> = (0..128)
        .map(|i| {
            let t = i as f64 / fs;
            (TAU * 16.0 * t).sin() + 0.6 * (TAU * 36.0 * t).cos()
        })
        .collect();
    let cwt = CwtData::morlet_logspace(&sig, fs, 6.0, 56.0, 16).unwrap();
    let cscale = ColorScale::new(cwt.value_range())
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut smith_state = SmithPlotState::default();
    let mut bode_state = BodePlotState::default();
    let mut ps: [PlotState; 4] = std::array::from_fn(|_| PlotState::default());
    let mut snapshot = None;

    ctx.run_ui(raw(), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, Default::default(), |ui| {
                let fig = Figure::new("export_stage15")
                    .columns(3)
                    .cell_height(240.0)
                    .show(ui, 6, |cell| match cell.index() {
                        0 => cell
                            .show_smithplot(SmithPlot::new("sm", &smith), &mut smith_state)
                            .into_plot_response(),
                        1 => cell.show_layers(
                            Plot::new("zp").zplane_axes(),
                            &mut ps[0],
                            &[PlotLayer::from(ZPlanePlot::new(&zp))],
                        ),
                        2 => cell
                            .show_bode(BodePlot::new("bd", &fr), &mut bode_state)
                            .into_plot_response(),
                        3 => cell.show_layers(
                            Plot::new("nyq").data_aspect(1.0),
                            &mut ps[1],
                            &[PlotLayer::from(NyquistPlot::new(&fr))],
                        ),
                        4 => cell.show_layers(
                            Plot::new("rl"),
                            &mut ps[2],
                            &[
                                PlotLayer::from(NicholsPlot::new(&fr).show_grid(false)),
                                PlotLayer::from(RootLocus::new(&rl)),
                            ],
                        ),
                        _ => cell.show_layers(
                            Plot::new("cwt"),
                            &mut ps[3],
                            &[PlotLayer::from(CwtPlot::new(&cwt, cscale))],
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
    assert!(unique.len() >= 4, "expected rich Stage 15 PNG colors");
}
