use std::{collections::HashSet, f64::consts::TAU};

use egui::{Id, IdSalt, Key, Modifiers, PointerButton, Pos2, Rect, Response, Ui, ViewportId};

use crate::{BoundsError, PolarTransform};

#[derive(Clone, Debug)]
struct PolarFrameSnapshot {
    context: egui::Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    base_r_lim: Option<[f64; 2]>,
    base_theta_offset: f64,
    drag_delta_x: Option<f32>,
    scroll_y: Option<f32>,
    reset: bool,
    legend_toggled: HashSet<IdSalt>,
}

/// Application-owned interaction state for [`crate::PolarPlot`] (`theta_offset`, radial limits, visibility).
#[derive(Clone, Debug, Default)]
pub struct PolarState {
    r_lim: Option<[f64; 2]>,
    theta_offset: f64,
    hidden: HashSet<IdSalt>,
    frame: Option<PolarFrameSnapshot>,
}

impl PolarState {
    pub fn r_lim(&self) -> Option<[f64; 2]> {
        self.r_lim
    }
    pub fn set_r_lim(&mut self, r_lim: [f64; 2]) -> Result<(), BoundsError> {
        if !r_lim[0].is_finite() || !r_lim[1].is_finite() || r_lim[1] <= r_lim[0] {
            return Err(BoundsError);
        }
        self.r_lim = Some(r_lim);
        self.frame = None;
        Ok(())
    }
    pub fn theta_offset(&self) -> f64 {
        self.theta_offset
    }
    pub fn set_theta_offset(&mut self, offset_rad: f64) {
        if offset_rad.is_finite() {
            self.theta_offset = offset_rad.rem_euclid(TAU);
            self.frame = None;
        }
    }
    pub fn reset(&mut self) {
        self.r_lim = None;
        self.theta_offset = 0.0;
        self.frame = None;
    }
    pub fn layer_visible(&self, id: IdSalt) -> bool {
        !self.hidden.contains(&id)
    }
    pub fn set_layer_visible(&mut self, id: IdSalt, visible: bool) {
        if visible {
            self.hidden.remove(&id);
        } else {
            self.hidden.insert(id);
        }
    }
    pub(crate) fn retain_layers(&mut self, present: &HashSet<IdSalt>) {
        self.hidden.retain(|id| present.contains(id));
    }

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(PolarFrameSnapshot {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                base_r_lim: self.r_lim,
                base_theta_offset: self.theta_offset,
                drag_delta_x: None,
                scroll_y: None,
                reset: false,
                legend_toggled: HashSet::new(),
            });
        }
    }

    pub(crate) fn toggle_layer(&mut self, id: IdSalt) -> bool {
        if !self
            .frame
            .as_mut()
            .expect("initialized polar frame")
            .legend_toggled
            .insert(id)
        {
            return false;
        }
        self.set_layer_visible(id, !self.layer_visible(id));
        true
    }
}

pub(crate) struct PolarNavigation {
    pub r_lim: [f64; 2],
    pub theta_offset: f64,
    pub changed: bool,
    pub hover_pos: Option<Pos2>,
}

/// Multi-pass idempotent polar navigation (horizontal drag rotates theta, wheel zooms r, double-click / Home resets).
pub(crate) fn navigate_polar(
    ui: &Ui,
    state: &mut PolarState,
    response: &mut Response,
    auto_r_lim: [f64; 2],
    lock_r_zero: bool,
    legend_rect: Option<Rect>,
    pixel_radius: f32,
) -> PolarNavigation {
    let hover_pos = response
        .hover_pos()
        .filter(|p| !legend_rect.is_some_and(|r| r.contains(*p)));
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    let frame = state.frame.as_mut().expect("initialized polar frame");
    if ui.is_enabled() {
        frame.reset |= (hover_pos.is_some() && response.double_clicked_by(PointerButton::Primary))
            || (response.has_focus()
                && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Home)));
        if response.dragged_by(PointerButton::Primary) && frame.drag_delta_x.is_none() {
            let dx = response.drag_delta().x;
            let from_legend = ui
                .input(|i| i.pointer.press_origin())
                .is_some_and(|p| legend_rect.is_some_and(|r| r.contains(p)));
            if dx.abs() > 0.0 && !from_legend {
                frame.drag_delta_x = Some(dx);
            }
        }
        if hover_pos.is_some() && frame.scroll_y.is_none() {
            let scroll = ui.input_mut(|i| {
                let d = i.smooth_scroll_delta.y;
                i.smooth_scroll_delta.y = 0.0;
                d
            });
            if scroll != 0.0 {
                frame.scroll_y = Some(scroll);
            }
        }
    }
    let mut r_lim = frame.base_r_lim.unwrap_or(auto_r_lim);
    let mut theta_offset = frame.base_theta_offset;
    let mut changed = false;
    if frame.reset {
        state.r_lim = None;
        state.theta_offset = 0.0;
        r_lim = auto_r_lim;
        theta_offset = 0.0;
        changed = frame.base_r_lim.is_some() || frame.base_theta_offset.abs() > 1e-12;
    } else {
        if let Some(dx) = frame.drag_delta_x {
            let delta = f64::from(dx) / f64::from(pixel_radius.max(24.0)) * 0.75;
            theta_offset = (theta_offset + delta).rem_euclid(TAU);
            state.theta_offset = theta_offset;
            changed = true;
        }
        if let Some(scroll) = frame.scroll_y {
            let factor = (f64::from(scroll) / 200.0).exp().clamp(0.5, 2.0);
            let dummy = PolarTransform::new(
                Pos2::ZERO,
                pixel_radius.max(1.0),
                r_lim,
                crate::ThetaZeroLocation::Right,
                crate::ThetaDirection::CounterClockwise,
                [0.0, TAU],
                0.0,
            )
            .expect("valid radial bounds");
            r_lim = dummy.zoom_radial(factor, lock_r_zero);
            state.r_lim = Some(r_lim);
            changed = true;
        }
    }
    PolarNavigation {
        r_lim,
        theta_offset,
        changed,
        hover_pos,
    }
}
