use egui::{Rect, Ui};

use crate::{Axis, AxisTick, PlotTransform, model::ResolvedPlotStyle};

/// Labels for the secondary Y axis. The primary grid and background are painted
/// once; this side never clears or repaints previously submitted data geometry.
pub(crate) fn paint_right_axis(
    ui: &Ui,
    outer: Rect,
    transform: PlotTransform,
    axis: &Axis,
    ticks: &[AxisTick],
    style: &ResolvedPlotStyle<'_>,
) {
    if !axis.visible {
        return;
    }
    let painter = ui.painter().with_clip_rect(outer);
    let rect = transform.rect();
    let font = egui::TextStyle::Small.resolve(ui.style());
    for tick in ticks.iter().filter(|t| t.major) {
        if !axis.tick_labels_visible || tick.label.is_empty() {
            continue;
        }
        let y = transform
            .position_from_point([transform.bounds().min()[0], tick.value])
            .y;
        painter.text(
            egui::pos2(rect.right() + 4.0, y),
            egui::Align2::LEFT_CENTER,
            &tick.label,
            font.clone(),
            style.text_color,
        );
    }
    if !axis.label.is_empty() {
        let galley = painter.layout_no_wrap(axis.label.clone(), font, style.text_color);
        let center = egui::pos2(outer.right() - 10.0, rect.center().y);
        let pos = center - galley.size() * 0.5;
        painter.add(
            egui::epaint::TextShape::new(pos, galley, style.text_color)
                .with_angle_and_anchor(std::f32::consts::FRAC_PI_2, egui::Align2::CENTER_CENTER),
        );
    }
}
