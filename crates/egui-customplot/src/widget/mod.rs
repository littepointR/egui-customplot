//! Public builder and thin per-pass orchestration. Algorithms live in their modules.

mod bode;
mod clustergram;
mod color_bar;
mod composite;
mod figure;
mod geo;
mod geoglobe;
mod hover;
mod plot3d;
mod polar;
mod response;
mod show;
mod sky;
mod smith;

use std::sync::Arc;

use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    Annotation, Axis, HoverInfo, PlotStyle, PlotTheme,
    interaction::{HoverFormatter, NavigationOptions},
};
pub use bode::{BodePlot, BodePlotResponse, BodePlotState};
pub use clustergram::{Clustergram, ClustergramResponse, ClustergramState};
pub use color_bar::{ColorBar, ColorBarResponse};
pub use composite::{
    ParetoChart, PlotMatrix, PlotYy, ScatterHistogram, ScatterHistogramState, StackedPlot,
    StackedPlotResponse,
};
pub use figure::{Figure, FigureCell, FigureResponse};
pub use geo::{GeoPlot, GeoPlotResponse};
pub use geoglobe::{GeoGlobePlot, GeoGlobeResponse};
pub use plot3d::{Plot3D, Plot3DResponse};
pub use polar::{PolarPlot, PolarPlotResponse};
pub use response::PlotResponse;
pub use sky::{SkyPlot, SkyPlotResponse};
pub use smith::{SmithPlot, SmithPlotResponse};

/// Native egui 2D scientific plot widget. Drag to pan; scroll wheel zooms when enabled;
/// double-click or focused `Home` resets.
///
/// 原生 `egui` 二维科学绘图控件：左键拖拽平移、右键/`Shift` 框选放大、双击或 `Home` 键重置视图，
/// 支持滚轮缩放、对数/反转/时间坐标轴、双 Y 轴、图面注解与自定义悬停提示。
pub struct Plot {
    id_salt: IdSalt,
    height: f32,
    show_grid: bool,
    show_frame: bool,
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
    bubble_legend: Option<crate::BubbleLegend>,
}

