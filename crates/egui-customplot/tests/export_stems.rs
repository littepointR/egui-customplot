#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use egui::{Color32, Stroke};
use egui_customplot::{
    ExportOptions, Plot, PlotBounds, PlotData, PlotState, StemOrientation, Stems,
};
use export_support::{decode_png, pixel, render_svg, snapshot};

#[test]
fn both_exports_keep_independent_rods_baseline_and_raw_endpoint_markers() {
    for orientation in [StemOrientation::Vertical, StemOrientation::Horizontal] {
        let swap = |p: [f64; 2]| {
            if orientation == StemOrientation::Horizontal {
                [p[1], p[0]]
            } else {
                p
            }
        };
        let mut transform = None;
        let snapshot = snapshot(|ui| {
            let data = PlotData::new([[1.0, 2.0], [3.0, -1.0]].map(swap).to_vec());
            let mut state = PlotState::default();
            state.set_bounds(if orientation == StemOrientation::Vertical {
                PlotBounds::new([0.0, -2.0], [4.0, 3.0]).unwrap()
            } else {
                PlotBounds::new([-2.0, 0.0], [3.0, 4.0]).unwrap()
            });
            let out = Plot::new("p").height(250.0).show_grid(false).show_layers(
                ui,
                &mut state,
                &[Stems::new(&data)
                    .orientation(orientation)
                    .color(Color32::RED)
                    .width(3.0)
                    .baseline_stroke(Stroke::new(3.0, Color32::GREEN))
                    .into()],
            );
            transform = Some(out.transform);
            out.outer_rect
        });
        let transform = transform.unwrap();
        let svg = snapshot.to_svg(ExportOptions::default()).unwrap();
        let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
        let rods: Vec<_> = xml
            .descendants()
            .filter(|n| n.has_tag_name("path") && n.attribute("stroke") == Some("#ff0000"))
            .collect();
        assert_eq!(rods.len(), 2);
        for (rod, point) in rods.iter().zip([[1.0, 2.0], [3.0, -1.0]]) {
            let base = transform.position_from_point(swap([point[0], 0.0]));
            let end = transform.position_from_point(swap(point));
            assert_eq!(
                rod.attribute("d"),
                Some(format!("M{} {}L{} {}", base.x, base.y, end.x, end.y).as_str())
            );
        }
        let (info, png) = decode_png(&snapshot.to_png(ExportOptions::default()).unwrap());
        let svg = render_svg(&svg, [info.width, info.height]);
        for (point, color) in [
            ([1.0, 1.0], 0),
            ([3.0, -0.5], 0),
            ([1.0, 2.0], 0),
            ([2.0, 0.0], 1),
            ([2.0, 0.5], 2),
        ] {
            let pos = transform.position_from_point(swap(point)) - snapshot.logical_rect().min;
            for image in [&png, &svg] {
                let p = pixel(image, info.width as usize, pos.x as usize, pos.y as usize);
                let red = p[0] > 220 && p[1] < 30 && p[2] < 30;
                let green = p[1] > 220 && p[0] < 30 && p[2] < 30;
                assert_eq!(
                    (red, green),
                    (color == 0, color == 1),
                    "{orientation:?} {point:?}: {p:?}"
                );
            }
        }
    }
}
