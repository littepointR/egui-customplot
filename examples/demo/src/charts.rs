//! Plot descriptors and multi-panel composition, separate from host lifecycle.
use eframe::egui;
use egui_customplot::{
    Axis, AxisLink, AxisScale, Figure, Line, LinePattern, MarkerStyle, PlotResponse, PlotState,
    Scatter, Series, YAxis,
};

use crate::{
    appearance::Presentation, controls::Options, data::DemoData, performance::Performance,
};

#[derive(Default)]
pub(super) struct MultiViewOptions {
    pub side_by_side: bool,
    pub link_x: bool,
    pub right_y: bool,
    pub right_log: bool,
    pub right_reversed: bool,
}

impl MultiViewOptions {
    pub fn show(&mut self, ui: &mut egui::Ui, second_view: &mut bool) {
        ui.checkbox(second_view, "Second Figure cell");
        ui.add_enabled_ui(*second_view, |ui| {
            ui.checkbox(&mut self.side_by_side, "Side-by-side cells");
            ui.checkbox(&mut self.link_x, "Link X (incompatible with equal units)");
        });
        ui.checkbox(&mut self.right_y, "Secondary signal on right Y");
        ui.add_enabled_ui(self.right_y, |ui| {
            ui.checkbox(&mut self.right_log, "Right log Y");
            ui.checkbox(&mut self.right_reversed, "Right reversed Y");
            ui.small("Static scene: independent pressure signal; Live mode omits it.");
        });
    }
}

pub(super) struct Charts {
    pub main: PlotState,
    pub detail: PlotState,
    link: AxisLink,
}

impl Default for Charts {
    fn default() -> Self {
        Self {
            main: PlotState::default(),
            detail: PlotState::default(),
            link: AxisLink::new([true, false]),
        }
    }
}

impl Charts {
    pub fn reset(&mut self) {
        self.main.reset();
        self.detail.reset();
        self.link.reset();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        options: &Options,
        presentation: &Presentation,
        data: &DemoData,
        performance: &mut Performance,
    ) -> Vec<PlotResponse> {
        let series = series(options, data, &performance.live.data);
        let count: usize = if options.second_view { 2 } else { 1 };
        let columns = if options.multi.side_by_side { 2 } else { 1 };
        let rows = count.div_ceil(columns);
        let height = ((ui.available_height() - 190.0) / rows as f32).max(80.0);
        let linked = options.second_view && options.multi.link_x && !options.equal_units;
        if !linked {
            self.main.clear_axis_links();
            self.detail.clear_axis_links();
            self.link.reset();
        }
        let responses = Figure::new("demo-figure")
            .columns(columns)
            .cell_height(height)
            .show(ui, count, |cell| {
                let mut plot = options.appearance.configure(
                    options.plot("plot", height),
                    presentation,
                    options.scene,
                );
                if options.multi.right_y {
                    plot = plot.right_y_axis(
                        Axis::new()
                            .label("Pressure")
                            .scale(if options.multi.right_log {
                                AxisScale::Log10
                            } else {
                                AxisScale::Linear
                            })
                            .reversed(options.multi.right_reversed),
                    );
                }
                let state = if cell.index() == 0 {
                    &mut self.main
                } else {
                    &mut self.detail
                };
                if options.performance.cached {
                    cell.show_series_cached(plot, state, &series, &mut performance.cache)
                } else {
                    cell.show_series(plot, state, &series)
                }
            })
            .expect("finite, fixed cell count")
            .inner;
        if linked
            && let Err(error) = self
                .link
                .synchronize(ui.ctx(), &mut [&mut self.main, &mut self.detail])
        {
            ui.colored_label(egui::Color32::YELLOW, error.to_string());
        }
        responses
    }
}

fn series<'a>(
    options: &Options,
    data: &'a DemoData,
    live: &'a egui_customplot::PlotData,
) -> Vec<Series<'a>> {
    let settings = &options.performance;
    let signal = if settings.streaming {
        live
    } else {
        &data.signal
    };
    let mut line = Line::new(signal)
        .id_salt("signal")
        .name("Signal")
        .sampling(settings.sampling)
        .width(options.width);
    if options.appearance.override_signal {
        line = line.color(options.color);
    }
    if options.markers {
        line = line.markers(MarkerStyle {
            shape: options.shape,
            radius: options.radius,
        });
    }
    if options.dashed {
        line = line.pattern(LinePattern::Dashed {
            length: 8.0,
            gap: 5.0,
        });
    }
    let mut series = vec![line.into()];
    if !settings.streaming {
        series.push(
            Line::new(&data.reference)
                .id_salt("reference")
                .name("Reference")
                .sampling(settings.sampling)
                .into(),
        );
        if options.observations {
            series.push(
                Scatter::new(&data.observations)
                    .id_salt("observations")
                    .name("Observations")
                    .shape(options.shape)
                    .colors(&data.colors)
                    .expect("matching colors")
                    .radii(&data.radii)
                    .expect("matching radii")
                    .into(),
            );
        }
        if options.multi.right_y {
            series.push(
                Line::new(&data.secondary)
                    .id_salt("pressure")
                    .name("Pressure (right Y)")
                    .y_axis(YAxis::Right)
                    .sampling(settings.sampling)
                    .into(),
            );
        }
    }
    series
}
