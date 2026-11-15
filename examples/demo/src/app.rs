use eframe::egui;
use egui_customplot::{Line, LinePattern, MarkerStyle, PlotState, Scatter, Series};

use crate::{
    appearance::Presentation,
    controls::{self, Options},
    data::DemoData,
    diagnostics,
};

pub(super) struct Demo {
    data: DemoData,
    options: Options,
    presentation: Presentation,
    state: PlotState,
    detail_state: PlotState,
}

impl Default for Demo {
    fn default() -> Self {
        let options = Options::default();
        Self {
            data: DemoData::new(options.scene, options.count, options.gaps),
            options,
            presentation: Presentation::default(),
            state: PlotState::default(),
            detail_state: PlotState::default(),
        }
    }
}

impl Demo {
    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("controls")
            .default_size(250.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let changes = controls::show(ui, &mut self.options);
                    if changes.rebuild {
                        self.data = DemoData::new(
                            self.options.scene,
                            self.options.count,
                            self.options.gaps,
                        );
                    }
                    if changes.reset {
                        self.state.reset();
                        self.detail_state.reset();
                    }
                });
            });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("egui-customplot / Stage 3");
            let mut line = Line::new(&self.data.signal)
                .id_salt("signal")
                .name("Signal")
                .width(self.options.width);
            if self.options.appearance.override_signal {
                line = line.color(self.options.color);
            }
            if self.options.markers {
                line = line.markers(MarkerStyle {
                    shape: self.options.shape,
                    radius: self.options.radius,
                });
            }
            if self.options.dashed {
                line = line.pattern(LinePattern::Dashed {
                    length: 8.0,
                    gap: 5.0,
                });
            }
            let mut series: Vec<Series<'_>> = vec![
                line.into(),
                Line::new(&self.data.reference)
                    .id_salt("reference")
                    .name("Reference")
                    .into(),
            ];
            if self.options.observations {
                series.push(
                    Scatter::new(&self.data.observations)
                        .id_salt("observations")
                        .name("Observations")
                        .shape(self.options.shape)
                        .colors(&self.data.colors)
                        .expect("matching colors")
                        .radii(&self.data.radii)
                        .expect("matching radii")
                        .into(),
                );
            }
            let height = ((ui.available_height() - 105.0)
                / if self.options.second_view { 2.0 } else { 1.0 })
            .max(80.0);
            let result = self
                .options
                .appearance
                .configure(
                    self.options.plot("main", height),
                    &self.presentation,
                    self.options.scene,
                )
                .show_series(ui, &mut self.state, &series);
            diagnostics::show(ui, &result.diagnostics);
            if self.options.second_view {
                self.options
                    .appearance
                    .configure(
                        self.options.plot("detail", height),
                        &self.presentation,
                        self.options.scene,
                    )
                    .show_series(ui, &mut self.detail_state, &series);
            }
        });
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{appearance::ThemeChoice, data::Scene};

    #[test]
    fn demo_scenes_embed_in_headless_egui_with_independent_views() {
        let ctx = egui::Context::default();
        let mut demo = Demo::default();
        demo.options.count = 1000;
        demo.options.second_view = true;
        demo.options.markers = true;
        demo.options.dashed = true;
        demo.options.equal_units = true;
        for scene in [Scene::Signal, Scene::PowerLaw] {
            demo.options.scene = scene;
            demo.options.log_axes = [scene == Scene::PowerLaw; 2];
            demo.data = DemoData::new(scene, 1000, true);
            for theme in [
                ThemeChoice::Host,
                ThemeChoice::Scientific,
                ThemeChoice::Monitoring,
            ] {
                demo.options.appearance.theme = theme;
                let output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1200.0, 820.0),
                        )),
                        ..Default::default()
                    },
                    |ui| demo.show(ui),
                );
                assert!(!output.shapes.is_empty());
                output.drop_without_applying_deltas();
            }
        }
        assert!(demo.state.bounds().is_none());
        assert!(demo.detail_state.bounds().is_none());
    }
}
