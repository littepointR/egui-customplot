use egui::Response;

use crate::{AxisTick, HoveredPoint, LegendEntry, PlotDiagnostics, PlotTransform};

pub struct PlotResponse {
    pub response: Response,
    pub transform: PlotTransform,
    pub diagnostics: PlotDiagnostics,
    /// Per-pass result, not a once-only external event. Read final state after UI.
    pub view_changed: bool,
    pub axis_ticks: [Vec<AxisTick>; 2],
    pub hovered_point: Option<HoveredPoint>,
    pub legend_entries: Vec<LegendEntry>,
}
