use std::sync::Arc;

use egui::{AsIdSalt, IdSalt, Rect, Response, Sense, Stroke, Ui};

use crate::{
    ColorScale, HoveredSkyItem, LegendEntry, MarkerShape, PlotDiagnostics, PlotResponse, PlotStyle,
    PlotTheme, SkyPlotData, SkyPlotState, SkyTransform,
    interaction::{navigate_sky, pick_sky_hover, show_sky_legend},
    layout::{allocate_plot, compute_polar_layout},
    model::resolve_plot_style,
    render::paint_sky,
};

/// Layout geometry, [`SkyTransform`], hover item, and diagnostics returned by [`SkyPlot::show`].
///
/// [`SkyPlot::show`] 返回的星空图区域、方位角-仰角投影、悬停卫星命中与诊断信息。
pub struct SkyPlotResponse {
    pub outer_rect: Rect,
    pub plot_area: Rect,
    pub response: Response,
    pub transform: SkyTransform,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_item: Option<HoveredSkyItem>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}

impl SkyPlotResponse {
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

/// Azimuth-elevation sky plot widget (`skyplot`, `EXT-07`).
///
/// 方位角-仰角极投影星空图控件：支持卫星轨迹、最低仰角掩膜环、图例交互与滚轮缩放。
pub struct SkyPlot<'a> {
    id_salt: IdSalt,
    data: &'a SkyPlotData,
    height: f32,
    title: String,
    min_elevation_deg: f64,
    mask_elevation_deg: f64,
    color_scale: Option<ColorScale>,
    marker_shape: MarkerShape,
    marker_radius: f32,
    show_labels: bool,
    show_grid: bool,
    show_legend: bool,
    show_hover: bool,
    allow_scroll_zoom: bool,
    pick_radius: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl<'a> SkyPlot<'a> {
    pub fn new(id_salt: impl AsIdSalt, data: &'a SkyPlotData) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            data,
            height: 290.0,
            title: String::new(),
            min_elevation_deg: 0.0,
            mask_elevation_deg: 0.0,
            color_scale: None,
            marker_shape: MarkerShape::Circle,
            marker_radius: 5.5,
            show_labels: true,
            show_grid: true,
            show_legend: true,
            show_hover: true,
            allow_scroll_zoom: false,
            pick_radius: 12.0,
            inherited_style: PlotStyle::default(),
            style: PlotStyle::default(),
            theme: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() && height > 0.0 {
            self.height = height.max(48.0);
        }
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn min_elevation(mut self, deg: f64) -> Self {
        if deg.is_finite() {
            self.min_elevation_deg = deg.clamp(0.0, 85.0);
        }
        self
    }
    pub fn mask_elevation(mut self, deg: f64) -> Self {
        if deg.is_finite() {
            self.mask_elevation_deg = deg.clamp(0.0, 89.0);
        }
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn marker_shape(mut self, shape: MarkerShape) -> Self {
        self.marker_shape = shape;
        self
    }
    pub fn marker_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() && radius > 0.0 {
            self.marker_radius = radius.max(1.0);
        }
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }
    pub fn show_legend(mut self, show: bool) -> Self {
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

    pub fn show(mut self, ui: &mut Ui, state: &mut SkyPlotState) -> SkyPlotResponse {
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
        let group_count = group_count(self.data);
        let legend_count = if self.show_legend { group_count } else { 0 };
        let layout = compute_polar_layout(outer, !self.title.is_empty(), legend_count, None);
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let mut response = ui.interact(layout.plot_area, id, Sense::click_and_drag());
        let (min_el, view_changed) = navigate_sky(
            ui,
            state,
            &mut response,
            self.min_elevation_deg,
            self.allow_scroll_zoom,
            layout.legend_rect,
        );
        if view_changed {
            response.mark_changed();
        }
        let transform = SkyTransform::new(layout.center, layout.pixel_radius, min_el);

        let mut diagnostics = PlotDiagnostics {
            painting_skipped: !ui.is_rect_visible(layout.outer),
            ..Default::default()
        };
        if !diagnostics.painting_skipped {
            let painter = ui.painter_at(layout.outer);
            paint_sky(
                &painter,
                state,
                self.data,
                transform,
                self.mask_elevation_deg,
                self.color_scale,
                self.marker_shape,
                self.marker_radius,
                self.show_labels,
                self.show_grid,
                &self.title,
                ui.ctx().pixels_per_point(),
                &style,
                &mut diagnostics.polar,
                &mut diagnostics.visible_series,
            );
        }

        let legend_entries = layout.legend_rect.map_or_else(Vec::new, |rect| {
            show_sky_legend(
                ui,
                state,
                self.data,
                group_count,
                rect,
                id.with("legend"),
                &style,
            )
        });

        let pointer = response
            .hover_pos()
            .filter(|p| !layout.legend_rect.is_some_and(|r| r.contains(*p)));
        let (hovered_item, hover_text) = if self.show_hover
            && ui.is_enabled()
            && !diagnostics.painting_skipped
            && let Some(pos) = pointer
        {
            diagnostics.polar.pick_queries += 1;
            let hit = pick_sky_hover(
                state,
                self.data,
                transform,
                self.mask_elevation_deg,
                pos,
                self.pick_radius,
            );
            let text = hit.as_ref().map(|h| {
                ui.painter_at(layout.plot_area).circle_stroke(
                    h.screen_pos,
                    self.marker_radius + 3.0,
                    Stroke::new(2.0, style.text_color),
                );
                let mask_note = if h.masked { " [masked]" } else { "" };
                match h.scalar {
                    Some(s) => format!(
                        "{}{mask_note}\naz={:.1}°  el={:.1}°  val={s:.3}",
                        h.label, h.azimuth_deg, h.elevation_deg
                    ),
                    None => format!(
                        "{}{mask_note}\naz={:.1}°  el={:.1}°",
                        h.label, h.azimuth_deg, h.elevation_deg
                    ),
                }
            });
            if let Some(ref msg) = text {
                response.clone().on_hover_ui_at_pointer(|ui| {
                    ui.monospace(msg);
                });
            }
            (hit, text)
        } else {
            (None, None)
        };

        SkyPlotResponse {
            outer_rect: layout.outer,
            plot_area: layout.plot_area,
            response,
            transform,
            diagnostics,
            view_changed,
            hovered_item,
            hover_text,
            legend_entries,
        }
    }
}

fn group_count(data: &SkyPlotData) -> usize {
    let named = data.group_names().len();
    let max_sat = data
        .satellites()
        .iter()
        .map(|s| s.group + 1)
        .max()
        .unwrap_or(0);
    let max_trk = data.tracks().iter().map(|t| t.group + 1).max().unwrap_or(0);
    let count = named.max(max_sat).max(max_trk);
    if count <= 1 && named == 0 { 0 } else { count }
}
