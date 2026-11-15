#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use egui::{Color32, Stroke};
use egui_customplot::{
    ColorMap, ColorScale, ContourLevels, ExportOptions, FilledContourData, FilledContours,
    ImageData, Patch, PatchData, Pcolor, PcolorShading, Plot, PlotBounds, PlotImage, PlotState,
    PolyRegion, PolyShape, PolyShapeData, ScalarField2D,
};
use export_support::{decode_png, pixel, render_svg, snapshot};

#[test]
fn polyshape_hole_is_white_in_png_and_resvg_and_emits_svg_vectors_without_text() {
    let region = PolyRegion::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .hole(vec![[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]]);
    let data = PolyShapeData::new(vec![region]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0, 0.0], [10.0, 10.0]).unwrap());
    let fill_color = Color32::from_rgb(20, 110, 210);

    let mut resp = None;
    let snap = snapshot(|ui| {
        let r = Plot::new("hole-export")
            .show_grid(false)
            .background(Color32::WHITE)
            .show_layers(
                ui,
                &mut state,
                &[PolyShape::new(&data)
                    .color(fill_color)
                    .border(Stroke::NONE)
                    .into()],
            );
        let outer = r.outer_rect;
        resp = Some(r);
        outer
    });

    let r = resp.unwrap();
    let options = ExportOptions {
        background: Color32::WHITE,
        pixels_per_point: 1.0,
        ..Default::default()
    };
    let svg = String::from_utf8(snap.to_svg(options).unwrap()).unwrap();
    assert!(!svg.contains("<text"));
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let path_count = doc.descendants().filter(|n| n.has_tag_name("path")).count();
    assert!(
        path_count >= 4,
        "expected vector triangles, got {path_count}"
    );

    let (png_info, png_rgba) = decode_png(&snap.to_png(options).unwrap());
    let svg_rgba = render_svg(svg.as_bytes(), [png_info.width, png_info.height]);

    let hole_pos = r.transform.position_from_point([5.0, 5.0]) - r.outer_rect.min;
    let ring_pos = r.transform.position_from_point([1.5, 5.0]) - r.outer_rect.min;
    let (hx, hy) = (hole_pos.x.round() as usize, hole_pos.y.round() as usize);
    let (rx, ry) = (ring_pos.x.round() as usize, ring_pos.y.round() as usize);
    let w = png_info.width as usize;

    assert_eq!(pixel(&png_rgba, w, hx, hy), [255, 255, 255, 255]);
    assert_eq!(pixel(&svg_rgba, w, hx, hy), [255, 255, 255, 255]);
    assert_eq!(pixel(&png_rgba, w, rx, ry), [20, 110, 210, 255]);
    assert_eq!(pixel(&svg_rgba, w, rx, ry), [20, 110, 210, 255]);
}

#[test]
fn gouraud_patch_and_interp_pcolor_use_svg_raster_fallback_while_flat_contourf_uses_paths() {
    let patch_data = PatchData::new(
        vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]],
        &[&[0, 1, 2, 3]],
    )
    .unwrap();
    let vcolors = [Color32::RED, Color32::GREEN, Color32::BLUE, Color32::YELLOW];
    let patch = Patch::new(&patch_data).vertex_colors(&vcolors).unwrap();

    let field = ScalarField2D::sample_function(
        PlotBounds::new([0.0, 0.0], [4.0, 4.0]).unwrap(),
        [5, 5],
        100,
        |x, y| x + y,
    )
    .unwrap();
    let scale = ColorScale::new([0.0, 8.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    let pcolor = Pcolor::new(&field, scale).shading(PcolorShading::Interp);
    let bands = FilledContourData::from_field(&field, ContourLevels::Count(3)).unwrap();
    let contourf = FilledContours::new(&bands, scale);
    let img_data = ImageData::rgba8(
        2,
        2,
        PlotBounds::new([0.0, 0.0], [2.0, 2.0]).unwrap(),
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ],
    )
    .unwrap();
    let img_layer = PlotImage::new(&img_data);

    let capture_layer = |id: &str, layer: egui_customplot::PlotLayer<'_>| -> String {
        let mut state = PlotState::default();
        let snap = snapshot(|ui| {
            Plot::new(id)
                .background(Color32::WHITE)
                .show_layers(ui, &mut state, &[layer])
                .outer_rect
        });
        let options = ExportOptions {
            background: Color32::WHITE,
            ..Default::default()
        };
        assert!(snap.to_png(options).unwrap().starts_with(b"\x89PNG"));
        String::from_utf8(snap.to_svg(options).unwrap()).unwrap()
    };

    let patch_svg = capture_layer("patch-gouraud", patch.into());
    let patch_doc = roxmltree::Document::parse(&patch_svg).unwrap();
    assert!(
        patch_doc.descendants().any(|n| n.has_tag_name("image")),
        "Gouraud patch should trigger embedded raster fallback in SVG"
    );
    assert!(!patch_svg.contains("<text"));

    let pcolor_svg = capture_layer("pcolor-interp", pcolor.into());
    let pcolor_doc = roxmltree::Document::parse(&pcolor_svg).unwrap();
    assert!(pcolor_doc.descendants().any(|n| n.has_tag_name("image")));
    assert!(!pcolor_svg.contains("<text"));

    let contour_svg = capture_layer("contourf-flat", contourf.into());
    let contour_doc = roxmltree::Document::parse(&contour_svg).unwrap();
    assert!(
        contour_doc
            .descendants()
            .filter(|n| n.has_tag_name("path"))
            .count()
            > 4
    );
    assert!(!contour_svg.contains("<text"));

    let img_svg = capture_layer("img-flat", img_layer.into());
    let img_doc = roxmltree::Document::parse(&img_svg).unwrap();
    assert!(
        img_doc
            .descendants()
            .filter(|n| n.has_tag_name("path"))
            .count()
            >= 4
    );
    assert!(!img_svg.contains("<text"));
}
