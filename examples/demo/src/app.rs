use eframe::egui;

use crate::{
    appearance::Presentation,
    charts::Charts,
    controls::{self, Options},
    data::DemoData,
    diagnostics,
    performance::Performance,
};

pub(super) struct Demo {
    data: DemoData,
    options: Options,
    presentation: Presentation,
    performance: Performance,
    views: Charts,
    scalar: crate::scalar::ScalarDemo,
    statistical: crate::statistical::StatisticalDemo,
    export: crate::export::ExportPanel,
}

impl Default for Demo {
    fn default() -> Self {
        let options = Options::default();
        Self {
            data: DemoData::new(options.scene, options.count, options.gaps),
            options,
            presentation: Presentation::default(),
            performance: Performance::default(),
            views: Charts::default(),
            scalar: crate::scalar::ScalarDemo::default(),
            statistical: crate::statistical::StatisticalDemo::default(),
            export: crate::export::ExportPanel::default(),
        }
    }
}

impl Demo {
    pub fn with_gallery(gallery: Option<&str>) -> Self {
        let mut demo = Self::default();
        demo.scalar.enabled = gallery == Some("--scalar");
        demo.statistical.enabled = gallery == Some("--statistical");
        demo
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        self.export.flush_previous(ui.ctx());
        egui::Panel::left("controls")
            .default_size(250.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let changes = controls::show(ui, &mut self.options);
                    self.scalar.controls(ui);
                    self.statistical.controls(ui);
                    self.export.controls(ui);
                    if changes.rebuild {
                        self.data = DemoData::new(
                            self.options.scene,
                            self.options.count,
                            self.options.gaps,
                        );
                        self.performance.live = crate::stream::LiveSignal::default();
                        self.performance.cache.clear();
                    }
                    if changes.rebuild || changes.reindex {
                        self.performance
                            .index(&mut self.data, &self.options.performance);
                    }
                    self.performance.status(ui);
                    if changes.reset {
                        self.views.reset();
                    }
                });
            });
        let performance = &self.options.performance;
        self.performance
            .cache
            .set_budget(performance.cache_budget_mib * 1_048_576);
        if performance.streaming {
            self.performance.live.tick(
                ui.ctx(),
                self.options.scene,
                self.options.gaps,
                performance
                    .indexed
                    .then_some(performance.index_budget_mib * 1_048_576),
            );
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("egui-customplot / Stage 7");
            let responses = if self.export.request.is_some() {
                let rect = ui.available_rect_before_wrap();
                let mut responses = Vec::new();
                let captured =
                    egui_customplot::PlotSnapshot::capture(ui, Default::default(), |ui| {
                        responses = self.show_charts(ui);
                        rect
                    });
                self.export.store(ui.ctx(), captured);
                responses
            } else {
                self.show_charts(ui)
            };
            diagnostics::show(ui, &responses[0].diagnostics);
        });
    }

    fn show_charts(&mut self, ui: &mut egui::Ui) -> Vec<egui_customplot::PlotResponse> {
        if self.scalar.enabled {
            return vec![self.scalar.show(ui)];
        }
        if self.statistical.enabled {
            return self.statistical.show(ui);
        }
        self.views.show(
            ui,
            &self.options,
            &self.presentation,
            &self.data,
            &mut self.performance,
        )
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
