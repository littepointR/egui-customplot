use std::sync::Arc;

use egui::{AsIdSalt, IdSalt, Rect, Response, Sense, Ui, UiBuilder, vec2};

use crate::{
    Axis, AxisLink, BarOrientation, ClustergramData, ColorBar, ColorMap, ColorScale,
    DendrogramPlot, Heatmap, Plot, PlotResponse, PlotState, PlotStyle, PlotTheme,
};

/// Persistent view state across passes for [`Clustergram`].
///
/// [`Clustergram`] 三面板联动（顶/左系统树与中心重排热力图）的宿主持有交互状态。
#[derive(Clone, Debug, Default)]
pub struct ClustergramState {
    pub top: PlotState,
    pub left: PlotState,
    pub heatmap: PlotState,
}

/// Output rects and aligned dendrogram/heatmap responses returned by [`Clustergram::show`].
///
/// [`Clustergram::show`] 返回的区域矩形及顶/左树状图与热力图子响应。
pub struct ClustergramResponse {
    pub outer_rect: Rect,
    pub heatmap_rect: Rect,
    pub response: Response,
    pub top_dendrogram: PlotResponse,
    pub left_dendrogram: PlotResponse,
    pub heatmap: PlotResponse,
    pub hover_text: Option<String>,
}

impl ClustergramResponse {
    pub fn into_plot_response(self) -> PlotResponse {
        PlotResponse::from_embedded(
            self.outer_rect,
            self.heatmap_rect,
            self.response,
            self.heatmap.diagnostics,
            self.heatmap.view_changed,
            self.hover_text,
            self.heatmap.legend_entries,
        )
    }
}

