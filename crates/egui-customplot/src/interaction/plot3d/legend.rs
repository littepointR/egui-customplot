use egui::{Id, Rect, Ui};

use super::Plot3DState;
use crate::{Layer3D, LegendEntry, model::ResolvedPlotStyle};

pub(crate) fn show_plot3d_legend(
    ui: &mut Ui,
    state: &mut Plot3DState,
    layers: &[Layer3D<'_>],
    rect: Rect,
    legend_id: Id,
    component: Option<&dyn crate::LegendComponent>,
    style: &ResolvedPlotStyle<'_>,
) -> Vec<LegendEntry> {
    let mut entries = Vec::new();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(legend_id)
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
                    for (idx, layer) in layers
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| !l.name().is_empty())
                    {
                        let id = layer.id(idx);
                        let visible = state.layer_visible(id);
                        let swatch = layer.resolved_style(idx, style).stroke.color;
                        let response = ui
                            .push_id(id, |ui| {
                                if let Some(c) = component {
                                    return c.item(ui, layer.name(), swatch, visible);
                                }
                                crate::layout::legend_item_button(
                                    ui,
                                    layer.name(),
                                    swatch,
                                    visible,
                                    style,
                                )
                            })
                            .inner;
                        if response.clicked() && state.toggle_layer(id) {
                            ui.ctx().request_repaint();
                        }
                        entries.push(LegendEntry {
                            id,
                            name: layer.name().to_owned(),
                            rect: response.rect.intersect(rect),
                            visible: state.layer_visible(id),
                        });
                    }
                });
        });
    entries
}
