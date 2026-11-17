#![cfg(feature = "export")]

use egui::{Context, Pos2, RawInput, Rect};
use egui_customplot::{
    AlphaShape3D, AlphaShape3DData, ColorMap, ColorScale, ConfusionChart, ConfusionMatrixData,
    ContourData, ContourLevels, Contours, CurvilinearField2D, CurvilinearPcolor, DepthMode,
    ExportOptions, Figure, Graph3D, Graph3DData, GraphLayout3D, Layer3D, Patch3D, Patch3DData,
    Plot, Plot3D, Plot3DState, PlotLayer, PlotSnapshot, PlotState, Point3D, SkyPlot, SkyPlotData,
    SkyPlotState, SurfaceShading,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1080.0, 720.0))),
        ..Default::default()
    }
}

#[test]
fn stage13_figure_exports_self_contained_svg_and_non_uniform_png() {
    let cm = ConfusionMatrixData::from_counts(&["A", "B", "C"], &[30, 2, 1, 3, 25, 2, 1, 1, 28])
        .unwrap();
    let sky = SkyPlotData::from_az_el(&[20.0, 140.0, 260.0], &[62.0, 25.0, 9.0]).unwrap();
    let (rows, cols) = (8, 12);
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut zs = Vec::new();
    for r in 0..rows {
        let rad = 0.6 + 1.4 * (r as f64 / (rows - 1) as f64);
        for c in 0..cols {
            let th = std::f64::consts::PI * (c as f64 / (cols - 1) as f64);
            xs.push(rad * th.cos());
            ys.push(rad * th.sin());
            zs.push((rad * 2.0).sin());
        }
    }
    let curv = CurvilinearField2D::from_nodes(rows, cols, xs, ys, zs).unwrap();
    let c_lines = ContourData::from_curvilinear(&curv, ContourLevels::Count(5)).unwrap();

    let patch = Patch3DData::new(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.4, 0.0],
            [0.4, 0.4, 0.0],
            [0.4, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        &[&[0, 1, 2, 3, 4, 5]],
    )
    .unwrap();

    let mut sites: Vec<Point3D> = Vec::new();
    for ix in 0..3 {
        for iy in 0..3 {
            for iz in 0..3 {
                sites.push([ix as f64 * 0.5, iy as f64 * 0.5, iz as f64 * 0.5]);
            }
        }
    }
    let crit = AlphaShape3DData::critical_alpha(&sites).unwrap();
    let ash = AlphaShape3DData::from_sites(&sites, crit).unwrap();
    let g3 = Graph3DData::directed(
        4,
        &[[0, 1], [1, 2], [2, 3], [3, 0]],
        GraphLayout3D::default(),
    )
    .unwrap();

    let cscale = ColorScale::new([-1.0, 1.0])
        .unwrap()
        .color_map(ColorMap::Parula);

    let ctx = Context::default();
    let mut ps = [PlotState::default(), PlotState::default()];
    let mut ss = SkyPlotState::default();
    let mut s3 = [
        Plot3DState::default(),
        Plot3DState::default(),
        Plot3DState::default(),
    ];
    let mut snapshot = None;

    ctx.run_ui(raw(), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, Default::default(), |ui| {
                let fig = Figure::new("export_stage13")
                    .columns(3)
                    .cell_height(240.0)
                    .show(ui, 6, |cell| match cell.index() {
                        0 => cell.show_layers(
                            Plot::new("cm").confusion_axes(&cm, true).title("confusion"),
                            &mut ps[0],
                            &[PlotLayer::from(ConfusionChart::new(&cm))],
                        ),
                        1 => cell
                            .show_skyplot(SkyPlot::new("sky", &sky).mask_elevation(15.0), &mut ss)
                            .into_plot_response(),
                        2 => cell.show_layers(
                            Plot::new("curv").data_aspect(1.0).title("curvilinear"),
                            &mut ps[1],
                            &[
                                PlotLayer::from(CurvilinearPcolor::new(&curv, cscale)),
                                PlotLayer::from(Contours::new(&c_lines)),
                            ],
                        ),
                        3 => cell
                            .show_plot3d(
                                Plot3D::new("p3").depth_mode(DepthMode::ZBuffer),
                                &mut s3[0],
                                &[Layer3D::from(
                                    Patch3D::new(&patch)
                                        .shading(SurfaceShading::Faceted)
                                        .specular(20.0, 0.35),
                                )],
                            )
                            .into_plot_response(),
                        4 => cell
                            .show_plot3d(
                                Plot3D::new("ash3").depth_mode(DepthMode::ZBuffer),
                                &mut s3[1],
                                &[Layer3D::from(AlphaShape3D::new(&ash).color_scale(cscale))],
                            )
                            .into_plot_response(),
                        _ => cell
                            .show_plot3d(
                                Plot3D::new("g3"),
                                &mut s3[2],
                                &[Layer3D::from(Graph3D::new(&g3))],
                            )
                            .into_plot_response(),
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
    assert!(
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .any(|px| px[0..3] != bytes[0..3])
    );
}
