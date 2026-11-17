use egui::{Rect, Response};

use crate::{
    AxisScale, AxisTick, HoveredBar, HoveredCell, HoveredPoint, HoveredRegion, HoveredVector,
    LegendEntry, PlotBounds, PlotDiagnostics, PlotTransform,
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
    pub hovered_region: Option<HoveredRegion>,
    /// Raw vector record plus the chosen origin/endpoint. hovered_point also
    /// contains this anchor, preserving the existing HoverInfo callback contract.
    pub hovered_vector: Option<HoveredVector>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
    pub brushed_bounds: Option<PlotBounds>,
    pub brush_finished: bool,
}

impl PlotResponse {
    pub(crate) fn from_embedded(
        outer_rect: Rect,
        plot_area: Rect,
        response: Response,
        diagnostics: PlotDiagnostics,
        view_changed: bool,
        hover_text: Option<String>,
        legend_entries: Vec<LegendEntry>,
    ) -> Self {
        let bounds = PlotBounds::new([0.0, 0.0], [1.0, 1.0]).expect("unit bounds");
        let transform =
            PlotTransform::with_axes(bounds, plot_area, [AxisScale::Linear; 2], [false; 2])
                .expect("valid embedded plot area");
        Self {
            outer_rect,
            response,
            transform,
            right_transform: None,
            diagnostics,
            view_changed,
            axis_ticks: [Vec::new(), Vec::new()],
            right_axis_ticks: Vec::new(),
            hovered_point: None,
            hovered_cell: None,
            hovered_bar: None,
            hovered_region: None,
            hovered_vector: None,
            hover_text,
            legend_entries,
            brushed_bounds: None,
            brush_finished: false,
        }
    }
}
