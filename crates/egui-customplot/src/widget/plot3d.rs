use std::sync::Arc;

use egui::{AsIdSalt, IdSalt, Response, Sense, Ui};

use crate::{
    Axis, Axis3D, AxisScale, Bounds3D, BoxStyle3D, BubbleLegend, Camera3D, ColorScale,
    ConstantPlane3D, DataAspect3D, DepthMode, Hovered3DBar, Hovered3DPoint, Hovered3DRegion,
    Layer3D, LegendEntry, Plot3DState, PlotDiagnostics, PlotResponse, PlotStyle, PlotTheme,
    Projection3D, Text3D, Transform3D,
    interaction::{navigate_plot3d, pick_plot3d_hover, show_plot3d_legend},
    layout::{
        allocate_plot, bubble_legend_size, compute_auto_bounds_3d, compute_plot3d_layout,
        paint_bubble_legend,
    },
    model::{resolve_plot_style, summarize_plot3d},
    render::{paint_color_bar, paint_plot3d},
};

/// 3D Cartesian axes widget (`plot3`, `surf`, `mesh`, `contour3`, `fimplicit3`, `trisurf`, etc.).
pub struct Plot3D {
    id_salt: IdSalt,
    height: f32,
    title: String,
    axes: [Axis3D; 3],
    camera: Camera3D,
    explicit_projection: Option<Projection3D>,
    aspect: DataAspect3D,
    depth_mode: DepthMode,
    box_style: BoxStyle3D,
    planes: Arc<[ConstantPlane3D]>,
    texts: Arc<[Text3D]>,
    bubble_legend: Option<BubbleLegend>,
    color_bar: Option<(ColorScale, String)>,
    allow_scroll_zoom: bool,
    show_legend: bool,
    show_hover: bool,
    pick_radius: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

pub struct Plot3DResponse {
    pub outer_rect: egui::Rect,
    pub plot_area: egui::Rect,
    pub response: Response,
    pub transform: Transform3D,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_point: Option<Hovered3DPoint>,
    pub hovered_bar: Option<Hovered3DBar>,
    pub hovered_region: Option<Hovered3DRegion>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}

impl Plot3DResponse {
    pub fn into_plot_response(self) -> PlotResponse {
        PlotResponse::from_embedded(
            self.outer_rect,
            self.plot_area,
            self.response,
            self.diagnostics,
            self.view_changed,
            self.hover_text,
            self.legend_entries,
        )
    }
}

impl Plot3D {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 300.0,
            title: String::new(),
            axes: [Axis3D::new(), Axis3D::new(), Axis3D::new()],
            camera: Camera3D::default(),
            explicit_projection: None,
            aspect: DataAspect3D::Auto,
            depth_mode: DepthMode::SortedPainter,
            box_style: BoxStyle3D::BackPlanes,
            planes: Arc::from([]),
            texts: Arc::from([]),
            bubble_legend: None,
            color_bar: None,
            allow_scroll_zoom: false,
            show_legend: true,
            show_hover: true,
            pick_radius: 10.0,
            inherited_style: PlotStyle::default(),
            style: PlotStyle::default(),
            theme: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() && height > 0.0 {
            self.height = height.max(32.0);
        }
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn x_axis(mut self, axis: Axis3D) -> Self {
        self.axes[0] = axis;
        self
    }
    pub fn y_axis(mut self, axis: Axis3D) -> Self {
        self.axes[1] = axis;
        self
    }
    pub fn z_axis(mut self, axis: Axis3D) -> Self {
        self.axes[2] = axis;
        self
    }
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.axes[0] = self.axes[0].clone().label(label);
        self
    }
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.axes[1] = self.axes[1].clone().label(label);
        self
    }
    pub fn z_label(mut self, label: impl Into<String>) -> Self {
        self.axes[2] = self.axes[2].clone().label(label);
        self
    }
    pub fn camera(mut self, mut camera: Camera3D) -> Self {
        if let Some(proj) = self.explicit_projection {
            camera.projection = proj;
        }
        self.camera = camera;
        self
    }
    pub fn view(mut self, azimuth_deg: f64, elevation_deg: f64) -> Self {
        self.camera.azimuth_deg = azimuth_deg;
        self.camera.elevation_deg = elevation_deg.clamp(-90.0, 90.0);
        self
    }
    pub fn projection(mut self, projection: Projection3D) -> Self {
        self.camera.projection = projection;
        self.explicit_projection = Some(projection);
        self
    }
    pub fn data_aspect(mut self, aspect: DataAspect3D) -> Self {
        self.aspect = aspect;
        self
    }
    pub fn depth_mode(mut self, mode: DepthMode) -> Self {
        self.depth_mode = mode;
        self
    }
    pub fn box_style(mut self, style: BoxStyle3D) -> Self {
        self.box_style = style;
        self
    }
    pub fn constant_planes(mut self, planes: impl Into<Arc<[ConstantPlane3D]>>) -> Self {
        self.planes = planes.into();
        self
    }
    pub fn texts(mut self, texts: impl Into<Arc<[Text3D]>>) -> Self {
        self.texts = texts.into();
        self
    }
    pub fn bubble_legend(mut self, legend: BubbleLegend) -> Self {
        self.bubble_legend = Some(legend);
        self
    }
    pub fn color_bar(mut self, scale: ColorScale, label: impl Into<String>) -> Self {
        self.color_bar = Some((scale, label.into()));
        self
    }
    pub fn allow_scroll_zoom(mut self, allow: bool) -> Self {
        self.allow_scroll_zoom = allow;
        self
    }
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }
    pub fn show_hover(mut self, show: bool) -> Self {
        self.show_hover = show;
        self
    }
    pub fn style(mut self, style: PlotStyle) -> Self {
        self.style = style;
        self
    }
    pub fn theme(mut self, theme: impl Into<Arc<PlotTheme>>) -> Self {
        self.theme = Some(theme.into());
        self
    }

    pub fn show(
        mut self,
        ui: &mut Ui,
        state: &mut Plot3DState,
        layers: &[Layer3D<'_>],
    ) -> Plot3DResponse {
        let mut forced_proj = self.explicit_projection;
        if let Some(defs) = crate::model::plot_defaults(ui.ctx()) {
            let mut dummy_grid = true;
            defs.apply_common(
                &mut self.theme,
                &mut self.inherited_style,
                &mut self.allow_scroll_zoom,
                &mut dummy_grid,
                &mut self.show_legend,
                &mut self.show_hover,
            );
            forced_proj = forced_proj.or(defs.projection_3d);
        }
        if let Some(proj) = forced_proj {
            self.camera.projection = proj;
        }
        let scales = [
            self.axes[0].axis_scale(),
            self.axes[1].axis_scale(),
            self.axes[2].axis_scale(),
        ];
        let reversed = [
            self.axes[0].is_reversed(),
            self.axes[1].is_reversed(),
            self.axes[2].is_reversed(),
        ];
        let (mut diagnostics, present, raw_extent) =
            summarize_plot3d(layers, scales, |id| state.layer_visible(id));
        state.retain_layers(&present);
        let auto_bounds = compute_auto_bounds_3d(
            raw_extent,
            [&self.axes[0], &self.axes[1], &self.axes[2]],
            scales,
        );

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
        let layout = compute_plot3d_layout(
            outer,
            !self.title.is_empty(),
            legend_count,
            bubble_size,
            self.color_bar.is_some(),
        );

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
            self.allow_scroll_zoom,
            layout.legend_rect,
        );
        if nav.changed {
            response.mark_changed();
        }

        let transform = Transform3D::new(
            nav.bounds,
            scales,
            reversed,
            self.aspect,
            layout.plot_area,
            nav.camera,
        )
        .unwrap_or_else(|_| {
            Transform3D::new(
                Bounds3D::default(),
                [AxisScale::Linear; 3],
                reversed,
                self.aspect,
                layout.plot_area,
                nav.camera,
            )
            .expect("default 3d transform")
        });

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
                &self.title,
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
                let ticks = Axis::new()
                    .scale(scale.scale())
                    .ticks(scale.range(), bar_rect.height());
                let outer_cb = egui::Rect::from_min_max(
                    egui::pos2(bar_rect.left(), bar_rect.top() - 20.0),
                    egui::pos2(bar_rect.left() + 72.0, bar_rect.bottom() + 8.0),
                )
                .intersect(layout.outer);
                paint_color_bar(ui, outer_cb, bar_rect, *scale, &ticks, label);
            }
        }

        let legend_entries = layout.legend_rect.map_or_else(Vec::new, |rect| {
            show_plot3d_legend(ui, state, layers, rect, id.with("legend"), &style)
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
