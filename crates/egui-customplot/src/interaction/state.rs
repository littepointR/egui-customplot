use std::collections::HashSet;

use egui::{Context, Id, IdSalt, Pos2, Ui, Vec2, ViewportId};

use crate::PlotBounds;

/// Application-owned view state, one instance per widget. None means auto-fit.
/// Do not put this in the same Context's IdTypeMap: frame snapshots hold a Context handle.
#[derive(Clone, Debug, Default)]
pub struct PlotState {
    pub(super) bounds: Option<PlotBounds>,
    hidden: HashSet<IdSalt>,
    pub(super) frame: Option<FrameSnapshot>,
}

#[derive(Clone, Debug)]
pub(super) struct FrameSnapshot {
    context: Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    pub base: Option<PlotBounds>,
    pub navigation: Navigation,
    legend_toggled: HashSet<IdSalt>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Navigation {
    pub drag: Option<Vec2>,
    pub scroll: Option<(f32, Pos2)>,
    pub reset: bool,
}

impl PlotState {
    pub fn bounds(&self) -> Option<PlotBounds> {
        self.bounds
    }
    /// Only resets navigation; legend visibility is a separate state.
    pub fn reset(&mut self) {
        self.bounds = None;
        self.frame = None;
    }
    pub fn set_bounds(&mut self, bounds: PlotBounds) {
        self.bounds = Some(bounds);
        self.frame = None;
    }
    pub fn series_visible(&self, id: IdSalt) -> bool {
        !self.hidden.contains(&id)
    }
    /// Use the series' explicit id_salt. Removed IDs are pruned during show.
    pub fn set_series_visible(&mut self, id: IdSalt, visible: bool) {
        if visible {
            self.hidden.remove(&id);
        } else {
            self.hidden.insert(id);
        }
    }

    pub(crate) fn retain_series(&mut self, present: &HashSet<IdSalt>) {
        self.hidden.retain(|id| present.contains(id));
    }

    /// Return whether a once-per-frame toggle was actually applied.
    pub(crate) fn toggle_series(&mut self, id: IdSalt) -> bool {
        if !self
            .frame
            .as_mut()
            .expect("initialized frame")
            .legend_toggled
            .insert(id)
        {
            return false;
        }
        self.set_series_visible(id, !self.series_visible(id));
        true
    }

    pub(crate) fn begin_frame(&mut self, ui: &Ui, id: Id) {
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !self.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            self.frame = Some(FrameSnapshot {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                base: self.bounds,
                navigation: Navigation::default(),
                legend_toggled: HashSet::new(),
            });
        }
    }
}
