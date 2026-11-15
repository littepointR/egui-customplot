use egui::{Key, Modifiers, PointerButton, Pos2, Rect, Response, Ui, Vec2};

use super::PlotState;
use crate::{PlotTransform, axis::apply_axis_ranges};

#[derive(Clone, Copy, Debug)]
pub(crate) struct NavigationOptions {
    pub allow_drag: bool,
    pub drag_axes: [bool; 2],
    pub zoom_axes: [bool; 2],
    pub allow_scroll_zoom: bool,
}

impl Default for NavigationOptions {
    fn default() -> Self {
        Self {
            allow_drag: true,
            drag_axes: [true; 2],
            zoom_axes: [true; 2],
            allow_scroll_zoom: false,
        }
    }
}

pub(crate) struct NavigationResult {
    pub transform: PlotTransform,
    pub changed: bool,
    pub invalid_manual_bounds: bool,
    pub hover_pos: Option<Pos2>,
    pub changed_axes: [bool; 2],
    pub reset: bool,
    pub invalid_linked_bounds: usize,
    pub applied_pan: Option<Vec2>,
    pub applied_zoom: Option<(Pos2, [f64; 2])>,
}

/// Capture input once, but derive the result again from the frame baseline and
/// final plot rectangle on every egui layout pass.
pub(crate) fn navigate(
    ui: &Ui,
    state: &mut PlotState,
    response: &mut Response,
    auto: PlotTransform,
    legend_rect: Option<Rect>,
    options: NavigationOptions,
) -> NavigationResult {
    let hover_pos = response
        .hover_pos()
        .filter(|p| !legend_rect.is_some_and(|r| r.contains(*p)));
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
    let frame = state.frame.as_mut().expect("initialized frame");
    if ui.is_enabled() {
        frame.navigation.reset |= (hover_pos.is_some()
            && response.double_clicked_by(PointerButton::Primary))
            || (response.has_focus()
                && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Home)));
        if options.allow_drag
            && response.dragged_by(PointerButton::Primary)
            && frame.navigation.drag.is_none()
        {
            let delta = response.drag_delta();
            let from_legend = ui
                .input(|i| i.pointer.press_origin())
                .is_some_and(|p| legend_rect.is_some_and(|r| r.contains(p)));
            if delta != Vec2::ZERO && !from_legend {
                frame.navigation.drag = Some(delta);
            }
        }
        if options.allow_scroll_zoom && hover_pos.is_some() {
            let scroll = ui.input_mut(|i| {
                if i.modifiers != Modifiers::NONE {
                    return 0.0;
                }
                let delta = i.smooth_scroll_delta.y;
                i.smooth_scroll_delta.y = 0.0;
                delta
            });
            if scroll != 0.0 && frame.navigation.scroll.is_none() {
                frame.navigation.scroll = hover_pos.map(|p| (scroll, p));
            }
        }
    }
    let base = frame.base;
    let mut candidate = base.and_then(|b| auto.with_bounds(b)).map(|t| t.bounds());
    let invalid_manual_bounds = base.is_some() && candidate.is_none();
    let local = candidate.and_then(|b| auto.with_bounds(b)).unwrap_or(auto);
    let has_input = frame.navigation.drag.is_some() || frame.navigation.scroll.is_some();
    let ranges = if has_input {
        frame.linked_base
    } else {
        state.linked_ranges
    };
    let (mut transform, mut invalid_linked_bounds) = apply_axis_ranges(local, ranges);
    let before = transform.bounds();
    let mut moved = false;
    let mut applied_pan = None;
    let mut applied_zoom = None;
    let reset = ui.is_enabled() && frame.navigation.reset;
    if ui.is_enabled() {
        if reset {
            candidate = None;
        } else {
            if options.allow_drag
                && response.dragged_by(PointerButton::Primary)
                && let Some(mut delta) = frame.navigation.drag
            {
                if !options.drag_axes[0] {
                    delta.x = 0.0;
                }
                if !options.drag_axes[1] {
                    delta.y = 0.0;
                }
                if delta != Vec2::ZERO
                    && let Ok(panned) = transform.pan(delta)
                {
                    candidate = Some(panned);
                    transform = auto.with_bounds(panned).expect("validated pan");
                    moved = true;
                    applied_pan = Some(delta);
                }
            }
            if options.allow_scroll_zoom
                && hover_pos.is_some()
                && let Some((scroll, pos)) = frame.navigation.scroll
                && auto.rect().contains(pos)
                && options.zoom_axes.iter().any(|enabled| *enabled)
                // Equal-aspect zoom cannot change only one axis while leaving
                // the other span locked. Honor the lock and leave zoom alone.
                && (auto.data_aspect().is_none() || options.zoom_axes.iter().all(|enabled| *enabled))
            {
                let factor = (-f64::from(scroll.clamp(-500.0, 500.0)) * 0.005).exp();
                let factors = options
                    .zoom_axes
                    .map(|allow| if allow { factor } else { 1.0 });
                if let Ok(zoomed) = transform.zoom_at(pos, factors) {
                    candidate = Some(zoomed);
                    moved = true;
                    applied_zoom = Some((pos, factors));
                }
            }
        }
    }
    state.bounds = candidate;
    let changed = candidate != base || (reset && frame.linked_base.iter().any(Option::is_some));
    if changed {
        response.mark_changed();
    }
    let local = candidate.and_then(|b| auto.with_bounds(b)).unwrap_or(auto);
    // Real navigation is always recomputed from this frame's original linked
    // baseline. Followers and reset views use the latest synchronized ranges.
    let transform = if moved {
        local
    } else {
        let (shown, rejected) = apply_axis_ranges(local, state.linked_ranges);
        invalid_linked_bounds = rejected;
        shown
    };
    let after = transform.bounds();
    let changed_axes = std::array::from_fn(|axis| {
        moved
            && (before.min()[axis] != after.min()[axis] || before.max()[axis] != after.max()[axis])
    });
    NavigationResult {
        transform,
        changed,
        invalid_manual_bounds,
        hover_pos,
        changed_axes,
        reset,
        invalid_linked_bounds,
        applied_pan,
        applied_zoom,
    }
}
