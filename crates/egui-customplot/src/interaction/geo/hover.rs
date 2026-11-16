use std::collections::HashSet;

use egui::{Color32, Id, IdSalt, Pos2, Rect, Ui};

use super::GeoPlotState;
use crate::{
    GeoLayer, LegendEntry,
    axis::{GeoPoint, GeoTransform},
    model::ResolvedPlotStyle,
};

/// Hovered raw point, bubble, or interpolated density readout from [`crate::GeoPlot`].
#[derive(Clone, Debug, PartialEq)]
pub struct HoveredGeoPoint {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub point_index: usize,
    pub point: GeoPoint,
    pub value: Option<f64>,
    pub category: Option<String>,
    pub label: Option<String>,
    pub screen_pos: Pos2,
    pub screen_distance: f32,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn pick_geo_hover(
    state: &GeoPlotState,
    layers: &[GeoLayer<'_>],
    transform: GeoTransform,
    pointer: Pos2,
    pick_radius: f32,
) -> (Option<HoveredGeoPoint>, Option<String>) {
    let mut best: Option<HoveredGeoPoint> = None;
    for (idx, layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        if !state.layer_visible(id) {
            continue;
        }
        match *layer {
            GeoLayer::Bubbles(b) => {
                let cats = b.data.categories();
                for (pi, pt) in b.data.points().iter().enumerate() {
                    if !pt.point.is_valid_domain() || !pt.size.is_finite() || pt.size < 0.0 {
                        continue;
                    }
                    let cat = pt.category.and_then(|c| cats.get(c).cloned());
                    if let Some(ref c) = cat
                        && !state.category_visible(c)
                    {
                        continue;
                    }
                    let hit_r = pick_radius.max(b.data.pixel_radius(pt.size));
                    consider_geo(
                        &mut best,
                        id,
                        idx,
                        pi,
                        pt.point,
                        Some(pt.size),
                        cat,
                        pt.label.clone(),
                        transform,
                        pointer,
                        hit_r,
                    );
                }
            }
            GeoLayer::Scatter(s) => {
                for (pi, pt) in s.data.points().iter().enumerate() {
                    let hit_r = pick_radius.max(pt.size.unwrap_or(4.5));
                    consider_geo(
                        &mut best,
                        id,
                        idx,
                        pi,
                        pt.point,
                        pt.color_value,
                        None,
                        None,
                        transform,
                        pointer,
                        hit_r,
                    );
                }
            }
            GeoLayer::Line(l) => {
                for (pi, &pt) in l.data.points().iter().enumerate() {
                    consider_geo(
                        &mut best,
                        id,
                        idx,
                        pi,
                        pt,
                        None,
                        None,
                        None,
                        transform,
                        pointer,
                        pick_radius,
                    );
                }
            }
            GeoLayer::Density(_) => {}
        }
    }

    if let Some(hit) = best {
        let mut msg = format!(
            "{} #{}\nlat={:.3}°  lon={:.3}°",
            layers[hit.layer_index].name(),
            hit.point_index,
            hit.point.lat,
            hit.point.lon
        );
        if let Some(ref lbl) = hit.label {
            msg.push_str(&format!("\nlabel={lbl}"));
        }
        if let Some(ref cat) = hit.category {
            msg.push_str(&format!("\ncategory={cat}"));
        }
        if let Some(v) = hit.value {
            msg.push_str(&format!("\nvalue={v:.3}"));
        }
        return (Some(hit), Some(msg));
    }

    let vp = transform.viewport();
    if vp.contains(pointer) {
        let cursor_geo = transform.unproject(pointer);
        if let Some(pt) = cursor_geo {
            for (idx, layer) in layers.iter().enumerate() {
                if let GeoLayer::Density(d) = *layer
                    && state.layer_visible(layer.id(idx))
                {
                    let grid = d.data.evaluate_viewport(transform);
                    let u = ((pointer.x - vp.left()) / vp.width().max(1.0)).clamp(0.0, 0.9999);
                    let v = ((pointer.y - vp.top()) / vp.height().max(1.0)).clamp(0.0, 0.9999);
                    let col = (u * grid.cols as f32) as usize;
                    let row = (v * grid.rows as f32) as usize;
                    let val = grid.values[row * grid.cols + col];
                    if val > 0.0 {
                        let hit = HoveredGeoPoint {
                            layer_id: layer.id(idx),
                            layer_index: idx,
                            point_index: row * grid.cols + col,
                            point: pt,
                            value: Some(val),
                            category: None,
                            label: None,
                            screen_pos: pointer,
                            screen_distance: 0.0,
                        };
                        let text = format!(
                            "{}\nlat={:.3}°  lon={:.3}°\ndensity={val:.3}",
                            d.name, pt.lat, pt.lon
                        );
                        return (Some(hit), Some(text));
                    }
                }
            }
            return (
                None,
                Some(format!("lat={:.3}°  lon={:.3}°", pt.lat, pt.lon)),
            );
        }
    }
    (None, None)
}

#[allow(clippy::too_many_arguments)]
fn consider_geo(
    best: &mut Option<HoveredGeoPoint>,
    layer_id: IdSalt,
    layer_index: usize,
    point_index: usize,
    point: GeoPoint,
    value: Option<f64>,
    category: Option<String>,
    label: Option<String>,
    transform: GeoTransform,
    pointer: Pos2,
    radius: f32,
) {
    let Some(pos) = transform.project(point) else {
        return;
    };
    let dist = pos.distance(pointer);
    if dist <= radius && best.as_ref().is_none_or(|cur| dist < cur.screen_distance) {
        *best = Some(HoveredGeoPoint {
            layer_id,
            layer_index,
            point_index,
            point,
            value,
            category,
            label,
            screen_pos: pos,
            screen_distance: dist,
        });
    }
}

pub(crate) fn show_geo_legend(
    ui: &mut Ui,
    state: &mut GeoPlotState,
    layers: &[GeoLayer<'_>],
    rect: Rect,
    legend_id: Id,
    style: &ResolvedPlotStyle<'_>,
) -> Vec<LegendEntry> {
    let mut entries = Vec::new();
    let mut cats = Vec::new();
    let mut seen_cats = HashSet::new();
    for (idx, layer) in layers.iter().enumerate() {
        if let GeoLayer::Bubbles(b) = *layer
            && state.layer_visible(layer.id(idx))
        {
            for c in b.data.categories() {
                if seen_cats.insert(c.clone()) {
                    cats.push(c.clone());
                }
            }
        }
    }
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
                        let response = ui
                            .push_id(id, |ui| {
                                ui.selectable_label(
                                    visible,
                                    egui::RichText::new(layer.name()).color(if visible {
                                        layer.resolved_style(idx, style).stroke.color
                                    } else {
                                        style.text_color.gamma_multiply(0.45)
                                    }),
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
                    for (ci, cat) in cats.iter().enumerate() {
                        let visible = state.category_visible(cat);
                        let color = style.palette[ci % style.palette.len()];
                        let response = ui
                            .push_id(("geo-cat", ci, cat.as_str()), |ui| {
                                ui.selectable_label(
                                    visible,
                                    egui::RichText::new(cat).color(if visible {
                                        color
                                    } else {
                                        Color32::GRAY
                                    }),
                                )
                            })
                            .inner;
                        if response.clicked() && state.toggle_category(cat) {
                            ui.ctx().request_repaint();
                        }
                        entries.push(LegendEntry {
                            id: IdSalt::new(("geo-cat", cat.as_str())),
                            name: cat.clone(),
                            rect: response.rect.intersect(rect),
                            visible: state.category_visible(cat),
                        });
                    }
                });
        });
    entries
}
