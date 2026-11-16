use std::{collections::HashSet, f64::consts::TAU, sync::Arc};

use egui::{AsIdSalt, IdSalt, Response, Sense, Ui};

use crate::{
    AxisScale, BubbleLegend, HoveredPolarBin, HoveredPolarPoint, LegendEntry, NegativeRadiusMode,
    PlotBounds, PlotDiagnostics, PlotResponse, PlotStyle, PlotTheme, PlotTransform, PolarLayer,
    PolarRegionAnnotation, PolarState, PolarTransform, ThetaDirection, ThetaZeroLocation,
    interaction::{navigate_polar, pick_polar_hover, show_polar_legend},
    layout::{allocate_plot, bubble_legend_size, compute_polar_layout, paint_bubble_legend},
    model::resolve_plot_style,
    render::paint_polar,
};

/// Polar coordinate chart widget (`polarplot`, `polarscatter`, `polarbubblechart`, `polarhistogram`, `compassplot`).
pub struct PolarPlot {
    id_salt: IdSalt,
    height: f32,
    title: String,
    theta_zero: ThetaZeroLocation,
    theta_dir: ThetaDirection,
    theta_lim: [f64; 2],
    neg_mode: NegativeRadiusMode,
    lock_r_zero: bool,
    regions: Arc<[PolarRegionAnnotation]>,
    bubble_legend: Option<BubbleLegend>,
    show_legend: bool,
    show_grid: bool,
    show_hover: bool,
    pick_radius: f32,
    pub(crate) inherited_style: PlotStyle,
    style: PlotStyle,
    pub(crate) theme: Option<Arc<PlotTheme>>,
}

pub struct PolarPlotResponse {
    pub outer_rect: egui::Rect,
    pub plot_area: egui::Rect,
    pub response: Response,
    pub transform: PolarTransform,
    pub diagnostics: PlotDiagnostics,
    pub view_changed: bool,
    pub hovered_point: Option<HoveredPolarPoint>,
    pub hovered_bin: Option<HoveredPolarBin>,
    pub hover_text: Option<String>,
    pub legend_entries: Vec<LegendEntry>,
}

impl PolarPlotResponse {
    pub fn into_plot_response(self) -> PlotResponse {
        let bounds = PlotBounds::new([0.0, 0.0], [1.0, 1.0]).expect("unit bounds");
        let transform =
            PlotTransform::with_axes(bounds, self.plot_area, [AxisScale::Linear; 2], [false; 2])
                .expect("valid polar plot area");
        PlotResponse {
            outer_rect: self.outer_rect,
            response: self.response,
            transform,
            right_transform: None,
            diagnostics: self.diagnostics,
            view_changed: self.view_changed,
            axis_ticks: [Vec::new(), Vec::new()],
            right_axis_ticks: Vec::new(),
            hovered_point: None,
            hovered_cell: None,
            hovered_bar: None,
            hovered_region: None,
            hovered_vector: None,
            hover_text: self.hover_text,
            legend_entries: self.legend_entries,
        }
    }
}

