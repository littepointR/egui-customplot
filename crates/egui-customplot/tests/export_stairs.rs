#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use egui::Color32;
use egui_customplot::{
    ExportOptions, Line, LineInterpolation, Plot, PlotBounds, PlotData, PlotState,
};
use export_support::{decode_png, pixel, render_svg, snapshot};

#[test]
fn exported_steps_keep_the_vector_corner_and_real_raster_ink_without_a_diagonal() {
    for interpolation in [LineInterpolation::StepAfter, LineInterpolation::StepBefore] {
        let mut transform = None;
        let snapshot = snapshot(|ui| {
            let data = PlotData::new(vec![[1.0, 2.0], [3.0, 8.0], [10.0, 4.0]]);
            let mut state = PlotState::default();
            state.set_bounds(PlotBounds::new([0.0; 2], [11.0, 10.0]).unwrap());
            let out = Plot::new("p").height(250.0).show_grid(false).show(
                ui,
                &mut state,
                &[Line::new(&data)
                    .interpolation(interpolation)
                    .color(Color32::RED)
                    .width(3.0)],
            );
            transform = Some(out.transform);
            out.outer_rect
        });
        let transform = transform.unwrap();
        let svg = snapshot.to_svg(ExportOptions::default()).unwrap();
        let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
        let red_paths: Vec<_> = xml
            .descendants()
            .filter(|n| n.has_tag_name("path") && n.attribute("stroke") == Some("#ff0000"))
            .collect();
        assert_eq!(red_paths.len(), 1);
        let points = if interpolation == LineInterpolation::StepAfter {
            [[1.0, 2.0], [3.0, 2.0], [3.0, 8.0], [10.0, 8.0], [10.0, 4.0]]
        } else {
            [[1.0, 2.0], [1.0, 8.0], [3.0, 8.0], [3.0, 4.0], [10.0, 4.0]]
        };
        let expected = points
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let p = transform.position_from_point(p);
                format!("{}{} {}", if i == 0 { 'M' } else { 'L' }, p.x, p.y)
            })
            .collect::<String>();
        assert_eq!(red_paths[0].attribute("d"), Some(expected.as_str()));
        let (info, png) = decode_png(&snapshot.to_png(ExportOptions::default()).unwrap());
        let svg = render_svg(&svg, [info.width, info.height]);
        let ink = if interpolation == LineInterpolation::StepAfter {
            [2.0, 2.0]
        } else {
            [2.0, 8.0]
        };
        for (point, expected_red) in [(ink, true), ([2.0, 5.0], false)] {
            let pos = transform.position_from_point(point) - snapshot.logical_rect().min;
            for image in [&png, &svg] {
                let p = pixel(image, info.width as usize, pos.x as usize, pos.y as usize);
                assert_eq!(
                    p[0] > 220 && p[1] < 30 && p[2] < 30,
                    expected_red,
                    "{point:?}: {p:?}"
                );
            }
        }
    }
}
