use crate::i18n::{lang, tr};
use eframe::egui;
use egui_customplot::{
    Axis, AxisLink, ColorMap, ColorScale, Heatmap, Line, Plot, PlotData, PlotResponse, PlotState,
    SharedCursors, SpectrogramData, StepResponseData, StepResponsePlot, TransferFunction,
};

pub(super) struct TimeMeasurements {
    waveform: PlotData,
    step: StepResponseData,
    states: [PlotState; 3],
    link: AxisLink,
    cursors: SharedCursors,
}
impl TimeMeasurements {
    pub fn new(samples: &[f64], fs: f64) -> Self {
        let waveform = PlotData::new(
            samples
                .iter()
                .enumerate()
                .map(|(i, &s)| [i as f64 / fs, s])
                .collect::<Vec<_>>(),
        );
        let plant = TransferFunction::new(&[16.0], &[1.0, 2.0, 16.0]).expect("second-order plant");
        Self {
            waveform,
            step: StepResponseData::new(&plant, 6.0, 1201).expect("stable response"),
            states: Default::default(),
            link: AxisLink::new([true, false]),
            cursors: Default::default(),
        }
    }
    pub fn reset(&mut self) {
        self.states.iter_mut().for_each(PlotState::reset);
        self.link = AxisLink::new([true, false]);
        self.cursors.clear();
    }
    pub fn show(&mut self, ui: &mut egui::Ui, spec: &SpectrogramData) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let h = ((ui.available_height() - 68.0) / 3.0).clamp(100.0, 300.0);
        let [wave_state, spec_state, step_state] = &mut self.states;
        let wave = Plot::new("measurement-wave")
            .title(tr(l, "Waveform", "时域波形", "波形"))
            .height(h)
            .x_axis(Axis::new().label("Time (s)"))
            .show(ui, wave_state, &[Line::new(&self.waveform)]);
        let scale = ColorScale::new(spec.value_range())
            .unwrap()
            .color_map(ColorMap::Inferno);
        let spectrum = Plot::new("measurement-stft")
            .title("STFT")
            .height(h)
            .color_bar(scale, "dB")
            .x_axis(Axis::new().label("Time (s)"))
            .y_axis(Axis::new().label("Hz"))
            .show_layers(ui, spec_state, &[Heatmap::new(spec.grid(), scale).into()]);
        let _ = self
            .link
            .synchronize(ui.ctx(), &mut [wave_state, spec_state]);
        let measures = self.cursors.show(ui, &[&wave, &spectrum]);
        ui.horizontal_wrapped(|ui| {
            ui.label(tr(
                l,
                "Click for A · Shift-click for B",
                "点击固定 A，Shift + 点击固定 B",
                "クリックで A、Shift + クリックで B",
            ));
            if let Some(dx) = measures.delta_x() {
                ui.label(format!("Δt = {dx:.4} s"));
            }
            if let Some(dy) = measures.delta_y() {
                ui.label(format!("Δy = {dy:.4}"));
            }
            if ui.button(tr(l, "Clear", "清除", "クリア")).clicked() {
                self.cursors.clear();
            }
        });
        let step = StepResponsePlot::new("measurement-step", &self.step)
            .height(h)
            .show(ui, step_state);
        vec![wave, spectrum, step]
    }
}
