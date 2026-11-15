//! Public builder and thin per-pass orchestration. Algorithms live in their modules.

mod response;
mod show;

use egui::{AsIdSalt, Color32, IdSalt};

use crate::{Axis, interaction::NavigationOptions};
pub use response::PlotResponse;

/// Native egui control. Drag to pan; double-click or focused Home resets.
pub struct Plot {
    id_salt: IdSalt,
    height: f32,
    show_grid: bool,
    navigation: NavigationOptions,
    background: Option<Color32>,
    axes: [Axis; 2],
    title: String,
    show_legend: bool,
    show_hover: bool,
    crosshair: bool,
    pick_radius: f32,
    data_aspect: Option<f64>,
}

impl Plot {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 320.0,
            show_grid: true,
            navigation: NavigationOptions::default(),
            background: None,
            axes: [Axis::default(), Axis::default()],
            title: String::new(),
            show_legend: false,
            show_hover: true,
            crosshair: true,
            pick_radius: 12.0,
            data_aspect: None,
        }
    }
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.max(1.0);
        }
        self
    }
    /// Ratio of X/Y screen unit lengths; 1.0 gives equal units (decades on log
    /// axes). Expands auto/manual bounds. With an axis zoom lock, aspect wins by
    /// leaving wheel zoom unchanged; panning can still be independently locked.
    pub fn data_aspect(mut self, ratio: f64) -> Self {
        if ratio.is_finite() && ratio > 0.0 {
            self.data_aspect = Some(ratio);
        }
        self
    }
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }
    pub fn allow_drag(mut self, allow: bool) -> Self {
        self.navigation.allow_drag = allow;
        self
    }
    pub fn drag_axes(mut self, axes: [bool; 2]) -> Self {
        self.navigation.drag_axes = axes;
        self
    }
    pub fn zoom_axes(mut self, axes: [bool; 2]) -> Self {
        self.navigation.zoom_axes = axes;
        self
    }
    /// Opt-in: surrounding ScrollArea owns the wheel by default. Ctrl/cmd-scroll
    /// remains available for egui global UI zoom, not plot zoom.
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.navigation.allow_scroll_zoom = allow;
        self
    }
    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }
    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.axes[0] = axis;
        self
    }
    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.axes[1] = axis;
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }
    pub fn show_hover(mut self, show: bool) -> Self {
        self.show_hover = show;
        self
    }
    pub fn crosshair(mut self, show: bool) -> Self {
        self.crosshair = show;
        self
    }
    pub fn pick_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() {
            self.pick_radius = radius.max(0.0);
        }
        self
    }
}
