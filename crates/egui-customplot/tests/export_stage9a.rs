#![cfg(feature = "export")]

use egui::{Color32, Context};
use egui_customplot::{
    Axis3D, Bar3D, Bar3DData, Bar3DMode, Bar3DOrientation, BoxStyle3D, Camera3D, CaptureLimits,
    ConstantPlane3D, DepthMode, ExportOptions, Figure, Fill3D, Layer3D, Line3D, Pie3D, Pie3DData,
    Plot3D, Plot3DState, PlotSnapshot, Point3DData, Polygon3DData, Quiver3D, Quiver3DData,
    Scatter3D, Stem3D, Stem3DData,
};

#[path = "support/export.rs"]
mod export_support;

#[test]
fn stage9a_3d_sorted_and_zbuffer_figures_export_svg_and_png_after_host_drop() {
    let line_pts = Point3DData::from_xyz(
        &[0.0, 1.0, 2.0, 3.0],
        &[0.0, 1.0, 0.2, 1.5],
        &[0.5, 1.2, 2.0, 1.0],
    )
    .unwrap();
    let bar_data = Bar3DData::from_matrix(
        &[1.0, 2.0, 1.5, 2.5],
        2,
        2,
        Bar3DMode::Grouped,
        Bar3DOrientation::Vertical,
        0.7,
    )
    .unwrap();
    let stem_data = Stem3DData::new(vec![[0.5, 0.5, 1.5], [2.5, 1.0, 2.2]], 0.0).unwrap();
    let quiver_data = Quiver3DData::from_components(
        &[0.5, 1.5],
        &[0.5, 0.5],
        &[0.5, 0.8],
        &[0.5, 0.3],
        &[0.4, 0.6],
        &[0.6, 0.4],
    )
    .unwrap();

    let quad_a = [
        [-0.8, -0.8, -0.8],
        [0.8, -0.8, 0.8],
        [0.8, 0.8, 0.8],
        [-0.8, 0.8, -0.8],
    ];
    let quad_b = [
        [-0.8, -0.8, 0.8],
        [0.8, -0.8, -0.8],
        [0.8, 0.8, -0.8],
        [-0.8, 0.8, 0.8],
    ];
    let poly_data = Polygon3DData::from_polygons(&[&quad_a, &quad_b]).unwrap();
    let pie_data = Pie3DData::new(&[40.0, 35.0, 25.0]).unwrap();

    let mut st0 = Plot3DState::default();
    let mut st1 = Plot3DState::default();
    let snap = export_support::snapshot(|ui| {
        let fig = Figure::new("export_9a")
            .columns(2)
            .cell_height(260.0)
            .show(ui, 2, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("sorted_cell")
                            .title("SortedPainter")
                            .depth_mode(DepthMode::SortedPainter),
                        &mut st0,
                        &[
                            Layer3D::from(Bar3D::new(&bar_data).name("bars")),
                            Layer3D::from(Line3D::new(&line_pts).name("line3")),
                            Layer3D::from(Scatter3D::new(&line_pts).name("scatter3")),
                            Layer3D::from(Stem3D::new(&stem_data).name("stem3")),
                            Layer3D::from(Quiver3D::new(&quiver_data).name("quiver3")),
                        ],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("zbuf_cell")
                            .title("ZBuffer")
                            .depth_mode(DepthMode::ZBuffer)
                            .constant_planes([ConstantPlane3D::z_plane(0.0).label("z=0")]),
                        &mut st1,
                        &[
                            Layer3D::from(
                                Fill3D::new(&poly_data)
                                    .face_colors(&[
                                        Color32::from_rgb(230, 50, 50),
                                        Color32::from_rgb(40, 80, 230),
                                    ])
                                    .name("fill3"),
                            ),
                            Layer3D::from(Pie3D::new(&pie_data).name("pie3")),
                        ],
                    )
                    .into_plot_response(),
            })
            .unwrap();
        fig.response.rect
    });
    drop((
        line_pts,
        bar_data,
        stem_data,
        quiver_data,
        poly_data,
        pie_data,
    ));

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
    let (chunks, _) = pixels.as_chunks::<4>();
    let first = chunks[0];
    assert!(chunks.iter().any(|&px| px != first));
}

