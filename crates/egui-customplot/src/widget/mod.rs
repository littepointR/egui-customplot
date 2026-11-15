//! Public builder and thin per-pass orchestration. Algorithms live in their modules.

mod color_bar;
mod figure;
mod hover;
mod response;
mod show;

use std::sync::Arc;

use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    Annotation, Axis, HoverInfo, PlotStyle, PlotTheme,
    interaction::{HoverFormatter, NavigationOptions},
};
pub use color_bar::{ColorBar, ColorBarResponse};
pub use figure::{Figure, FigureCell, FigureResponse};
pub use response::PlotResponse;

/// Native egui control. Drag to pan; double-click or focused Home resets.
pub struct Plot {
    id_salt: IdSalt,
    height: f32,
    show_grid: bool,
    navigation: NavigationOptions,
    theme: Option<Arc<PlotTheme>>,
    inherited_style: PlotStyle,
    style: PlotStyle,
    axes: [Axis; 2],
    right_y_axis: Option<Axis>,
    title: String,
    show_legend: bool,
    show_hover: bool,
    crosshair: bool,
    pick_radius: f32,
    data_aspect: Option<f64>,
    hover_formatter: Option<Arc<HoverFormatter>>,
    annotations: Arc<[Annotation]>,
}

impl Plot {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 320.0,
            show_grid: true,
            navigation: NavigationOptions::default(),
            theme: None,
            inherited_style: PlotStyle::default(),
            style: PlotStyle::default(),
            axes: [Axis::default(), Axis::default()],
            right_y_axis: None,
            title: String::new(),
            show_legend: false,
            show_hover: true,
            crosshair: true,
            pick_radius: 12.0,
            data_aspect: None,
            hover_formatter: None,
            annotations: Arc::from([]),
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
        self.style.background = Some(color);
        self
    }
    pub fn theme(mut self, theme: impl Into<Arc<PlotTheme>>) -> Self {
        self.theme = Some(theme.into());
        self
    }
    /// Retain and share an Arc slice to avoid cloning labels each frame.
    /// Replaces the previous annotation slice; annotations do not affect auto-fit.
    pub fn annotations(mut self, annotations: impl Into<Arc<[Annotation]>>) -> Self {
        self.annotations = annotations.into();
        self
    }
    /// Parent figure/group defaults. Does not mutate egui's local Style.
    pub fn inherited_style(mut self, style: PlotStyle) -> Self {
        self.inherited_style = style.over(self.inherited_style);
        self
    }
    pub fn style(mut self, style: PlotStyle) -> Self {
        self.style = style.over(self.style);
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
    /// Independent scale, direction, ticks and label for Right series. X remains
    /// shared. Wheel/pan navigate both Y axes using the same screen-space action.
    pub fn right_y_axis(mut self, axis: Axis) -> Self {
        self.right_y_axis = Some(axis);
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
    /// Runs outside Context locks, once per hovered show pass (including discarded
    /// passes), even before the delayed tooltip opens. Keep it pure and cheap.
    pub fn hover_formatter(
        mut self,
        formatter: impl for<'a> Fn(HoverInfo<'a>) -> String + Send + Sync + 'static,
    ) -> Self {
        self.hover_formatter = Some(Arc::new(formatter));
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
