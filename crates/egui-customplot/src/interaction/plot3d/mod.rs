mod hover;
mod legend;
mod surface_hover;
mod volume_hover;

use std::collections::HashSet;

use egui::{Id, IdSalt, Key, Modifiers, PointerButton, Pos2, Rect, Response, Ui, ViewportId};

pub(crate) use hover::pick_plot3d_hover;
pub use hover::{Hovered3DBar, Hovered3DPoint, Hovered3DRegion};
pub(crate) use legend::show_plot3d_legend;

use crate::{Bounds3D, Camera3D};

#[derive(Clone, Debug)]
struct Plot3DFrameSnapshot {
    context: egui::Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    base_camera: Camera3D,
    base_bounds: Option<Bounds3D>,
    orbit_delta: Option<[f32; 2]>,
    pan_delta: Option<[f32; 2]>,
    scroll_y: Option<f32>,
    reset: bool,
    legend_toggled: HashSet<IdSalt>,
}

/// Application-owned interaction state for [`crate::Plot3D`] (`Camera3D`, manual bounds, visibility).
#[derive(Clone, Debug, Default)]
pub struct Plot3DState {
    camera_override: Option<Camera3D>,
    bounds_override: Option<Bounds3D>,
    hidden: HashSet<IdSalt>,
    frame: Option<Plot3DFrameSnapshot>,
}

impl Plot3DState {
    pub fn camera(&self) -> Option<Camera3D> {
        self.camera_override
    }
    pub fn set_camera(&mut self, camera: Camera3D) {
        self.camera_override = Some(Camera3D {
            azimuth_deg: camera.azimuth_deg.rem_euclid(360.0),
            elevation_deg: camera.elevation_deg.clamp(-90.0, 90.0),
            zoom: camera.zoom.clamp(0.1, 20.0),
            ..camera
        });
        self.frame = None;
    }
    pub fn bounds(&self) -> Option<Bounds3D> {
        self.bounds_override
    }
    pub fn set_bounds(&mut self, bounds: Bounds3D) {
        self.bounds_override = Some(bounds);
        self.frame = None;
    }
    pub fn reset(&mut self) {
        self.camera_override = None;
        self.bounds_override = None;
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

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id, default_camera: Camera3D) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(Plot3DFrameSnapshot {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                base_camera: self.camera_override.unwrap_or(default_camera),
                base_bounds: self.bounds_override,
                orbit_delta: None,
                pan_delta: None,
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
            .expect("initialized 3D frame")
            .legend_toggled
            .insert(id)
        {
            return false;
        }
        self.set_layer_visible(id, !self.layer_visible(id));
        true
    }
}

pub(crate) struct Plot3DNavigation {
    pub camera: Camera3D,
    pub bounds: Bounds3D,
    pub changed: bool,
    pub hover_pos: Option<Pos2>,
}

/// Multi-pass idempotent 3D orbit / pan / wheel zoom / Home-reset input arbitration.
pub(crate) fn navigate_plot3d(
    ui: &Ui,
    state: &mut Plot3DState,
    response: &mut Response,
    auto_bounds: Bounds3D,
    default_camera: Camera3D,
    allow_scroll_zoom: bool,
    legend_rect: Option<Rect>,
) -> Plot3DNavigation {
    let hover_pos = response
        .hover_pos()
        .filter(|p| !legend_rect.is_some_and(|r| r.contains(*p)));
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    let frame = state.frame.as_mut().expect("initialized 3D frame");
    if ui.is_enabled() {
        frame.reset |= (hover_pos.is_some() && response.double_clicked_by(PointerButton::Primary))
            || (response.has_focus()
                && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Home)));
        let from_legend = ui
            .input(|i| i.pointer.press_origin())
            .is_some_and(|p| legend_rect.is_some_and(|r| r.contains(p)));
        if !from_legend {
            if response.dragged_by(PointerButton::Primary) && frame.orbit_delta.is_none() {
                let d = response.drag_delta();
                if d.x.abs() > 0.0 || d.y.abs() > 0.0 {
                    frame.orbit_delta = Some([d.x, d.y]);
                }
            }
            if response.dragged_by(PointerButton::Secondary) && frame.pan_delta.is_none() {
                let d = response.drag_delta();
                if d.x.abs() > 0.0 || d.y.abs() > 0.0 {
                    frame.pan_delta = Some([d.x, d.y]);
                }
            }
        }
        if allow_scroll_zoom && hover_pos.is_some() && frame.scroll_y.is_none() {
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

    let mut camera = frame.base_camera;
    let mut bounds = frame.base_bounds.unwrap_or(auto_bounds);
    let mut changed = false;
    if frame.reset {
        state.camera_override = None;
        state.bounds_override = None;
        camera = default_camera;
        bounds = auto_bounds;
        changed = camera != frame.base_camera || frame.base_bounds.is_some();
    } else {
        if let Some([dx, dy]) = frame.orbit_delta {
            camera.azimuth_deg = (camera.azimuth_deg - f64::from(dx) * 0.45).rem_euclid(360.0);
            camera.elevation_deg = (camera.elevation_deg + f64::from(dy) * 0.45).clamp(-89.5, 89.5);
            state.camera_override = Some(camera);
            changed = true;
        }
        if let Some([dx, dy]) = frame.pan_delta {
            let scale = f64::from(response.rect.width().min(response.rect.height())).max(40.0)
                * 0.44
                * camera.zoom;
            camera.pan[0] = (camera.pan[0] + f64::from(dx) / scale).clamp(-2.5, 2.5);
            camera.pan[1] = (camera.pan[1] - f64::from(dy) / scale).clamp(-2.5, 2.5);
            state.camera_override = Some(camera);
            changed = true;
        }
        if let Some(scroll) = frame.scroll_y {
            let factor = (f64::from(scroll) / 220.0).exp().clamp(0.5, 2.0);
            camera.zoom = (camera.zoom * factor).clamp(0.15, 20.0);
            state.camera_override = Some(camera);
            changed = true;
        }
    }
    Plot3DNavigation {
        camera,
        bounds,
        changed,
        hover_pos,
    }
}
