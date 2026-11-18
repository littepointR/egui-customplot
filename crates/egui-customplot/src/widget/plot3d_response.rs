use crate::{
    Hovered3DBar, Hovered3DPoint, Hovered3DRegion, LegendEntry, PlotDiagnostics, PlotResponse,
    Transform3D,
};
use egui::Response;

/// Output geometry, hit-test result, and diagnostics returned by [`crate::Plot3D::show`].
///
/// [`crate::Plot3D::show`] 返回的布局信息、三维拾取命中结果与渲染诊断计数。
pub struct Plot3DResponse {
    pub outer_rect: egui::Rect,
    pub plot_area: egui::Rect,
    pub response: Response,
    pub transform: Transform3D,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_point: Option<Hovered3DPoint>,
    pub hovered_bar: Option<Hovered3DBar>,
    pub hovered_region: Option<Hovered3DRegion>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}

impl Plot3DResponse {
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