impl Plot {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 320.0,
            show_grid: true,
            show_frame: true,
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
            bubble_legend: None,
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
    pub fn show_frame(mut self, show: bool) -> Self {
        self.show_frame = show;
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
    /// Clamps interactive panning to `[min, max]` per axis without spring-back jitter.
    ///
    /// 按轴将交互式平移限制在 `[min, max]` 区间内，贴边即停且无回弹抖动。
    pub fn pan_limits(mut self, x: Option<[f64; 2]>, y: Option<[f64; 2]>) -> Self {
        self.navigation.pan_limits = [x, y];
        self
    }
    pub fn zoom_axes(mut self, axes: [bool; 2]) -> Self {
        self.navigation.zoom_axes = axes;
        self
    }
    /// Opt-in: surrounding ScrollArea owns the wheel by default. Ctrl/cmd-scroll
    /// remains available for egui global UI zoom, not plot zoom.
    ///
    /// 是否开启滚轮缩放（默认关闭以便外层 `ScrollArea` 接管滚轮；也可通过 `PlotDefaults` 全局启用）。
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.navigation.allow_scroll_zoom = allow;
        self
    }
    /// Enables rubber-band box zoom via secondary mouse drag or Shift+primary drag.
    pub fn allow_box_zoom(mut self, allow: bool) -> Self {
        self.navigation.allow_box_zoom = allow;
        self
    }
    /// Enables persistent rectangular data brushing via Alt+primary drag (or primary drag when pan is off).
    pub fn allow_brush(mut self, allow: bool) -> Self {
        self.navigation.allow_brush = allow;
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
    /// Configure image display axes (`imshow` / `EX-025`): top-down reversed Y axis
    /// and 1:1 data aspect ratio while keeping any user axis labels/scales.
    pub fn imshow_axes(mut self) -> Self {
        self.axes[1].reversed = true;
        self.data_aspect = Some(1.0);
        self
    }

    /// Configure sparsity matrix display axes (`spy` / `ML-020`): top-down reversed Y axis
    /// (row 1 at top) and 1:1 data aspect ratio.
    pub fn spy_axes(mut self) -> Self {
        self.axes[1].reversed = true;
        self.data_aspect = Some(1.0);
        self
    }

    pub fn bubble_legend(mut self, legend: crate::BubbleLegend) -> Self {
        self.bubble_legend = Some(legend);
        self
    }

    /// Configures a clean aspect-1:1 frame-free canvas for pie, donut, word, and bubble clouds.
    pub fn canvas_axes(mut self) -> Self {
        self.axes[0] = self.axes[0].clone().visible(false);
        self.axes[1] = self.axes[1].clone().visible(false);
        self.show_grid = false;
        self.show_frame = false;
        self.data_aspect = Some(1.0);
        self
    }

    /// Configures explicit category ticks and top-down row order for [`CategoricalHeatmapData`](crate::CategoricalHeatmapData).
    ///
    /// 为 [`CategoricalHeatmapData`](crate::CategoricalHeatmapData) 自动配置行列分类刻度与自顶向下行序。
    pub fn categorical_heatmap_axes(mut self, data: &crate::CategoricalHeatmapData) -> Self {
        self.axes[0] = self.axes[0].clone().explicit_ticks(data.x_ticks());
        self.axes[1] = self.axes[1]
            .clone()
            .explicit_ticks(data.y_ticks())
            .reversed(true);
        self.show_grid = false;
        self
    }

    /// Configures class + summary ticks and top-down row order for [`crate::ConfusionMatrixData`].
    pub fn confusion_axes(mut self, data: &crate::ConfusionMatrixData, summaries: bool) -> Self {
        self.axes[0] = self.axes[0].clone().explicit_ticks(data.x_ticks(summaries));
        self.axes[1] = self.axes[1]
            .clone()
            .explicit_ticks(data.y_ticks(summaries))
            .reversed(true);
        self.show_grid = false;
        self
    }

    /// Configures 1:1 aspect ratio and hides axis tick labels/grid for clean network & tree views.
    pub fn graph_axes(mut self) -> Self {
        self.axes[0] = self.axes[0].clone().tick_labels_visible(false);
        self.axes[1] = self.axes[1].clone().tick_labels_visible(false);
        self.show_grid = false;
        self.data_aspect = Some(1.0);
        self
    }

    /// Configures recommended `Probit`, `Weibull`, or linear QQ scales for [`crate::ProbabilityPlotData`].
    pub fn probplot_axes(mut self, data: &crate::ProbabilityPlotData) -> Self {
        let [sx, sy] = data.recommended_scales();
        self.axes[0] = self.axes[0].clone().scale(sx);
        self.axes[1] = self.axes[1].clone().scale(sy);
        self
    }

    /// Configures named category ticks along the X axis for [`crate::Waterfall2DData`].
    pub fn waterfall_axes(mut self, data: &crate::Waterfall2DData) -> Self {
        self.axes[0] = self.axes[0].clone().explicit_ticks(data.ticks());
        self
    }

    /// Configures 1:1 data aspect ratio and Real/Imaginary labels for [`crate::ZPlanePlot`].
    pub fn zplane_axes(mut self) -> Self {
        self.axes[0] = self.axes[0].clone().label("Real Part");
        self.axes[1] = self.axes[1].clone().label("Imaginary Part");
        self.data_aspect = Some(1.0);
        self
    }

    /// Configures task labels along the Y axis in top-down order for [`crate::GanttData`].
    pub fn gantt_axes(mut self, data: &crate::GanttData) -> Self {
        self.axes[1] = self.axes[1]
            .clone()
            .explicit_ticks(data.y_ticks())
            .reversed(true);
        self
    }
}
