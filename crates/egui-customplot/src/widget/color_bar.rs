use crate::{Axis, AxisTick, ColorScale, layout::allocate_color_bar, render::paint_color_bar};
use egui::{Rect, Response, Ui};

/// A separate native egui widget; share the exact ColorScale with a Heatmap.
/// No global current axes or color limits, texture, window or device is created.
pub struct ColorBar {
    scale: ColorScale,
    label: String,
    height: f32,
}

pub struct ColorBarResponse {
    pub response: Response,
    pub bar_rect: Rect,
    pub axis_ticks: Vec<AxisTick>,
}

impl ColorBar {
    pub fn new(scale: ColorScale) -> Self {
        Self {
            scale,
            label: String::new(),
            height: 260.0,
        }
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.clamp(48.0, 16_384.0);
        }
        self
    }
    pub fn show(self, ui: &mut Ui) -> ColorBarResponse {
        let (outer, bar_rect, response) =
            allocate_color_bar(ui, self.height, !self.label.is_empty());
        let axis_ticks = Axis::new()
            .scale(self.scale.scale())
            .ticks(self.scale.range(), bar_rect.height());
        if ui.is_rect_visible(outer) {
            paint_color_bar(ui, outer, bar_rect, self.scale, &axis_ticks, &self.label);
        }
        ColorBarResponse {
            response,
            bar_rect,
            axis_ticks,
        }
    }
}
