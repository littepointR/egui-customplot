mod support;

use egui::{Color32, Context, Event, Shape};
use egui_customplot::{
    ColorMap, ColorScale, Fill, ImageData, Pcolor, PcolorShading, Plot, PlotBounds, PlotImage,
    PlotState, PolygonData, RegionKind, ScalarField2D,
};
use support::{capture, run};

#[test]
fn pcolor_uses_nx_minus_1_cells_and_supports_flat_and_interp_shading() {
    let field = ScalarField2D::new(
        vec![0.0, 1.0, 3.0, 6.0],
        vec![0.0, 2.0, 5.0],
        vec![
            0.0,
            1.0,
            2.0,
            99.0, // row 0 (col 3 is outer right node)
            2.0,
            f64::NAN,
            4.0,
            99.0, // row 1 (col 1 has NaN -> hole)
            99.0,
            99.0,
            99.0,
            99.0, // top row bounds geometry only in Flat
        ],
    )
    .unwrap();
    assert_eq!(field.cell_cols(), 3);
    assert_eq!(field.cell_rows(), 2);

    let scale = ColorScale::new([0.0, 4.0])
        .unwrap()
        .color_map(ColorMap::Gray);
    let ctx = Context::default();
    let mut state = PlotState::default();
    let flat_out = run(&ctx, vec![], false, |ui| {
        Plot::new("pcolor-flat").show_layers(ui, &mut state, &[Pcolor::new(&field, scale).into()])
    });
    assert_eq!(flat_out.diagnostics.images.input_cells, 6);
    assert_eq!(flat_out.diagnostics.images.non_finite_cells, 1);
    assert_eq!(flat_out.diagnostics.images.output_cells, 5);

    // Hover flat cell (2, 1) where value = 4.0 inside [3..6] x [2..5].
    let c21 = flat_out.transform.position_from_point([4.5, 3.5]);
    let hit = run(&ctx, vec![Event::PointerMoved(c21)], false, |ui| {
        Plot::new("pcolor-flat").show_layers(ui, &mut state, &[Pcolor::new(&field, scale).into()])
    });
    let reg = hit.hovered_region.unwrap();
    assert_eq!(reg.kind, RegionKind::PcolorCell);
    assert_eq!(reg.grid_index, Some([2, 1]));
    assert_eq!(reg.scalar_value, Some(4.0));

    // In Interp shading, all 4 cells touching node (1, 1) = NaN are omitted (2 remain).
    let (interp_out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("pcolor-interp").show_layers(
            ui,
            &mut state,
            &[Pcolor::new(&field, scale)
                .shading(PcolorShading::Interp)
                .into()],
        )
    });
    assert_eq!(interp_out.diagnostics.images.output_cells, 2);
    let mesh = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Mesh(m) => Some(m),
            _ => None,
        })
        .unwrap();
    assert!(mesh.vertices.windows(2).any(|w| w[0].color != w[1].color));
    output.drop_without_applying_deltas();
}

#[test]
fn image_constructors_imshow_axes_and_transparent_pixel_hover_passthrough() {
    let bounds = PlotBounds::new([0.0, 0.0], [2.0, 2.0]).unwrap();
    // Row 0 bottom (y 0..1): left opaque blue, right transparent.
    // Row 1 top    (y 1..2): both opaque green.
    let img = ImageData::rgba8(
        2,
        2,
        bounds,
        vec![
            0, 0, 255, 255, 0, 0, 0, 0, // row 0
            0, 255, 0, 255, 0, 255, 0, 255, // row 1
        ],
    )
    .unwrap();
    assert_eq!(img.transparent_pixels(), 1);

    let bg = PolygonData::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]);
    let ctx = Context::default();
    let mut state = PlotState::default();
    let layers = [
        Fill::new(&bg).name("bg").color(Color32::GRAY).into(),
        PlotImage::new(&img).name("img").into(),
    ];
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("imshow")
            .imshow_axes()
            .show_layers(ui, &mut state, &layers)
    });
    // Reversed Y puts y=0 above y=2 in screen space.
    assert!(
        first.transform.position_from_point([0.5, 0.2]).y
            < first.transform.position_from_point([0.5, 1.8]).y
    );
    assert_eq!(first.diagnostics.images.input_cells, 4);
    assert_eq!(first.diagnostics.images.output_cells, 3);

    let opaque_pos = first.transform.position_from_point([0.5, 0.5]);
    let trans_pos = first.transform.position_from_point([1.5, 0.5]);

    let opaque_hit = run(&ctx, vec![Event::PointerMoved(opaque_pos)], false, |ui| {
        Plot::new("imshow")
            .imshow_axes()
            .show_layers(ui, &mut state, &layers)
    });
    let r = opaque_hit.hovered_region.unwrap();
    assert_eq!(r.kind, RegionKind::ImagePixel);
    assert_eq!(r.grid_index, Some([0, 0]));
    assert_eq!(r.rgba, Some([0, 0, 255, 255]));

    // Transparent pixel passes through to underlying Fill.
    let trans_hit = run(&ctx, vec![Event::PointerMoved(trans_pos)], false, |ui| {
        Plot::new("imshow")
            .imshow_axes()
            .show_layers(ui, &mut state, &layers)
    });
    assert_eq!(trans_hit.hovered_region.unwrap().kind, RegionKind::Fill);

    // Verify grayscale and pixel-center expansion helpers.
    let gray = ImageData::grayscale(2, 1, bounds, &[0.0, 10.0], Some([2.0, 6.0])).unwrap();
    assert_eq!(gray.pixel(0, 0), Some([0, 0, 0, 255]));
    assert_eq!(gray.pixel(1, 0), Some([255, 255, 255, 255]));
    let centered =
        ImageData::from_pixel_centers(2, 2, [0.5, 1.5], [0.5, 1.5], vec![255; 16]).unwrap();
    assert_eq!(centered.x_edges(), &[0.0, 1.0, 2.0]);
}
