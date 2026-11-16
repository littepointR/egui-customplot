use std::collections::HashSet;

use egui::{Context, Id, IdSalt, Pos2, Ui, Vec2, ViewportId};

use crate::{PlotBounds, PlotTransform};

/// Application-owned view state, one instance per widget. None means auto-fit.
/// Do not put this in the same Context's IdTypeMap: frame snapshots hold a Context handle.
#[derive(Clone, Debug)]
pub struct PlotState {
    pub(super) bounds: Option<PlotBounds>,
    pub(super) linked_ranges: [Option<[f64; 2]>; 2],
    pub(super) right_y_range: Option<[f64; 2]>,
    hidden: HashSet<IdSalt>,
    pub(super) frame: Option<FrameSnapshot>,
    pub(crate) auto_cache: crate::PlotCache,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            bounds: None,
            linked_ranges: [None; 2],
            right_y_range: None,
            hidden: HashSet::new(),
            frame: None,
            auto_cache: crate::PlotCache::new(2 * 1024 * 1024),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct FrameSnapshot {
    context: Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    pub base: Option<PlotBounds>,
    pub linked_base: [Option<[f64; 2]>; 2],
    pub right_y_base: Option<[f64; 2]>,
    pub navigation: Navigation,
    pub view: Option<ViewSnapshot>,
    legend_toggled: HashSet<IdSalt>,
}

impl FrameSnapshot {
    pub(super) fn is_current(&self, ctx: &Context) -> bool {
        self.context == *ctx
            && self.viewport == ctx.viewport_id()
            && self.frame_nr == ctx.cumulative_frame_nr()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewSnapshot {
    pub auto: PlotTransform,
    pub shown: PlotTransform,
    pub changed_axes: [bool; 2],
    pub reset: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Navigation {
    pub drag: Option<Vec2>,
    pub scroll: Option<(f32, Pos2)>,
    pub reset: bool,
}

impl PlotState {
    /// Local manual bounds, excluding externally linked ranges. The effective
    /// displayed range is always available in PlotResponse::transform.
    pub fn bounds(&self) -> Option<PlotBounds> {
        self.bounds
    }
    /// Only resets navigation; legend visibility is a separate state.
    pub fn reset(&mut self) {
        self.bounds = None;
        self.right_y_range = None;
        self.frame = None;
        self.auto_cache.clear();
    }
    pub fn set_bounds(&mut self, bounds: PlotBounds) {
        self.bounds = Some(bounds);
        self.frame = None;
    }
    pub fn right_y_range(&self) -> Option<[f64; 2]> {
        self.right_y_range
    }

    /// Manual secondary Y range. Log-domain compatibility is checked at show.
    pub fn set_right_y_range(&mut self, range: [f64; 2]) -> Result<(), crate::BoundsError> {
        PlotBounds::new([0.0, range[0]], [1.0, range[1]])?;
        self.right_y_range = Some(range);
        self.frame = None;
        Ok(())
    }
    /// Stop using shared ranges. Also remove this state from the host's next
    /// AxisLink::synchronize call, otherwise the group will link it again.
    pub fn clear_axis_links(&mut self) {
        self.linked_ranges = [None; 2];
    }

    pub(crate) fn record_view(
        &mut self,
        auto: PlotTransform,
        shown: PlotTransform,
        changed_axes: [bool; 2],
        reset: bool,
    ) {
        self.frame.as_mut().expect("initialized frame").view = Some(ViewSnapshot {
            auto,
            shown,
            changed_axes,
            reset,
        });
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

    pub(crate) fn split_visibility_and_auto_cache(
        &mut self,
    ) -> (&HashSet<IdSalt>, &mut crate::PlotCache) {
        (&self.hidden, &mut self.auto_cache)
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
                linked_base: self.linked_ranges,
                right_y_base: self.right_y_range,
                navigation: Navigation::default(),
                view: None,
                legend_toggled: HashSet::new(),
            });
        }
    }
}
