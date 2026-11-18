use egui::{Color32, IdSalt, Rect, Sense, Stroke, Ui};

use crate::{PlotLayer, PlotState, model::ResolvedPlotStyle};

/// Placement of a Cartesian plot legend within the allocated widget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegendPosition {
    #[default]
    InsideTopRight,
    OutsideRight,
    OutsideBottom,
}

pub(crate) fn reserve_legend(
    ui: &Ui,
    outer: &mut Rect,
    series: &[PlotLayer<'_>],
    placement: LegendPosition,
) -> Option<Rect> {
    if placement == LegendPosition::InsideTopRight {
        return None;
    }
    let names = series.iter().map(|s| s.name()).filter(|n| !n.is_empty());
    let count = names.clone().count();
    if count == 0 || outer.width() < 96.0 || outer.height() < 60.0 {
        return None;
    }
    let mut legend = *outer;
    if placement == LegendPosition::OutsideRight {
        let font = egui::TextStyle::Button.resolve(ui.style());
        let width = names
            .map(|n| {
                ui.painter()
                    .layout_no_wrap(n.to_owned(), font.clone(), ui.visuals().text_color())
                    .size()
                    .x
            })
            .fold(0.0, f32::max)
            + 56.0;
        legend.min.x = outer.right() - width.min(outer.width() * 0.42);
        outer.max.x = legend.left() - 4.0;
    } else {
        legend.min.y = outer.bottom() - (24.0 * count as f32 + 18.0).min(outer.height() * 0.4);
        outer.max.y = legend.top() - 4.0;
    }
    Some(legend)
}

#[derive(Clone, Debug)]
pub struct LegendEntry {
    pub id: IdSalt,
    pub name: String,
    pub rect: Rect,
    pub visible: bool,
}

pub(crate) fn legend_rect(rect: Rect, series: &[PlotLayer<'_>]) -> Option<Rect> {
    let count = series.iter().filter(|s| !s.name().is_empty()).count();
    if count == 0 || rect.width() < 100.0 || rect.height() < 50.0 {
        return None;
    }
    let size = egui::vec2(
        (rect.width() * 0.35).clamp(80.0, 220.0),
        (count as f32 * 24.0 + 18.0).min(rect.height() - 12.0),
    );
    Some(Rect::from_min_size(
        egui::pos2(rect.right() - size.x - 6.0, rect.top() + 6.0),
        size,
    ))
}

#[inline]
fn color_luma(c: Color32) -> f32 {
    0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
}

pub(crate) fn legend_item_button(
    ui: &mut Ui,
    name: &str,
    swatch: Color32,
    visible: bool,
    style: &ResolvedPlotStyle<'_>,
) -> egui::Response {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let fg = if visible {
        style.text_color
    } else {
        style.text_color.gamma_multiply(0.42)
    };
    let wrap_width = (ui.available_width() - 18.0 - 2.0 * ui.spacing().button_padding.x).max(1.0);
    let galley = ui
        .painter()
        .layout(name.to_owned(), font_id, fg, wrap_width);
    let pad = ui.spacing().button_padding;
    let swatch_w = 12.0_f32;
    let gap = 6.0_f32;
    let width =
        (pad.x * 2.0 + swatch_w + gap + galley.size().x).min(ui.available_width().max(24.0));
    let height = galley.size().y.max(12.0) + pad.y * 2.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(rect, 3.0, style.text_color.gamma_multiply(0.08));
        }
        let sw_rect = Rect::from_center_size(
            egui::pos2(rect.left() + pad.x + swatch_w * 0.5, rect.center().y),
            egui::vec2(swatch_w, 3.5),
        );
        let sw_col = if visible {
            swatch
        } else {
            swatch.gamma_multiply(0.32)
        };
        painter.rect_filled(sw_rect, 1.5, sw_col);
        if visible && (color_luma(swatch) - color_luma(style.background)).abs() < 45.0 {
            painter.rect_stroke(
                sw_rect,
                1.5,
                Stroke::new(0.7, style.text_color.gamma_multiply(0.35)),
                egui::StrokeKind::Outside,
            );
        }
        let text_pos = egui::pos2(
            rect.left() + pad.x + swatch_w + gap,
            rect.center().y - galley.size().y * 0.5,
        );
        painter.with_clip_rect(rect).galley(text_pos, galley, fg);
    }
    response
}

/// Submit legend last for ordered painting; visibility takes effect on the next
/// layout pass/repaint. Only state::toggle_series owns once-per-frame effects.
pub(crate) fn show_legend(
    ui: &mut Ui,
    state: &mut PlotState,
    series: &[PlotLayer<'_>],
    rect: Rect,
    id_salt: IdSalt,
    component: Option<&dyn crate::LegendComponent>,
    style: &ResolvedPlotStyle<'_>,
) -> Vec<LegendEntry> {
    let mut entries = Vec::new();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((id_salt, "legend"))
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    egui::Frame::popup(ui.style())
        .fill(style.background)
        .stroke(style.border_stroke)
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(&mut child, |ui| {
            ui.set_max_width((rect.width() - 16.0).max(1.0));
            egui::ScrollArea::vertical()
                .max_height((rect.height() - 16.0).max(1.0))
                .show(ui, |ui| {
                    for (index, s) in series
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| !s.name().is_empty())
                    {
                        let id = s.id(index);
                        let visible = state.series_visible(id);
                        let swatch = s.resolved_style(index, style).stroke.color;
                        let response = ui
                            .push_id(id, |ui| {
                                if let Some(c) = component {
                                    c.item(ui, s.name(), swatch, visible)
                                } else {
                                    legend_item_button(ui, s.name(), swatch, visible, style)
                                }
                            })
                            .inner;
                        if response.clicked() && state.toggle_series(id) {
                            ui.ctx().request_repaint();
                        }
                        entries.push(LegendEntry {
                            id,
                            name: s.name().to_owned(),
                            rect: response.rect.intersect(rect),
                            visible: state.series_visible(id),
                        });
                    }
                });
        });
    entries
}
