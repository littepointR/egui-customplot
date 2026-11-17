use egui::{Key, Modifiers, PointerButton, Pos2, Rect, Response, Ui};

use crate::{PlotBounds, PlotTransform};

use super::NavigationOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GestureKind {
    BoxZoom,
    Brush,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ActiveGesture {
    pub kind: GestureKind,
    pub button: PointerButton,
    pub start: Pos2,
    pub current: Pos2,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SelectionCapture {
    pub raw_press: Option<ActiveGesture>,
    pub cur_pos: Option<Pos2>,
    pub released: bool,
    pub escape_pressed: bool,
}

pub(super) struct SelectionOutcome {
    pub zoom_bounds: Option<PlotBounds>,
    pub applied_box_zoom: Option<Rect>,
    pub brush_finished: bool,
}

fn shift_held(i: &egui::InputState) -> bool {
    i.modifiers.shift
        || i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::PointerButton {
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.shift
            )
        })
}

fn alt_held(i: &egui::InputState) -> bool {
    i.modifiers.alt
        || i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::PointerButton {
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.alt
            )
        })
}

pub(super) fn is_primary_claimed_by_selection(
    ui: &Ui,
    gesture_base: Option<ActiveGesture>,
    cap: SelectionCapture,
    options: NavigationOptions,
) -> bool {
    if gesture_base.is_some_and(|g| g.button == PointerButton::Primary)
        || cap
            .raw_press
            .is_some_and(|g| g.button == PointerButton::Primary)
    {
        return true;
    }
    if gesture_base.is_none() {
        return ui.input(|i| {
            let active = i.pointer.button_pressed(PointerButton::Primary)
                || i.pointer.button_down(PointerButton::Primary);
            active
                && ((options.allow_box_zoom && shift_held(i))
                    || (options.allow_brush && alt_held(i)))
        });
    }
    false
}

fn detect_press_candidate(ui: &Ui, options: NavigationOptions) -> Option<ActiveGesture> {
    ui.input(|i| {
        let press_origin = i.pointer.press_origin();
        if options.allow_box_zoom {
            if i.pointer.button_pressed(PointerButton::Secondary) {
                let start = press_origin
                    .or_else(|| i.pointer.interact_pos())
                    .or_else(|| i.pointer.latest_pos())?;
                return Some(ActiveGesture {
                    kind: GestureKind::BoxZoom,
                    button: PointerButton::Secondary,
                    start,
                    current: start,
                });
            }
            if shift_held(i) && i.pointer.button_pressed(PointerButton::Primary) {
                let start = press_origin
                    .or_else(|| i.pointer.interact_pos())
                    .or_else(|| i.pointer.latest_pos())?;
                return Some(ActiveGesture {
                    kind: GestureKind::BoxZoom,
                    button: PointerButton::Primary,
                    start,
                    current: start,
                });
            }
        }
        if options.allow_brush
            && i.pointer.button_pressed(PointerButton::Primary)
            && (alt_held(i) || (!options.allow_drag && !shift_held(i)))
        {
            let start = press_origin
                .or_else(|| i.pointer.interact_pos())
                .or_else(|| i.pointer.latest_pos())?;
            return Some(ActiveGesture {
                kind: GestureKind::Brush,
                button: PointerButton::Primary,
                start,
                current: start,
            });
        }
        None
    })
}

pub(super) fn capture_frame_input(
    ui: &Ui,
    response: &Response,
    options: NavigationOptions,
    gesture_base: Option<ActiveGesture>,
    cap: &mut SelectionCapture,
) {
    if !ui.is_enabled() {
        return;
    }
    if (response.has_focus() || response.hovered())
        && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
    {
        cap.escape_pressed = true;
    }
    if gesture_base.is_none() && cap.raw_press.is_none() {
        cap.raw_press = detect_press_candidate(ui, options);
    }
    if let Some(active) = gesture_base.or(cap.raw_press) {
        ui.input(|i| {
            if let Some(p) = i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()) {
                cap.cur_pos = Some(p);
            }
            if i.pointer.button_released(active.button) || !i.pointer.button_down(active.button) {
                cap.released = true;
            }
        });
    }
}

