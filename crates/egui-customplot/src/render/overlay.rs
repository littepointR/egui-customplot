use egui::{Pos2, Stroke, Ui};

use crate::{HoveredPoint, PlotTransform};

pub(crate) fn paint_hover(
    ui: &Ui,
    transform: PlotTransform,
    position: Pos2,
    closest: Option<&HoveredPoint>,
    crosshair: bool,
) {
    let painter = ui.painter().with_clip_rect(transform.rect());
    if crosshair {
        let rect = transform.rect();
        let stroke = Stroke::new(1.0, ui.visuals().weak_text_color());
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
            Stroke::new(1.5, ui.visuals().text_color()),
        );
    }
}
