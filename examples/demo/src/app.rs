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
    pub(super) lang: crate::i18n::Language,
    pub(super) gallery: crate::gallery::ActiveGallery,
    pub(super) global_controls: crate::global_controls::GlobalPlotControls,
    os_theme: Option<crate::os_theme::OsThemeSync>,
    prev_plot3d_perspective: bool,
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
    stage13: crate::stage13::Stage13Demo,
    stage14: crate::stage14::Stage14Demo,
    stage15: crate::stage15::Stage15Demo,
    stage16: crate::stage16::Stage16Demo,
    dynamic10m: crate::dynamic10m::Dynamic10mDemo,
    custom_axes: crate::custom_axes::CustomAxesDemo,
    custom_annotations: crate::custom_annotations::CustomAnnotationsDemo,
    custom_animation: crate::custom_animation::CustomAnimationDemo,
    custom_styling: crate::custom_styling::CustomStylingDemo,
    export: crate::export::ExportPanel,
}

impl Default for Demo {
    fn default() -> Self {
        let options = Options::default();
        let plot3d = crate::plot3d::Plot3DDemo::default();
        let global_controls = crate::global_controls::GlobalPlotControls {
            perspective_3d: plot3d.perspective,
            ..Default::default()
        };
        Self {
            lang: crate::i18n::Language::default(),
            gallery: crate::gallery::ActiveGallery::default(),
            global_controls,
            os_theme: None,
            prev_plot3d_perspective: plot3d.perspective,
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
            plot3d,
            surfaces3d: crate::surfaces3d::Surfaces3DDemo::default(),
            volumes3d: crate::volumes3d::Volumes3DDemo::default(),
            geo: crate::geo::GeoDemo::default(),
            stage13: crate::stage13::Stage13Demo::default(),
            stage14: crate::stage14::Stage14Demo::default(),
            stage15: crate::stage15::Stage15Demo::default(),
            stage16: crate::stage16::Stage16Demo::default(),
            dynamic10m: crate::dynamic10m::Dynamic10mDemo::default(),
            custom_axes: crate::custom_axes::CustomAxesDemo::default(),
            custom_annotations: crate::custom_annotations::CustomAnnotationsDemo::default(),
            custom_animation: crate::custom_animation::CustomAnimationDemo::default(),
            custom_styling: crate::custom_styling::CustomStylingDemo::default(),
            export: crate::export::ExportPanel::default(),
        }
    }
}

impl Demo {
    pub(super) fn select_gallery(&mut self, next: crate::gallery::ActiveGallery) {
        if self.gallery == next {
            return;
        }
        let was_stairs = self.gallery == crate::gallery::ActiveGallery::Stairs;
        self.gallery = next;
        match next {
            crate::gallery::ActiveGallery::Stairs => {
                self.options.interpolation = egui_customplot::LineInterpolation::StepAfter;
                self.options.count = 8;
                self.options.markers = true;
                self.options.observations = false;
                self.data = DemoData::stairs();
                self.views.reset();
            }
            crate::gallery::ActiveGallery::Default2D if was_stairs => {
                self.options.interpolation = egui_customplot::LineInterpolation::Linear;
                self.options.count = 10_000;
                self.options.markers = false;
                self.options.observations = true;
                self.data =
                    DemoData::new(self.options.scene, self.options.count, self.options.gaps);
                self.performance
                    .index(&mut self.data, &self.options.performance);
                self.views.reset();
            }
            crate::gallery::ActiveGallery::Dynamic10m => {
                self.dynamic10m.reset_view();
            }
            _ => {}
        }
    }

    pub fn with_gallery(gallery: Option<&str>) -> Self {
        let mut demo = Self::default();
        demo.select_gallery(crate::gallery::ActiveGallery::from_cli(gallery));
        demo
    }

    pub fn set_language(&mut self, lang: crate::i18n::Language) {
        self.lang = lang;
    }

    pub fn install_os_theme_sync(&mut self, ctx: &egui::Context) {
        self.os_theme = Some(crate::os_theme::OsThemeSync::install(ctx));
    }

    pub fn set_wgpu_info(&mut self, registered: bool, adapter_info: String) {
        self.dynamic10m.set_wgpu_info(registered, adapter_info);
    }

    pub fn configure_window_bench(&mut self, csv_path: std::path::PathBuf) {
        self.select_gallery(crate::gallery::ActiveGallery::Dynamic10m);
        self.dynamic10m.configure_window_bench(csv_path);
    }

