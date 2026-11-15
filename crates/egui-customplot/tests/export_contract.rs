#![cfg(feature = "export")]

#[path = "support/export.rs"]
mod export_support;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use egui::{Color32, Context, Mesh, Rect, Shape, TextureId, pos2, vec2};
use egui_customplot::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
use export_support::{decode_png, snapshot};

fn rect() -> Rect {
    Rect::from_min_size(pos2(20.0, 20.0), vec2(50.0, 40.0))
}

fn attempt(
    limits: CaptureLimits,
    mut draw: impl FnMut(&mut egui::Ui) -> Rect,
) -> Result<PlotSnapshot, ExportError> {
    let ctx = Context::default();
    let mut result = None;
    ctx.run_ui(egui::RawInput::default(), |ui| {
        result = Some(PlotSnapshot::capture(ui, limits, |ui| draw(ui)));
    })
    .drop_without_applying_deltas();
    result.unwrap()
}

#[test]
fn capture_limits_reject_without_replaying_draw_or_consuming_host_texture_uploads() {
    for limits in [
        CaptureLimits {
            max_shapes: 0,
            ..Default::default()
        },
        CaptureLimits {
            max_vertices: 1,
            ..Default::default()
        },
        CaptureLimits {
            max_indices: 1,
            ..Default::default()
        },
        CaptureLimits {
            max_font_pixels: 1,
            ..Default::default()
        },
    ] {
        let mut calls = 0;
        let result = attempt(limits, |ui| {
            calls += 1;
            ui.painter().rect_filled(rect(), 0.0, Color32::RED);
            rect()
        });
        assert!(matches!(result, Err(ExportError::LimitExceeded(_))));
        assert_eq!(calls, 1);
    }
    let ctx = Context::default();
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        PlotSnapshot::capture(ui, CaptureLimits::default(), |ui| {
            ui.painter().text(
                rect().min,
                egui::Align2::LEFT_TOP,
                "font upload",
                egui::FontId::proportional(20.0),
                Color32::RED,
            );
            rect()
        })
        .unwrap();
    });
    assert!(
        !output.textures_delta.set.is_empty(),
        "snapshot must not consume the host atlas delta"
    );
    output.drop_without_applying_deltas();
}

#[test]
fn unsupported_textures_callbacks_additive_and_malformed_meshes_fail_explicitly() {
    for texture in [TextureId::User(42), TextureId::Managed(77)] {
        let result = attempt(CaptureLimits::default(), |ui| {
            let mut mesh = Mesh::with_texture(texture);
            mesh.add_rect_with_uv(
                rect(),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(Shape::Mesh(Arc::new(mesh)));
            rect()
        });
        assert!(matches!(result, Err(ExportError::UnsupportedTexture(t)) if t == texture));
    }
    let callback = attempt(CaptureLimits::default(), |ui| {
        ui.painter()
            .add(Shape::Callback(egui::epaint::PaintCallback {
                rect: rect(),
                callback: Arc::new(()),
            }));
        rect()
    });
    assert!(matches!(callback, Err(ExportError::UnsupportedCallback)));
    let bad = attempt(CaptureLimits::default(), |ui| {
        let mesh = Mesh {
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        ui.painter().add(Shape::Mesh(Arc::new(mesh)));
        rect()
    });
    assert!(matches!(bad, Err(ExportError::InvalidGeometry)));
    let additive = attempt(CaptureLimits::default(), |ui| {
        ui.painter()
            .rect_filled(rect(), 0.0, Color32::from_rgb_additive(255, 0, 0));
        rect()
    });
    assert!(matches!(
        additive,
        Err(ExportError::UnsupportedAdditiveColor)
    ));
}

#[test]
fn invalid_export_size_output_and_work_limits_fail_without_mutating_snapshot() {
    let snapshot = snapshot(|ui| {
        ui.painter().rect_filled(rect(), 0.0, Color32::RED);
        rect()
    });
    for options in [
        ExportOptions {
            pixels_per_point: f32::NAN,
            ..Default::default()
        },
        ExportOptions {
            pixels_per_point: 0.0,
            ..Default::default()
        },
        ExportOptions {
            pixels_per_point: f32::MAX,
            ..Default::default()
        },
        ExportOptions {
            dpi: f32::INFINITY,
            ..Default::default()
        },
        ExportOptions {
            dpi: -1.0,
            ..Default::default()
        },
        ExportOptions {
            max_pixels: 4,
            ..Default::default()
        },
        ExportOptions {
            max_output_bytes: 4,
            ..Default::default()
        },
    ] {
        assert!(snapshot.to_png(options).is_err());
        assert!(snapshot.to_svg(options).is_err());
    }
    assert!(matches!(
        snapshot.to_png(ExportOptions {
            max_raster_steps: 0,
            ..Default::default()
        }),
        Err(ExportError::LimitExceeded("raster work"))
    ));
    let (info, _) = decode_png(&snapshot.to_png(ExportOptions::default()).unwrap());
    assert_eq!([info.width, info.height], [50, 40]);
}

#[test]
fn scope_excludes_other_layers_and_earlier_later_ui_and_files_are_real() {
    let snapshot = snapshot(|ui| {
        ui.painter().rect_filled(rect(), 0.0, Color32::RED);
        // Nested snapshot starts after the red item and ends before the blue one.
        let captured = PlotSnapshot::capture(ui, CaptureLimits::default(), |ui| {
            ui.ctx()
                .layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::unique("not-captured"),
                ))
                .rect_filled(rect(), 0.0, Color32::GREEN);
            rect()
        })
        .unwrap();
        ui.painter().rect_filled(rect(), 0.0, Color32::BLUE);
        let (_, rgba) = decode_png(&captured.to_png(ExportOptions::default()).unwrap());
        assert!(rgba.iter().all(|b| *b == 0));
        rect()
    });
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/export-test-output");
    std::fs::create_dir_all(&directory).unwrap();
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let name = format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    for (extension, bytes) in [
        ("png", snapshot.to_png(ExportOptions::default()).unwrap()),
        ("svg", snapshot.to_svg(ExportOptions::default()).unwrap()),
    ] {
        let file = directory.join(format!("{name}.{extension}"));
        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)
            .unwrap();
        std::io::Write::write_all(&mut writer, &bytes).unwrap();
        drop(writer);
        let actual = std::fs::read(&file).unwrap();
        assert_eq!(actual, bytes);
        if extension == "png" {
            decode_png(&actual);
        } else {
            roxmltree::Document::parse(std::str::from_utf8(&actual).unwrap()).unwrap();
        }
        std::fs::remove_file(&file).unwrap();
    }
}
