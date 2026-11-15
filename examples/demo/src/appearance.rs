use std::sync::Arc;

use eframe::egui;
use egui_customplot::{Annotation, Plot, PlotStyle, PlotTheme, SeriesStyle};

use crate::data::Scene;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ThemeChoice {
    Host,
    #[default]
    Scientific,
    Monitoring,
}

pub(super) struct Appearance {
    pub theme: ThemeChoice,
    pub override_signal: bool,
    pub annotations: bool,
    pub custom_tooltip: bool,
    inherit: bool,
    parent_width: f32,
    override_background: bool,
    background: egui::Color32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Scientific,
            override_signal: false,
            annotations: true,
            custom_tooltip: true,
            inherit: true,
            parent_width: 2.5,
            override_background: false,
            background: egui::Color32::from_rgb(235, 240, 245),
        }
    }
}

/// Themes and labels are allocated once, not rebuilt in each UI pass.
pub(super) struct Presentation {
    light: Arc<PlotTheme>,
    dark: Arc<PlotTheme>,
    overlays: [Arc<[Annotation]>; 2],
}

impl Default for Presentation {
    fn default() -> Self {
        let band = egui::Color32::from_rgba_unmultiplied(180, 140, 20, 45);
        Self {
            light: Arc::new(PlotTheme::scientific_light()),
            dark: Arc::new(PlotTheme::monitoring_dark()),
            overlays: [
                vec![
                    Annotation::x_region([8.0, 8.7], band),
                    Annotation::y_line(0.0, "zero"),
                    Annotation::x_line(8.0, "gap starts"),
                    Annotation::text([3.0, 0.8], "signal + observations"),
                ]
                .into(),
                vec![
                    Annotation::x_region([f64::NEG_INFINITY, 0.1], band),
                    Annotation::x_line(1.0, "x = 1"),
                    Annotation::y_line(1.0, "y = 1"),
                    Annotation::text([10.0, 10.0_f64.powf(0.65)], "y = x^0.65"),
                ]
                .into(),
            ],
        }
    }
}

impl Appearance {
    pub fn configure(&self, mut plot: Plot, presentation: &Presentation, scene: Scene) -> Plot {
        plot = match self.theme {
            ThemeChoice::Host => plot,
            ThemeChoice::Scientific => plot.theme(presentation.light.clone()),
            ThemeChoice::Monitoring => plot.theme(presentation.dark.clone()),
        };
        if self.inherit {
            plot = plot.inherited_style(PlotStyle {
                series: SeriesStyle {
                    line_width: Some(self.parent_width),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
        if self.override_background {
            plot = plot.background(self.background);
        }
        if self.annotations {
            plot = plot
                .annotations(presentation.overlays[usize::from(scene == Scene::PowerLaw)].clone());
        }
        if self.custom_tooltip {
            plot = plot.hover_formatter(move |info| {
                let point = info.hovered_point.map_or(info.cursor, |p| p.point);
                let units = if scene == Scene::Signal {
                    "s / V"
                } else {
                    "x / power"
                };
                let source = if info.hovered_point.is_some() {
                    info.series_name
                } else {
                    "cursor"
                };
                format!(
                    "{source} ({units})\nx = {:.5}, y = {:.5}",
                    point[0], point[1]
                )
            });
        }
        plot
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Theme / inherited properties", |ui| {
            for (value, label) in [
                (ThemeChoice::Host, "Host egui style"),
                (ThemeChoice::Scientific, "Scientific light"),
                (ThemeChoice::Monitoring, "Monitoring dark"),
            ] {
                ui.selectable_value(&mut self.theme, value, label);
            }
            ui.checkbox(&mut self.inherit, "Parent style defaults");
            ui.add_enabled(
                self.inherit,
                egui::Slider::new(&mut self.parent_width, 0.0..=5.0).text("Parent line width"),
            );
            ui.small("Reference inherits; Signal has a local width override.");
            ui.checkbox(&mut self.override_background, "Local background override");
            if self.override_background {
                ui.color_edit_button_srgba(&mut self.background);
            }
            ui.checkbox(&mut self.override_signal, "Override Signal palette color");
            ui.checkbox(&mut self.annotations, "Text / reference lines / regions");
            ui.checkbox(&mut self.custom_tooltip, "Custom data tooltip with units");
        });
    }
}
