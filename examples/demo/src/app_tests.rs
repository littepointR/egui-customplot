use super::*;
use crate::{appearance::ThemeChoice, data::Scene};

fn input(time: f64) -> egui::RawInput {
    egui::RawInput {
        time: Some(time),
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1200.0, 820.0),
        )),
        ..Default::default()
    }
}

#[test]
fn demo_scenes_embed_in_headless_egui_with_independent_views() {
    let ctx = egui::Context::default();
    let mut demo = Demo::default();
    demo.options.count = 1000;
    demo.options.second_view = true;
    demo.options.markers = true;
    demo.options.dashed = true;
    demo.options.equal_units = true;
    let mut time = 0.0;
    for scene in [Scene::Signal, Scene::PowerLaw] {
        demo.options.scene = scene;
        demo.options.log_axes = [scene == Scene::PowerLaw; 2];
        demo.data = DemoData::new(scene, 1000, true);
        for theme in [
            ThemeChoice::Host,
            ThemeChoice::Scientific,
            ThemeChoice::Monitoring,
        ] {
            demo.options.appearance.theme = theme;
            let output = ctx.run_ui(input(time), |ui| demo.show(ui));
            assert!(!output.shapes.is_empty());
            output.drop_without_applying_deltas();
            time += 0.1;
        }
    }
    assert!(demo.views.main.bounds().is_none());
    assert!(demo.views.detail.bounds().is_none());
}

