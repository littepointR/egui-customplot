use super::Plot3D;
use crate::{
    Axis3D, BubbleLegend, Layer3D, Plot3DResponse, Plot3DState, Transform3D,
    interaction::{navigate_plot3d, pick_plot3d_hover, show_plot3d_legend},
    layout::{
        allocate_plot, bubble_legend_size, compute_auto_bounds_3d, compute_plot3d_layout,
        paint_bubble_legend,
    },
    model::{resolve_plot_style, summarize_plot3d},
    render::paint_plot3d,
};
use egui::{Sense, Ui};

impl Plot3D {
    pub fn show(
        mut self,
        ui: &mut Ui,
        state: &mut Plot3DState,
        layers: &[Layer3D<'_>],
    ) -> Plot3DResponse {
        let mut forced_proj = self.explicit_projection;
        let mut show_grid = true;
        if let Some(defs) = crate::model::plot_defaults(ui.ctx()) {
            defs.apply_common(
                &mut self.theme,
                &mut self.inherited_style,
                &mut self.allow_scroll_zoom,
                &mut show_grid,
                &mut self.show_legend,
                &mut self.show_hover,
            );
            forced_proj = forced_proj.or(defs.projection_3d);
        }
        if let Some(proj) = forced_proj {
            self.camera.projection = proj;
        }
        let axes = self.axes.each_ref();
        let mappings = axes.map(Axis3D::mapping);
        let scales = mappings.map(|m| m.domain());
        let reversed = axes.map(Axis3D::is_reversed);
        let (mut diagnostics, present, raw_extent) =
            summarize_plot3d(layers, scales, |id| state.layer_visible(id));
        state.retain_layers(&present);
        let auto_bounds = compute_auto_bounds_3d(raw_extent, axes, mappings);

        let theme = self.theme.as_deref();
        let style = resolve_plot_style(ui, theme, self.inherited_style, self.style);
        diagnostics.numeric_rejections += style.invalid_strokes;

        let outer = allocate_plot(ui, self.height);
        let legend_count = if self.show_legend {
            layers.iter().filter(|l| !l.name().is_empty()).count()
        } else {
            0
        };
        let bubble_samples = self
            .bubble_legend
            .as_ref()
            .map(BubbleLegend::resolved_samples)
            .filter(|s| !s.is_empty());
        let bubble_size = self
            .bubble_legend
            .as_ref()
            .zip(bubble_samples.as_ref())
            .map(|(b, s)| bubble_legend_size(b, s));
        let mut body = outer;
        let placed = self
            .decorations
            .reserve(ui, &mut body, &self.title, self.color_bar.as_ref());
        let title = if self.decorations.title.is_some() {
            ""
        } else {
            &self.title
        };
        let custom_rect = self
            .legend_component
            .as_deref()
            .filter(|_| self.show_legend)
            .and_then(|c| {
                crate::layout::reserve_component(
                    ui,
                    &mut body,
                    layers.iter().map(|l| l.name()),
                    c,
                    crate::LegendPosition::OutsideRight,
                )
            });
        let mut layout = compute_plot3d_layout(
            body,
            !title.is_empty(),
            if self.legend_component.is_some() {
                0
            } else {
                legend_count
            },
            bubble_size,
            self.color_bar.is_some() && self.decorations.color_bar.is_none(),
        );

        layout.outer = outer;
        layout.legend_rect = custom_rect.or(layout.legend_rect);
        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id, self.camera, forced_proj);
        let mut response = ui.interact(layout.plot_area, id, Sense::click_and_drag());
        let nav = navigate_plot3d(
            ui,
            state,
            &mut response,
            auto_bounds,
            self.camera,
            forced_proj,
            self.controller.as_deref(),
            self.allow_scroll_zoom,
            layout.legend_rect,
        );
        if nav.changed {
            response.mark_changed();
        }

        let transform = Transform3D::with_mappings(
            nav.bounds.for_scales(mappings, auto_bounds),
            mappings,
            reversed,
            self.aspect,
            layout.plot_area,
            nav.camera,
        )
        .expect("positive plot area and validated per-axis bounds");

        let transform = self
            .lens
            .map_or(transform, |lens| transform.with_lens(lens));
        diagnostics.painting_skipped = !ui.is_rect_visible(layout.outer);
        if !diagnostics.painting_skipped {
            let painter = ui.painter_at(layout.outer);
            paint_plot3d(
                &painter,
                state,
                layers,
                &self.planes,
                &self.texts,
                [&self.axes[0], &self.axes[1], &self.axes[2]],
                transform,
                self.depth_mode,
                self.box_style,
                show_grid,
                title,
                &style,
                &mut diagnostics.plot3d,
            );
            if let (Some(area), Some(legend), Some(samples)) = (
                layout.bubble_rect,
                &self.bubble_legend,
                bubble_samples.as_ref(),
            ) {
                paint_bubble_legend(ui, area, legend, samples, style.text_color);
            }
            if let (Some(bar_rect), Some((scale, label))) = (layout.color_bar_rect, &self.color_bar)
            {
                crate::widget::color_bar::paint_inset(
                    ui,
                    layout.outer,
                    bar_rect,
                    *scale,
                    label,
                    style.text_color,
                );
            }
        }

        crate::layout::show_decorations(ui, id, placed, style.text_color);
        let legend_entries = layout.legend_rect.map_or_else(Vec::new, |rect| {
            show_plot3d_legend(
                ui,
                state,
                layers,
                rect,
                id.with("legend"),
                self.legend_component.as_deref(),
                &style,
            )
        });

        let pick = if self.show_hover
            && ui.is_enabled()
            && !diagnostics.painting_skipped
            && let Some(pointer) = nav.hover_pos
        {
            let res = pick_plot3d_hover(
                state,
                layers,
                transform,
                pointer,
                self.pick_radius,
                &mut diagnostics,
            );
            if let Some(ref msg) = res.text {
                response.clone().on_hover_ui_at_pointer(|ui| {
                    ui.monospace(msg);
                });
            }
            res
        } else {
            Default::default()
        };

        Plot3DResponse {
            outer_rect: layout.outer,
            plot_area: layout.plot_area,
            response,
            transform,
            diagnostics,
            view_changed: nav.changed,
            hovered_point: pick.point,
            hovered_bar: pick.bar,
            hovered_region: pick.region,
            hover_text: pick.text,
            legend_entries,
        }
    }
}
