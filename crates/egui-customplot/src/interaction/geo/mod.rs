mod hover;

use std::collections::HashSet;

use egui::{Id, IdSalt, Key, Modifiers, PointerButton, Pos2, Rect, Response, Ui, Vec2, ViewportId};

pub use hover::HoveredGeoPoint;
pub(crate) use hover::{pick_geo_hover, show_geo_legend};

use crate::axis::{GeoBounds, GeoError, GeoProjection, GeoTransform, wrap_lon_180};

#[derive(Clone, Debug)]
struct GeoFrameSnapshot {
    context: egui::Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    base_bounds: Option<GeoBounds>,
    drag_delta: Option<Vec2>,
    scroll_y: Option<f32>,
    cursor_pos: Option<Pos2>,
    hover_pos: Option<Pos2>,
    reset: bool,
    legend_toggled: HashSet<IdSalt>,
    cat_toggled: HashSet<String>,
}

/// Application-owned interaction state for [`crate::GeoPlot`].
#[derive(Clone, Debug, Default)]
pub struct GeoPlotState {
    bounds: Option<GeoBounds>,
    hidden_layers: HashSet<IdSalt>,
    hidden_categories: HashSet<String>,
    frame: Option<GeoFrameSnapshot>,
}

impl GeoPlotState {
    pub fn bounds(&self) -> Option<GeoBounds> {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: GeoBounds) -> Result<(), GeoError> {
        if !bounds.lat_min.is_finite()
            || !bounds.lat_max.is_finite()
            || bounds.lat_max <= bounds.lat_min
            || !bounds.lon_span.is_finite()
            || !(1e-6..=360.0).contains(&bounds.lon_span)
        {
            return Err(GeoError::InvalidBounds(
                "non-finite or degenerate geo bounds",
            ));
        }
        self.bounds = Some(bounds);
        self.frame = None;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.bounds = None;
        self.frame = None;
    }

    pub fn layer_visible(&self, id: IdSalt) -> bool {
        !self.hidden_layers.contains(&id)
    }

    pub fn set_layer_visible(&mut self, id: IdSalt, visible: bool) {
        if visible {
            self.hidden_layers.remove(&id);
        } else {
            self.hidden_layers.insert(id);
        }
    }

    pub fn category_visible(&self, category: &str) -> bool {
        !self.hidden_categories.contains(category)
    }

    pub fn set_category_visible(&mut self, category: &str, visible: bool) {
        if visible {
            self.hidden_categories.remove(category);
        } else {
            self.hidden_categories.insert(category.to_owned());
        }
    }

    pub(crate) fn retain_layers(&mut self, present: &HashSet<IdSalt>) {
        self.hidden_layers.retain(|id| present.contains(id));
    }

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(GeoFrameSnapshot {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                base_bounds: self.bounds,
                drag_delta: None,
                scroll_y: None,
                cursor_pos: None,
                hover_pos: None,
                reset: false,
                legend_toggled: HashSet::new(),
                cat_toggled: HashSet::new(),
            });
        }
    }

    pub(crate) fn toggle_layer(&mut self, id: IdSalt) -> bool {
        if !self
            .frame
            .as_mut()
            .expect("initialized geo frame")
            .legend_toggled
            .insert(id)
        {
            return false;
        }
        let next = !self.layer_visible(id);
        self.set_layer_visible(id, next);
        true
    }

    pub(crate) fn toggle_category(&mut self, cat: &str) -> bool {
        if !self
            .frame
            .as_mut()
            .expect("initialized geo frame")
            .cat_toggled
            .insert(cat.to_owned())
        {
            return false;
        }
        let next = !self.category_visible(cat);
        self.set_category_visible(cat, next);
        true
    }
}

