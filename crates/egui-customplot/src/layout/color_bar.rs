use egui::{Rect, Response, Ui};

pub(crate) fn allocate_color_bar(
    ui: &mut Ui,
    height: f32,
    has_label: bool,
) -> (Rect, Rect, Response) {
    let (outer, response) = ui.allocate_exact_size(egui::vec2(104.0, height), egui::Sense::hover());
    let top = if has_label { 24.0 } else { 8.0 };
    let bar = Rect::from_min_max(
        outer.min + egui::vec2(0.0, top),
        egui::pos2(outer.left() + 16.0, outer.bottom() - 8.0),
    );
    (outer, bar, response)
}
