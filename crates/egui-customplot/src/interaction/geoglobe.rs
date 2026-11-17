use egui::{Id, Pos2, Response, Ui, ViewportId};

use crate::{GeoGlobeData, GeoGlobeTransform};

#[derive(Clone, Debug)]
struct GlobeFrameGuard {
    context: egui::Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    navigated: bool,
}

/// Application-owned orbit/zoom view state for [`crate::GeoGlobePlot`] (`EXT-25`).
#[derive(Clone, Debug)]
pub struct GeoGlobeState {
    center_lat_deg: f64,
    center_lon_deg: f64,
    zoom: f64,
    frame: Option<GlobeFrameGuard>,
}

impl Default for GeoGlobeState {
    fn default() -> Self {
        Self {
            center_lat_deg: 25.0,
            center_lon_deg: 105.0,
            zoom: 1.0,
            frame: None,
        }
    }
}

impl GeoGlobeState {
    pub fn center_lat_lon(&self) -> (f64, f64) {
        (self.center_lat_deg, self.center_lon_deg)
    }
    pub fn zoom(&self) -> f64 {
        self.zoom
    }
    pub fn set_view(&mut self, lat_deg: f64, lon_deg: f64, zoom: f64) {
        if lat_deg.is_finite() {
            self.center_lat_deg = lat_deg.clamp(-85.0, 85.0);
        }
        if lon_deg.is_finite() {
            self.center_lon_deg = crate::wrap_lon_180(lon_deg);
        }
        if zoom.is_finite() && zoom > 0.0 {
            self.zoom = zoom.clamp(0.4, 8.0);
        }
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(GlobeFrameGuard {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                navigated: false,
            });
        }
    }

    pub(crate) fn navigate(
        &mut self,
        ui: &Ui,
        response: &Response,
        transform: GeoGlobeTransform,
        allow_scroll_zoom: bool,
    ) -> bool {
        let Some(guard) = self.frame.as_mut() else {
            return false;
        };
        if guard.navigated {
            return false;
        }
        if response.double_clicked()
            || (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Home)))
        {
            self.center_lat_deg = 25.0;
            self.center_lon_deg = 105.0;
            self.zoom = 1.0;
            guard.navigated = true;
            return true;
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            let d = response.drag_delta();
            if d.length_sq() > 0.0 {
                let (lat, lon) = transform.orbit(d);
                self.center_lat_deg = lat;
                self.center_lon_deg = lon;
                guard.navigated = true;
                return true;
            }
        }
        if allow_scroll_zoom && response.hovered() {
            let scroll = ui.input_mut(|i| {
                if i.modifiers != egui::Modifiers::NONE {
                    return 0.0;
                }
                let d = i.smooth_scroll_delta.y;
                i.smooth_scroll_delta.y = 0.0;
                d
            });
            if scroll != 0.0 {
                let factor = (f64::from(scroll) * 0.0025).exp().clamp(0.5, 2.0);
                self.zoom = (self.zoom * factor).clamp(0.4, 8.0);
                guard.navigated = true;
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct HoveredGlobeItem {
    pub label: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: f64,
    pub screen_pos: Pos2,
    pub screen_distance: f32,
}

impl HoveredGlobeItem {
    pub fn default_text(&self) -> String {
        format!(
            "{}\nlat={:.2}° lon={:.2}° alt={:.0} m",
            self.label, self.lat_deg, self.lon_deg, self.alt_m
        )
    }
}

pub(crate) fn pick_globe_hover(
    data: &GeoGlobeData,
    transform: GeoGlobeTransform,
    pointer: Pos2,
    pick_radius: f32,
) -> Option<HoveredGlobeItem> {
    let mut best: Option<HoveredGlobeItem> = None;
    for m in data.markers() {
        let Some((pos, _)) = transform.project(m.lat_deg, m.lon_deg, m.alt_m) else {
            continue;
        };
        let d = pos.distance(pointer);
        if d <= pick_radius.max(m.radius_px + 3.0)
            && best.as_ref().is_none_or(|b| d < b.screen_distance)
        {
            best = Some(HoveredGlobeItem {
                label: m.label.clone(),
                lat_deg: m.lat_deg,
                lon_deg: m.lon_deg,
                alt_m: m.alt_m,
                screen_pos: pos,
                screen_distance: d,
            });
        }
    }
    if best.is_some() {
        return best;
    }
    for trk in data.tracks() {
        for i in 0..trk.lat_deg.len() {
            let Some((pos, _)) = transform.project(trk.lat_deg[i], trk.lon_deg[i], trk.alt_m[i])
            else {
                continue;
            };
            let d = pos.distance(pointer);
            if d <= pick_radius && best.as_ref().is_none_or(|b| d < b.screen_distance) {
                best = Some(HoveredGlobeItem {
                    label: trk.name.clone(),
                    lat_deg: trk.lat_deg[i],
                    lon_deg: trk.lon_deg[i],
                    alt_m: trk.alt_m[i],
                    screen_pos: pos,
                    screen_distance: d,
                });
            }
        }
    }
    best
}