/// Hierarchical clustergram combining row/column dendrograms, reordered heatmap, and colorbar (`EXT-15`).
///
/// 层次聚类热力图控件：组合行列聚类树状图、重排序标准化热力矩阵与可选色条。
pub struct Clustergram<'a> {
    id_salt: IdSalt,
    data: &'a ClustergramData,
    height: f32,
    title: String,
    color_scale: Option<ColorScale>,
    color_map: ColorMap,
    show_colorbar: bool,
    allow_scroll_zoom: bool,
    dendro_ratio: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl<'a> Clustergram<'a> {
    pub fn new(id_salt: impl AsIdSalt, data: &'a ClustergramData) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            data,
            height: 320.0,
            title: String::new(),
            color_scale: None,
            color_map: ColorMap::BlueRed,
            show_colorbar: true,
            allow_scroll_zoom: false,
            dendro_ratio: 0.22,
            inherited_style: PlotStyle::default(),
            style: PlotStyle::default(),
            theme: None,
        }
    }
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.clamp(160.0, 4096.0);
        }
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn show_colorbar(mut self, show: bool) -> Self {
        self.show_colorbar = show;
        self
    }
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }
    pub fn dendro_ratio(mut self, ratio: f32) -> Self {
        if ratio.is_finite() {
            self.dendro_ratio = ratio.clamp(0.12, 0.40);
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

    pub fn show(self, ui: &mut Ui, state: &mut ClustergramState) -> ClustergramResponse {
        let avail_w = ui.available_width().max(220.0);
        let total_size = vec2(avail_w, self.height);
        let (outer_rect, response) = ui.allocate_exact_size(total_size, Sense::hover());

        let title_h = if self.title.is_empty() { 0.0 } else { 22.0 };
        if !self.title.is_empty() && ui.is_rect_visible(outer_rect) {
            let text_col = ui.visuals().text_color();
            ui.painter().text(
                egui::pos2(outer_rect.center().x, outer_rect.top() + 3.0),
                egui::Align2::CENTER_TOP,
                &self.title,
                egui::FontId::proportional(13.0),
                text_col,
            );
        }

        let body = Rect::from_min_max(
            egui::pos2(outer_rect.left(), outer_rect.top() + title_h),
            outer_rect.max,
        );
        let cb_w = if self.show_colorbar && body.width() > 260.0 {
            62.0
        } else {
            0.0
        };
        let main_w = (body.width() - cb_w).max(160.0);
        let left_w = (main_w * self.dendro_ratio).clamp(54.0, main_w * 0.42);
        let top_h = (body.height() * self.dendro_ratio).clamp(44.0, body.height() * 0.40);
        let gap = 4.0;

        let x_left_end = body.left() + left_w;
        let x_heat_start = (x_left_end + gap).min(body.right() - cb_w - 60.0);
        let x_heat_end =
            (body.right() - cb_w - if cb_w > 0.0 { gap } else { 0.0 }).max(x_heat_start + 60.0);
        let y_top_end = body.top() + top_h;
        let y_heat_start = (y_top_end + gap).min(body.bottom() - 60.0);

        let top_rect = Rect::from_min_max(
            egui::pos2(x_heat_start, body.top()),
            egui::pos2(x_heat_end, y_top_end),
        );
        let left_rect = Rect::from_min_max(
            egui::pos2(body.left(), y_heat_start),
            egui::pos2(x_left_end, body.bottom()),
        );
        let heat_rect = Rect::from_min_max(
            egui::pos2(x_heat_start, y_heat_start),
            egui::pos2(x_heat_end, body.bottom()),
        );

        let scale = self.color_scale.unwrap_or_else(|| {
            ColorScale::new(self.data.value_range())
                .expect("clustergram value range")
                .color_map(self.color_map)
        });

        let mut top_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id_salt, "top"))
                .max_rect(top_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        top_ui.set_clip_rect(ui.clip_rect().intersect(top_rect));
        let mut top_plot = Plot::new((self.id_salt, "top_dendro"))
            .height(top_rect.height())
            .allow_scroll_zoom(self.allow_scroll_zoom)
            .x_axis(Axis::new().visible(false))
            .y_axis(Axis::new().visible(false))
            .show_grid(false)
            .show_frame(false)
            .inherited_style(self.inherited_style)
            .style(self.style);
        if let Some(t) = self.theme.as_ref() {
            top_plot = top_plot.theme(t.clone());
        }
        let top_layer =
            DendrogramPlot::new(self.data.col_dendrogram()).orientation(BarOrientation::Vertical);
        let top_dendrogram = top_plot.show_layers(&mut top_ui, &mut state.top, &[top_layer.into()]);

        let mut left_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id_salt, "left"))
                .max_rect(left_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        left_ui.set_clip_rect(ui.clip_rect().intersect(left_rect));
        let mut left_plot = Plot::new((self.id_salt, "left_dendro"))
            .height(left_rect.height())
            .allow_scroll_zoom(self.allow_scroll_zoom)
            .x_axis(Axis::new().reversed(true).formatter(|_| String::from(" ")))
            .y_axis(
                Axis::new()
                    .reversed(true)
                    .explicit_ticks(self.data.ordered_row_ticks()),
            )
            .show_grid(false)
            .show_frame(false)
            .inherited_style(self.inherited_style)
            .style(self.style);
        if let Some(t) = self.theme.as_ref() {
            left_plot = left_plot.theme(t.clone());
        }
        let left_layer =
            DendrogramPlot::new(self.data.row_dendrogram()).orientation(BarOrientation::Horizontal);
        let left_dendrogram =
            left_plot.show_layers(&mut left_ui, &mut state.left, &[left_layer.into()]);

        let mut heat_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id_salt, "heat"))
                .max_rect(heat_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        heat_ui.set_clip_rect(ui.clip_rect().intersect(heat_rect));
        let mut heat_plot = Plot::new((self.id_salt, "heatmap"))
            .height(heat_rect.height())
            .allow_scroll_zoom(self.allow_scroll_zoom)
            .x_axis(Axis::new().explicit_ticks(self.data.ordered_col_ticks()))
            .y_axis(Axis::new().reversed(true).tick_labels_visible(false))
            .show_grid(false)
            .inherited_style(self.inherited_style)
            .style(self.style);
        if let Some(t) = self.theme.as_ref() {
            heat_plot = heat_plot.theme(t.clone());
        }
        let heat_layer = Heatmap::new(self.data.grid(), scale);
        let heatmap = heat_plot.show_layers(&mut heat_ui, &mut state.heatmap, &[heat_layer.into()]);

        if cb_w > 0.0 {
            let cb_rect = Rect::from_min_max(
                egui::pos2(x_heat_end + gap, y_heat_start),
                egui::pos2(body.right(), body.bottom()),
            );
            let mut cb_ui = ui.new_child(
                UiBuilder::new()
                    .id_salt((self.id_salt, "cb"))
                    .max_rect(cb_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            cb_ui.set_clip_rect(ui.clip_rect().intersect(cb_rect));
            ColorBar::new(scale)
                .height(cb_rect.height())
                .show(&mut cb_ui);
        }

        let ctx = ui.ctx().clone();
        let mut x_refs = [&mut state.top, &mut state.heatmap];
        let _ = AxisLink::new([true, false]).synchronize(&ctx, &mut x_refs);
        let mut y_refs = [&mut state.left, &mut state.heatmap];
        let _ = AxisLink::new([false, true]).synchronize(&ctx, &mut y_refs);

        let heatmap_rect = heatmap.transform.rect();
        let hover_text = heatmap
            .hover_text
            .clone()
            .or_else(|| top_dendrogram.hover_text.clone())
            .or_else(|| left_dendrogram.hover_text.clone());

        ClustergramResponse {
            outer_rect,
            heatmap_rect,
            response,
            top_dendrogram,
            left_dendrogram,
            heatmap,
            hover_text,
        }
    }
}
