use crate::{Annotation, Axis, Line, Plot, PlotResponse, PlotState, StepResponseData};

/// Unit-step chart with final value, observed overshoot and sampled 2% settling time.
pub struct StepResponsePlot<'a> {
    plot: Plot,
    curve: Line<'a>,
}

impl<'a> StepResponsePlot<'a> {
    pub fn new(id: impl egui::AsIdSalt, data: &'a StepResponseData) -> Self {
        let mut markers = vec![Annotation::y_line(
            data.steady_state(),
            format!("Final {:.3}", data.steady_state()),
        )];
        if let Some(t) = data.settling_time() {
            markers.push(Annotation::x_line(t, format!("2%: {t:.3} s")));
        }
        let title = data.overshoot_percent().map_or_else(
            || "Unit step response".into(),
            |v| format!("Unit step response · overshoot {v:.1}%"),
        );
        Self {
            plot: Plot::new(id)
                .title(title)
                .x_axis(Axis::new().label("Time (s)"))
                .y_axis(Axis::new().label("Output"))
                .annotations(markers),
            curve: Line::new(data.curve()).name("Unit step"),
        }
    }
    /// Configure the underlying plot, including axes, title, annotations and legend.
    pub fn configure_plot(mut self, configure: impl FnOnce(Plot) -> Plot) -> Self {
        self.plot = configure(self.plot);
        self
    }
    /// Configure the named step-response curve.
    pub fn configure_curve(mut self, configure: impl FnOnce(Line<'a>) -> Line<'a>) -> Self {
        self.curve = configure(self.curve);
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.plot = self.plot.height(height);
        self
    }
    pub fn show(self, ui: &mut egui::Ui, state: &mut PlotState) -> PlotResponse {
        self.plot.show(ui, state, &[self.curve])
    }
}
