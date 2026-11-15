use egui::{IdSalt, Rect, Ui};

use crate::{PlotState, Series};

#[derive(Clone, Debug)]
pub struct LegendEntry {
    pub id: IdSalt,
    pub name: String,
    pub rect: Rect,
    pub visible: bool,
}

pub(crate) fn legend_rect(rect: Rect, series: &[Series<'_>]) -> Option<Rect> {
    let count = series.iter().filter(|s| !s.name().is_empty()).count();
    if count == 0 || rect.width() < 100.0 || rect.height() < 50.0 {
        return None;
    }
    let size = egui::vec2(
        (rect.width() * 0.35).clamp(80.0, 220.0),
        (count as f32 * 24.0 + 16.0).min(rect.height() - 12.0),
    );
    Some(Rect::from_min_size(
        egui::pos2(rect.right() - size.x - 6.0, rect.top() + 6.0),
        size,
    ))
}

/// Submit legend last for ordered painting; visibility takes effect on the next
/// layout pass/repaint. Only state::toggle_series owns once-per-frame effects.
pub(crate) fn show_legend(
    ui: &mut Ui,
    state: &mut PlotState,
    series: &[Series<'_>],
    rect: Rect,
    id_salt: IdSalt,
) -> Vec<LegendEntry> {
    let mut entries = Vec::new();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((id_salt, "legend"))
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    egui::Frame::popup(ui.style()).show(&mut child, |ui| {
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
                    let response = ui
                        .push_id(id, |ui| {
                            ui.selectable_label(
                                visible,
                                egui::RichText::new(s.name()).color(if visible {
                                    s.color(index)
                                } else {
                                    ui.visuals().weak_text_color()
                                }),
                            )
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
