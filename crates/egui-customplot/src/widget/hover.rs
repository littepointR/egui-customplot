use crate::{
    HoverInfo, PlotDiagnostics, PlotLayer, PlotState,
    axis::SeriesTransforms,
    interaction::{HoverPick, pick_hover, show_tooltip},
    model::ResolvedPlotStyle,
    render::{paint_bar_hover, paint_cell_hover, paint_hover, paint_region_hover},
};
use egui::{Pos2, Response, Ui};

pub(super) struct HoverContext<'a> {
    pub plot: &'a super::Plot,
    pub response: &'a Response,
    pub transforms: SeriesTransforms,
    pub position: Pos2,
}

pub(super) fn show_hover(
    ui: &Ui,
    state: &PlotState,
    layers: &[PlotLayer<'_>],
    hover: HoverContext<'_>,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
) -> (HoverPick, String) {
    let picked = pick_hover(
        state,
        layers,
        hover.transforms,
        hover.position,
        hover.plot.pick_radius,
        diagnostics,
        style,
    );
    let axis = picked
        .point
        .as_ref()
        .map(|p| p.y_axis)
        .or(picked.cell.as_ref().map(|c| c.y_axis))
        .or(picked.bar.as_ref().map(|b| b.y_axis))
        .or(picked.region.as_ref().map(|r| r.y_axis));
    let transform = axis.map_or(hover.transforms.left, |a| hover.transforms.for_axis(a));
    paint_hover(
        ui,
        transform,
        hover.position,
        picked.point.as_ref(),
        hover.plot.crosshair,
        style,
    );
    if let Some(cell) = &picked.cell {
        paint_cell_hover(ui, transform, cell, style);
    }
    if let Some(bar) = &picked.bar {
        paint_bar_hover(ui, transform, bar, style);
    }
    if let Some(region) = &picked.region {
        paint_region_hover(ui, transform, region, style);
    }
    let index = picked
        .point
        .as_ref()
        .map(|p| p.series_index)
        .or(picked.cell.as_ref().map(|c| c.layer_index))
        .or(picked.bar.as_ref().map(|b| b.layer_index))
        .or(picked.region.as_ref().map(|r| r.layer_index));
    let name = index.map_or("", |i| layers[i].name());
    let bar_point = picked.bar.as_ref().map(|b| b.as_point());
    let info = HoverInfo {
        hovered_point: picked.point.as_ref().or(bar_point.as_ref()),
        cursor: transform.point_from_position(hover.position),
        series_name: name,
    };
    let text = hover.plot.hover_formatter.as_ref().map_or_else(
        || {
            if let Some(vector) = &picked.vector {
                vector.default_text(name)
            } else if let Some(bar) = &picked.bar {
                bar.default_text(name)
            } else if let Some(cell) = &picked.cell {
                cell.default_text(name)
            } else if let Some(region) = &picked.region {
                region.default_text(name)
            } else {
                info.default_text()
            }
        },
        |format| format(info),
    );
    show_tooltip(hover.response, &text);
    (picked, text)
}
