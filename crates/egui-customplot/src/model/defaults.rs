use std::sync::Arc;

use egui::{Context, Id};

use crate::{PlotStyle, PlotTheme, Projection3D};

const DEFAULTS_KEY: &str = "egui_customplot_plot_defaults";

/// Context-scoped default appearance & interaction overrides applied by widgets unless disabled.
///
/// 挂载于 `egui::Context` 上的全局默认绘图配置（主题、父级样式、滚轮/框选缩放、网格/图例/悬停/十字线与 3D 投影默认项）。
#[derive(Clone, Debug, Default)]
pub struct PlotDefaults {
    pub theme: Option<Arc<PlotTheme>>,
    pub inherited_style: PlotStyle,
    pub allow_scroll_zoom: Option<bool>,
    pub allow_box_zoom: Option<bool>,
    pub show_grid: Option<bool>,
    pub show_legend: Option<bool>,
    pub show_hover: Option<bool>,
    pub crosshair: Option<bool>,
    pub projection_3d: Option<Projection3D>,
}

/// Set or clear context-scoped [`PlotDefaults`] outside locks.
///
/// 在锁外设置或清除上下文作用域的全局 [`PlotDefaults`]，供画廊内全部图表统一继承。
pub fn set_plot_defaults(ctx: &Context, defaults: Option<PlotDefaults>) {
    ctx.data_mut(|data| {
        let id = Id::NULL.with(DEFAULTS_KEY);
        match defaults {
            Some(d) => {
                data.insert_temp(id, d);
            }
            None => data.remove::<PlotDefaults>(id),
        }
    });
}

/// Read the active context-scoped [`PlotDefaults`], if any.
///
/// 读取当前 `egui::Context` 中生效的 [`PlotDefaults`]。
pub fn plot_defaults(ctx: &Context) -> Option<PlotDefaults> {
    ctx.data(|data| data.get_temp::<PlotDefaults>(Id::NULL.with(DEFAULTS_KEY)))
}

impl PlotDefaults {
    pub(crate) fn resolve_scroll_zoom(&self, local: bool) -> bool {
        self.allow_scroll_zoom.unwrap_or(local)
    }

    pub(crate) fn resolve_box_zoom(&self, local: bool) -> bool {
        self.allow_box_zoom.unwrap_or(local)
    }

    pub(crate) fn resolve_grid(&self, local: bool) -> bool {
        self.show_grid.unwrap_or(true) && local
    }

    pub(crate) fn resolve_legend(&self, local: bool) -> bool {
        self.show_legend.unwrap_or(true) && local
    }

    pub(crate) fn resolve_hover(&self, local: bool) -> bool {
        self.show_hover.unwrap_or(true) && local
    }

    pub(crate) fn resolve_crosshair(&self, local: bool) -> bool {
        self.crosshair.unwrap_or(true) && local
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_common(
        &self,
        theme: &mut Option<Arc<PlotTheme>>,
        inherited: &mut PlotStyle,
        scroll_zoom: &mut bool,
        show_grid: &mut bool,
        show_legend: &mut bool,
        show_hover: &mut bool,
    ) {
        if theme.is_none() {
            *theme = self.theme.clone();
        }
        *inherited = inherited.over(self.inherited_style);
        *scroll_zoom = self.resolve_scroll_zoom(*scroll_zoom);
        *show_grid = self.resolve_grid(*show_grid);
        *show_legend = self.resolve_legend(*show_legend);
        *show_hover = self.resolve_hover(*show_hover);
    }
}
