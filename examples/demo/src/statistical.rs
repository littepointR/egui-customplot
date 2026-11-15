//! Host-owned preprocessing and a native two-panel statistical gallery.
use eframe::egui;
use egui_customplot::{
    Axis, Bar, BarData, BarOrientation, Bars, ErrorBar, ErrorBarData, ErrorBars, Figure,
    HistogramData, HistogramNormalization, Plot, PlotResponse, PlotState,
};

pub(super) struct StatisticalDemo {
    pub enabled: bool,
    pub horizontal: bool,
    pub normalization: HistogramNormalization,
    pub cap_width: f32,
    histogram: HistogramData,
    histogram_bars: BarData,
    built_normalization: HistogramNormalization,
    bars: BarData,
    errors: [ErrorBarData; 2],
    states: [PlotState; 2],
}

impl Default for StatisticalDemo {
    fn default() -> Self {
        let mut seed = 42u64;
        let samples = (0..4000)
            .map(|_| {
                (0..6)
                    .map(|_| {
                        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                        (seed >> 32) as f64 / u32::MAX as f64
                    })
                    .sum::<f64>()
                    - 3.0
            })
            .collect::<Vec<_>>();
        let histogram = HistogramData::new(
            &samples,
            (0..=24).map(|i| -3.0 + i as f64 * 0.25).collect::<Vec<_>>(),
        )
        .unwrap();
        let normalization = HistogramNormalization::Count;
        let records = [(1.0, 2.0), (2.0, -1.0), (3.0, 3.5), (4.0, 1.5)];
        let errors = [false, true].map(|horizontal| {
            ErrorBarData::new(
                records
                    .map(|(x, y)| {
                        if horizontal {
                            ErrorBar::new([y, x]).x_error([0.4, 0.7])
                        } else {
                            ErrorBar::new([x, y]).y_error([0.4, 0.7])
                        }
                    })
                    .to_vec(),
            )
        });
        Self {
            enabled: false,
            horizontal: false,
            normalization,
            cap_width: 10.0,
            histogram_bars: histogram.to_bars(normalization).unwrap(),
            histogram,
            built_normalization: normalization,
            bars: BarData::new(records.map(|(x, y)| Bar::new(x, y).width(0.65)).to_vec()),
            errors,
            states: Default::default(),
        }
    }
}

impl StatisticalDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Bars / errors / explicit-bin histogram");
        ui.add_enabled_ui(self.enabled, |ui| {
            if ui.checkbox(&mut self.horizontal, "Horizontal bars").changed() {
                self.states[1].reset();
            }
            ui.add(egui::Slider::new(&mut self.cap_width, 0.0..=24.0).text("Error cap (points)"));
            egui::ComboBox::from_id_salt("histogram-normalization").selected_text(format!("{:?}", self.normalization))
                .show_ui(ui, |ui| {
                    for norm in [HistogramNormalization::Count, HistogramNormalization::Probability,
                        HistogramNormalization::Density, HistogramNormalization::CumulativeCount,
                        HistogramNormalization::CumulativeProbability] {
                        ui.selectable_value(&mut self.normalization, norm, format!("{norm:?}"));
                    }
                });
            if ui.button("Fit statistical panels").clicked() { self.states.iter_mut().for_each(PlotState::reset); }
            ui.small("4000 fixed observations; explicit 0.25 bins. Normalization uses in-range finite samples. Whiskers are asymmetric offsets, not inferred confidence intervals.");
        });
        if self.built_normalization != self.normalization {
            self.histogram_bars = self.histogram.to_bars(self.normalization).unwrap();
            self.built_normalization = self.normalization;
            self.states[0].reset();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 110.0) * 0.5).max(80.0);
        Figure::new("statistical-panels")
            .cell_height(height)
            .show(ui, 2, |cell| {
                let index = cell.index();
                let plot = Plot::new("plot").allow_scroll_zoom(true).height(height);
                if index == 0 {
                    cell.show_layers(
                        plot.title("Explicit-bin histogram")
                            .x_axis(Axis::new().label("Observation"))
                            .y_axis(Axis::new().label(format!("{:?}", self.normalization))),
                        &mut self.states[index],
                        &[Bars::new(&self.histogram_bars).name("Histogram").into()],
                    )
                } else {
                    let orientation = if self.horizontal {
                        BarOrientation::Horizontal
                    } else {
                        BarOrientation::Vertical
                    };
                    cell.show_layers(
                        plot.title("Signed bars + asymmetric errors"),
                        &mut self.states[index],
                        &[
                            Bars::new(&self.bars)
                                .orientation(orientation)
                                .name("Measurement")
                                .into(),
                            ErrorBars::new(&self.errors[usize::from(self.horizontal)])
                                .color(egui::Color32::LIGHT_RED)
                                .cap_width(self.cap_width)
                                .name("Error offsets")
                                .into(),
                        ],
                    )
                }
            })
            .unwrap()
            .inner
    }
}
