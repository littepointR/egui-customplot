//! Fixed host-owned discrete signals; no data construction inside plot passes.
use eframe::egui;
use egui_customplot::{
    Axis, AxisScale, Figure, Line, LinePattern, Plot, PlotData, PlotResponse, PlotState,
    StemOrientation, Stems, YAxis,
};

pub(super) struct StemDemo {
    pub enabled: bool,
    pub horizontal: bool,
    pub markers: bool,
    pub baseline_visible: bool,
    pub dashed: bool,
    pub baseline: f64,
    signal: PlotData,
    positive: PlotData,
    reference: PlotData,
    states: [PlotState; 2],
}

impl Default for StemDemo {
    fn default() -> Self {
        Self {
            enabled: false,
            horizontal: false,
            markers: true,
            baseline_visible: true,
            dashed: false,
            baseline: 0.0,
            signal: PlotData::new(vec![
                [0.5, 1.0],
                [1.0, -1.5],
                [2.0, 2.5],
                [2.0, 1.5],
                [f64::NAN; 2],
                [4.0, 0.0],
                [7.0, -0.8],
                [9.0, 2.0],
            ]),
            positive: PlotData::new(vec![
                [0.5, 10.0],
                [1.0, 40.0],
                [2.0, 100.0],
                [4.0, 25.0],
                [9.0, 60.0],
            ]),
            reference: PlotData::new(vec![[0.5, -1.0], [9.0, 1.0]]),
            states: Default::default(),
        }
    }
}

impl StemDemo {
    #[cfg(test)]
    pub fn data_versions(&self) -> [u64; 3] {
        [
            self.signal.version(),
            self.positive.version(),
            self.reference.version(),
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Discrete stems / baseline / markers");
        ui.add_enabled_ui(self.enabled, |ui| {
            if ui.checkbox(&mut self.horizontal, "Horizontal top-panel rods (raw XY unchanged)").changed() {
                self.states[0].reset();
            }
            ui.checkbox(&mut self.markers, "Raw endpoint markers");
            ui.checkbox(&mut self.baseline_visible, "Show baseline");
            ui.checkbox(&mut self.dashed, "Dashed rods (baseline stays solid)");
            ui.add(egui::Slider::new(&mut self.baseline, -2.0..=2.0).text("Top baseline"));
            if ui.button("Fit stem panels").clicked() { self.states.iter_mut().for_each(PlotState::reset); }
            ui.small("Independent rods; NaN omits only its record. Hover raw endpoints, not rods or baseline. Rendering and markers scan all points; no stem LOD/cache.");
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 110.0) * 0.5).max(80.0);
        let pattern = if self.dashed {
            LinePattern::Dashed {
                length: 8.0,
                gap: 4.0,
            }
        } else {
            LinePattern::Solid
        };
        Figure::new("stem-panels")
            .cell_height(height)
            .show(ui, 2, |cell| {
                let index = cell.index();
                let plot = Plot::new("plot")
                    .height(height)
                    .allow_scroll_zoom(true)
                    .legend(true);
                if index == 0 {
                    cell.show_layers(
                        plot.title("Independent signed stems"),
                        &mut self.states[0],
                        &[Stems::new(&self.signal)
                            .name("Discrete signal")
                            .baseline(self.baseline)
                            .orientation(if self.horizontal {
                                StemOrientation::Horizontal
                            } else {
                                StemOrientation::Vertical
                            })
                            .show_markers(self.markers)
                            .show_baseline(self.baseline_visible)
                            .pattern(pattern)
                            .into()],
                    )
                } else {
                    cell.show_layers(
                        plot.title("Raw endpoints on a right logarithmic Y axis")
                            .right_y_axis(
                                Axis::new().scale(AxisScale::Log10).label("Amplitude (log)"),
                            ),
                        &mut self.states[1],
                        &[
                            Line::new(&self.reference).name("Left reference").into(),
                            Stems::new(&self.positive)
                                .name("Right discrete signal")
                                .y_axis(YAxis::Right)
                                .baseline(1.0)
                                .color(egui::Color32::LIGHT_RED)
                                .show_markers(self.markers)
                                .show_baseline(self.baseline_visible)
                                .pattern(pattern)
                                .into(),
                        ],
                    )
                }
            })
            .unwrap()
            .inner
    }
}