pub(crate) struct GeoNavigation {
    pub bounds: GeoBounds,
    pub changed: bool,
    pub hover_pos: Option<Pos2>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn navigate_geo(
    ui: &Ui,
    state: &mut GeoPlotState,
    response: &mut Response,
    auto_bounds: GeoBounds,
    projection: GeoProjection,
    allow_scroll_zoom: bool,
    legend_rect: Option<Rect>,
    plot_area: Rect,
) -> GeoNavigation {
    let snap = state.frame.as_mut().expect("initialized geo frame");
    let hover = response.hover_pos();
    if let Some(pos) = hover {
        snap.hover_pos = Some(pos);
    }

    if response.double_clicked() || (response.has_focus() && ui.input(|i| i.key_pressed(Key::Home)))
    {
        snap.reset = true;
    }

    if response.dragged_by(PointerButton::Primary) {
        let d = response.drag_delta();
        if d.length_sq() > 0.0 && snap.drag_delta.is_none() {
            snap.drag_delta = Some(d);
        }
    }

    if allow_scroll_zoom
        && let Some(pointer) = hover
        && legend_rect.is_none_or(|r| !r.contains(pointer))
    {
        let scroll_y = ui.input_mut(|i| {
            if i.modifiers != Modifiers::NONE {
                return 0.0;
            }
            let d = i.smooth_scroll_delta.y;
            i.smooth_scroll_delta.y = 0.0;
            d
        });
        if scroll_y.abs() > 0.0 && snap.scroll_y.is_none() {
            snap.scroll_y = Some(scroll_y);
            snap.cursor_pos = Some(pointer);
        }
    }

    if snap.reset {
        let changed = state.bounds.is_some();
        state.bounds = None;
        return GeoNavigation {
            bounds: auto_bounds,
            changed,
            hover_pos: snap.hover_pos,
        };
    }

    let mut cur = snap.base_bounds.unwrap_or(auto_bounds);
    let mut changed = false;

    if let Some(delta) = snap.drag_delta
        && let Some(tf) = GeoTransform::new(cur, projection, plot_area)
    {
        let vp = tf.viewport();
        let dlon = f64::from(delta.x) / f64::from(vp.width().max(1.0)) * cur.lon_span;
        let y0 = projection.lat_to_norm_y(cur.lat_max);
        let y1 = projection.lat_to_norm_y(cur.lat_min);
        let dy = -f64::from(delta.y) / f64::from(vp.height().max(1.0)) * (y1 - y0);
        let y_span = (y1 - y0).clamp(1e-5, 1.0);
        let ny0 = (y0 + dy).clamp(0.0, 1.0 - y_span);
        let ny1 = ny0 + y_span;
        cur.center_lon = wrap_lon_180(cur.center_lon - dlon);
        cur.lat_max = projection.norm_y_to_lat(ny0);
        cur.lat_min = projection.norm_y_to_lat(ny1).min(cur.lat_max - 1e-4);
        changed = true;
    }

    if let Some(scroll_y) = snap.scroll_y
        && let Some(tf) = GeoTransform::new(cur, projection, plot_area)
    {
        let factor = (-f64::from(scroll_y) * 0.0025).exp().clamp(0.5, 2.0);
        let vp = tf.viewport();
        let pointer = snap.cursor_pos.unwrap_or_else(|| vp.center());
        let anchor = tf.unproject(pointer).unwrap_or(crate::axis::GeoPoint::new(
            0.5 * (cur.lat_min + cur.lat_max),
            cur.center_lon,
        ));
        let u = f64::from((pointer.x - vp.left()) / vp.width().max(1.0)).clamp(0.0, 1.0);
        let v = f64::from((pointer.y - vp.top()) / vp.height().max(1.0)).clamp(0.0, 1.0);
        let new_lon_span = (cur.lon_span * factor).clamp(0.05, 360.0);
        let old_y0 = projection.lat_to_norm_y(cur.lat_max);
        let old_y1 = projection.lat_to_norm_y(cur.lat_min);
        let old_y_span = (old_y1 - old_y0).max(1e-5);
        let new_y_span = (old_y_span * factor).clamp(1e-5, 1.0);
        let anchor_y = projection.lat_to_norm_y(anchor.lat);
        let ny0 = (anchor_y - v * new_y_span).clamp(0.0, 1.0 - new_y_span);
        let ny1 = ny0 + new_y_span;
        cur.lon_span = new_lon_span;
        cur.center_lon = wrap_lon_180(anchor.lon - (u - 0.5) * new_lon_span);
        cur.lat_max = projection.norm_y_to_lat(ny0);
        cur.lat_min = projection.norm_y_to_lat(ny1).min(cur.lat_max - 1e-4);
        changed = true;
    }

    if changed {
        state.bounds = Some(cur);
    }

    GeoNavigation {
        bounds: cur,
        changed,
        hover_pos: snap.hover_pos,
    }
}
