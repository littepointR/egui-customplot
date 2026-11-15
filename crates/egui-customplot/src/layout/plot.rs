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
    let y_labels = axes[1].visible && axes[1].tick_labels_visible;
    let x_labels = axes[0].visible && axes[0].tick_labels_visible;
    let right_labels = right_axis.is_some_and(|axis| axis.visible && axis.tick_labels_visible);
    let any_axis = axes[0].visible || axes[1].visible || right_axis.is_some_and(|a| a.visible);

    let left_base = if y_labels { 52.0_f32 } else { 8.0_f32 };
    let right_base = if right_axis.is_some() {
        if right_labels { 52.0_f32 } else { 8.0_f32 }
    } else if any_axis && (x_labels || y_labels) {
        52.0_f32
    } else {
        8.0_f32
    };
    let top_base = if any_axis { 22.0_f32 } else { 8.0_f32 };
    let bottom_base = if x_labels { 22.0_f32 } else { 8.0_f32 };

    let left = (left_base
        + if axes[1].visible && !axes[1].label.is_empty() {
            24.0
        } else {
            0.0
        })
    .min(outer.width() * 0.2);
    let right = (right_base
        + if right_axis.is_some_and(|axis| axis.visible && !axis.label.is_empty()) {
            24.0
        } else {
            0.0
        })
    .min(outer.width() * 0.2);
    let top = (top_base + if title.is_empty() { 0.0 } else { 24.0 }).min(outer.height() * 0.2);
    let bottom = (bottom_base
        + if axes[0].visible && !axes[0].label.is_empty() {
            24.0
        } else {
            0.0
        })
    .min(outer.height() * 0.2);
    Rect::from_min_max(
        outer.min + egui::vec2(left, top),
        outer.max - egui::vec2(right, bottom),
    )
}
