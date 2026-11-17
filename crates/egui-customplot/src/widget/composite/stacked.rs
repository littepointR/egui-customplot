use egui::Ui;

use crate::{
    Axis, AxisLink, Figure, FigureError, FigureResponse, LinkReport, Plot, PlotLayer, PlotResponse,
    PlotState,
};

/// Stacked sub-plot figures and X-axis link arbitration summary returned by `StackedPlot::show`.
///
/// `StackedPlot::show` 返回的多行子图结果与水平轴联动报告。
pub struct StackedPlotResponse {
    pub figure: FigureResponse<PlotResponse>,
    pub link_report: Option<LinkReport>,
}

/// Vertically stacked plots sharing a synchronized X axis (`stackedplot`, `ML-006`).
///
/// 共享同步 X 轴的垂直堆叠多图控件（同一 X 视窗联动平移与缩放）。
pub struct StackedPlot<'a> {
    id_salt: &'a str,
    panels: Vec<(&'a str, Vec<PlotLayer<'a>>)>,
    row_height: f32,
    x_label: String,
    title: String,
    allow_scroll_zoom: bool,
}

impl<'a> StackedPlot<'a> {
    pub fn new(id_salt: &'a str, panels: Vec<(&'a str, Vec<PlotLayer<'a>>)>) -> Self {
        Self {
            id_salt,
            panels,
            row_height: 150.0,
            x_label: String::new(),
            title: String::new(),
            allow_scroll_zoom: false,
        }
    }

    pub fn row_height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.row_height = height.max(64.0);
        }
        self
    }

    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = label.into();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        states: &mut [PlotState],
    ) -> Result<StackedPlotResponse, FigureError> {
        let count = self.panels.len().min(states.len());
        if count == 0 {
            let empty = Figure::new(self.id_salt)
                .columns(1)
                .show(ui, 0, |_| unreachable!())?;
            return Ok(StackedPlotResponse {
                figure: empty,
                link_report: None,
            });
        }
        let ctx = ui.ctx().clone();
        ui.vertical(|ui| {
            if !self.title.is_empty() {
                ui.strong(&self.title);
            }
            let fig = Figure::new(self.id_salt)
                .columns(1)
                .cell_height(self.row_height)
                .spacing(egui::vec2(0.0, 6.0));
            let figure = fig.show(ui, count, |cell| {
                let idx = cell.index();
                let (y_label, layers) = &self.panels[idx];
                let is_bottom = idx + 1 == count;
                let mut x_axis = Axis::new().tick_labels_visible(is_bottom);
                if is_bottom && !self.x_label.is_empty() {
                    x_axis = x_axis.label(self.x_label.clone());
                }
                let plot = Plot::new((self.id_salt, idx))
                    .x_axis(x_axis)
                    .y_axis(Axis::new().label(*y_label))
                    .allow_scroll_zoom(self.allow_scroll_zoom)
                    .legend(layers.len() > 1);
                cell.show_layers(plot, &mut states[idx], layers)
            })?;
            let link_report = if count >= 2 {
                let mut refs: Vec<&mut PlotState> = states[..count].iter_mut().collect();
                AxisLink::new([true, false])
                    .synchronize(&ctx, &mut refs)
                    .ok()
            } else {
                None
            };
            Ok(StackedPlotResponse {
                figure,
                link_report,
            })
        })
        .inner
    }
}
