use egui::{Key, Modifiers, PointerButton, Pos2, Rect, Response, Ui, Vec2};

use super::PlotState;
use crate::PlotTransform;

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
    let mut transform = candidate.and_then(|b| auto.with_bounds(b)).unwrap_or(auto);
    if ui.is_enabled() {
        if frame.navigation.reset {
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
                if let Ok(zoomed) = transform.zoom_at(
                    pos,
                    options
                        .zoom_axes
                        .map(|allow| if allow { factor } else { 1.0 }),
                ) {
                    candidate = Some(zoomed);
                }
            }
        }
    }
    state.bounds = candidate;
    let changed = candidate != base;
    if changed {
        response.mark_changed();
    }
    NavigationResult {
        transform: candidate.and_then(|b| auto.with_bounds(b)).unwrap_or(auto),
        changed,
        invalid_manual_bounds,
        hover_pos,
    }
}
