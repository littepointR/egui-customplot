#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use std::f64::consts::TAU;

use egui::{Color32, Stroke};
use egui_customplot::{
    AlphaShape, AlphaShapeData, Annotation, AnnotationCoord, Comet, CometPlayback, ExportOptions,
    Feather, FeatherData, FunctionCurveData, FunctionPlot, FunctionPlotOptions, Plot, PlotBounds,
    PlotData, PlotState, Spy, SpyData, StreamlineData, Streamlines, TriMesh2D, TriPlot,
    VectorField2D, Voronoi, VoronoiData,
};
use export_support::{decode_png, pixel, render_svg, snapshot};

#[test]
fn alphashape_hole_exports_cleanly_to_png_svg_and_resvg() {
    let mut ring = Vec::new();
    for &r in &[2.0_f64, 4.0_f64] {
        for k in 0..24 {
            let a = TAU * (k as f64 / 24.0);
            ring.push([5.0 + r * a.cos(), 5.0 + r * a.sin()]);
        }
    }
    let alpha = AlphaShapeData::from_sites(&ring, 1.35).unwrap();
    let bounds = PlotBounds::new([0.0, 0.0], [10.0, 10.0]).unwrap();

    let mut state = PlotState::default();
    state.set_bounds(bounds);
    let fill = Color32::from_rgb(35, 130, 215);

    let mut resp = None;
    let snap = snapshot(|ui| {
        let r = Plot::new("alpha-export")
            .show_grid(false)
            .background(Color32::WHITE)
            .show_layers(
                ui,
                &mut state,
                &[AlphaShape::new(&alpha)
                    .fill_color(fill)
                    .border(Stroke::new(1.5, Color32::BLACK))
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
    assert!(doc.descendants().filter(|n| n.has_tag_name("path")).count() > 10);

    let (png_info, png_rgba) = decode_png(&snap.to_png(options).unwrap());
    let svg_rgba = render_svg(svg.as_bytes(), [png_info.width, png_info.height]);
    let w = png_info.width as usize;

    let hole_pos = r.transform.position_from_point([5.0, 5.0]) - r.outer_rect.min;
    let ring_pos = r.transform.position_from_point([8.0, 5.0]) - r.outer_rect.min;
    let (hx, hy) = (hole_pos.x.round() as usize, hole_pos.y.round() as usize);
    let (rx, ry) = (ring_pos.x.round() as usize, ring_pos.y.round() as usize);

    assert_eq!(pixel(&png_rgba, w, hx, hy), [255, 255, 255, 255]);
    assert_eq!(pixel(&svg_rgba, w, hx, hy), [255, 255, 255, 255]);
    assert_ne!(pixel(&png_rgba, w, rx, ry), [255, 255, 255, 255]);
    assert_ne!(pixel(&svg_rgba, w, rx, ry), [255, 255, 255, 255]);
}

#[test]
fn functions_triplot_voronoi_flow_spy_comet_and_ex027_annotations_export_as_valid_svg_and_png() {
    let sites = vec![[0.5, -1.0], [2.0, 1.0], [3.5, -0.8], [5.0, 0.9], [2.8, 0.1]];
    let mesh = TriMesh2D::delaunay(&sites).unwrap();
    let vor =
        VoronoiData::from_sites(&sites, PlotBounds::new([0.0, -1.5], [TAU, 1.5]).unwrap()).unwrap();
    let curve =
        FunctionCurveData::explicit([0.0, TAU], FunctionPlotOptions::default(), f64::sin).unwrap();
    let field = VectorField2D::uniform(
        16,
        16,
        PlotBounds::new([0.0, -1.5], [TAU, 1.5]).unwrap(),
        vec![1.0; 256],
        vec![0.4; 256],
    )
    .unwrap();
    let slice = StreamlineData::streamslice(&field, 1.0).unwrap();
    let feather = FeatherData::from_components(&[0.5, 0.8, 0.2], &[0.4, 0.3, -0.3])
        .unwrap()
        .with_baseline(1.0, 1.5, -0.8);
    let spy = SpyData::from_coords(6, 6, &[(0, 0), (1, 1), (2, 2), (3, 3)]).unwrap();
    let comet_pts = PlotData::new(vec![[0.5, -1.0], [2.0, 0.8], [4.0, -0.4], [5.5, 1.1]]);
    let mut playback = CometPlayback::new(10.0);
    playback.set_progress_points(2.5, comet_pts.len());

    let annots = vec![
        Annotation::line(AnnotationCoord::Normalized, [0.1, 0.1], [0.9, 0.1])
            .with_stroke(Stroke::new(1.5, Color32::RED)),
        Annotation::arrow(AnnotationCoord::Data, [1.0, -1.0], [2.5, 0.5])
            .with_stroke(Stroke::new(1.5, Color32::BLUE)),
        Annotation::double_arrow(AnnotationCoord::Normalized, [0.2, 0.85], [0.8, 0.85])
            .with_stroke(Stroke::new(1.5, Color32::GREEN)),
        Annotation::text_arrow(AnnotationCoord::Data, [3.5, 0.8], [5.0, 1.0], "target")
            .with_stroke(Stroke::new(1.2, Color32::BLACK)),
        Annotation::textbox(
            AnnotationCoord::Normalized,
            [[0.08, 0.72], [0.35, 0.88]],
            "note",
        )
        .with_stroke(Stroke::new(1.0, Color32::DARK_GRAY)),
        Annotation::rectangle(AnnotationCoord::Data, [[0.5, -0.5], [1.8, 0.5]])
            .with_stroke(Stroke::new(1.0, Color32::DARK_RED)),
        Annotation::ellipse(AnnotationCoord::Normalized, [[0.6, 0.2], [0.85, 0.45]])
            .with_stroke(Stroke::new(1.2, Color32::BROWN)),
    ];

    let mut state = PlotState::default();
    let snap = snapshot(|ui| {
        Plot::new("stage8b-all")
            .background(Color32::WHITE)
            .annotations(annots.as_slice())
            .show_layers(
                ui,
                &mut state,
                &[
                    FunctionPlot::new(&curve).into(),
                    TriPlot::new(&mesh).color(Color32::GRAY).into(),
                    Voronoi::new(&vor)
                        .show_sites(true)
                        .color(Color32::DARK_BLUE)
                        .into(),
                    Streamlines::new(&slice).into(),
                    Feather::new(&feather).into(),
                    Spy::new(&spy).into(),
                    Comet::new(&comet_pts, &playback).into(),
                ],
            )
            .outer_rect
    });

    let options = ExportOptions {
        background: Color32::WHITE,
        ..Default::default()
    };
    assert!(snap.to_png(options).unwrap().starts_with(b"\x89PNG"));
    let svg = String::from_utf8(snap.to_svg(options).unwrap()).unwrap();
    assert!(!svg.contains("<text"));
    let doc = roxmltree::Document::parse(&svg).unwrap();
    assert!(doc.descendants().filter(|n| n.has_tag_name("path")).count() >= 10);
    assert!(
        doc.descendants()
            .any(|n| n.has_tag_name("circle") || n.has_tag_name("path"))
    );
}