impl PolarPlot {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            height: 280.0,
            title: String::new(),
            theta_zero: ThetaZeroLocation::Right,
            theta_dir: ThetaDirection::CounterClockwise,
            theta_lim: [0.0, TAU],
            neg_mode: NegativeRadiusMode::ShiftMinRadius,
            lock_r_zero: true,
            regions: Arc::from([]),
            bubble_legend: None,
            show_legend: true,
            show_grid: true,
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
    pub fn theta_zero(mut self, zero: ThetaZeroLocation) -> Self {
        self.theta_zero = zero;
        self
    }
    pub fn theta_direction(mut self, dir: ThetaDirection) -> Self {
        self.theta_dir = dir;
        self
    }
    pub fn theta_lim(mut self, lim_rad: [f64; 2]) -> Self {
        if lim_rad[0].is_finite() && lim_rad[1].is_finite() && lim_rad[1] > lim_rad[0] {
            self.theta_lim = lim_rad;
        }
        self
    }
    pub fn negative_radius_mode(mut self, mode: NegativeRadiusMode) -> Self {
        self.neg_mode = mode;
        self
    }
    pub fn lock_r_zero(mut self, lock: bool) -> Self {
        self.lock_r_zero = lock;
        self
    }
    pub fn regions(mut self, regions: impl Into<Arc<[PolarRegionAnnotation]>>) -> Self {
        self.regions = regions.into();
        self
    }
    pub fn bubble_legend(mut self, legend: BubbleLegend) -> Self {
        self.bubble_legend = Some(legend);
        self
    }
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
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
        self,
        ui: &mut Ui,
        state: &mut PolarState,
        layers: &[PolarLayer<'_>],
    ) -> PolarPlotResponse {
        let outer = allocate_plot(ui, self.height);
        let style = resolve_plot_style(ui, self.theme.as_deref(), self.inherited_style, self.style);
        let (mut diagnostics, present, auto_r_lim) =
            summarize_polar(layers, state, self.neg_mode, self.lock_r_zero);
        state.retain_layers(&present);

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
        let layout = compute_polar_layout(outer, !self.title.is_empty(), legend_count, bubble_size);

        let id = ui.make_persistent_id(self.id_salt);
        state.begin_frame(ui, id);
        let mut response = ui.interact(layout.plot_area, id, Sense::click_and_drag());
        let nav = navigate_polar(
            ui,
            state,
            &mut response,
            auto_r_lim,
            self.lock_r_zero,
            layout.legend_rect,
            layout.pixel_radius,
        );
        if nav.changed {
            response.mark_changed();
        }
        let transform = PolarTransform::new(
            layout.center,
            layout.pixel_radius,
            nav.r_lim,
            self.theta_zero,
            self.theta_dir,
            self.theta_lim,
            nav.theta_offset,
        )
        .expect("validated polar transform");

        diagnostics.painting_skipped = !ui.is_rect_visible(layout.outer);
        if !diagnostics.painting_skipped {
            let painter = ui.painter_at(layout.outer);
            paint_polar(
                &painter,
                state,
                layers,
                &self.regions,
                transform,
                self.neg_mode,
                &self.title,
                self.show_grid,
                ui.ctx().pixels_per_point(),
                &style,
                &mut diagnostics.polar,
            );
            if let (Some(area), Some(legend), Some(samples)) = (
                layout.bubble_rect,
                &self.bubble_legend,
                bubble_samples.as_ref(),
            ) {
                paint_bubble_legend(ui, area, legend, samples, style.text_color);
            }
        }

        let legend_entries = layout.legend_rect.map_or_else(Vec::new, |rect| {
            show_polar_legend(ui, state, layers, rect, id.with("legend"), &style)
        });

        let (hovered_point, hovered_bin, hover_text) = if self.show_hover
            && ui.is_enabled()
            && !diagnostics.painting_skipped
            && let Some(pointer) = nav.hover_pos
        {
            diagnostics.polar.pick_queries += 1;
            let (pt, bin, text) = pick_polar_hover(
                state,
                layers,
                transform,
                self.neg_mode,
                pointer,
                self.pick_radius,
            );
            if let Some(ref msg) = text {
                response.clone().on_hover_ui_at_pointer(|ui| {
                    ui.monospace(msg);
                });
            }
            (pt, bin, text)
        } else {
            (None, None, None)
        };

        PolarPlotResponse {
            outer_rect: layout.outer,
            plot_area: layout.plot_area,
            response,
            transform,
            diagnostics,
            view_changed: nav.changed,
            hovered_point,
            hovered_bin,
            hover_text,
            legend_entries,
        }
    }
}

fn summarize_polar(
    layers: &[PolarLayer<'_>],
    state: &PolarState,
    neg_mode: NegativeRadiusMode,
    lock_r_zero: bool,
) -> (PlotDiagnostics, HashSet<IdSalt>, [f64; 2]) {
    let mut diag = PlotDiagnostics::default();
    let mut present = HashSet::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut range: Option<[f64; 2]> = None;

    for (idx, layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        present.insert(id);
        let (alloc, pts, non_fin) = match *layer {
            PolarLayer::Line(l) => (
                l.data.allocation(),
                l.data.len(),
                l.data.non_finite_points(),
            ),
            PolarLayer::Scatter(s) => (
                s.data.allocation(),
                s.data.len(),
                s.data.non_finite_points(),
            ),
            PolarLayer::Bubble(b) => (
                b.data.allocation(),
                b.data.points().len(),
                b.data.non_finite_points(),
            ),
            PolarLayer::Histogram(h) => {
                (h.data.allocation(), h.data.bin_count(), h.data.non_finite())
            }
            PolarLayer::Compass(c) => (c.data.allocation(), c.data.len(), c.data.non_finite()),
        };
        diag.polar.input_points += pts;
        diag.polar.non_finite_points += non_fin;
        if seen.insert(alloc) {
            diag.polar.unique_bytes += alloc.1;
        }
        if state.layer_visible(id) {
            diag.visible_series += 1;
            if let Some([lo, hi]) = layer.r_extent(neg_mode) {
                let r = range.get_or_insert([lo, hi]);
                r[0] = r[0].min(lo);
                r[1] = r[1].max(hi);
            }
        }
    }
    let [mut r0, mut r1] = range.unwrap_or([0.0, 1.0]);
    if lock_r_zero && r0 >= 0.0 {
        r0 = 0.0;
    }
    if (r1 - r0).abs() <= 1e-12 {
        r1 = r0 + 1.0;
    }
    (diag, present, [r0, r1])
}
