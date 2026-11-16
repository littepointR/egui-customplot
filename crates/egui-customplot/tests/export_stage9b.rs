#![cfg(feature = "export")]

use egui::{Color32, Context};
use egui_customplot::{
    Axis3D, Bounds3D, BoxStyle3D, Camera3D, CaptureLimits, ColorMap, ColorScale, Contour3D,
    Contour3DData, ContourLevels, DepthMode, ExportOptions, Figure, ImplicitSurface3DData, Layer3D,
    Plot3D, Plot3DState, PlotSnapshot, Surface3D, Surface3DData, SurfaceShading, TriSurf3D,
};

#[path = "support/export.rs"]
mod export_support;

#[test]
fn stage9b_sorted_and_zbuffer_surface_figures_export_after_host_drop() {
    let peaks =
        Surface3DData::from_explicit_fn([-2.0, 2.0], [-2.0, 2.0], [18, 18], 2_000, |x, y| {
            (x * x + y * y).sqrt().cos()
        })
        .unwrap()
        .with_contours(ContourLevels::Count(5))
        .unwrap();
    let c3 = Contour3DData::from_surface(&peaks, ContourLevels::Count(5)).unwrap();
    let cs = ColorScale::new([-1.0, 1.0]).unwrap();

    let sph = Surface3DData::sphere(14).unwrap();
    let cyl = Surface3DData::cylinder(&[0.6, 0.8, 0.4], 14).unwrap();
    let imp = ImplicitSurface3DData::from_function(
        Bounds3D::new([-1.4; 3], [1.4; 3]).unwrap(),
        [14, 14, 14],
        10_000,
        |x, y, z| x * x + y * y + z * z - 0.8,
    )
    .unwrap();

    let mut st0 = Plot3DState::default();
    let mut st1 = Plot3DState::default();
    let snap = export_support::snapshot(|ui| {
        let fig = Figure::new("export_9b")
            .columns(2)
            .cell_height(260.0)
            .show(ui, 2, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("surfc_sorted")
                            .title("Sorted surfc")
                            .depth_mode(DepthMode::SortedPainter)
                            .color_bar(cs, "Z"),
                        &mut st0,
                        &[
                            Layer3D::from(
                                Surface3D::new(&peaks)
                                    .name("peaks")
                                    .shading(SurfaceShading::Interp)
                                    .color_scale(cs)
                                    .floor_contours(true),
                            ),
                            Layer3D::from(Contour3D::new(&c3).name("c3")),
                        ],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("implicit_zbuf")
                            .title("ZBuffer implicit")
                            .depth_mode(DepthMode::ZBuffer),
                        &mut st1,
                        &[
                            Layer3D::from(TriSurf3D::from_implicit(&imp).name("imp")),
                            Layer3D::from(
                                Surface3D::new(&sph)
                                    .name("sphere")
                                    .shading(SurfaceShading::Flat)
                                    .wireframe_stroke(None)
                                    .alpha(0.5),
                            ),
                            Layer3D::from(
                                Surface3D::new(&cyl)
                                    .name("cylinder")
                                    .shading(SurfaceShading::Flat),
                            ),
                        ],
                    )
                    .into_plot_response(),
            });
        fig.unwrap().response.rect
    });

    drop(peaks);
    drop(c3);
    drop(sph);
    drop(cyl);
    drop(imp);

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
    let unique_colors = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[..3] != [255, 255, 255])
        .count();
    assert!(
        unique_colors > 500,
        "exported PNG must contain rendered 3D surfaces"
    );
}

#[test]
fn gouraud_interp_shading_smoothly_interpolates_vertex_colors_in_zbuffer() {
    let flat_quad = Surface3DData::from_grid(&[-1.0, 1.0], &[-1.0, 1.0], &[0.0, 0.0, 0.0, 0.0])
        .unwrap()
        .with_scalars(&[0.0, 1.0, 0.0, 1.0])
        .unwrap();
    let cs = ColorScale::new([0.0, 1.0])
        .unwrap()
        .color_map(ColorMap::BlueRed);

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let mut captured = None;
    let mut resp = None;

    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(480.0, 320.0),
            )),
            ..Default::default()
        },
        |ui| {
            captured = Some(
                PlotSnapshot::capture(ui, CaptureLimits::default(), |ui| {
                    let r = Plot3D::new("gouraud_zbuffer")
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
                            &[Layer3D::from(
                                Surface3D::new(&flat_quad)
                                    .shading(SurfaceShading::Interp)
                                    .color_scale(cs)
                                    .light(None)
                                    .wireframe_stroke(None)
                                    .alpha(1.0),
                            )],
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
    let snap = captured.unwrap();
    let png = snap
        .to_png(ExportOptions {
            pixels_per_point: 1.0,
            background: Color32::WHITE,
            ..Default::default()
        })
        .unwrap();
    let (info, rgba) = export_support::decode_png(&png);
    let rect = snap.logical_rect();

    let px_at_x = |x: f64| -> [u8; 4] {
        let p = resp.transform.project_data([x, 0.0, 0.0]).unwrap().0;
        let px = ((p.x - rect.left()).round() as usize).min(info.width as usize - 1);
        let py = ((p.y - rect.top()).round() as usize).min(info.height as usize - 1);
        export_support::pixel(&rgba, info.width as usize, px, py)
    };

    let left = px_at_x(-0.7);
    let mid = px_at_x(0.0);
    let right = px_at_x(0.7);

    assert!(
        i16::from(left[2]) > i16::from(left[0]) + 50,
        "left blue: {left:?}"
    );
    assert!(
        i16::from(right[0]) > i16::from(right[2]) + 50,
        "right red: {right:?}"
    );
    assert!(
        mid[0].min(mid[1]).min(mid[2]) < 150,
        "mid rendered, not white: {mid:?}"
    );
    assert!(
        left[0] + 25 < mid[0] && mid[0] + 25 < right[0],
        "expected smooth red ramp across Gouraud quad: left={left:?}, mid={mid:?}, right={right:?}"
    );
    assert!(
        left[2] > mid[2] + 25 && mid[2] > right[2] + 25,
        "expected smooth blue ramp across Gouraud quad: left={left:?}, mid={mid:?}, right={right:?}"
    );
}
