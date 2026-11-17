#![cfg(feature = "export")]

use std::{collections::HashSet, f64::consts::TAU};

use egui::{Color32, Context, Pos2, RawInput, Rect};
use egui_customplot::{
    Bounds3D, ColorMap, DepthMode, ExportOptions, Figure, GanttChart, GanttData, GanttTask,
    GeoGlobeData, GeoGlobePlot, GeoGlobeState, GlobeMarker3D, Layer3D, Plot, Plot3D, Plot3DState,
    PlotLayer, PlotSnapshot, PlotState, PointCloud3D, PointCloud3DData, RadarChart, RadarData,
    SankeyChart, SankeyData, ScalarVolume3D, SunburstChart, SunburstData, SunburstInput,
    TreemapChart, TreemapData, VolumeRender3D, VolumeRenderMode, VolumeTransferFunction,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 680.0))),
        ..Default::default()
    }
}

#[test]
fn stage16_figure_exports_self_contained_svg_and_non_uniform_png() {
    let bounds = Bounds3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [20, 20, 20], 20_000, |x, y, z| {
        (-(x * x + y * y + z * z) * 2.5).exp()
    })
    .unwrap();
    let tf = VolumeTransferFunction::ramp(ColorMap::Plasma, 0.0, 0.85);

    let pts: Vec<[f64; 3]> = (0..12_000)
        .map(|i| {
            let t = i as f64 / 12_000.0;
            let a = 10.0 * TAU * t;
            [
                (0.2 + 0.8 * t) * a.cos(),
                (0.2 + 0.8 * t) * a.sin(),
                2.0 * t - 1.0,
            ]
        })
        .collect();
    let cloud = PointCloud3DData::from_xyz(pts).unwrap();

    let globe = GeoGlobeData::new()
        .with_extruded_track(
            "Orbit",
            &[15.0, 32.0, 48.0],
            &[85.0, 105.0, 125.0],
            &[150_000.0, 480_000.0, 220_000.0],
            Color32::from_rgb(255, 165, 45),
        )
        .unwrap()
        .with_markers(vec![GlobeMarker3D {
            label: "Center".into(),
            lat_deg: 31.2,
            lon_deg: 121.5,
            alt_m: 10_000.0,
            radius_px: 5.0,
            color: Color32::LIGHT_BLUE,
        }])
        .unwrap();

    let radar = RadarData::new(
        &["A", "B", "C", "D", "E"],
        &[10.0; 5],
        &[("S1", &[8.0, 6.0, 9.0, 7.0, 8.5])],
    )
    .unwrap();
    let sankey = SankeyData::new(
        &["Src1", "Src2", "Mid", "Out1", "Out2"],
        &[(0, 2, 12.0), (1, 2, 8.0), (2, 3, 14.0), (2, 4, 6.0)],
    )
    .unwrap();
    let treemap =
        TreemapData::squarified(&[("Alpha", 0, 5.0), ("Beta", 1, 3.0), ("Gamma", 2, 2.0)]).unwrap();
    let sunburst = SunburstData::new(&[
        SunburstInput {
            label: "Root".into(),
            parent: None,
            value: 0.0,
            group: 0,
        },
        SunburstInput {
            label: "Left".into(),
            parent: Some(0),
            value: 6.0,
            group: 1,
        },
        SunburstInput {
            label: "Right".into(),
            parent: Some(0),
            value: 4.0,
            group: 2,
        },
    ])
    .unwrap();
    let gantt = GanttData::new(
        vec![
            GanttTask {
                label: "T1".into(),
                start: 0.0,
                end: 4.0,
                progress: 1.0,
                group: 0,
                milestone: false,
            },
            GanttTask {
                label: "T2".into(),
                start: 3.0,
                end: 8.0,
                progress: 0.5,
                group: 1,
                milestone: false,
            },
        ],
        vec![[0, 1]],
    )
    .unwrap();

    let ctx = Context::default();
    let mut s_vol = Plot3DState::default();
    let mut s_pc = Plot3DState::default();
    let mut s_globe = GeoGlobeState::default();
    let mut ps: [PlotState; 5] = std::array::from_fn(|_| PlotState::default());
    let mut snapshot = None;

    ctx.run_ui(raw(), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, Default::default(), |ui| {
                let fig = Figure::new("export_stage16")
                    .columns(4)
                    .cell_height(260.0)
                    .show(ui, 8, |cell| match cell.index() {
                        0 => cell
                            .show_plot3d(
                                Plot3D::new("vol").depth_mode(DepthMode::ZBuffer),
                                &mut s_vol,
                                &[Layer3D::from(
                                    VolumeRender3D::new(&vol)
                                        .transfer(tf)
                                        .mode(VolumeRenderMode::DirectAlpha),
                                )],
                            )
                            .into_plot_response(),
                        1 => cell
                            .show_plot3d(
                                Plot3D::new("pc").depth_mode(DepthMode::ZBuffer),
                                &mut s_pc,
                                &[Layer3D::from(PointCloud3D::new(&cloud).max_points(4_000))],
                            )
                            .into_plot_response(),
                        2 => cell
                            .show_geoglobe(GeoGlobePlot::new("globe", &globe), &mut s_globe)
                            .into_plot_response(),
                        3 => cell.show_layers(
                            Plot::new("radar").canvas_axes(),
                            &mut ps[0],
                            &[PlotLayer::from(RadarChart::new(&radar))],
                        ),
                        4 => cell.show_layers(
                            Plot::new("sankey").canvas_axes(),
                            &mut ps[1],
                            &[PlotLayer::from(SankeyChart::new(&sankey))],
                        ),
                        5 => cell.show_layers(
                            Plot::new("treemap").canvas_axes(),
                            &mut ps[2],
                            &[PlotLayer::from(TreemapChart::new(&treemap))],
                        ),
                        6 => cell.show_layers(
                            Plot::new("sunburst").canvas_axes(),
                            &mut ps[3],
                            &[PlotLayer::from(SunburstChart::new(&sunburst))],
                        ),
                        _ => cell.show_layers(
                            Plot::new("gantt").gantt_axes(&gantt),
                            &mut ps[4],
                            &[PlotLayer::from(GanttChart::new(&gantt))],
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
        .take(50_000)
        .map(|px| [px[0], px[1], px[2]])
        .collect();
    assert!(
        unique.len() >= 8,
        "expected rich Stage 16 PNG colors, got {}",
        unique.len()
    );
}
