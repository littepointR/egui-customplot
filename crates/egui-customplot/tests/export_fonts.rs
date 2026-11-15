#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use egui::{Align2, Color32, FontId, Rect, Stroke, pos2, vec2};
use egui_customplot::ExportOptions;
use export_support::{png, render_svg, snapshot};

#[test]
fn rotated_host_glyphs_overrides_and_clipping_survive_without_installed_svg_fonts() {
    let snapshot = snapshot(|ui| {
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(100.0, 90.0));
        let painter = ui.painter().with_clip_rect(rect.shrink(4.0));
        let galley =
            painter.layout_no_wrap("A<&> Ω漢".into(), FontId::proportional(19.0), Color32::RED);
        let mut text = egui::epaint::TextShape::new(pos2(30.0, 28.0), galley, Color32::RED)
            .with_angle_and_anchor(std::f32::consts::FRAC_PI_4, Align2::CENTER_CENTER);
        text.override_text_color = Some(Color32::GREEN);
        text.underline = Stroke::new(1.0, Color32::BLUE);
        painter.add(text);
        rect
    });
    // The Context and host font manager have already been dropped.
    let (_, png) = png(&snapshot);
    assert!(png.as_chunks::<4>().0.iter().any(|p| p[1] > 0 && p[3] > 0));
    assert!(png.as_chunks::<4>().0.iter().any(|p| p[2] > 0 && p[3] > 0));
    assert!(png.as_chunks::<4>().0.iter().all(|p| p[0] == 0));
    let svg = snapshot.to_svg(ExportOptions::default()).unwrap();
    let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
    assert!(
        !xml.descendants()
            .any(|n| n.has_tag_name("text") || n.has_tag_name("script"))
    );
    let images: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("image"))
        .collect();
    assert_eq!(images.len(), 1);
    assert!(
        images[0]
            .attribute("href")
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
    let svg = render_svg(&svg, [100, 90]);
    // SVG pixels are premultiplied, PNG pixels unassociated: compare ink alpha.
    let difference: usize = png
        .as_chunks::<4>()
        .0
        .iter()
        .zip(svg.as_chunks::<4>().0.iter())
        .map(|(a, b)| a[3].abs_diff(b[3]) as usize)
        .sum();
    assert!(difference < 150, "embedded glyph alpha drift={difference}");
}
