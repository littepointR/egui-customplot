use egui::{Pos2, Stroke, Ui};

use crate::{
    HoveredBar, HoveredCell, HoveredPoint, PlotTransform,
    axis::{clipped_data_rect, grid_cell_rect},
    model::ResolvedPlotStyle,
};

pub(crate) fn paint_bar_hover(
    ui: &Ui,
    transform: PlotTransform,
    bar: &HoveredBar,
    style: &ResolvedPlotStyle<'_>,
) {
    if let Some((lo, hi)) = bar.bar.extent(bar.orientation)
        && let Some(rect) = clipped_data_rect(lo, hi, transform)
    {
        ui.painter().with_clip_rect(transform.rect()).rect_stroke(
            rect,
            0.0,
            Stroke::new(1.5, style.text_color),
            egui::StrokeKind::Inside,
        );
    }
}

pub(crate) fn paint_cell_hover(
    ui: &Ui,
    transform: PlotTransform,
    cell: &HoveredCell,
    style: &ResolvedPlotStyle<'_>,
) {
    if let Some(rect) = grid_cell_rect(cell.bounds, transform) {
        ui.painter().with_clip_rect(transform.rect()).rect_stroke(
            rect,
            0.0,
            Stroke::new(1.5, style.text_color),
            egui::StrokeKind::Inside,
        );
    }
}

pub(crate) fn paint_hover(
    ui: &Ui,
    transform: PlotTransform,
    position: Pos2,
    closest: Option<&HoveredPoint>,
    crosshair: bool,
    style: &ResolvedPlotStyle<'_>,
) {
    let painter = ui.painter().with_clip_rect(transform.rect());
    if crosshair {
        let rect = transform.rect();
        let stroke = style.crosshair_stroke;
        painter.line_segment(
            [
                egui::pos2(position.x, rect.top()),
                egui::pos2(position.x, rect.bottom()),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(rect.left(), position.y),
                egui::pos2(rect.right(), position.y),
            ],
            stroke,
        );
    }
    if let Some(point) = closest {
        painter.circle_stroke(
            transform.position_from_point(point.point),
            5.0,
            Stroke::new(1.5, style.text_color),
        );
    }
}