#[test]
fn zbuffer_resolves_mutually_intersecting_3d_quads_per_pixel() {
    // Quad A sits on z = x (Red); Quad B sits on z = -x (Blue).
    // From Camera3D::view_2d_xy() (+X right, +Z toward camera):
    // - At x < 0: z_Blue > z_Red => Blue must win at every pixel on the left.
    // - At x > 0: z_Red > z_Blue => Red must win at every pixel on the right.
    // Notice Painter's sort alone cannot split intersecting polygons and draws all of B over A!
    let quad_red = [
        [-0.8, -0.8, -0.8],
        [0.8, -0.8, 0.8],
        [0.8, 0.8, 0.8],
        [-0.8, 0.8, -0.8],
    ];
    let quad_blue = [
        [-0.8, -0.8, 0.8],
        [0.8, -0.8, -0.8],
        [0.8, 0.8, -0.8],
        [-0.8, 0.8, 0.8],
    ];
    let poly_red = Polygon3DData::from_polygons(&[&quad_red]).unwrap();
    let poly_blue = Polygon3DData::from_polygons(&[&quad_blue]).unwrap();

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let mut captured = None;
    let mut resp = None;
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 280.0),
            )),
            ..Default::default()
        },
        |ui| {
            captured = Some(
                PlotSnapshot::capture(ui, CaptureLimits::default(), |ui| {
                    let r = Plot3D::new("zbuffer_intersect")
                        .height(260.0)
                        .camera(Camera3D::view_2d_xy())
                        .box_style(BoxStyle3D::AxesOnly)
                        .depth_mode(DepthMode::ZBuffer)
                        .show_legend(false)
                        .x_axis(Axis3D::new().bounds(-1.0, 1.0).visible(false).grid(false))
                        .y_axis(Axis3D::new().bounds(-1.0, 1.0).visible(false).grid(false))
                        .z_axis(Axis3D::new().bounds(-1.0, 1.0).visible(false).grid(false))
                        .show(
                            ui,
                            &mut state,
                            &[
                                Layer3D::from(
                                    Fill3D::new(&poly_red)
                                        .color(Color32::from_rgb(240, 20, 20))
                                        .border(None)
                                        .alpha(1.0),
                                ),
                                Layer3D::from(
                                    Fill3D::new(&poly_blue)
                                        .color(Color32::from_rgb(20, 60, 240))
                                        .border(None)
                                        .alpha(1.0),
                                ),
                            ],
                        );
                    let outer = r.outer_rect;
                    resp = Some(r);
                    outer
                })
                .unwrap(),
            );
        },
    )
    .drop_without_applying_deltas();

    let resp = resp.unwrap();
    assert!(resp.diagnostics.plot3d.zbuffer_pixels > 500);
    assert!(resp.diagnostics.plot3d.zbuffer_spans > 20);

    let snap = captured.unwrap();
    let opts = ExportOptions {
        pixels_per_point: 1.0,
        background: Color32::WHITE,
        ..Default::default()
    };
    let png = snap.to_png(opts).unwrap();
    let (info, rgba) = export_support::decode_png(&png);
    let rect = snap.logical_rect();

    let left_pt = resp.transform.project_data([-0.45, 0.0, 0.0]).unwrap().0;
    let right_pt = resp.transform.project_data([0.45, 0.0, 0.0]).unwrap().0;
    let to_px = |p: egui::Pos2| -> (usize, usize) {
        let px = ((p.x - rect.left()).round() as usize).min(info.width as usize - 1);
        let py = ((p.y - rect.top()).round() as usize).min(info.height as usize - 1);
        (px, py)
    };
    let (lx, ly) = to_px(left_pt);
    let (rx, ry) = to_px(right_pt);
    let left_rgba = export_support::pixel(&rgba, info.width as usize, lx, ly);
    let right_rgba = export_support::pixel(&rgba, info.width as usize, rx, ry);

    assert!(
        i16::from(left_rgba[2]) > i16::from(left_rgba[0]) + 60,
        "x < 0 must show Blue in front (z_blue > z_red), got {left_rgba:?}"
    );
    assert!(
        i16::from(right_rgba[0]) > i16::from(right_rgba[2]) + 60,
        "x > 0 must show Red in front (z_red > z_blue), got {right_rgba:?}"
    );
}
