use egui::{Sense, Ui};

use super::{Plot, PlotResponse};
use crate::{
    Line, PlotCache, PlotLayer, PlotState, PlotTransform, Series,
    axis::{SeriesTransforms, secondary_auto},
    interaction::{navigate, navigate_secondary},
    layout::{allocate_plot, data_rect, legend_rect, show_legend},
    model::{resolve_plot_style, summarize_series},
    render::{
        AxesStyle, paint_annotations, paint_axes, paint_right_axis, paint_selection_overlays,
        paint_series,
    },
};

impl Plot {
    /// Compatibility shortcut. Only series descriptors are collected, not data.
    pub fn show(self, ui: &mut Ui, state: &mut PlotState, lines: &[Line<'_>]) -> PlotResponse {
        let layers: Vec<_> = lines.iter().copied().map(PlotLayer::from).collect();
        self.show_impl(ui, state, &layers, None)
    }

    pub fn show_series(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        series: &[Series<'_>],
    ) -> PlotResponse {
        let layers: Vec<_> = series.iter().copied().map(PlotLayer::from).collect();
        self.show_impl(ui, state, &layers, None)
    }

    pub fn show_cached(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        lines: &[Line<'_>],
        cache: &mut PlotCache,
    ) -> PlotResponse {
        let layers: Vec<_> = lines.iter().copied().map(PlotLayer::from).collect();
        self.show_impl(ui, state, &layers, Some(cache))
    }

    pub fn show_series_cached(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        series: &[Series<'_>],
        cache: &mut PlotCache,
    ) -> PlotResponse {
        let layers: Vec<_> = series.iter().copied().map(PlotLayer::from).collect();
        self.show_impl(ui, state, &layers, Some(cache))
    }

    /// Ordered point-series and scalar-cell layers, sharing layout/navigation.
    pub fn show_layers(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        layers: &[PlotLayer<'_>],
    ) -> PlotResponse {
        self.show_impl(ui, state, layers, None)
    }

    /// The cache still retains line geometry only; heatmaps are queried afresh.
    pub fn show_layers_cached(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        layers: &[PlotLayer<'_>],
        cache: &mut PlotCache,
    ) -> PlotResponse {
        self.show_impl(ui, state, layers, Some(cache))
    }

    pub fn show_stream(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        lines: &[crate::StreamLine<'_>],
    ) -> PlotResponse {
        let layers: Vec<_> = lines.iter().copied().map(PlotLayer::from).collect();
        self.show_impl(ui, state, &layers, None)
    }

    pub fn show_waveform(
        self,
        ui: &mut Ui,
        state: &mut PlotState,
        lines: &[crate::WaveformLine<'_>],
    ) -> PlotResponse {
        let layers: Vec<_> = lines.iter().copied().map(PlotLayer::from).collect();
        self.show_impl(ui, state, &layers, None)
    }

    fn show_impl(
        mut self,
        ui: &mut Ui,
        state: &mut PlotState,
        series: &[PlotLayer<'_>],
        mut cache: Option<&mut PlotCache>,
    ) -> PlotResponse {
        if let Some(defs) = crate::model::plot_defaults(ui.ctx()) {
            let mut scroll_zoom = self.navigation.allow_scroll_zoom;
            defs.apply_common(
                &mut self.theme,
                &mut self.inherited_style,
                &mut scroll_zoom,
                &mut self.show_grid,
                &mut self.show_legend,
                &mut self.show_hover,
            );
            if self.navigation.allow_drag {
                self.navigation.allow_scroll_zoom = scroll_zoom;
                self.navigation.allow_box_zoom =
                    defs.resolve_box_zoom(self.navigation.allow_box_zoom);
            }
            self.crosshair = defs.resolve_crosshair(self.crosshair);
        }
        let outer = allocate_plot(ui, self.height);
        let style = resolve_plot_style(ui, self.theme.as_deref(), self.inherited_style, self.style);
        let rect = data_rect(outer, &self.axes, &self.title, self.right_y_axis.as_ref());
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let scales = [self.axes[0].scale, self.axes[1].scale];
        let reversed = [self.axes[0].reversed, self.axes[1].reversed];
        let summary = summarize_series(
            series,
            scales,
            self.right_y_axis.as_ref().map(|a| a.scale),
            |id| state.series_visible(id),
        );
        state.retain_series(&summary.present);
        let mut diagnostics = summary.diagnostics;
        diagnostics.numeric_rejections += style.invalid_strokes;
        let auto_transform = PlotTransform::with_axes(summary.bounds[0], rect, scales, reversed)
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
        let sense = if self.navigation.allow_drag
            || self.navigation.allow_box_zoom
            || self.navigation.allow_brush
        {
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
        state.record_view(
            auto_transform,
            transform,
            navigation.changed_axes,
            navigation.reset,
        );
        diagnostics.invalid_manual_bounds = navigation.invalid_manual_bounds;
        diagnostics.invalid_linked_bounds = navigation.invalid_linked_bounds;
        let secondary = self.right_y_axis.as_ref().map(|axis| {
            navigate_secondary(
                state,
                secondary_auto(transform, summary.bounds[1], axis),
                &navigation,
            )
        });
        let right_transform = secondary.as_ref().map(|s| s.transform);
        diagnostics.invalid_right_y_range =
            secondary.as_ref().is_some_and(|s| s.invalid_manual_range);
        let view_changed = navigation.changed || secondary.as_ref().is_some_and(|s| s.changed);
        if view_changed {
            response.mark_changed();
        }
        let transforms = SeriesTransforms {
            left: transform,
            right: right_transform,
        };
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
        let right_axis_ticks = self
            .right_y_axis
            .as_ref()
            .zip(right_transform)
            .map_or_else(Vec::new, |(axis, t)| {
                axis.ticks([t.bounds().min()[1], t.bounds().max()[1]], rect.height())
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
                    show_frame: self.show_frame,
                },
            );
            if let Some((axis, t)) = self.right_y_axis.as_ref().zip(right_transform) {
                paint_right_axis(ui, outer, t, axis, &right_axis_ticks, &style);
            }
            let (hidden_series, auto_cache) = state.split_visibility_and_auto_cache();
            paint_series(
                ui,
                hidden_series,
                series,
                transforms,
                &mut diagnostics,
                &style,
                cache.as_deref_mut(),
                Some(auto_cache),
            );
            paint_annotations(ui, &self.annotations, transform, &style, &mut diagnostics);
            paint_selection_overlays(
                ui,
                navigation.brushed_bounds,
                state.box_zoom_preview_rect(),
                transform,
            );
            if let Some(legend) = &self.bubble_legend {
                let samples = legend.resolved_samples();
                if !samples.is_empty() {
                    let size = crate::layout::bubble_legend_size(legend, &samples);
                    let r = transform.rect();
                    if r.width() >= size.x + 12.0 && r.height() >= size.y + 12.0 {
                        let area =
                            egui::Rect::from_min_size(r.max - size - egui::vec2(8.0, 8.0), size);
                        crate::layout::paint_bubble_legend(
                            ui,
                            area,
                            legend,
                            &samples,
                            style.text_color,
                        );
                    }
                }
            }
        }
        let legend_entries = legend_rect.map_or_else(Vec::new, |area| {
            show_legend(ui, state, series, area, self.id_salt, &style)
        });
        if let Some(cache) = cache {
            diagnostics.line_cache_bytes = cache.retained_bytes();
        }
        let (hovered_point, hovered_cell, hovered_bar, hovered_region, hovered_vector, hover_text) =
            if self.show_hover && ui.is_enabled() && !diagnostics.painting_skipped {
                navigation
                    .hover_pos
                    .map_or((None, None, None, None, None, None), |position| {
                        let (picked, text) = super::hover::show_hover(
                            ui,
                            state,
                            series,
                            super::hover::HoverContext {
                                plot: &self,
                                response: &response,
                                transforms,
                                position,
                            },
                            &mut diagnostics,
                            &style,
                        );
                        (
                            picked.point,
                            picked.cell,
                            picked.bar,
                            picked.region,
                            picked.vector,
                            Some(text),
                        )
                    })
            } else {
                (None, None, None, None, None, None)
            };
        PlotResponse {
            outer_rect: outer,
            response,
            transform,
            right_transform,
            diagnostics,
            view_changed,
            axis_ticks,
            right_axis_ticks,
            hovered_point,
            hovered_cell,
            hovered_bar,
            hovered_region,
            hovered_vector,
            hover_text,
            legend_entries,
            brushed_bounds: navigation.brushed_bounds,
            brush_finished: navigation.brush_finished,
        }
    }
}
