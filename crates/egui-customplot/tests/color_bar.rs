#[path = "support/input.rs"]
mod support;

use egui::{Context, Shape};
use egui_customplot::{AxisScale, ColorBar, ColorMap, ColorScale};

#[test]
fn color_bar_uses_the_shared_scale_ticks_and_native_parent_clip() {
    let ctx = Context::default();
    let colors = ColorScale::with_scale([1.0, 1000.0], AxisScale::Log10)
        .unwrap()
        .color_map(ColorMap::Gray);
    let mut bar = None;
    let output = ctx.run_ui(support::input(vec![]), |ui| {
        let clip = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(80.0, 180.0));
        ui.set_clip_rect(clip);
        let out = ColorBar::new(colors)
            .label("Pressure")
            .height(260.0)
            .show(ui);
        assert!(out.response.rect.contains_rect(out.bar_rect));
        assert!(out.axis_ticks.iter().all(|t| t.value > 0.0));
        assert!(out.axis_ticks.iter().any(|t| t.major && t.value == 1000.0));
        bar = Some(out.bar_rect);
    });
    let bar = bar.unwrap();
    let mesh = output
        .shapes
        .iter()
        .find_map(|s| {
            if let Shape::Mesh(m) = &s.shape {
                Some(m)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!((mesh.vertices.len(), mesh.indices.len()), (256, 384));
    assert!(mesh.vertices.iter().all(|v| bar.contains(v.pos)));
    assert_eq!(mesh.vertices[0].color, colors.map().sample(0.5 / 64.0));
    assert_eq!(
        mesh.vertices.last().unwrap().color,
        colors.map().sample(63.5 / 64.0)
    );
    output.drop_without_applying_deltas();
}

#[test]
fn hidden_color_bar_skips_geometry_and_invalid_heights_stay_finite() {
    let ctx = Context::default();
    for height in [f32::NAN, f32::INFINITY, -1.0, 0.0, 1e30] {
        let output = ctx.run_ui(support::input(vec![]), |ui| {
            ui.set_clip_rect(egui::Rect::NOTHING);
            let out = ColorBar::new(ColorScale::new([0.0, 1.0]).unwrap())
                .height(height)
                .show(ui);
            assert!(out.response.rect.is_finite() && out.bar_rect.is_positive());
        });
        assert!(output.shapes.is_empty());
        output.drop_without_applying_deltas();
    }
}
