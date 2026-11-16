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
    stems: crate::stems::StemDemo,
    quiver: crate::quiver::QuiverDemo,
    polygons: crate::polygons::PolygonDemo,
    contours: crate::contours::ContourDemo,
    functions: crate::functions::FunctionsDemo,
    geometry: crate::geometry::GeometryDemo,
    distributions: crate::distributions::DistributionsDemo,
    multivariate: crate::multivariate::MultivariateDemo,
    polar: crate::polar::PolarDemo,
    networks: crate::networks::NetworksDemo,
    plot3d: crate::plot3d::Plot3DDemo,
    surfaces3d: crate::surfaces3d::Surfaces3DDemo,
    volumes3d: crate::volumes3d::Volumes3DDemo,
    geo: crate::geo::GeoDemo,
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
            stems: crate::stems::StemDemo::default(),
            quiver: crate::quiver::QuiverDemo::default(),
            polygons: crate::polygons::PolygonDemo::default(),
            contours: crate::contours::ContourDemo::default(),
            functions: crate::functions::FunctionsDemo::default(),
            geometry: crate::geometry::GeometryDemo::default(),
            distributions: crate::distributions::DistributionsDemo::default(),
            multivariate: crate::multivariate::MultivariateDemo::default(),
            polar: crate::polar::PolarDemo::default(),
            networks: crate::networks::NetworksDemo::default(),
            plot3d: crate::plot3d::Plot3DDemo::default(),
            surfaces3d: crate::surfaces3d::Surfaces3DDemo::default(),
            volumes3d: crate::volumes3d::Volumes3DDemo::default(),
            geo: crate::geo::GeoDemo::default(),
            export: crate::export::ExportPanel::default(),
        }
    }
}

impl Demo {
    pub fn with_gallery(gallery: Option<&str>) -> Self {
        let mut demo = Self::default();
        demo.scalar.enabled = gallery == Some("--scalar");
        demo.statistical.enabled = gallery == Some("--statistical");
        demo.stems.enabled = gallery == Some("--stems");
        demo.quiver.enabled = gallery == Some("--quiver");
        demo.polygons.enabled = gallery == Some("--polygons");
        demo.contours.enabled = gallery == Some("--contours");
        demo.functions.enabled = gallery == Some("--functions");
        demo.geometry.enabled = gallery == Some("--geometry");
        demo.distributions.enabled = gallery == Some("--distributions");
        demo.multivariate.enabled = gallery == Some("--multivariate");
        demo.polar.enabled = gallery == Some("--polar");
        demo.networks.enabled = gallery == Some("--networks");
        demo.plot3d.enabled = gallery == Some("--plot3d");
        demo.surfaces3d.enabled = gallery == Some("--surfaces3d");
        demo.volumes3d.enabled = gallery == Some("--volumes3d");
        demo.geo.enabled = gallery == Some("--geo");
        if gallery == Some("--stairs") {
            demo.options.interpolation = egui_customplot::LineInterpolation::StepAfter;
            demo.options.count = 8;
            demo.options.markers = true;
            demo.options.observations = false;
            demo.data = DemoData::stairs();
        }
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
                    self.stems.controls(ui);
                    self.quiver.controls(ui);
                    self.polygons.controls(ui);
                    self.contours.controls(ui);
                    self.functions.controls(ui);
                    self.geometry.controls(ui);
                    self.distributions.controls(ui);
                    self.multivariate.controls(ui);
                    self.polar.controls(ui);
                    self.networks.controls(ui);
                    self.plot3d.controls(ui);
                    self.surfaces3d.controls(ui);
                    self.volumes3d.controls(ui);
                    self.geo.controls(ui);
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
            ui.heading("egui-customplot / Stage 11: Full 133-Item Readiness");
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
        if self.stems.enabled {
            return self.stems.show(ui);
        }
        if self.quiver.enabled {
            return self.quiver.show(ui);
        }
        if self.polygons.enabled {
            return self.polygons.show(ui);
        }
        if self.contours.enabled {
            return self.contours.show(ui);
        }
        if self.functions.enabled {
            return self.functions.show(ui);
        }
        if self.geometry.enabled {
            return self.geometry.show(ui);
        }
        if self.distributions.enabled {
            return self.distributions.show(ui);
        }
        if self.multivariate.enabled {
            return self.multivariate.show(ui);
        }
        if self.polar.enabled {
            return self.polar.show(ui);
        }
        if self.networks.enabled {
            return self.networks.show(ui);
        }
        if self.plot3d.enabled {
            return self.plot3d.show(ui);
        }
        if self.surfaces3d.enabled {
            return self.surfaces3d.show(ui);
        }
        if self.volumes3d.enabled {
            return self.volumes3d.show(ui);
        }
        if self.geo.enabled {
            return self.geo.show(ui);
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
