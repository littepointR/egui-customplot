use super::Demo;
use eframe::egui::{self, Color32, Event, Modifiers, PointerButton, Pos2, Rect};
use egui_customplot::{ExportOptions, PlotSnapshot};
use std::path::Path;

const DENSITY: f32 = 3.0;

fn save_plot(
    ctx: &egui::Context,
    canvas: egui::Vec2,
    dest: &Path,
    name: &str,
    draw: impl FnMut(&mut egui::Ui) -> Rect,
) {
    let snapshot = PlotSnapshot::render_at(ctx, canvas, DENSITY, Default::default(), draw)
        .unwrap_or_else(|e| panic!("render {name}: {e:?}"));
    let png = snapshot
        .to_png(ExportOptions {
            pixels_per_point: DENSITY,
            background: Color32::WHITE,
            ..Default::default()
        })
        .unwrap();
    std::fs::write(dest.join(format!("{name}.png")), png).unwrap();
}

/// Drive the same click and Shift-click gestures as the interactive gallery before export.
fn measurement_plots(dest: &Path) {
    let ctx = egui::Context::default();
    ctx.set_theme(egui::ThemePreference::Light);
    let canvas = egui::vec2(600.0, 680.0);
    let mut demo = Demo::with_gallery(Some("--stage15"));
    demo.stage15.show_measurements = true;
    let mut frame = |events| {
        let mut wave = Rect::NOTHING;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, canvas)),
            events,
            ..Default::default()
        };
        ctx.run_ui(input, |ui| {
            wave = demo.show_charts(ui)[0].transform.rect();
        })
        .drop_without_applying_deltas();
        wave
    };
    let wave = frame(vec![]);
    for (x, modifiers) in [(0.28, Modifiers::NONE), (0.70, Modifiers::SHIFT)] {
        let pos = wave.lerp_inside(egui::vec2(x, 0.5));
        for pressed in [true, false] {
            frame(vec![
                Event::ModifiersChanged(modifiers),
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers,
                },
            ]);
        }
    }
    frame(vec![Event::PointerGone]);
    save_plot(&ctx, canvas, dest, "shared-cursors", |ui| {
        let plots = demo.show_charts(ui);
        plots[0].outer_rect.union(plots[1].outer_rect)
    });
    save_plot(&ctx, canvas, dest, "step-response", |ui| {
        demo.show_charts(ui)[2].outer_rect
    });
}

/// Regenerate the README illustrations from the interactive gallery models.
#[test]
#[ignore = "writes documentation assets"]
fn render_readme_plots() {
    let dest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/images");
    for (gallery, index, columns, name) in [
        ("--surfaces3d", 0, 3, "surface"),
        ("--contours", 0, 2, "contours"),
        ("--custom-axes", 1, 2, "dual-axis"),
        ("--geo", 0, 2, "pacific-route"),
        ("--stage15", 0, 4, "smith"),
        ("--stage15", 7, 4, "spectrogram"),
        ("--stage15", 2, 4, "bode"),
        ("--stage15", 1, 4, "pole-zero"),
        ("--distributions", 4, 4, "raincloud"),
        ("--stage14", 1, 4, "survival"),
        ("--stage14", 4, 4, "biplot"),
        ("--stage14", 6, 4, "candlestick"),
        ("--stage13", 0, 4, "confusion"),
        ("--stage13", 2, 4, "curvilinear"),
        ("--volumes3d", 2, 3, "isosurface"),
        ("--stage16", 1, 4, "point-cloud"),
        ("--stage16", 2, 4, "globe"),
        ("--stage16", 4, 4, "sankey"),
        ("--stage16", 0, 4, "volume"),
        ("--stage16", 7, 4, "gantt"),
        ("--networks", 1, 2, "network"),
        ("--polar", 2, 3, "polar-histogram"),
        ("--custom-annotations", 0, 2, "annotations"),
        ("--quiver", 0, 2, "quiver"),
        ("--custom-animation", 0, 2, "animation"),
        ("--stage13", 1, 4, "sky"),
        ("", 0, 1, "signal"),
        ("--stairs", 0, 1, "stairs"),
        ("--scalar", 0, 1, "scalar"),
        ("--statistical", 1, 2, "bar-errors"),
        ("--stems", 0, 2, "stems"),
        ("--polygons", 1, 2, "polygons"),
        ("--functions", 2, 2, "implicit"),
        ("--geometry", 0, 3, "triangulation"),
        ("--multivariate", 0, 4, "bubbles"),
        ("--plot3d", 1, 3, "scatter3d"),
        ("--dynamic-10m", 0, 1, "streaming"),
        ("--custom-styling", 2, 2, "styling"),
    ] {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Light);
        let mut demo = Demo::with_gallery((!gallery.is_empty()).then_some(gallery));
        if gallery == "--dynamic-10m" {
            // At full extent the 10M samples fill every pixel column. Show a 90 s window.
            let bounds = egui_customplot::PlotBounds::new([100.0, -1.1], [190.0, 1.1]).unwrap();
            demo.dynamic10m.set_view_bounds(bounds);
        }
        save_plot(
            &ctx,
            egui::vec2(
                600.0 * columns as f32,
                if columns == 1 { 465.0 } else { 820.0 },
            ),
            &dest,
            name,
            |ui| demo.show_charts(ui)[index].outer_rect,
        );
    }
    measurement_plots(&dest);
}
