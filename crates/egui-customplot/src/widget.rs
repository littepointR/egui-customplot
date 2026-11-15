use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use egui::{
    AsIdSalt, Color32, Context, Id, IdSalt, Key, Modifiers, PointerButton, Pos2, Rect, Response,
    Sense, Stroke, StrokeKind, Ui, Vec2, ViewportId,
};

use crate::{Line, PlotBounds, PlotTransform, render::prepare_line};

/// Application-owned navigation. Keep one state per plot, outside egui memory.
/// `None` bounds means auto-fit (continues following data until navigation).
#[derive(Clone, Debug, Default)]
pub struct PlotState {
    bounds: Option<PlotBounds>,
    frame: Option<FrameSnapshot>,
}

#[derive(Clone, Debug)]
struct FrameSnapshot {
    context: Context,
    viewport: ViewportId,
    id: Id,
    frame_nr: u64,
    base: Option<PlotBounds>,
    navigation: Navigation,
}

#[derive(Clone, Copy, Debug, Default)]
struct Navigation {
    drag: Option<Vec2>,
    scroll: Option<(f32, Pos2)>,
    reset: bool,
}

impl PlotState {
    pub fn bounds(&self) -> Option<PlotBounds> {
        self.bounds
    }
    /// Return to auto-fit. Safe to call repeatedly, including on layout re-runs.
    pub fn reset(&mut self) {
        self.bounds = None;
        self.frame = None;
    }
    pub fn set_bounds(&mut self, bounds: PlotBounds) {
        self.bounds = Some(bounds);
        self.frame = None;
    }
}

/// Per-show-pass CPU counters. Not whole-process memory or GPU measurements.
#[derive(Clone, Debug, Default)]
pub struct PlotDiagnostics {
    /// Across series references, so shared data drawn twice counts twice here.
    pub input_points: usize,
    /// Unique shared sample payloads, excluding allocator metadata.
    pub unique_sample_bytes: usize,
    pub non_finite_points: usize,
    pub visited_segments: usize,
    pub output_points: usize,
    pub output_paths: usize,
    /// Screen-point Vec capacities handed to egui, not tessellated mesh bytes.
    pub geometry_capacity_bytes: usize,
    pub numeric_rejections: usize,
    pub auto_bounds_error: bool,
    pub painting_skipped: bool,
    /// Geometry preparation and shape submission, excluding axes/tessellation/GPU.
    pub prepare_duration: Duration,
}

pub struct PlotResponse {
    pub response: Response,
    pub transform: PlotTransform,
    pub diagnostics: PlotDiagnostics,
    /// Relative to the frame's initial bounds, not a once-only external event.
    /// UI closures can run multiple times; consume final state outside the closure.
    pub view_changed: bool,
}

/// Native egui control. Drag to pan, double-click/Home to reset.
/// Hover+wheel zooms; wheel zoom is opt-in inside scrolling containers.
pub struct Plot {
    id_salt: IdSalt,
    height: f32,
    show_grid: bool,
    allow_drag: bool,
    allow_scroll_zoom: bool,
    background: Option<Color32>,
}

