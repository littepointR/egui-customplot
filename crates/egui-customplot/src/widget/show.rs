use std::collections::HashSet;

use egui::{Sense, Ui};

use super::{Plot, PlotResponse};
use crate::{
    AxisScale, HoverInfo, Line, PlotBounds, PlotDiagnostics, PlotPoint, PlotState, PlotTransform,
    Series,
    interaction::{navigate, pick_point, show_tooltip},
    layout::{allocate_plot, data_rect, legend_rect, show_legend},
    model::resolve_plot_style,
    render::{AxesStyle, paint_annotations, paint_axes, paint_hover, paint_series},
};

impl Plot {
    /// Compatibility shortcut. Only series descriptors are collected, not data.
    pub fn show(self, ui: &mut Ui, state: &mut PlotState, lines: &[Line<'_>]) -> PlotResponse {
        let series: Vec<_> = lines.iter().copied().map(Series::Line).collect();
        self.show_series(ui, state, &series)
    }

    pub fn show_series(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        series: &[Series<'_>],
    ) -> PlotResponse {
        let outer = allocate_plot(ui, self.height);
        let style = resolve_plot_style(ui, self.theme.as_deref(), self.inherited_style, self.style);
        let rect = data_rect(outer, &self.axes, &self.title);
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let scales = [self.axes[0].scale, self.axes[1].scale];
        let reversed = [self.axes[0].reversed, self.axes[1].reversed];
        let (auto, mut diagnostics) = summarize_series(state, series, scales);
        diagnostics.numeric_rejections += style.invalid_strokes;
        let auto_transform = PlotTransform::with_axes(auto, rect, scales, reversed)
            .expect("valid layout and fitted bounds");
        let auto_transform = self.data_aspect.map_or(auto_transform, |ratio| {
            auto_transform.with_data_aspect(ratio).unwrap_or_else(|| {
                diagnostics.aspect_constraint_failed = true;
                auto_transform
            })
        });
        let legend_rect = self
            .show_legend
            .then(|| legend_rect(rect, series))
            .flatten();
        let sense = if self.navigation.allow_drag {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let mut response = ui.interact(rect, id, sense);
        let navigation = navigate(
            ui,
            state,
            &mut response,
            auto_transform,
            legend_rect,
            self.navigation,
        );
        let transform = navigation.transform;
        diagnostics.invalid_manual_bounds = navigation.invalid_manual_bounds;
        let bounds = transform.bounds();
        let axis_ticks = std::array::from_fn(|axis| {
            self.axes[axis].ticks(
                [bounds.min()[axis], bounds.max()[axis]],
                if axis == 0 {
                    rect.width()
                } else {
                    rect.height()
                },
            )
        });
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::Canvas,
                ui.is_enabled(),
                if self.title.is_empty() {
                    "2D plot; drag to pan, Home to reset"
                } else {
                    &self.title
                },
            )
        });
        diagnostics.painting_skipped = !ui.is_rect_visible(rect);
        if !diagnostics.painting_skipped {
            paint_axes(
                ui,
                outer,
                transform,
                &axis_ticks,
                AxesStyle {
                    axes: &self.axes,
                    title: &self.title,
                    style: &style,
                    show_grid: self.show_grid,
                },
            );
            paint_series(ui, state, series, transform, &mut diagnostics, &style);
            paint_annotations(ui, &self.annotations, transform, &style, &mut diagnostics);
        }
        let legend_entries = legend_rect.map_or_else(Vec::new, |area| {
            show_legend(ui, state, series, area, self.id_salt, &style)
        });
        let (hovered_point, hover_text) =
            if self.show_hover && ui.is_enabled() && !diagnostics.painting_skipped {
                navigation.hover_pos.map_or((None, None), |position| {
                    let point = pick_point(
                        state,
                        series,
                        transform,
                        position,
                        self.pick_radius,
                        &mut diagnostics,
                        &style,
                    );
                    paint_hover(
                        ui,
                        transform,
                        position,
                        point.as_ref(),
                        self.crosshair,
                        &style,
                    );
                    let info = HoverInfo {
                        hovered_point: point.as_ref(),
                        cursor: transform.point_from_position(position),
                        series_name: point.as_ref().map_or("", |p| series[p.series_index].name()),
                    };
                    let text = self
                        .hover_formatter
                        .as_ref()
                        .map_or_else(|| info.default_text(), |format| format(info));
                    show_tooltip(&response, &text);
                    (point, Some(text))
                })
            } else {
                (None, None)
            };
        PlotResponse {
            response,
            transform,
            diagnostics,
            view_changed: navigation.changed,
            axis_ticks,
            hovered_point,
            hover_text,
            legend_entries,
        }
    }
}

/// Merge cached extents, not raw samples. Counts refer to submitted descriptors;
/// shared raw sample payload is deduplicated independently of series visibility.
fn summarize_series(
    state: &mut PlotState,
    series: &[Series<'_>],
    scales: [AxisScale; 2],
) -> (PlotBounds, PlotDiagnostics) {
    let mut diagnostics = PlotDiagnostics::default();
    let mut present = HashSet::new();
    for (index, s) in series.iter().enumerate() {
        if !present.insert(s.id(index)) {
            diagnostics.duplicate_series_ids += 1;
        }
    }
    state.retain_series(&present);
    let mut seen = HashSet::new();
    let mut extent: Option<(PlotPoint, PlotPoint)> = None;
    for (index, s) in series.iter().enumerate() {
        let data = s.data();
        diagnostics.input_points += data.len();
        diagnostics.non_finite_points += data.non_finite_points();
        diagnostics.domain_invalid_points += data.invalid_points(scales) - data.non_finite_points();
        if seen.insert((data.identity(), data.len())) {
            diagnostics.unique_sample_bytes += data.sample_bytes();
        }
        if !state.series_visible(s.id(index)) {
            continue;
        }
        diagnostics.visible_series += 1;
        if let Some((lo, hi)) = data.extent(scales) {
            let (min, max) = extent.get_or_insert((lo, hi));
            for axis in 0..2 {
                min[axis] = min[axis].min(lo[axis]);
                max[axis] = max[axis].max(hi[axis]);
            }
        }
    }
    let default = PlotBounds::default_scaled(scales);
    let auto = match extent
        .map(|(min, max)| PlotBounds::fit_scaled(min, max, scales))
        .transpose()
    {
        Ok(bounds) => bounds.unwrap_or(default),
        Err(_) => {
            diagnostics.auto_bounds_error = true;
            default
        }
    };
    (auto, diagnostics)
}