#[test]
fn live_indexed_cached_demo_survives_repeated_layout_without_duplicate_batches() {
    let ctx = egui::Context::default();
    let mut demo = Demo::default();
    demo.options.performance.streaming = true;
    demo.options.performance.indexed = true;
    demo.options.performance.cached = true;
    demo.options.performance.sampling = egui_customplot::LineSampling::MinMax;
    demo.options.second_view = true;
    for frame in 0..4 {
        let mut passes = 0;
        let mut version = None;
        ctx.run_ui(input(frame as f64 * 0.1), |ui| {
            demo.show(ui);
            if let Some(version) = version {
                assert_eq!(demo.performance.live.data.version(), version);
            }
            version = Some(demo.performance.live.data.version());
            if passes == 0 {
                ui.ctx().request_discard("live demo final layout");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
        assert_eq!(demo.performance.live.data.len(), (frame + 1) * 256);
        assert!(demo.performance.live.data.line_index_bytes() > 0);
        assert!(demo.performance.live.error.is_none());
        assert!(demo.performance.cache.retained_bytes() <= demo.performance.cache.budget_bytes());
    }
}

#[test]
fn figure_linked_dual_y_demo_and_scalar_gallery_survive_layout_retries() {
    let ctx = egui::Context::default();
    ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(4).unwrap());
    let mut demo = Demo::default();
    demo.options.second_view = true;
    demo.options.multi.link_x = true;
    demo.options.multi.side_by_side = true;
    demo.options.multi.right_y = true;
    demo.options.multi.right_log = true;
    demo.options.multi.right_reversed = true;
    for frame in 0..2 {
        ctx.run_ui(input(frame as f64 * 0.1), |ui| {
            let out = demo.views.show(
                ui,
                &demo.options,
                &demo.presentation,
                &demo.data,
                &mut demo.performance,
            );
            assert_eq!(out.len(), 2);
            assert!(
                out.iter()
                    .all(|r| r.right_transform.is_some() && !r.right_axis_ticks.is_empty())
            );
            assert_eq!(
                out[0].transform.bounds().min()[0],
                out[1].transform.bounds().min()[0]
            );
        })
        .drop_without_applying_deltas();
    }
    demo.scalar.enabled = true;
    for map in [
        egui_customplot::ColorMap::Gray,
        egui_customplot::ColorMap::Viridis,
    ] {
        demo.scalar.colors = map;
        demo.scalar.log_axes = true;
        demo.scalar.log_colors = true;
        demo.scalar.reversed_y = true;
        let mut passes = 0;
        ctx.run_ui(input(1.0), |ui| {
            let out = demo.scalar.show(ui);
            assert_eq!(out.diagnostics.input_heatmap_cells, 64 * 48);
            assert!(out.diagnostics.output_heatmap_cells > 2500);
            assert_eq!(out.diagnostics.output_paths, 1);
            if passes == 0 {
                ui.ctx().request_discard("scalar gallery final layout");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
    }
}

#[test]
fn statistical_gallery_keeps_preprocessing_and_geometry_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::default();
    demo.statistical.enabled = true;
    for horizontal in [false, true] {
        demo.statistical.horizontal = horizontal;
        let mut passes = 0;
        ctx.run_ui(input(0.0), |ui| {
            let out = demo.statistical.show(ui);
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].diagnostics.input_bars, 24);
            assert_eq!(out[1].diagnostics.output_bars, 4);
            assert_eq!(out[1].diagnostics.output_error_bar_segments, 12);
            if passes == 0 {
                ui.ctx().request_discard("statistical final layout");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
    }
}

#[test]
fn export_file_side_effect_waits_for_the_final_pass_and_refuses_overwrite() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--statistical"));
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/export-test-output/demo-{}",
        std::process::id()
    ));
    let path = base.with_extension("png");
    assert!(!path.exists());
    demo.export.path = base.to_string_lossy().into_owned();
    demo.export.pixels_per_point = 0.5;
    demo.export.request = Some(crate::export::Format::Png);
    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        demo.show(ui);
        assert!(!path.exists(), "no disk writes during capture/layout");
        if passes == 0 {
            ui.ctx().request_discard("export final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);
    ctx.run_ui(input(1.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
    assert!(
        demo.export.status.starts_with("Saved"),
        "{}",
        demo.export.status
    );
    let saved = std::fs::read(&path).unwrap();
    assert_eq!(&saved[..8], b"\x89PNG\r\n\x1a\n");
    demo.export.request = Some(crate::export::Format::Png);
    ctx.run_ui(input(2.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
    ctx.run_ui(input(3.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
    assert!(!demo.export.status.starts_with("Saved"));
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn every_gallery_exports_self_contained_svg_and_keeps_geometry_after_host_drop() {
    for gallery in [
        None,
        Some("--scalar"),
        Some("--statistical"),
        Some("--stairs"),
        Some("--stems"),
        Some("--quiver"),
        Some("--polygons"),
        Some("--contours"),
        Some("--functions"),
        Some("--geometry"),
        Some("--distributions"),
        Some("--multivariate"),
        Some("--polar"),
        Some("--networks"),
    ] {
        let ctx = egui::Context::default();
        let mut demo = Demo::with_gallery(gallery);
        if gallery != Some("--stairs") {
            demo.options.count = 1000;
            demo.data = DemoData::new(demo.options.scene, 1000, false);
        }
        let mut snapshot = None;
        ctx.run_ui(input(0.0), |ui| {
            snapshot = Some(
                egui_customplot::PlotSnapshot::capture(ui, Default::default(), |ui| {
                    demo.show_charts(ui)
                        .iter()
                        .fold(egui::Rect::NOTHING, |rect, r| rect.union(r.outer_rect))
                })
                .unwrap(),
            );
        })
        .drop_without_applying_deltas();
        drop((demo, ctx));
        let options = egui_customplot::ExportOptions {
            pixels_per_point: 0.5,
            ..Default::default()
        };
        let snapshot = snapshot.unwrap();
        let svg = String::from_utf8(snapshot.to_svg(options).unwrap()).unwrap();
        assert!(
            svg.contains("<svg")
                && svg.contains("clipPath")
                && svg.contains("data:image/png;base64,")
        );
        assert!(!svg.contains("<text"));
        assert!(snapshot.to_png(options).unwrap().starts_with(b"\x89PNG"));
    }
}

#[path = "stems_tests.rs"]
mod stems_tests;

#[path = "quiver_tests.rs"]
mod quiver_tests;

#[path = "stairs_tests.rs"]
mod stairs_tests;

#[path = "polygons_tests.rs"]
mod polygons_tests;

#[path = "contours_tests.rs"]
mod contours_tests;

#[path = "functions_tests.rs"]
mod functions_tests;

#[path = "geometry_tests.rs"]
mod geometry_tests;

#[path = "distributions_tests.rs"]
mod distributions_tests;

#[path = "multivariate_tests.rs"]
mod multivariate_tests;

#[path = "polar_tests.rs"]
mod polar_tests;

#[path = "networks_tests.rs"]
mod networks_tests;
