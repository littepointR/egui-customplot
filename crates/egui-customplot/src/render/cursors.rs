use crate::{CursorMeasurements, PlotTransform};
use egui::{Align2, Pos2, Stroke, TextStyle, Ui};

pub(crate) fn paint_shared_cursors(
    ui: &Ui,
    transform: PlotTransform,
    hover: Option<f64>,
    pins: CursorMeasurements,
) {
    let rect = transform.rect();
    let painter = ui.painter().with_clip_rect(rect);
    for (value, label, color) in [
        (hover, "", ui.visuals().weak_text_color()),
        (pins.a.map(|r| r.point[0]), "A", ui.visuals().text_color()),
        (
            pins.b.map(|r| r.point[0]),
            "B",
            ui.visuals().selection.stroke.color,
        ),
    ] {
        let Some(value) = value else {
            continue;
        };
        if value < transform.bounds().min()[0] || value > transform.bounds().max()[0] {
            continue;
        }
        let x = transform
            .position_from_point([value, transform.bounds().min()[1]])
            .x;
        if !x.is_finite() {
            continue;
        }
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(1.0, color),
        );
        if !label.is_empty() {
            painter.text(
                Pos2::new(x + 2.0, rect.top() + 2.0),
                Align2::LEFT_TOP,
                label,
                TextStyle::Small.resolve(ui.style()),
                color,
            );
        }
    }
}
