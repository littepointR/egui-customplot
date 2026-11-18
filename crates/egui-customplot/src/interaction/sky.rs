use std::collections::HashSet;

use egui::{Id, IdSalt, Pos2, Rect, Response, Ui, ViewportId};

use crate::{LegendEntry, SkyPlotData, SkyTransform, model::ResolvedPlotStyle};

#[derive(Clone, Debug)]
struct SkyFrameGuard {
    context: egui::Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    scroll_y: Option<f32>,
    reset: bool,
    legend_toggled: HashSet<usize>,
}

/// Application-owned interaction state for [`crate::SkyPlot`] (group visibility & zenith zoom with multi-pass guard).
#[derive(Clone, Debug, Default)]
pub struct SkyPlotState {
    min_el_override: Option<f64>,
    hidden_groups: HashSet<usize>,
    frame: Option<SkyFrameGuard>,
}

impl SkyPlotState {
    pub fn min_elevation(&self) -> Option<f64> {
        self.min_el_override
    }

    pub fn group_visible(&self, group: usize) -> bool {
        !self.hidden_groups.contains(&group)
    }

    pub fn set_group_visible(&mut self, group: usize, visible: bool) {
        if visible {
            self.hidden_groups.remove(&group);
        } else {
            self.hidden_groups.insert(group);
        }
    }

    pub fn reset(&mut self) {
        self.min_el_override = None;
        self.hidden_groups.clear();
        self.frame = None;
    }

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(SkyFrameGuard {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                scroll_y: None,
                reset: false,
                legend_toggled: HashSet::new(),
            });
        }
    }

    pub(crate) fn toggle_group(&mut self, group: usize) -> bool {
        if !self
            .frame
            .as_mut()
            .expect("initialized skyplot frame")
            .legend_toggled
            .insert(group)
        {
            return false;
        }
        self.set_group_visible(group, !self.group_visible(group));
        true
    }
}

pub(crate) fn navigate_sky(
    ui: &Ui,
    state: &mut SkyPlotState,
    response: &mut Response,
    default_min_el: f64,
    allow_scroll_zoom: bool,
    legend_rect: Option<Rect>,
) -> (f64, bool) {
    let hover_pos = response
        .hover_pos()
        .filter(|p| !legend_rect.is_some_and(|r| r.contains(*p)));
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    let Some(guard) = state.frame.as_mut() else {
        return (state.min_el_override.unwrap_or(default_min_el), false);
    };
    if ui.is_enabled() {
        guard.reset |= (hover_pos.is_some()
            && response.double_clicked_by(egui::PointerButton::Primary))
            || (response.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Home)));
        if allow_scroll_zoom && hover_pos.is_some() && guard.scroll_y.is_none() {
            let scroll = ui.input_mut(|i| {
                if i.modifiers != egui::Modifiers::NONE {
                    return 0.0;
                }
                let d = i.smooth_scroll_delta.y;
                i.smooth_scroll_delta.y = 0.0;
                d
            });
            if scroll != 0.0 {
                guard.scroll_y = Some(scroll);
            }
        }
    }

    if guard.reset {
        let changed = state.min_el_override.is_some();
        state.min_el_override = None;
        return (default_min_el, changed);
    }
    if let Some(scroll) = guard.scroll_y.take() {
        let cur = state.min_el_override.unwrap_or(default_min_el);
        let factor = (-f64::from(scroll) * 0.0025).exp().clamp(0.5, 2.0);
        let span = ((90.0 - cur) * factor).clamp(5.0, 90.0);
        let next = (90.0 - span).clamp(0.0, 85.0);
        let changed = (next - cur).abs() > 1e-12;
        state.min_el_override = Some(next);
        return (next, changed);
    }
    (state.min_el_override.unwrap_or(default_min_el), false)
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoveredSkyItem {
    pub satellite_index: Option<usize>,
    pub track_index: Option<(usize, usize)>,
    pub label: String,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub scalar: Option<f64>,
    pub masked: bool,
    pub screen_pos: Pos2,
    pub screen_distance: f32,
}

pub(crate) fn show_sky_legend(
    ui: &mut Ui,
    state: &mut SkyPlotState,
    data: &SkyPlotData,
    rect: Rect,
    legend_id: Id,
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
            for &g in data.legend_groups() {
                let name = data
                    .group_names()
                    .get(g)
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .unwrap_or_else(|| format!("Group {}", g as u128 + 1));
                let visible = state.group_visible(g);
                let c = style.resolve_series(g, Default::default()).stroke.color;
                let resp = ui
                    .push_id(g, |ui| {
                        crate::layout::legend_item_button(ui, &name, c, visible, style)
                    })
                    .inner;
                if resp.clicked() && state.toggle_group(g) {
                    ui.ctx().request_repaint();
                }
                entries.push(LegendEntry {
                    id: IdSalt::new(("sky_group", g)),
                    name,
                    rect: resp.rect.intersect(rect),
                    visible: state.group_visible(g),
                });
            }
        });
    entries
}

pub(crate) fn pick_sky_hover(
    state: &SkyPlotState,
    data: &SkyPlotData,
    transform: SkyTransform,
    mask_elevation_deg: f64,
    pointer: Pos2,
    pick_radius: f32,
) -> Option<HoveredSkyItem> {
    let mut best: Option<HoveredSkyItem> = None;
    for (si, sat) in data.satellites().iter().enumerate() {
        if !state.group_visible(sat.group) {
            continue;
        }
        let Some(pos) = transform.project(sat.azimuth_deg, sat.elevation_deg) else {
            continue;
        };
        let dist = pos.distance(pointer);
        if dist <= pick_radius && best.as_ref().is_none_or(|b| dist < b.screen_distance) {
            best = Some(HoveredSkyItem {
                satellite_index: Some(si),
                track_index: None,
                label: sat.label.clone(),
                azimuth_deg: sat.azimuth_deg,
                elevation_deg: sat.elevation_deg,
                scalar: None,
                masked: sat.elevation_deg < mask_elevation_deg,
                screen_pos: pos,
                screen_distance: dist,
            });
        }
    }
    if best.is_some() {
        return best;
    }
    for (ti, trk) in data.tracks().iter().enumerate() {
        if !state.group_visible(trk.group) {
            continue;
        }
        for pi in 0..trk.azimuth_deg.len() {
            let az = trk.azimuth_deg[pi];
            let el = trk.elevation_deg[pi];
            let Some(pos) = transform.project(az, el) else {
                continue;
            };
            let dist = pos.distance(pointer);
            if dist <= pick_radius && best.as_ref().is_none_or(|b| dist < b.screen_distance) {
                best = Some(HoveredSkyItem {
                    satellite_index: None,
                    track_index: Some((ti, pi)),
                    label: trk.label.clone(),
                    azimuth_deg: az,
                    elevation_deg: el,
                    scalar: trk.scalars.as_ref().and_then(|s| s.get(pi).copied()),
                    masked: el < mask_elevation_deg,
                    screen_pos: pos,
                    screen_distance: dist,
                });
            }
        }
    }
    best
}
