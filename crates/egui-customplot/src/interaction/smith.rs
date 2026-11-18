use super::view_gesture::ViewGesture;
use std::collections::HashSet;

use egui::{Id, Pos2, Rect, Response, Ui, ViewportId};

use crate::{LegendEntry, SmithPlotData, SmithTransform, model::ResolvedPlotStyle};

#[derive(Clone, Debug)]
struct SmithFrameGuard {
    context: egui::Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    gesture: Option<ViewGesture>,
    initial_center: [f64; 2],
    initial_span: f64,
    legend_toggled: HashSet<usize>,
}

/// Application-owned zoom/pan & trace-visibility state for [`crate::SmithPlot`] (`EXT-19`).
#[derive(Clone, Debug)]
pub struct SmithPlotState {
    center_gamma: [f64; 2],
    half_span: f64,
    hidden_traces: HashSet<usize>,
    frame: Option<SmithFrameGuard>,
}

impl Default for SmithPlotState {
    fn default() -> Self {
        Self {
            center_gamma: [0.0, 0.0],
            half_span: 1.08,
            hidden_traces: HashSet::new(),
            frame: None,
        }
    }
}

impl SmithPlotState {
    pub fn center_gamma(&self) -> [f64; 2] {
        self.center_gamma
    }
    pub fn half_span(&self) -> f64 {
        self.half_span
    }
    pub fn trace_visible(&self, index: usize) -> bool {
        !self.hidden_traces.contains(&index)
    }
    pub fn set_trace_visible(&mut self, index: usize, visible: bool) {
        if visible {
            self.hidden_traces.remove(&index);
        } else {
            self.hidden_traces.insert(index);
        }
    }
    pub fn reset(&mut self) {
        self.center_gamma = [0.0, 0.0];
        self.half_span = 1.08;
        self.hidden_traces.clear();
        self.frame = None;
    }

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(SmithFrameGuard {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                gesture: None,
                initial_center: self.center_gamma,
                initial_span: self.half_span,
                legend_toggled: HashSet::new(),
            });
        }
    }

    pub(crate) fn navigate(
        &mut self,
        ui: &Ui,
        response: &Response,
        plot_area: egui::Rect,
        allow_scroll: bool,
    ) -> bool {
        let Some(guard) = self.frame.as_mut() else {
            return false;
        };
        let Some(gesture) = guard
            .gesture
            .or_else(|| ViewGesture::read(ui, response, allow_scroll))
        else {
            return false;
        };
        guard.gesture = Some(gesture);
        let transform = SmithTransform::new(plot_area, guard.initial_center, guard.initial_span);
        (self.center_gamma, self.half_span) = match gesture {
            ViewGesture::Reset => ([0.0, 0.0], 1.08),
            ViewGesture::Drag(delta) => transform.pan(delta),
            ViewGesture::Scroll { anchor, delta } => {
                transform.zoom_at(anchor, (-f64::from(delta) * 0.0025).exp().clamp(0.5, 2.0))
            }
        };
        true
    }

    pub(crate) fn toggle_trace(&mut self, idx: usize) -> bool {
        let Some(guard) = self.frame.as_mut() else {
            return false;
        };
        if !guard.legend_toggled.insert(idx) {
            return false;
        }
        if !self.hidden_traces.remove(&idx) {
            self.hidden_traces.insert(idx);
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct HoveredSmithPoint {
    pub trace_index: usize,
    pub point_index: usize,
    pub trace_name: String,
    pub freq_hz: f64,
    pub gamma: [f64; 2],
    pub z_norm: Option<[f64; 2]>,
    pub z_ohms: Option<[f64; 2]>,
    pub vswr: f64,
    pub return_loss_db: f64,
    pub screen_pos: Pos2,
    pub screen_distance: f32,
}

impl HoveredSmithPoint {
    pub fn default_text(&self) -> String {
        let z_txt = match self.z_ohms {
            Some([r, x]) => format!("Z = {r:.2} {:+.2}j Ohm", x),
            None => String::from("Z = open/inf"),
        };
        format!(
            "{} @ {:.3e} Hz\nGamma = ({:+.4}, {:+.4}j)\n{z_txt}\nVSWR = {:.2}  RL = {:.2} dB",
            self.trace_name,
            self.freq_hz,
            self.gamma[0],
            self.gamma[1],
            self.vswr,
            self.return_loss_db,
        )
    }
}

pub(crate) fn pick_smith_hover(
    state: &SmithPlotState,
    data: &SmithPlotData,
    transform: SmithTransform,
    pointer: Pos2,
    pick_radius: f32,
) -> Option<HoveredSmithPoint> {
    let mut best: Option<HoveredSmithPoint> = None;
    for (ti, trace) in data.traces().iter().enumerate() {
        if !state.trace_visible(ti) {
            continue;
        }
        for (pi, &gamma) in trace.gamma.iter().enumerate() {
            let pos = transform.gamma_to_pos(gamma);
            if !pos.is_finite() {
                continue;
            }
            let dist = pos.distance(pointer);
            if dist <= pick_radius && best.as_ref().is_none_or(|b| dist < b.screen_distance) {
                let z_norm = SmithTransform::gamma_to_z(gamma);
                let z_ohms = z_norm.map(|[r, x]| [r * trace.z0_ohms, x * trace.z0_ohms]);
                best = Some(HoveredSmithPoint {
                    trace_index: ti,
                    point_index: pi,
                    trace_name: trace.name.clone(),
                    freq_hz: trace.freq_hz[pi],
                    gamma,
                    z_norm,
                    z_ohms,
                    vswr: SmithTransform::vswr_from_gamma(gamma),
                    return_loss_db: SmithTransform::return_loss_db(gamma),
                    screen_pos: pos,
                    screen_distance: dist,
                });
            }
        }
    }
    best
}

pub(crate) fn show_smith_legend(
    ui: &mut Ui,
    state: &mut SmithPlotState,
    data: &SmithPlotData,
    rect: Rect,
    id: Id,
    style: &ResolvedPlotStyle<'_>,
) -> Vec<LegendEntry> {
    let mut entries = Vec::with_capacity(data.traces().len());
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    egui::Frame::popup(ui.style())
        .fill(style.background)
        .stroke(style.border_stroke)
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(&mut child, |ui| {
            for (ti, trace) in data.traces().iter().enumerate() {
                let resolved = style.resolve_series(ti, Default::default());
                let vis = state.trace_visible(ti);
                let resp = ui
                    .push_id(ti, |ui| {
                        crate::layout::legend_item_button(
                            ui,
                            &trace.name,
                            resolved.stroke.color,
                            vis,
                            style,
                        )
                    })
                    .inner;
                if resp.clicked() && state.toggle_trace(ti) {
                    ui.ctx().request_repaint();
                }
                entries.push(LegendEntry {
                    id: egui::IdSalt::new(("smith", ti)),
                    name: trace.name.clone(),
                    rect: resp.rect.intersect(rect),
                    visible: state.trace_visible(ti),
                });
            }
        });
    entries
}
