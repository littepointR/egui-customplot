use std::f64::consts::TAU;

use egui::{Id, IdSalt, Pos2, Rect, Ui};

use crate::{
    LegendEntry, NegativeRadiusMode, PolarLayer, PolarState, PolarTransform,
    model::ResolvedPlotStyle,
};

#[derive(Clone, Debug, PartialEq)]
pub struct HoveredPolarPoint {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub point_index: usize,
    pub theta_rad: f64,
    pub r: f64,
    pub size: Option<f64>,
    pub screen_distance: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoveredPolarBin {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub bin_index: usize,
    pub theta_range: [f64; 2],
    pub value: f64,
    pub count: usize,
}

pub(crate) fn pick_polar_hover(
    state: &PolarState,
    layers: &[PolarLayer<'_>],
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    pointer: Pos2,
    pick_radius: f32,
) -> (
    Option<HoveredPolarPoint>,
    Option<HoveredPolarBin>,
    Option<String>,
) {
    let Some([cursor_t, cursor_r]) = transform.polar_from_position(pointer) else {
        return (None, None, None);
    };
    let mut best_pt: Option<HoveredPolarPoint> = None;
    for (idx, layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        if !state.layer_visible(id) {
            continue;
        }
        match *layer {
            PolarLayer::Scatter(s) => {
                for (pi, &[th, r]) in s.data.points().iter().enumerate() {
                    consider_polar(
                        &mut best_pt,
                        id,
                        idx,
                        pi,
                        th,
                        r,
                        None,
                        transform,
                        neg_mode,
                        pointer,
                        pick_radius,
                    );
                }
            }
            PolarLayer::Bubble(b) => {
                for (pi, pt) in b.data.points().iter().enumerate() {
                    let hit_r = pick_radius.max(b.data.pixel_radius(pt.size));
                    consider_polar(
                        &mut best_pt,
                        id,
                        idx,
                        pi,
                        pt.theta,
                        pt.r,
                        Some(pt.size),
                        transform,
                        neg_mode,
                        pointer,
                        hit_r,
                    );
                }
            }
            PolarLayer::Line(l) => {
                for (pi, &[th, r]) in l.data.points().iter().enumerate() {
                    consider_polar(
                        &mut best_pt,
                        id,
                        idx,
                        pi,
                        th,
                        r,
                        None,
                        transform,
                        neg_mode,
                        pointer,
                        pick_radius,
                    );
                }
            }
            PolarLayer::Compass(c) => {
                for (pi, &[th, rho]) in c.data.arrows().iter().enumerate() {
                    consider_polar(
                        &mut best_pt,
                        id,
                        idx,
                        pi,
                        th,
                        rho,
                        None,
                        transform,
                        neg_mode,
                        pointer,
                        pick_radius,
                    );
                }
            }
            PolarLayer::Histogram(_) => {}
        }
    }
    if let Some(hit) = best_pt {
        let name = layers[hit.layer_index].name();
        let size_s = hit
            .size
            .map_or(String::new(), |sz| format!("  size={sz:.3}"));
        let text = if name.is_empty() {
            format!(
                "[{}] theta={:.1}°  r={:.4}{size_s}",
                hit.point_index,
                hit.theta_rad.to_degrees(),
                hit.r
            )
        } else {
            format!(
                "{name}\n[{}] theta={:.1}°  r={:.4}{size_s}",
                hit.point_index,
                hit.theta_rad.to_degrees(),
                hit.r
            )
        };
        return (Some(hit), None, Some(text));
    }
    for (idx, layer) in layers.iter().enumerate().rev() {
        let id = layer.id(idx);
        if !state.layer_visible(id) {
            continue;
        }
        if let PolarLayer::Histogram(h) = *layer {
            let vals = h.data.values(h.normalization);
            let edges = h.data.edges();
            for (bi, &val) in vals.iter().enumerate() {
                if val <= 0.0 || cursor_r > val {
                    continue;
                }
                let e0 = edges[bi];
                let e1 = edges[bi + 1];
                let rel = (cursor_t - e0).rem_euclid(TAU);
                if rel <= (e1 - e0) + 1e-9 {
                    let bin = HoveredPolarBin {
                        layer_id: id,
                        layer_index: idx,
                        bin_index: bi,
                        theta_range: [e0, e1],
                        value: val,
                        count: h.data.counts()[bi],
                    };
                    let name = layer.name();
                    let text = if name.is_empty() {
                        format!(
                            "[{:.0}°..{:.0}°]  val={val:.4} (count={})",
                            e0.to_degrees(),
                            e1.to_degrees(),
                            bin.count
                        )
                    } else {
                        format!(
                            "{name}\n[{:.0}°..{:.0}°]  val={val:.4} (count={})",
                            e0.to_degrees(),
                            e1.to_degrees(),
                            bin.count
                        )
                    };
                    return (None, Some(bin), Some(text));
                }
            }
        }
    }
    (
        None,
        None,
        Some(format!(
            "theta={:.1}°  r={cursor_r:.4}",
            cursor_t.to_degrees()
        )),
    )
}

#[allow(clippy::too_many_arguments)]
fn consider_polar(
    best: &mut Option<HoveredPolarPoint>,
    layer_id: IdSalt,
    layer_index: usize,
    point_index: usize,
    theta: f64,
    r: f64,
    size: Option<f64>,
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    pointer: Pos2,
    radius: f32,
) {
    let Some(pos) = transform.position_from_polar(theta, r, neg_mode) else {
        return;
    };
    let dist = pos.distance(pointer);
    if dist <= radius && best.as_ref().is_none_or(|cur| dist < cur.screen_distance) {
        *best = Some(HoveredPolarPoint {
            layer_id,
            layer_index,
            point_index,
            theta_rad: theta,
            r,
            size,
            screen_distance: dist,
        });
    }
}

pub(crate) fn show_polar_legend(
    ui: &mut Ui,
    state: &mut PolarState,
    layers: &[PolarLayer<'_>],
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
