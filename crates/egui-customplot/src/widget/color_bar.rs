use crate::{Axis, AxisTick, ColorScale, layout::allocate_color_bar, render::paint_color_bar};
use egui::{Rect, Response, Ui};

/// A separate native egui widget; share the exact [`ColorScale`] with a [`crate::Heatmap`].
/// No global current axes or color limits, texture, window or device is created.
///
/// 独立标量色条控件：与热力图/等高线共享同一 [`ColorScale`]，自动生成线性或对数刻度与渐变色柱。
pub struct ColorBar {
    scale: ColorScale,
    label: String,
    height: f32,
}

/// Screen geometry and generated color-scale ticks returned by [`ColorBar::show`].
///
/// [`ColorBar::show`] 返回的色条矩形区域与刻度分布。
pub struct ColorBarResponse {
    pub response: Response,
    pub bar_rect: Rect,
    pub axis_ticks: Vec<AxisTick>,
}

pub(super) fn paint_aligned(
    ui: &Ui,
    area: Rect,
    plot: Rect,
    scale: ColorScale,
    label: &str,
    text_color: egui::Color32,
) {
    let bar = Rect::from_min_max(
        egui::pos2(area.left(), plot.top()),
        egui::pos2(area.left() + 14.0, plot.bottom()),
    );
    let range = scale.range();
    let mut ticks = Axis::new()
        .scale(scale.scale())
        .ticks(range, bar.height() * 2.0);
    ticks.retain(|tick| {
        scale
            .fraction(tick.value)
            .is_some_and(|f| f.min(1.0 - f) * f64::from(bar.height()) > 16.0)
    });
    for value in range {
        ticks.push(AxisTick {
            value,
            major: true,
            label: crate::axis::format_tick(value, (range[1] - range[0]) / 100.0),
        });
    }
    paint_color_bar(ui, area, bar, scale, &ticks, label, text_color);
}

pub(super) fn paint_inset(
    ui: &Ui,
    outer: Rect,
    bar: Rect,
    scale: ColorScale,
    label: &str,
    color: egui::Color32,
) {
    let ticks = Axis::new()
        .scale(scale.scale())
        .ticks(scale.range(), bar.height());
    let area = Rect::from_min_max(
        egui::pos2(bar.left(), bar.top() - 20.),
        egui::pos2(bar.left() + 72., bar.bottom() + 8.),
    )
    .intersect(outer);
    paint_color_bar(ui, area, bar, scale, &ticks, label, color);
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
            paint_color_bar(
                ui,
                outer,
                bar_rect,
                self.scale,
                &axis_ticks,
                &self.label,
                ui.visuals().text_color(),
            );
        }
        ColorBarResponse {
            response,
            bar_rect,
            axis_ticks,
        }
    }
}
