#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use egui::{Color32, Mesh, Rect, Shape, Stroke, pos2, vec2};
use egui_customplot::ExportOptions;
use export_support::{decode_png, pixel, png, render_svg, snapshot};

fn rect() -> Rect {
    Rect::from_min_size(pos2(10.0, 20.0), vec2(32.0, 24.0))
}

#[test]
fn png_dimensions_dpi_srgb_transparency_and_shared_edges_are_real_pixels() {
    let snapshot = snapshot(|ui| {
        let r = rect();
        let mut mesh = Mesh::default();
        mesh.add_colored_rect(
            r.shrink(4.0),
            Color32::from_rgba_unmultiplied(255, 0, 0, 128),
        );
        ui.painter().add(Shape::mesh(mesh));
        r
    });
    for ppp in [1.0, 1.5, 2.0] {
        let bytes = snapshot
            .to_png(ExportOptions {
                pixels_per_point: ppp,
                dpi: 144.0,
                ..Default::default()
            })
            .unwrap();
        let (info, rgba) = decode_png(&bytes);
        assert_eq!(
            [info.width, info.height],
            [(32.0 * ppp) as u32, (24.0 * ppp) as u32]
        );
        assert_eq!(info.color_type, png::ColorType::Rgba);
        assert_eq!(pixel(&rgba, info.width as usize, 0, 0), [0; 4]);
        // Both triangles meet through the middle: no crack or double alpha.
        for i in 8..16 {
            assert_eq!(
                pixel(
                    &rgba,
                    info.width as usize,
                    (i as f32 * ppp) as usize,
                    (i as f32 * ppp) as usize
                ),
                [255, 0, 0, 128]
            );
        }
        let reader = png::Decoder::new(std::io::Cursor::new(&bytes))
            .read_info()
            .unwrap();
        assert_eq!(reader.info().pixel_dims.unwrap().xppu, 5669);
        assert_eq!(
            reader.info().srgb,
            Some(png::SrgbRenderingIntent::Perceptual)
        );
    }
}

#[test]
fn clipping_layer_order_and_background_apply_to_both_real_backends() {
    let snapshot = snapshot(|ui| {
        let r = rect();
        ui.painter().rect_filled(r.shrink(3.0), 0.0, Color32::RED);
        let clip = Rect::from_min_max(pos2(26.0, 20.0), r.max);
        ui.painter().with_clip_rect(clip).rect_filled(
            r,
            0.0,
            Color32::from_rgba_unmultiplied(0, 0, 255, 128),
        );
        r
    });
    let options = ExportOptions {
        background: Color32::WHITE,
        ..Default::default()
    };
    let (_, png) = decode_png(&snapshot.to_png(options).unwrap());
    let svg = render_svg(&snapshot.to_svg(options).unwrap(), [32, 24]);
    for output in [&png, &svg] {
        assert_eq!(pixel(output, 32, 1, 1), [255; 4]);
        assert_eq!(pixel(output, 32, 8, 8), [255, 0, 0, 255]);
        let overlap = pixel(output, 32, 24, 12);
        assert!(
            overlap[0].abs_diff(127) <= 1
                && overlap[1] == 0
                && overlap[2].abs_diff(128) <= 1
                && overlap[3] == 255,
            "{overlap:?}"
        );
    }
}

#[test]
fn mixed_winding_gradient_triangles_keep_interpolation_and_svg_raster_fallback() {
    let snapshot = snapshot(|ui| {
        let mut mesh = Mesh::default();
        for (p, c) in [
            (pos2(10.0, 20.0), Color32::RED),
            (pos2(42.0, 20.0), Color32::BLUE),
            (pos2(42.0, 44.0), Color32::BLUE),
            (pos2(10.0, 44.0), Color32::RED),
        ] {
            mesh.colored_vertex(p, c);
        }
        mesh.indices = vec![0, 1, 2, 3, 2, 0];
        ui.painter().add(Shape::mesh(mesh));
        rect()
    });
    let (_, png) = png(&snapshot);
    let svg = snapshot.to_svg(ExportOptions::default()).unwrap();
    let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
    assert_eq!(
        xml.descendants()
            .filter(|n| n.has_tag_name("image"))
            .count(),
        1
    );
    let svg = render_svg(&svg, [32, 24]);
    for x in [2, 8, 16, 24, 30] {
        let a = pixel(&png, 32, x, 12);
        let b = pixel(&svg, 32, x, 12);
        assert!(a[0].abs_diff(b[0]) <= 1 && a[2].abs_diff(b[2]) <= 1);
        assert_eq!(a[3], 255);
    }
    assert!(pixel(&png, 32, 2, 12)[0] > pixel(&png, 32, 24, 12)[0]);
}

#[test]
fn vector_paths_and_inside_borders_are_not_a_whole_plot_screenshot() {
    let snapshot = snapshot(|ui| {
        ui.painter().rect_stroke(
            rect().shrink(3.0),
            0.0,
            Stroke::new(2.0, Color32::RED),
            egui::StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [pos2(15.0, 25.0), pos2(35.0, 38.0)],
            Stroke::new(1.5, Color32::GREEN),
        );
        rect()
    });
    let svg = snapshot.to_svg(ExportOptions::default()).unwrap();
    let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
    assert!(xml.descendants().any(|n| n.has_tag_name("path")));
    assert!(!xml.descendants().any(|n| n.has_tag_name("image")));
    let svg = render_svg(&svg, [32, 24]);
    assert_eq!(pixel(&svg, 32, 4, 12), [255, 0, 0, 255]);
    assert_eq!(pixel(&svg, 32, 1, 12), [0; 4]);
}
