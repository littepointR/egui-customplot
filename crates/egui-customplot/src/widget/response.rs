use egui::Response;

use crate::{
    AxisTick, HoveredBar, HoveredCell, HoveredPoint, HoveredVector, LegendEntry, PlotDiagnostics,
    PlotTransform,
};

pub struct PlotResponse {
    /// Includes title, tick labels and axis labels, unlike `response.rect`.
    pub outer_rect: egui::Rect,
    pub response: Response,
    pub transform: PlotTransform,
    pub right_transform: Option<PlotTransform>,
    pub diagnostics: PlotDiagnostics,
    /// Per-pass result, not a once-only external event. Read final state after UI.
    pub view_changed: bool,
    pub axis_ticks: [Vec<AxisTick>; 2],
    pub right_axis_ticks: Vec<AxisTick>,
    pub hovered_point: Option<HoveredPoint>,
    pub hovered_cell: Option<HoveredCell>,
    pub hovered_bar: Option<HoveredBar>,
    /// Raw vector record plus the chosen origin/endpoint. hovered_point also
    /// contains this anchor, preserving the existing HoverInfo callback contract.
    pub hovered_vector: Option<HoveredVector>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}