impl Plot {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 320.0,
            show_grid: true,
            allow_drag: true,
            allow_scroll_zoom: false,
            background: None,
        }
    }
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.max(1.0);
        }
        self
    }
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }
    pub fn allow_drag(mut self, allow: bool) -> Self {
        self.allow_drag = allow;
        self
    }
    /// Defaults to false to let surrounding ScrollArea widgets own the wheel.
    /// Enabled plots consume hovered vertical smooth-scroll, without modifiers.
    /// Ctrl/cmd-scroll remains available for egui's global UI zoom.
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }
    pub fn background(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn show(self, ui: &mut Ui, state: &mut PlotState, lines: &[Line<'_>]) -> PlotResponse {
        // Horizontal ScrollArea layouts may offer infinite width. A plot still
        // needs finite screen geometry; default to the visible viewport width.
        let available_width = ui.available_width();
        let width = if available_width.is_finite() {
            available_width.max(1.0)
        } else {
            ui.clip_rect()
                .intersect(ui.ctx().viewport_rect())
                .width()
                .max(1.0)
        };
        let size = egui::vec2(width, self.height);
        let (_, outer) = ui.allocate_space(size);
        // Stable user ID, independent of insertion/removal of preceding widgets.
        let id = ui.make_persistent_id(self.id_salt);
        let margin = egui::vec2(
            52.0_f32.min(outer.width() * 0.2),
            22.0_f32.min(outer.height() * 0.2),
        );
        let rect = outer.shrink2(margin);
        let sense = if self.allow_drag {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let mut response = ui.interact(rect, id, sense);
        let mut diagnostics = PlotDiagnostics::default();
        let mut seen = HashSet::new();
        let mut extent: Option<([f64; 2], [f64; 2])> = None;
        for line in lines {
            diagnostics.input_points += line.data.len();
            diagnostics.non_finite_points += line.data.non_finite_points();
            if seen.insert((line.data.identity(), line.data.len())) {
                diagnostics.unique_sample_bytes += line.data.sample_bytes();
            }
            if let Some((lo, hi)) = line.data.extent {
                let (min, max) = extent.get_or_insert((lo, hi));
                for axis in 0..2 {
                    min[axis] = min[axis].min(lo[axis]);
                    max[axis] = max[axis].max(hi[axis]);
                }
            }
        }
        let auto = match extent.map(|(lo, hi)| PlotBounds::fit(lo, hi)).transpose() {
            Ok(bounds) => bounds.unwrap_or_default(),
            Err(_) => {
                diagnostics.auto_bounds_error = true;
                PlotBounds::default()
            }
        };
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let viewport = ui.ctx().viewport_id();
        if !state.frame.as_ref().is_some_and(|f| {
            f.context == *ui.ctx() && f.viewport == viewport && f.id == id && f.frame_nr == frame_nr
        }) {
            state.frame = Some(FrameSnapshot {
                context: ui.ctx().clone(),
                viewport,
                id,
                frame_nr,
                base: state.bounds,
                navigation: Navigation::default(),
            });
        }
        let frame = state.frame.as_mut().expect("snapshot initialized");
        if response.clicked() || response.drag_started() {
            response.request_focus();
        }
        if ui.is_enabled() {
            frame.navigation.reset |= response.double_clicked_by(PointerButton::Primary)
                || (response.has_focus()
                    && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Home)));
            if self.allow_drag && response.dragged_by(PointerButton::Primary) {
                let delta = response.drag_delta();
                if delta != Vec2::ZERO && frame.navigation.drag.is_none() {
                    frame.navigation.drag = Some(delta);
                }
            }
            if self.allow_scroll_zoom && response.hovered() {
                let scroll = ui.input_mut(|i| {
                    if i.modifiers == Modifiers::NONE {
                        let delta = i.smooth_scroll_delta.y;
                        i.smooth_scroll_delta.y = 0.0;
                        delta
                    } else {
                        0.0
                    }
                });
                if frame.navigation.scroll.is_none()
                    && scroll != 0.0
                    && let Some(pos) = response.hover_pos()
                {
                    frame.navigation.scroll = Some((scroll, pos));
                }
            }
        }
        // Re-evaluate the saved input against the FINAL pass's rectangle. egui
        // can clear raw events/deltas between passes, so skipping later passes
        // or merely reading their input would lose navigation or use old layout.
        let base = frame.base;
        let mut candidate = base;
        let mut bounds = base.unwrap_or(auto);
        if ui.is_enabled() && !frame.navigation.reset {
            if self.allow_drag
                && response.dragged_by(PointerButton::Primary)
                && let Some(delta) = frame.navigation.drag
                && let Some(transform) = PlotTransform::new(bounds, rect)
                && let Ok(panned) = transform.pan(delta)
            {
                bounds = panned;
                candidate = Some(bounds);
            }
            if self.allow_scroll_zoom
                && response.hovered()
                && let Some((scroll, pos)) = frame.navigation.scroll
                && rect.contains(pos)
                && let Some(transform) = PlotTransform::new(bounds, rect)
                && let Ok(zoomed) = bounds.zoomed(
                    transform.point_from_position(pos),
                    (-f64::from(scroll.clamp(-500.0, 500.0)) * 0.005).exp(),
                )
            {
                bounds = zoomed;
                candidate = Some(bounds);
            }
        } else if ui.is_enabled() && frame.navigation.reset {
            candidate = None;
        }
        state.bounds = candidate;
        let view_changed = candidate != base;
        if view_changed {
            response.mark_changed();
        }
        let transform = PlotTransform::new(candidate.unwrap_or(auto), rect)
            .expect("allocated positive finite rectangle");
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::Canvas,
                ui.is_enabled(),
                "2D plot; drag to pan, Home to reset",
            )
        });
        diagnostics.painting_skipped = !ui.is_rect_visible(rect);
        if !diagnostics.painting_skipped {
            self.paint_axes(ui, outer, transform);
            let painter = ui.painter().with_clip_rect(rect);
            let start = Instant::now();
            let palette = [
                Color32::from_rgb(70, 160, 235),
                Color32::from_rgb(235, 145, 65),
                Color32::from_rgb(90, 190, 130),
            ];
            for (index, line) in lines.iter().enumerate() {
                let stroke = line
                    .stroke
                    .unwrap_or(Stroke::new(1.5, palette[index % palette.len()]));
                if !stroke.width.is_finite()
                    || stroke.width <= 0.0
                    || stroke.color == Color32::TRANSPARENT
                {
                    continue;
                }
                let prepared = prepare_line(line.data.points(), transform);
                diagnostics.visited_segments += prepared.visited_segments;
                diagnostics.numeric_rejections += prepared.numeric_rejections;
                diagnostics.output_paths += prepared.paths.len();
                for path in prepared.paths {
                    diagnostics.output_points += path.len();
                    diagnostics.geometry_capacity_bytes +=
                        path.capacity() * std::mem::size_of::<Pos2>();
                    painter.add(egui::Shape::line(path, stroke));
                }
            }
            diagnostics.prepare_duration = start.elapsed();
        }
        PlotResponse {
            response,
            transform,
            diagnostics,
            view_changed,
        }
    }

    fn paint_axes(&self, ui: &Ui, outer: Rect, transform: PlotTransform) {
        let painter = ui.painter().with_clip_rect(outer);
        let rect = transform.rect();
        let visuals = ui.visuals();
        painter.rect_filled(
            rect,
            0.0,
            self.background.unwrap_or(visuals.extreme_bg_color),
        );
        let grid = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
        let font = egui::TextStyle::Small.resolve(ui.style());
        for index in 0..=4 {
            let t = index as f32 / 4.0;
            let x = egui::lerp(rect.x_range(), t);
            let y = egui::lerp(rect.y_range(), t);
            if self.show_grid {
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    grid,
                );
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    grid,
                );
            }
            // Basic fixed divisions only; nice ticks/custom formatters come later.
            let x_value = transform.point_from_position(egui::pos2(x, rect.bottom()))[0];
            let y_value = transform.point_from_position(egui::pos2(rect.left(), y))[1];
            painter.text(
                egui::pos2(x, rect.bottom() + 4.0),
                egui::Align2::CENTER_TOP,
                format_tick(x_value),
                font.clone(),
                visuals.text_color(),
            );
            painter.text(
                egui::pos2(rect.left() - 4.0, y),
                egui::Align2::RIGHT_CENTER,
                format_tick(y_value),
                font.clone(),
                visuals.text_color(),
            );
        }
        painter.rect_stroke(
            rect,
            0.0,
            visuals.widgets.noninteractive.bg_stroke,
            StrokeKind::Inside,
        );
    }
}

fn format_tick(value: f64) -> String {
    if value != 0.0 && !(0.001..10_000.0).contains(&value.abs()) {
        format!("{value:.1e}")
    } else {
        format!("{value:.2}")
    }
}
