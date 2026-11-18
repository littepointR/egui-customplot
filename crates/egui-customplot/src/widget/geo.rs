use super::GeoPlotResponse;
use std::sync::Arc;

use egui::{AsIdSalt, Color32, IdSalt, Sense, Stroke, Ui};

use crate::{
    BubbleLegend, ColorScale, GeoAttribution, GeoBasemapStyle, GeoBounds, GeoLayer, GeoPlotState,
    GeoProjection, GeoTileCache, GeoTransform, PlotStyle, PlotTheme,
    interaction::{navigate_geo, pick_geo_hover, show_geo_legend},
    layout::{allocate_plot, bubble_legend_size, compute_geo_layout, paint_bubble_legend},
    model::{resolve_plot_style, summarize_geo},
    render::paint_geo,
};

/// 2D geographic coordinate map widget (`geoplot`, `geoscatter`, `geodensityplot`, `geobubble`).
/// Never uses MathWorks basemap services or initiates network I/O.
///
/// 离线二维地理坐标地图控件：支持等经纬/墨卡托投影、本地离线底图/合成地形瓦片，不触发任何网络请求。
pub struct GeoPlot {
    id_salt: IdSalt,
    height: f32,
    title: String,
    projection: GeoProjection,
    basemap: GeoBasemapStyle,
    attribution: Option<GeoAttribution>,
    manual_bounds: Option<GeoBounds>,
    manual_lat_lim: Option<[f64; 2]>,
    manual_lon_lim: Option<[f64; 2]>,
    allow_scroll_zoom: bool,
    show_grid: bool,
    decorations: crate::layout::DecorationSlots,
    legend_component: Option<Arc<dyn crate::LegendComponent>>,
    show_legend: bool,
    show_hover: bool,
    pick_radius: f32,
    bubble_legend: Option<BubbleLegend>,
    color_bar: Option<(ColorScale, String)>,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl GeoPlot {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 300.0,
            title: String::new(),
            projection: GeoProjection::default(),
            basemap: GeoBasemapStyle::default(),
            attribution: None,
            manual_bounds: None,
            manual_lat_lim: None,
            manual_lon_lim: None,
            allow_scroll_zoom: false,
            show_grid: true,
            decorations: Default::default(),
            legend_component: None,
            show_legend: true,
            show_hover: true,
            pick_radius: 10.0,
            bubble_legend: None,
            color_bar: None,
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

    pub fn projection(mut self, projection: GeoProjection) -> Self {
        self.projection = projection;
        self
    }

    pub fn basemap(mut self, basemap: GeoBasemapStyle) -> Self {
        self.basemap = basemap;
        self
    }

    pub fn attribution(mut self, attribution: GeoAttribution) -> Self {
        self.attribution = Some(attribution);
        self
    }

    pub fn geo_bounds(mut self, bounds: GeoBounds) -> Self {
        self.manual_bounds = Some(bounds);
        self
    }

    pub fn lat_lim(mut self, lat_lim: [f64; 2]) -> Self {
        self.manual_lat_lim = Some(lat_lim);
        self
    }

    pub fn lon_lim(mut self, lon_lim: [f64; 2]) -> Self {
        self.manual_lon_lim = Some(lon_lim);
        self
    }

    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }

    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    pub fn title_component(mut self, component: Arc<dyn crate::PlotDecoration>) -> Self {
        self.decorations.title = Some(component);
        self
    }
    pub fn legend_component(mut self, component: Arc<dyn crate::LegendComponent>) -> Self {
        self.legend_component = Some(component);
        self.show_legend = true;
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

    pub fn pick_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() {
            self.pick_radius = radius.max(1.0);
        }
        self
    }

    pub fn bubble_legend(mut self, legend: BubbleLegend) -> Self {
        self.bubble_legend = Some(legend);
        self
    }

    pub fn color_bar_component(mut self, c: Arc<dyn crate::PlotDecoration>) -> Self {
        self.decorations.color_bar = Some(c);
        self
    }
    pub fn color_bar(mut self, scale: ColorScale, label: impl Into<String>) -> Self {
        self.color_bar = Some((scale, label.into()));
        self
    }