fn clamped_rect(a: Pos2, b: Pos2, bounds: Rect) -> Rect {
    let x0 = a.x.clamp(bounds.left(), bounds.right());
    let x1 = b.x.clamp(bounds.left(), bounds.right());
    let y0 = a.y.clamp(bounds.top(), bounds.bottom());
    let y1 = b.y.clamp(bounds.top(), bounds.bottom());
    Rect::from_two_pos(Pos2::new(x0, y0), Pos2::new(x1, y1))
}

fn data_bounds_from_rect(screen: Rect, transform: PlotTransform) -> Option<PlotBounds> {
    let p0 = transform.point_from_position(screen.left_top());
    let p1 = transform.point_from_position(screen.right_bottom());
    if !p0[0].is_finite() || !p0[1].is_finite() || !p1[0].is_finite() || !p1[1].is_finite() {
        return None;
    }
    let lo = [p0[0].min(p1[0]), p0[1].min(p1[1])];
    let hi = [p0[0].max(p1[0]), p0[1].max(p1[1])];
    PlotBounds::new(lo, hi).ok()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_selection(
    ui: &Ui,
    state: &mut super::PlotState,
    auto: PlotTransform,
    transform: PlotTransform,
    legend_rect: Option<Rect>,
    options: NavigationOptions,
    gesture_base: Option<ActiveGesture>,
    brush_base: Option<PlotBounds>,
    cap: SelectionCapture,
) -> SelectionOutcome {
    let mut out = SelectionOutcome {
        zoom_bounds: None,
        applied_box_zoom: None,
        brush_finished: false,
    };
    state.active_gesture = gesture_base;
    state.brushed_bounds = brush_base;

    if !ui.is_enabled() {
        state.active_gesture = None;
        return out;
    }
    if cap.escape_pressed {
        state.active_gesture = None;
        state.brushed_bounds = None;
        return out;
    }

    let valid_origin =
        |p: Pos2| auto.rect().contains(p) && !legend_rect.is_some_and(|lr| lr.contains(p));
    let gesture = match gesture_base {
        Some(g) => Some(g),
        None => cap.raw_press.filter(|g| valid_origin(g.start)),
    };
    let Some(mut g) = gesture else {
        return out;
    };

    let cur = cap.cur_pos.unwrap_or(g.current);
    let released = cap.released;
    g.current = cur;
    let screen = clamped_rect(g.start, cur, transform.rect());

    if !released {
        state.active_gesture = Some(g);
        if g.kind == GestureKind::Brush
            && screen.width() >= 3.0
            && screen.height() >= 3.0
            && let Some(bounds) = data_bounds_from_rect(screen, transform)
        {
            state.brushed_bounds = Some(bounds);
        }
        return out;
    }

    state.active_gesture = None;
    match g.kind {
        GestureKind::BoxZoom => {
            let zoom_ok = screen.width() >= 4.0
                && screen.height() >= 4.0
                && options.zoom_axes.iter().any(|&a| a)
                && (auto.data_aspect().is_none() || options.zoom_axes.iter().all(|&a| a));
            if zoom_ok && let Some(rect_bounds) = data_bounds_from_rect(screen, transform) {
                let cur_bounds = transform.bounds();
                let mut lo = rect_bounds.min();
                let mut hi = rect_bounds.max();
                for axis in 0..2 {
                    if !options.zoom_axes[axis] {
                        lo[axis] = cur_bounds.min()[axis];
                        hi[axis] = cur_bounds.max()[axis];
                    }
                }
                if let Ok(req) = PlotBounds::new(lo, hi)
                    && let Some(zoomed) = auto.with_bounds(req)
                {
                    out.zoom_bounds = Some(zoomed.bounds());
                    out.applied_box_zoom = Some(screen);
                }
            }
        }
        GestureKind::Brush => {
            if screen.width() >= 3.0 && screen.height() >= 3.0 {
                if let Some(bounds) = data_bounds_from_rect(screen, transform) {
                    state.brushed_bounds = Some(bounds);
                    out.brush_finished = true;
                }
            } else {
                state.brushed_bounds = None;
            }
        }
    }
    out
}
