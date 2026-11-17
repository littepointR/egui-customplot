use std::sync::Arc;

use egui::{AsIdSalt, IdSalt, Rect, Response, Sense, Ui};

use crate::{
    GeoGlobeData, GeoGlobeState, GeoGlobeTransform, HoveredGlobeItem, PlotDiagnostics,
    PlotResponse, PlotStyle, PlotTheme, interaction::pick_globe_hover, layout::allocate_plot,
    model::resolve_plot_style, render::paint_geoglobe,
};

/// Layout rect, WGS84 transform, hover hit, and diagnostics returned by [`GeoGlobePlot::show`].
///
/// [`GeoGlobePlot::show`] 返回的三维地球仪布局、变换矩阵、悬停命中与诊断信息。
pub struct GeoGlobeResponse {
    pub outer_rect: Rect,
    pub response: Response,
    pub transform: GeoGlobeTransform,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_item: Option<HoveredGlobeItem>,
    pub hover_text: Option<String>,
}

impl GeoGlobeResponse {
    pub fn into_plot_response(self) -> PlotResponse {
        PlotResponse::from_embedded(
            self.outer_rect,
            self.outer_rect,
            self.response,
            self.diagnostics,
            self.view_changed,
            self.hover_text,
            Vec::new(),
        )
    }
}

/// Interactive 3D WGS84 Globe widget (`geoglobe` / `geoplot3`, `EXT-25`).
///
/// 交互式 WGS84 三维地球仪控件：内置离线海岸线、经纬网格与高程挤出轨迹，支持背面消隐、旋转拖拽与滚轮缩放。
pub struct GeoGlobePlot<'a> {
    id_salt: IdSalt,
    data: &'a GeoGlobeData,
    height: f32,
    title: String,
    show_graticule: bool,
    show_coastlines: bool,
    show_hover: bool,
    allow_scroll_zoom: bool,
    pick_radius: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl<'a> GeoGlobePlot<'a> {
    pub fn new(id_salt: impl AsIdSalt, data: &'a GeoGlobeData) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            data,
            height: 290.0,
            title: String::new(),
            show_graticule: true,
            show_coastlines: true,
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
    pub fn show_graticule(mut self, show: bool) -> Self {
        self.show_graticule = show;
        self
    }
    pub fn show_coastlines(mut self, show: bool) -> Self {
        self.show_coastlines = show;
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

    pub fn show(mut self, ui: &mut Ui, state: &mut GeoGlobeState) -> GeoGlobeResponse {
        if let Some(defs) = crate::model::plot_defaults(ui.ctx()) {
            let mut show_legend = false;
            let mut show_grid = self.show_graticule;
            defs.apply_common(
                &mut self.theme,
                &mut self.inherited_style,
                &mut self.allow_scroll_zoom,
                &mut show_grid,
                &mut show_legend,
                &mut self.show_hover,
            );
            self.show_graticule = show_grid;
        }
        let outer = allocate_plot(ui, self.height);
        let style = resolve_plot_style(ui, self.theme.as_deref(), self.inherited_style, self.style);
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let response = ui.interact(outer, id, Sense::click_and_drag());
        let (lat0, lon0) = state.center_lat_lon();
        let pre_tf = GeoGlobeTransform::new(
            outer,
            lat0,
            lon0,
            state.zoom(),
            self.data.get_alt_exaggeration(),
        );
        let view_changed = state.navigate(ui, &response, pre_tf, self.allow_scroll_zoom);
        let (lat, lon) = state.center_lat_lon();
        let transform = GeoGlobeTransform::new(
            outer,
            lat,
            lon,
            state.zoom(),
            self.data.get_alt_exaggeration(),
        );

        let mut diagnostics = PlotDiagnostics {
            painting_skipped: !ui.is_rect_visible(outer),
            ..Default::default()
        };
        diagnostics.geo.unique_bytes = self.data.sample_bytes();
        let pointer = response.hover_pos();
        let hovered_item = if self.show_hover
            && ui.is_enabled()
            && !diagnostics.painting_skipped
            && let Some(pos) = pointer
        {
            diagnostics.geo.pick_queries += 1;
            pick_globe_hover(self.data, transform, pos, self.pick_radius)
        } else {
            None
        };

        if !diagnostics.painting_skipped {
            let painter = ui.painter_at(outer);
            paint_geoglobe(
                &painter,
                self.data,
                transform,
                self.show_graticule,
                self.show_coastlines,
                &self.title,
                &style,
                hovered_item.as_ref(),
                &mut diagnostics.geo.output_paths,
            );
        }

        let hover_text = hovered_item.as_ref().map(HoveredGlobeItem::default_text);
        if let Some(ref msg) = hover_text {
            response.clone().on_hover_ui_at_pointer(|ui| {
                ui.label(msg);
            });
        }

        GeoGlobeResponse {
            outer_rect: outer,
            response,
            transform,
            diagnostics,
            view_changed,
            hovered_item,
            hover_text,
        }
    }
}
