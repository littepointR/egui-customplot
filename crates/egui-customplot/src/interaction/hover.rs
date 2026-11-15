use super::region::{HoveredRegion, RegionKind, pick_region};
use crate::{
    HoveredBar, HoveredCell, HoveredPoint, HoveredVector, PlotDiagnostics, PlotLayer, PlotState,
    axis::SeriesTransforms, model::ResolvedPlotStyle,
};
use egui::Pos2;
use std::time::Instant;

pub(crate) struct HoverPick {
    pub point: Option<HoveredPoint>,
    pub cell: Option<HoveredCell>,
    pub bar: Option<HoveredBar>,
    pub vector: Option<HoveredVector>,
    pub region: Option<HoveredRegion>,
}

/// Filled layers (heatmap cells, bars, patches/areas/images) occlude earlier layers
/// strictly below them; `Area`'s upper stroke shares its own layer index.
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
    let mut region = pick_region(state, layers, transforms, position, diagnostics, style);
    let top_layer = cell
        .as_ref()
        .map(|c| c.layer_index)
        .into_iter()
        .chain(bar.as_ref().map(|b| b.layer_index))
        .chain(region.as_ref().map(|r| r.layer_index))
        .max();
    if let Some(top) = top_layer {
        if cell.as_ref().is_some_and(|c| c.layer_index != top) {
            cell = None;
        }
        if bar.as_ref().is_some_and(|b| b.layer_index != top) {
            bar = None;
        }
        if region.as_ref().is_some_and(|r| r.layer_index != top) {
            region = None;
        }
    }
    let first_layer = match top_layer {
        Some(top) if region.as_ref().is_some_and(|r| r.kind == RegionKind::Area) => top,
        Some(top) => top + 1,
        None => 0,
    };
    let options = super::picking::PickOptions {
        position,
        radius,
        first_layer,
    };
    let point = super::picking::pick_point(state, layers, transforms, options, diagnostics, style);
    let vector = point.as_ref().and_then(|p| match layers[p.series_index] {
        PlotLayer::Quiver(quiver) => Some(HoveredVector::from_point(p, quiver)),
        _ => None,
    });
    let (cell, bar, region) = if point.is_some() {
        (None, None, None)
    } else {
        (cell, bar, region)
    };
    diagnostics.pick_duration = start.elapsed();
    HoverPick {
        point,
        cell,
        bar,
        vector,
        region,
    }
}
