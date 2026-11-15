use egui::{Rect, Ui};

use crate::Axis;

pub(crate) fn allocate_plot(ui: &mut Ui, height: f32) -> Rect {
    let width = if ui.available_width().is_finite() {
        ui.available_width().max(1.0)
    } else {
        ui.clip_rect()
            .intersect(ui.ctx().viewport_rect())
            .width()
            .max(1.0)
    };
    ui.allocate_space(egui::vec2(width, height)).1
}

pub(crate) fn data_rect(
    outer: Rect,
    axes: &[Axis; 2],
    title: &str,
    right_axis: Option<&Axis>,
) -> Rect {
    let left =
        (52.0_f32 + if axes[1].label.is_empty() { 0.0 } else { 24.0 }).min(outer.width() * 0.2);
    let right = (52.0_f32
        + if right_axis.is_some_and(|axis| !axis.label.is_empty()) {
            24.0
        } else {
            0.0
        })
    .min(outer.width() * 0.2);
    let top = (22.0_f32 + if title.is_empty() { 0.0 } else { 24.0 }).min(outer.height() * 0.2);
    let bottom =
        (22.0_f32 + if axes[0].label.is_empty() { 0.0 } else { 24.0 }).min(outer.height() * 0.2);
    Rect::from_min_max(
        outer.min + egui::vec2(left, top),
        outer.max - egui::vec2(right, bottom),
    )
}
