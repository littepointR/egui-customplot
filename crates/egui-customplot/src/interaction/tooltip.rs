use egui::Response;

use crate::{HoveredPoint, PlotPoint};

#[derive(Clone, Copy, Debug)]
pub struct HoverInfo<'a> {
    pub hovered_point: Option<&'a HoveredPoint>,
    pub cursor: PlotPoint,
    pub series_name: &'a str,
}

impl HoverInfo<'_> {
    pub fn default_text(self) -> String {
        if let Some(point) = self.hovered_point {
            let prefix = if self.series_name.is_empty() {
                String::new()
            } else {
                format!("{}\n", self.series_name)
            };
            format!(
                "{prefix}[{}]  x={:.8e}  y={:.8e}",
                point.point_index, point.point[0], point.point[1]
            )
        } else {
            format!("x={:.8e}  y={:.8e}", self.cursor[0], self.cursor[1])
        }
    }
}

pub(crate) type HoverFormatter = dyn for<'a> Fn(HoverInfo<'a>) -> String + Send + Sync;

pub(crate) fn show_tooltip(response: &Response, text: &str) {
    response.clone().on_hover_ui_at_pointer(|ui| {
        ui.monospace(text);
    });
}
