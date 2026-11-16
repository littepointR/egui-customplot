#![cfg(feature = "export")]

use egui::Color32;
use egui_customplot::{
    Bounds3D, ColorScale, ConePlot3D, ConePlot3DData, Contour3D, Contour3DData, ContourLevels,
    DepthMode, ExportOptions, Figure, IsocapThreshold, Isocaps3D, Isocaps3DData, Isosurface3D,
    Isosurface3DData, Layer3D, Plot3D, Plot3DState, ScalarVolume3D, SlicePlane3D, Stream3DOptions,
    StreamTube3D, StreamTube3DData, VectorField3D, VolumeSlice3D, VolumeSliceData,
};

#[path = "support/export.rs"]
mod export_support;

#[test]
fn stage9c_sorted_and_zbuffer_volume_flow_figures_export_after_host_drop() {
    let bounds = Bounds3D::new([-0.7, -0.7, -0.7], [1.3, 1.3, 1.3]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [16, 16, 16], 10_000, |x, y, z| {
        x * x + y * y + z * z
    })
    .unwrap();
    let slices =
        VolumeSliceData::from_volume(&vol, &[SlicePlane3D::X(0.0), SlicePlane3D::Z(0.0)]).unwrap();
    let cslice =
        Contour3DData::from_volume_slices(&vol, &[SlicePlane3D::Z(0.0)], ContourLevels::Count(5))
            .unwrap();

    let iso = Isosurface3DData::from_volume(&vol, 1.0).unwrap();
    let caps = Isocaps3DData::from_volume(&vol, 1.0, IsocapThreshold::Below).unwrap();
    let vf = VectorField3D::from_function(bounds, [10, 10, 10], 5_000, |x, y, _| {
        [-y + 0.1, x + 0.1, 1.0]
    })
    .unwrap();
    let tubes = StreamTube3DData::from_field(
        &vf,
        &[[0.2, 0.1, -0.6]],
        0.12,
        10,
        Stream3DOptions {
            max_steps: 45,
            ..Default::default()
        },
    )
    .unwrap();
    let cones =
        ConePlot3DData::from_field(&vf, &[[0.4, 0.4, 0.2], [0.8, 0.2, 0.5]], 0.25, 8).unwrap();

    let cs = ColorScale::new([0.0, 4.0]).unwrap();
    let mut st0 = Plot3DState::default();
    let mut st1 = Plot3DState::default();
    let snap = export_support::snapshot(|ui| {
        let fig = Figure::new("export_9c")
            .columns(2)
            .cell_height(260.0)
            .show(ui, 2, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("slice_sorted")
                            .title("Sorted slices")
                            .depth_mode(DepthMode::SortedPainter)
                            .color_bar(cs, "V"),
                        &mut st0,
                        &[
                            Layer3D::from(
                                VolumeSlice3D::new(&slices).name("slice").color_scale(cs),
                            ),
                            Layer3D::from(Contour3D::new(&cslice).name("contourslice")),
                        ],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("iso_zbuf")
                            .title("ZBuffer iso+flow")
                            .depth_mode(DepthMode::ZBuffer),
                        &mut st1,
                        &[
                            Layer3D::from(Isosurface3D::new(&iso).name("iso")),
                            Layer3D::from(Isocaps3D::new(&caps).name("caps").color_scale(cs)),
                            Layer3D::from(StreamTube3D::new(&tubes).name("tubes")),
                            Layer3D::from(ConePlot3D::new(&cones).name("cones")),
                        ],
                    )
                    .into_plot_response(),
            });
        fig.unwrap().response.rect
    });

    drop((vol, slices, cslice, iso, caps, vf, tubes, cones));

    let opts = ExportOptions {
        pixels_per_point: 1.0,
        background: Color32::WHITE,
        ..Default::default()
    };
    let svg = String::from_utf8(snap.to_svg(opts).unwrap()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("clipPath"));
    assert!(!svg.contains("<text"));
    assert!(svg.contains("data:image/png;base64,"));

    let png = snap.to_png(opts).unwrap();
    let (info, rgba) = export_support::decode_png(&png);
    assert!(info.width > 200 && info.height > 150);
    let non_white = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[..3] != [255, 255, 255])
        .count();
    assert!(
        non_white > 500,
        "exported PNG must contain rendered 3D volumes"
    );
}
