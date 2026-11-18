use std::sync::Arc;

use egui::{AsIdSalt, IdSalt, Rect, Response, Sense, Ui};

use crate::{
    HoveredSmithPoint, LegendEntry, PlotDiagnostics, PlotResponse, PlotStyle, PlotTheme,
    SmithGridKind, SmithPlotData, SmithPlotState, SmithTransform,
    interaction::{pick_smith_hover, show_smith_legend},
    layout::{allocate_plot, compute_polar_layout},
    model::resolve_plot_style,
    render::paint_smith,
};

/// Layout geometry, [`SmithTransform`], hover point, and diagnostics returned by [`SmithPlot::show`].
///
/// [`SmithPlot::show`] 返回的史密斯圆图区域、复反射系数坐标变换、悬停点与诊断信息。
pub struct SmithPlotResponse {
    pub outer_rect: Rect,
    pub plot_area: Rect,
    pub response: Response,
    pub transform: SmithTransform,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_point: Option<HoveredSmithPoint>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}

impl SmithPlotResponse {
    pub fn into_plot_response(self) -> PlotResponse {
        PlotResponse::from_embedded(
            self.outer_rect,
            self.plot_area,
            self.response,
            self.diagnostics,
            self.view_changed,
            self.hover_text,
            self.legend_entries,
        )
    }
}

/// Interactive RF Smith chart widget (`smithplot`, `EXT-19`).
///
/// 射频阻抗/导纳史密斯圆图控件：支持阻抗/导纳/混合网格、散射参数轨迹悬停读数与缩放拖拽。
pub struct SmithPlot<'a> {
    id_salt: IdSalt,
    data: &'a SmithPlotData,
    height: f32,
    title: String,
    grid_kind: SmithGridKind,
    show_grid: bool,
    show_legend: bool,
    show_hover: bool,
    allow_scroll_zoom: bool,
    pick_radius: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl<'a> SmithPlot<'a> {
    pub fn new(id_salt: impl AsIdSalt, data: &'a SmithPlotData) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            data,
            height: 290.0,
            title: String::new(),
            grid_kind: SmithGridKind::Impedance,
            show_grid: true,
            show_legend: true,
            show_hover: true,
            allow_scroll_zoom: false,
            pick_radius: 14.0,
            inherited_style: PlotStyle::default(),
            style: PlotStyle::default(),
            theme: None,
        }
    }
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.clamp(120.0, 4096.0);
        }
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn grid_kind(mut self, kind: SmithGridKind) -> Self {
        self.grid_kind = kind;
        self
    }
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
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
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }
    pub fn pick_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() && radius > 0.0 {
            self.pick_radius = radius;
        }
        self
    }
    pub fn style(mut self, style: PlotStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn theme(mut self, theme: impl Into<Arc<PlotTheme>>) -> Self {
        self.theme = Some(theme.into());
        self
    }

    pub fn show(mut self, ui: &mut Ui, state: &mut SmithPlotState) -> SmithPlotResponse {
        if let Some(defs) = crate::model::plot_defaults(ui.ctx()) {
            defs.apply_common(
                &mut self.theme,
                &mut self.inherited_style,
                &mut self.allow_scroll_zoom,
                &mut self.show_grid,
                &mut self.show_legend,
                &mut self.show_hover,
            );
        }
        let outer = allocate_plot(ui, self.height);
        let style = resolve_plot_style(ui, self.theme.as_deref(), self.inherited_style, self.style);
        let legend_count = if self.show_legend {
            self.data.traces().len()
        } else {
            0
        };
        let layout = compute_polar_layout(outer, !self.title.is_empty(), legend_count, None);
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let response = ui.interact(layout.plot_area, id, Sense::click_and_drag());
        let view_changed = state.navigate(ui, &response, layout.plot_area, self.allow_scroll_zoom);
        let transform =
            SmithTransform::new(layout.plot_area, state.center_gamma(), state.half_span());

        let mut diagnostics = PlotDiagnostics {
            painting_skipped: !ui.is_rect_visible(layout.outer),
            ..Default::default()
        };
        diagnostics.polar.input_points = self.data.traces().iter().map(|t| t.len()).sum();

        let pointer = response
            .hover_pos()
            .filter(|p| !layout.legend_rect.is_some_and(|r| r.contains(*p)));
        let hovered_point = if self.show_hover
            && ui.is_enabled()
            && !diagnostics.painting_skipped
            && let Some(pos) = pointer
        {
            diagnostics.polar.pick_queries += 1;
            pick_smith_hover(state, self.data, transform, pos, self.pick_radius)
        } else {
            None
        };

        if !diagnostics.painting_skipped {
            let painter = ui.painter_at(layout.plot_area);
            paint_smith(
                &painter,
                state,
                self.data,
                transform,
                self.grid_kind,
                self.show_grid,
                &self.title,
                &style,
                hovered_point.as_ref(),
                &mut diagnostics.visible_series,
            );
        }

        let legend_entries = layout.legend_rect.map_or_else(Vec::new, |rect| {
            show_smith_legend(ui, state, self.data, rect, id.with("legend"), &style)
        });
        let hover_text = hovered_point.as_ref().map(HoveredSmithPoint::default_text);
        if let Some(ref msg) = hover_text {
            response.clone().on_hover_ui_at_pointer(|ui| {
                ui.label(msg);
            });
        }

        SmithPlotResponse {
            outer_rect: layout.outer,
            plot_area: layout.plot_area,
            response,
            transform,
            diagnostics,
            view_changed,
            hovered_point,
            hover_text,
            legend_entries,
        }
    }
}
