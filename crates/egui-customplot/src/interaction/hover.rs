use crate::{
    HoveredBar, HoveredCell, HoveredPoint, PlotDiagnostics, PlotLayer, PlotState,
    axis::SeriesTransforms, model::ResolvedPlotStyle,
};
use egui::Pos2;
use std::time::Instant;

pub(crate) struct HoverPick {
    pub point: Option<HoveredPoint>,
    pub cell: Option<HoveredCell>,
    pub bar: Option<HoveredBar>,
}

/// Filled heatmap cells occlude earlier point layers. Among eligible point
/// layers retain the established closest-distance / first-submission tie rule.
pub(crate) fn pick_hover(
    state: &PlotState,
    layers: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    position: Pos2,
    radius: f32,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
) -> HoverPick {
    let start = Instant::now();
    let mut cell = super::pick_cell(state, layers, transforms, position, diagnostics);
    let mut bar = super::bar::pick_bar(state, layers, transforms, position, diagnostics, style);
    if let (Some(c), Some(b)) = (&cell, &bar) {
        if c.layer_index > b.layer_index {
            bar = None;
        } else {
            cell = None;
        }
    }
    let first_layer = cell
        .as_ref()
        .map(|c| c.layer_index)
        .or(bar.as_ref().map(|b| b.layer_index))
        .map_or(0, |i| i + 1);
    let options = super::picking::PickOptions {
        position,
        radius,
        first_layer,
    };
    let point = super::picking::pick_point(state, layers, transforms, options, diagnostics, style);
    let cell = if point.is_some() { None } else { cell };
    let bar = if point.is_some() { None } else { bar };
    diagnostics.pick_duration = start.elapsed();
    HoverPick { point, cell, bar }
}
