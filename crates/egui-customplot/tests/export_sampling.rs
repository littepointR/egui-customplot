#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use std::sync::Arc;

use egui::{Color32, Rect, pos2, vec2};
use egui_customplot::{ExportOptions, Line, LineSampling, Plot, PlotData, PlotState};
use export_support::{decode_png, pixel, render_svg, snapshot};

#[test]
fn snapshot_keeps_displayed_lod_without_retaining_samples_or_promising_recovery() {
    let points: Arc<[[f64; 2]]> = (0..20_000)
        .map(|i| [i as f64, (i as f64 * 0.35).sin()])
        .collect::<Vec<_>>()
        .into();
    let mut lengths = Vec::new();
    for sampling in [LineSampling::Exact, LineSampling::MinMax] {
        let mut data = PlotData::new(points.clone());
        data.build_line_index(1024 * 1024).unwrap();
        let mut state = PlotState::default();
        let snapshot = snapshot(|ui| {
            Plot::new("p")
                .height(200.0)
                .show(
                    ui,
                    &mut state,
                    &[Line::new(&data).sampling(sampling).color(Color32::RED)],
                )
                .outer_rect
        });
        drop((data, state));
        assert_eq!(
            Arc::strong_count(&points),
            1,
            "no raw sample ownership in a snapshot"
        );
        let path = |ppp| {
            let svg = snapshot
                .to_svg(ExportOptions {
                    pixels_per_point: ppp,
                    ..Default::default()
                })
                .unwrap();
            let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).unwrap()).unwrap();
            xml.descendants()
                .filter(|n| n.has_tag_name("path") && n.attribute("stroke") == Some("#ff0000"))
                .map(|n| n.attribute("d").unwrap().to_owned())
                .collect::<Vec<_>>()
        };
        let a = path(1.0);
        assert_eq!(
            a,
            path(3.0),
            "larger output does not secretly query/recover raw data"
        );
        lengths.push(a.iter().map(|d| d.matches('L').count()).sum::<usize>());
    }
    assert!(
        lengths[0] > 19_000 && lengths[1] < lengths[0] / 2,
        "{lengths:?}"
    );
}

#[test]
fn fractional_rectangles_do_not_stretch_svg_contents_to_ceil_pixel_size() {
    let snapshot = snapshot(|ui| {
        let rect = Rect::from_min_size(pos2(10.25, 10.5), vec2(32.2, 24.2));
        let mut mesh = egui::Mesh::default();
        mesh.add_colored_rect(
            Rect::from_min_max(pos2(26.25, 10.5), rect.max),
            Color32::RED,
        );
        ui.painter().add(egui::Shape::mesh(mesh));
        rect
    });
    let options = ExportOptions::default();
    let (info, png) = decode_png(&snapshot.to_png(options).unwrap());
    assert_eq!([info.width, info.height], [33, 25]);
    let svg = render_svg(&snapshot.to_svg(options).unwrap(), [33, 25]);
    for x in [2, 14, 16, 24, 30] {
        assert_eq!(pixel(&png, 33, x, 12), pixel(&svg, 33, x, 12));
    }
}
