#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use egui::Color32;
use egui_customplot::{
    ExportOptions, Plot, PlotBounds, PlotState, Quiver, QuiverArrow, QuiverData,
};
use export_support::{decode_png, pixel, render_svg, snapshot};

#[test]
fn real_svg_paths_and_two_pixel_backends_preserve_shafts_heads_and_no_false_clipped_tip() {
    let mut transform = None;
    let snapshot = snapshot(|ui| {
        let data = QuiverData::new(vec![
            QuiverArrow::new([-1.0, 0.25], [3.0, 0.0]),
            QuiverArrow::new([0.2, 0.6], [0.3, 0.0]),
        ]);
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
        let out = Plot::new("p").height(250.0).show_grid(false).show_layers(
            ui,
            &mut state,
            &[Quiver::new(&data)
                .color(Color32::RED)
                .width(3.0)
                .head_size(20.0, 16.0)
                .into()],
        );
        transform = Some(out.transform);
        out.outer_rect
    });
    let transform = transform.unwrap();
    let svg = snapshot.to_svg(ExportOptions::default()).unwrap();
    let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
    let paths: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("path") && n.attribute("stroke") == Some("#ff0000"))
        .collect();
    assert_eq!(paths.len(), 3);
    for (path, [a, b]) in paths[..2]
        .iter()
        .zip([[[0.0, 0.25], [1.0, 0.25]], [[0.2, 0.6], [0.5, 0.6]]])
    {
        let a = transform.position_from_point(a);
        let b = transform.position_from_point(b);
        assert_eq!(
            path.attribute("d"),
            Some(format!("M{} {}L{} {}", a.x, a.y, b.x, b.y).as_str())
        );
    }
    let tip = transform.position_from_point([0.5, 0.6]);
    assert_eq!(
        paths[2].attribute("d"),
        Some(
            format!(
                "M{} {}L{} {}L{} {}",
                tip.x - 20.0,
                tip.y + 8.0,
                tip.x,
                tip.y,
                tip.x - 20.0,
                tip.y - 8.0
            )
            .as_str()
        )
    );
    assert!(!xml.descendants().any(|n| n.has_tag_name("text")));
    for scale in [1.0, 2.0] {
        let options = ExportOptions {
            pixels_per_point: scale,
            ..Default::default()
        };
        let (info, png) = decode_png(&snapshot.to_png(options).unwrap());
        let svg = render_svg(
            &snapshot.to_svg(options).unwrap(),
            [info.width, info.height],
        );
        let edge = transform.position_from_point([1.0, 0.25]);
        for (screen, red) in [
            (transform.position_from_point([0.3, 0.25]), true),
            (transform.position_from_point([0.3, 0.6]), true),
            (tip + egui::vec2(-10.0, 4.0), true),
            (tip + egui::vec2(-10.0, -4.0), true),
            (edge + egui::vec2(-10.0, 4.0), false),
            (transform.position_from_point([0.35, 0.42]), false),
        ] {
            let p = (screen - snapshot.logical_rect().min) * scale;
            for image in [&png, &svg] {
                let rgba = pixel(image, info.width as usize, p.x as usize, p.y as usize);
                assert_eq!(
                    rgba[0] > 200 && rgba[1] < 80 && rgba[2] < 80,
                    red,
                    "{screen:?} scale={scale}: {rgba:?}"
                );
            }
        }
    }
}