    pub fn inherited_style(mut self, style: PlotStyle) -> Self {
        self.inherited_style = style.over(self.inherited_style);
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

    pub fn show(
        self,
        ui: &mut Ui,
        state: &mut GeoPlotState,
        layers: &[GeoLayer<'_>],
    ) -> GeoPlotResponse {
        self.show_internal(ui, state, layers, None)
    }

    pub fn show_with_tiles(
        self,
        ui: &mut Ui,
        state: &mut GeoPlotState,
        layers: &[GeoLayer<'_>],
        tiles: &mut GeoTileCache,
    ) -> GeoPlotResponse {
        self.show_internal(ui, state, layers, Some(tiles))
    }

    fn resolved_auto_bounds(&self, auto: GeoBounds) -> (GeoBounds, bool) {
        if let Some(b) = self.manual_bounds {
            return (b, false);
        }
        if self.manual_lat_lim.is_none() && self.manual_lon_lim.is_none() {
            return (auto, false);
        }
        let lat = self.manual_lat_lim.unwrap_or([auto.lat_min, auto.lat_max]);
        let lon = self.manual_lon_lim.unwrap_or([
            crate::axis::wrap_lon_180(auto.center_lon - 0.5 * auto.lon_span),
            crate::axis::wrap_lon_180(auto.center_lon + 0.5 * auto.lon_span),
        ]);
        match GeoBounds::from_lat_lon(lat, lon) {
            Ok(b) => (b, false),
            Err(_) => (auto, true),
        }
    }

    fn show_internal(
        mut self,
        ui: &mut Ui,
        state: &mut GeoPlotState,
        layers: &[GeoLayer<'_>],
        tile_cache: Option<&mut GeoTileCache>,
    ) -> GeoPlotResponse {
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
        let mut body = outer;
        let placed = self
            .decorations
            .reserve(ui, &mut body, &self.title, self.color_bar.as_ref());
        let title = if self.decorations.title.is_some() {
            ""
        } else {
            &self.title
        };
        let style = resolve_plot_style(ui, self.theme.as_deref(), self.inherited_style, self.style);
        let (mut diagnostics, present, auto_raw) = summarize_geo(layers, state);
        state.retain_layers(&present);
        let (auto_bounds, invalid_manual) = self.resolved_auto_bounds(auto_raw);
        if invalid_manual {
            diagnostics.invalid_manual_bounds = true;
        }

        let names = if self.show_legend {
            crate::layout::geo_legend_names(layers, state)
        } else {
            Vec::new()
        };
        let custom = self.legend_component.as_deref().and_then(|c| {
            crate::layout::reserve_component(
                ui,
                &mut body,
                names.iter().copied(),
                c,
                crate::LegendPosition::OutsideRight,
            )
        });
        let legend_rows = if self.legend_component.is_some() {
            0
        } else {
            names.len()
        };
        let bubble_samples = self
            .bubble_legend
            .as_ref()
            .map(BubbleLegend::resolved_samples)
            .filter(|s| !s.is_empty());
        let bubble_size = self
            .bubble_legend
            .as_ref()
            .zip(bubble_samples.as_ref())
            .map(|(b, s)| bubble_legend_size(b, s));
        let mut layout = compute_geo_layout(
            body,
            !title.is_empty(),
            legend_rows,
            bubble_size,
            self.color_bar.is_some() && self.decorations.color_bar.is_none(),
        );

        layout.outer = outer;
        layout.legend_rect = custom.or(layout.legend_rect);
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let mut response = ui.interact(layout.plot_area, id, Sense::click_and_drag());
        let nav = navigate_geo(
            ui,
            state,
            &mut response,
            auto_bounds,
            self.projection,
            self.allow_scroll_zoom,
            layout.legend_rect,
            layout.plot_area,
        );
        if nav.changed {
            response.mark_changed();
        }
        let transform = GeoTransform::new(nav.bounds, self.projection, layout.plot_area)
            .unwrap_or_else(|| {
                GeoTransform::new(GeoBounds::world(), self.projection, layout.plot_area)
                    .expect("valid default geo viewport")
            });

        diagnostics.painting_skipped = !ui.is_rect_visible(layout.outer);
        if !diagnostics.painting_skipped {
            let painter = ui.painter_at(layout.outer);
            paint_geo(
                &painter,
                ui,
                state,
                layers,
                transform,
                self.basemap,
                tile_cache,
                self.attribution.as_ref(),
                title,
                self.show_grid,
                ui.ctx().pixels_per_point(),
                &style,
                &mut diagnostics.geo,
            );
            if let (Some(area), Some(legend), Some(samples)) = (
                layout.bubble_rect,
                &self.bubble_legend,
                bubble_samples.as_ref(),
            ) {
                paint_bubble_legend(ui, area, legend, samples, style.text_color);
            }
            if let (Some(bar_rect), Some((scale, label))) = (layout.color_bar_rect, &self.color_bar)
            {
                super::color_bar::paint_inset(
                    ui,
                    layout.outer,
                    bar_rect,
                    *scale,
                    label,
                    style.text_color,
                );
            }
        }

        crate::layout::show_decorations(ui, id, placed, style.text_color);
        let legend_entries = layout.legend_rect.map_or_else(Vec::new, |rect| {
            show_geo_legend(
                ui,
                state,
                layers,
                rect,
                id.with("legend"),
                self.legend_component.as_deref(),
                &style,
            )
        });

        let (hovered_point, hover_text) = if self.show_hover
            && ui.is_enabled()
            && !diagnostics.painting_skipped
            && let Some(pointer) = nav.hover_pos
        {
            diagnostics.geo.pick_queries += 1;
            let (hit, text) = pick_geo_hover(state, layers, transform, pointer, self.pick_radius);
            if let Some(ref h) = hit
                && !diagnostics.painting_skipped
            {
                ui.painter_at(transform.viewport()).circle_stroke(
                    h.screen_pos,
                    6.0,
                    Stroke::new(2.0, Color32::WHITE),
                );
            }
            if let Some(ref msg) = text {
                response.clone().on_hover_ui_at_pointer(|ui| {
                    ui.monospace(msg);
                });
            }
            (hit, text)
        } else {
            (None, None)
        };

        GeoPlotResponse {
            outer_rect: layout.outer,
            plot_area: layout.plot_area,
            viewport: transform.viewport(),
            response,
            transform,
            diagnostics,
            view_changed: nav.changed,
            hovered_point,
            hover_text,
            legend_entries,
        }
    }
}
