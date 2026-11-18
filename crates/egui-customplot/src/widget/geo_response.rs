use crate::{GeoTransform, HoveredGeoPoint, LegendEntry, PlotDiagnostics, PlotResponse};
use egui::Response;
/// Layout geometry, [`GeoTransform`], hover point, and diagnostics returned by `GeoPlot::show`.
///
/// `GeoPlot::show` 返回的地图区域、地理坐标变换、悬停命中与诊断信息。
pub struct GeoPlotResponse {
    pub outer_rect: egui::Rect,
    pub plot_area: egui::Rect,
    pub viewport: egui::Rect,
    pub response: Response,
    pub transform: GeoTransform,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_point: Option<HoveredGeoPoint>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}

impl GeoPlotResponse {
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
