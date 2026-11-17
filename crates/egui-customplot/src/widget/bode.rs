use std::sync::Arc;

use egui::{AsIdSalt, Color32, IdSalt, Rect, Response, Sense, Ui, UiBuilder, vec2};

use crate::{
    Annotation, Axis, AxisLink, AxisScale, FrequencyResponseData, Line, Plot, PlotResponse,
    PlotState, PlotStyle, PlotTheme,
};

/// Caller-owned plot states for magnitude (top) and phase (bottom) panels of [`BodePlot`].
///
/// 伯德图组合视图状态（分别持久化幅频子图与相频子图的交互视图）。
#[derive(Clone, Debug, Default)]
pub struct BodePlotState {
    pub mag: PlotState,
    pub phase: PlotState,
}

/// Output geometry and synced sub-plot responses returned by [`BodePlot::show`].
///
/// [`BodePlot::show`] 返回的外框区域及幅频/相频联动画布响应。
pub struct BodePlotResponse {
    pub outer_rect: Rect,
    pub response: Response,
    pub mag: PlotResponse,
    pub phase: PlotResponse,
    pub hover_text: Option<String>,
}

impl BodePlotResponse {
    pub fn into_plot_response(self) -> PlotResponse {
        PlotResponse::from_embedded(
            self.outer_rect,
            self.mag.transform.rect(),
            self.response,
            self.mag.diagnostics,
            self.mag.view_changed || self.phase.view_changed,
            self.hover_text,
            self.mag.legend_entries,
        )
    }
}

/// Paired Bode Magnitude (dB) + Unwrapped Phase (deg) widget with X-axis link & margins (`EXT-21`).
///
/// 对数频率轴联动伯德图控件：上下同步显示幅值(dB)与解卷绕相位(deg)，可选自动标注增益裕度与相位裕度。
pub struct BodePlot<'a> {
    id_salt: IdSalt,
    data: &'a FrequencyResponseData,
    height: f32,
    title: String,
    show_margins: bool,
    allow_scroll_zoom: bool,
    color: Option<Color32>,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

impl<'a> BodePlot<'a> {
    pub fn new(id_salt: impl AsIdSalt, data: &'a FrequencyResponseData) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            data,
            height: 290.0,
            title: String::new(),
            show_margins: true,
            allow_scroll_zoom: false,
            color: None,
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
    pub fn show_margins(mut self, show: bool) -> Self {
        self.show_margins = show;
        self
    }
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
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

    pub fn show(self, ui: &mut Ui, state: &mut BodePlotState) -> BodePlotResponse {
        let avail_w = ui.available_width().max(200.0);
        let (outer_rect, response) =
            ui.allocate_exact_size(vec2(avail_w, self.height), Sense::hover());

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

        let body_top = outer_rect.top() + title_h;
        let body_h = (outer_rect.bottom() - body_top).max(120.0);
        let gap = 4.0;
        let mag_h = ((body_h - gap) * 0.5).max(58.0);
        let mag_rect = Rect::from_min_max(
            egui::pos2(outer_rect.left(), body_top),
            egui::pos2(outer_rect.right(), body_top + mag_h),
        );
        let phase_rect = Rect::from_min_max(
            egui::pos2(outer_rect.left(), body_top + mag_h + gap),
            outer_rect.max,
        );

        let margins = self.data.margins();
        let mut mag_ann = vec![Annotation::y_line(0.0, "0 dB")];
        let mut phase_ann = vec![Annotation::y_line(-180.0, "-180°")];
        if self.show_margins {
            if let Some(w_pc) = margins.phase_crossover_rad_s {
                let lbl = margins
                    .gain_margin_db
                    .map_or_else(|| String::from("GM"), |g| format!("GM {g:.1} dB"));
                mag_ann.push(Annotation::x_line(w_pc, lbl.clone()));
                phase_ann.push(Annotation::x_line(w_pc, lbl));
            }
            if let Some(w_gc) = margins.gain_crossover_rad_s {
                let lbl = margins
                    .phase_margin_deg
                    .map_or_else(|| String::from("PM"), |p| format!("PM {p:.1}°"));
                mag_ann.push(Annotation::x_line(w_gc, lbl.clone()));
                phase_ann.push(Annotation::x_line(w_gc, lbl));
            }
        }

        let line_col = self.color.unwrap_or_else(|| Color32::from_rgb(0, 114, 189));

        let mut mag_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id_salt, "mag"))
                .max_rect(mag_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        mag_ui.set_clip_rect(ui.clip_rect().intersect(mag_rect));
        let mut mag_plot = Plot::new((self.id_salt, "mag_p"))
            .height(mag_rect.height())
            .allow_scroll_zoom(self.allow_scroll_zoom)
            .x_axis(
                Axis::new()
                    .scale(AxisScale::Log10)
                    .tick_labels_visible(false),
            )
            .y_axis(Axis::new().label("Mag (dB)"))
            .annotations(mag_ann)
            .inherited_style(self.inherited_style)
            .style(self.style);
        if let Some(t) = self.theme.as_ref() {
            mag_plot = mag_plot.theme(t.clone());
        }
        let mag = mag_plot.show(
            &mut mag_ui,
            &mut state.mag,
            &[Line::new(self.data.mag_curve()).color(line_col).width(1.8)],
        );

        let mut phase_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id_salt, "phase"))
                .max_rect(phase_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        phase_ui.set_clip_rect(ui.clip_rect().intersect(phase_rect));
        let mut phase_plot = Plot::new((self.id_salt, "phase_p"))
            .height(phase_rect.height())
            .allow_scroll_zoom(self.allow_scroll_zoom)
            .x_axis(
                Axis::new()
                    .scale(AxisScale::Log10)
                    .label("Frequency (rad/s)"),
            )
            .y_axis(Axis::new().label("Phase (deg)"))
            .annotations(phase_ann)
            .inherited_style(self.inherited_style)
            .style(self.style);
        if let Some(t) = self.theme.as_ref() {
            phase_plot = phase_plot.theme(t.clone());
        }
        let phase = phase_plot.show(
            &mut phase_ui,
            &mut state.phase,
            &[Line::new(self.data.phase_curve())
                .color(line_col)
                .width(1.8)],
        );

        let ctx = ui.ctx().clone();
        let mut x_refs = [&mut state.mag, &mut state.phase];
        let _ = AxisLink::new([true, false]).synchronize(&ctx, &mut x_refs);

        let hover_text = mag.hover_text.clone().or_else(|| phase.hover_text.clone());
        BodePlotResponse {
            outer_rect,
            response,
            mag,
            phase,
            hover_text,
        }
    }
}