    fn sync_plot_defaults(&mut self, ctx: &egui::Context) {
        if self.plot3d.perspective != self.prev_plot3d_perspective {
            self.global_controls.perspective_3d = self.plot3d.perspective;
            self.prev_plot3d_perspective = self.plot3d.perspective;
        } else if self.global_controls.perspective_3d != self.prev_plot3d_perspective {
            self.plot3d.perspective = self.global_controls.perspective_3d;
            self.prev_plot3d_perspective = self.global_controls.perspective_3d;
        }
        self.global_controls
            .sync_defaults(ctx, &self.options.appearance, &self.presentation);
    }

    pub(super) fn reset_active_gallery(&mut self) {
        use crate::gallery::ActiveGallery;
        match self.gallery {
            ActiveGallery::Default2D | ActiveGallery::Stairs => self.views.reset(),
            ActiveGallery::Scalar => self.scalar.reset_views(),
            ActiveGallery::Statistical => self.statistical.reset_views(),
            ActiveGallery::Stems => self.stems.reset_views(),
            ActiveGallery::Quiver => self.quiver.reset_views(),
            ActiveGallery::Polygons => self.polygons.reset_views(),
            ActiveGallery::Contours => self.contours.reset_views(),
            ActiveGallery::Functions => self.functions.reset_views(),
            ActiveGallery::Geometry => self.geometry.reset_views(),
            ActiveGallery::Distributions => self.distributions.reset_views(),
            ActiveGallery::Multivariate => self.multivariate.reset_views(),
            ActiveGallery::Polar => self.polar.reset_views(),
            ActiveGallery::Networks => self.networks.reset_views(),
            ActiveGallery::Plot3D => self.plot3d.reset_views(),
            ActiveGallery::Surfaces3D => self.surfaces3d.reset_views(),
            ActiveGallery::Volumes3D => self.volumes3d.reset_views(),
            ActiveGallery::Geo => self.geo.reset_views(),
            ActiveGallery::Stage13 => self.stage13.reset_views(),
            ActiveGallery::Stage14 => self.stage14.reset_views(),
            ActiveGallery::Stage15 => self.stage15.reset_views(),
            ActiveGallery::Stage16 => self.stage16.reset_views(),
            ActiveGallery::Dynamic10m => self.dynamic10m.reset_view(),
            ActiveGallery::CustomAxes => self.custom_axes.reset_views(),
            ActiveGallery::CustomAnnotations => self.custom_annotations.reset_views(),
            ActiveGallery::CustomAnimation => self.custom_animation.reset_views(),
            ActiveGallery::CustomStyling => self.custom_styling.reset_views(),
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        use crate::gallery::ActiveGallery;
        if let Some(sync) = &mut self.os_theme {
            sync.sync_ui(ui);
        }
        crate::i18n::sync_context(ui.ctx(), self.lang);
        self.export.flush_previous(ui.ctx());
        egui::Panel::left("controls")
            .default_size(258.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    crate::i18n::language_selector(ui, &mut self.lang);
                    crate::i18n::sync_context(ui.ctx(), self.lang);
                    crate::i18n::theme_preference_selector(ui, self.lang);
                    ui.separator();
                    let mut selected = self.gallery;
                    if crate::gallery::show_selector(ui, &mut selected, self.lang) {
                        self.select_gallery(selected);
                    }
                    ui.separator();
                    match self.gallery {
                        ActiveGallery::Default2D | ActiveGallery::Stairs => {
                            let changes = controls::show(ui, &mut self.options);
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
                        }
                        ActiveGallery::Scalar => self.scalar.controls(ui),
                        ActiveGallery::Statistical => self.statistical.controls(ui),
                        ActiveGallery::Stems => self.stems.controls(ui),
                        ActiveGallery::Quiver => self.quiver.controls(ui),
                        ActiveGallery::Polygons => self.polygons.controls(ui),
                        ActiveGallery::Contours => self.contours.controls(ui),
                        ActiveGallery::Functions => self.functions.controls(ui),
                        ActiveGallery::Geometry => self.geometry.controls(ui),
                        ActiveGallery::Distributions => self.distributions.controls(ui),
                        ActiveGallery::Multivariate => self.multivariate.controls(ui),
                        ActiveGallery::Polar => self.polar.controls(ui),
                        ActiveGallery::Networks => self.networks.controls(ui),
                        ActiveGallery::Plot3D => self.plot3d.controls(ui),
                        ActiveGallery::Surfaces3D => self.surfaces3d.controls(ui),
                        ActiveGallery::Volumes3D => self.volumes3d.controls(ui),
                        ActiveGallery::Geo => self.geo.controls(ui),
                        ActiveGallery::Stage13 => self.stage13.controls(ui),
                        ActiveGallery::Stage14 => self.stage14.controls(ui),
                        ActiveGallery::Stage15 => self.stage15.controls(ui),
                        ActiveGallery::Stage16 => self.stage16.controls(ui),
                        ActiveGallery::Dynamic10m => self.dynamic10m.controls(ui),
                        ActiveGallery::CustomAxes => self.custom_axes.controls(ui),
                        ActiveGallery::CustomAnnotations => self.custom_annotations.controls(ui),
                        ActiveGallery::CustomAnimation => self.custom_animation.controls(ui),
                        ActiveGallery::CustomStyling => self.custom_styling.controls(ui),
                    }
                });
            });
        if let Some(sync) = &self.os_theme {
            crate::os_theme::sync_pass_theme(ui, sync.current_theme(), false);
        }
        self.sync_plot_defaults(ui.ctx());
        if self.global_controls.open {
            let mut reset_clicked = false;
            egui::Panel::right("global_plot_controls")
                .default_size(244.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        reset_clicked = self.global_controls.show_panel(
                            ui,
                            &mut self.options.appearance,
                            &mut self.export,
                        );
                    });
                });
            if reset_clicked {
                self.reset_active_gallery();
            }
            self.sync_plot_defaults(ui.ctx());
        }
        let performance = &self.options.performance;
        self.performance
            .cache
            .set_budget(performance.cache_budget_mib * 1_048_576);
        if matches!(
            self.gallery,
            ActiveGallery::Default2D | ActiveGallery::Stairs
        ) && performance.streaming
        {
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
            let l = crate::i18n::lang(ui.ctx());
            ui.horizontal(|ui| {
                ui.heading(format!("egui-customplot — {}", self.gallery.title(l)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let toggle_lbl =
                        crate::i18n::tr(l, "Plot Controls", "绘图控制面板", "プロット制御");
                    if ui
                        .selectable_label(self.global_controls.open, toggle_lbl)
                        .clicked()
                    {
                        self.global_controls.open = !self.global_controls.open;
                    }
                });
            });
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
        use crate::gallery::ActiveGallery;
        self.sync_plot_defaults(ui.ctx());
        match self.gallery {
            ActiveGallery::Default2D | ActiveGallery::Stairs => self.views.show(
                ui,
                &self.options,
                &self.presentation,
                &self.data,
                &mut self.performance,
            ),
            ActiveGallery::Scalar => vec![self.scalar.show(ui)],
            ActiveGallery::Statistical => self.statistical.show(ui),
            ActiveGallery::Stems => self.stems.show(ui),
            ActiveGallery::Quiver => self.quiver.show(ui),
            ActiveGallery::Polygons => self.polygons.show(ui),
            ActiveGallery::Contours => self.contours.show(ui),
            ActiveGallery::Functions => self.functions.show(ui),
            ActiveGallery::Geometry => self.geometry.show(ui),
            ActiveGallery::Distributions => self.distributions.show(ui),
            ActiveGallery::Multivariate => self.multivariate.show(ui),
            ActiveGallery::Polar => self.polar.show(ui),
            ActiveGallery::Networks => self.networks.show(ui),
            ActiveGallery::Plot3D => self.plot3d.show(ui),
            ActiveGallery::Surfaces3D => self.surfaces3d.show(ui),
            ActiveGallery::Volumes3D => self.volumes3d.show(ui),
            ActiveGallery::Geo => self.geo.show(ui),
            ActiveGallery::Stage13 => self.stage13.show(ui),
            ActiveGallery::Stage14 => self.stage14.show(ui),
            ActiveGallery::Stage15 => self.stage15.show(ui),
            ActiveGallery::Stage16 => self.stage16.show(ui),
            ActiveGallery::Dynamic10m => self.dynamic10m.show(ui),
            ActiveGallery::CustomAxes => self.custom_axes.show(ui),
            ActiveGallery::CustomAnnotations => self.custom_annotations.show(ui),
            ActiveGallery::CustomAnimation => self.custom_animation.show(ui),
            ActiveGallery::CustomStyling => self.custom_styling.show(ui),
        }
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
