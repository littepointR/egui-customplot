use std::sync::Arc;

use eframe::egui;
use egui_customplot::{Annotation, Plot, PlotStyle, PlotTheme, SeriesStyle};

use crate::{
    data::Scene,
    i18n::{Language, lang, tr},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ThemeChoice {
    #[default]
    Host,
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
            theme: ThemeChoice::Host,
            override_signal: false,
            annotations: true,
            custom_tooltip: true,
            inherit: false,
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
    overlays: [[Arc<[Annotation]>; 2]; 3],
}

impl Default for Presentation {
    fn default() -> Self {
        let band = egui::Color32::from_rgba_unmultiplied(180, 140, 20, 45);
        let mk_overlay = |zero_lbl: &str, gap_lbl: &str, sig_lbl: &str| -> [Arc<[Annotation]>; 2] {
            [
                vec![
                    Annotation::x_region([8.0, 8.7], band),
                    Annotation::y_line(0.0, zero_lbl),
                    Annotation::x_line(8.0, gap_lbl),
                    Annotation::text([3.0, 0.8], sig_lbl),
                ]
                .into(),
                vec![
                    Annotation::x_region([f64::NEG_INFINITY, 0.1], band),
                    Annotation::x_line(1.0, "x = 1"),
                    Annotation::y_line(1.0, "y = 1"),
                    Annotation::text([10.0, 10.0_f64.powf(0.65)], "y = x^0.65"),
                ]
                .into(),
            ]
        };
        Self {
            light: Arc::new(PlotTheme::scientific_light()),
            dark: Arc::new(PlotTheme::monitoring_dark()),
            overlays: [
                mk_overlay("zero", "gap starts", "signal + observations"),
                mk_overlay("零线", "断口起点", "信号 + 观测散点"),
                mk_overlay("ゼロ", "欠損開始", "信号 + 観測点"),
            ],
        }
    }
}

impl Appearance {
    pub fn resolved_theme(&self, presentation: &Presentation) -> Option<Arc<PlotTheme>> {
        match self.theme {
            ThemeChoice::Host => None,
            ThemeChoice::Scientific => Some(presentation.light.clone()),
            ThemeChoice::Monitoring => Some(presentation.dark.clone()),
        }
    }

    pub fn inherited_plot_style(&self) -> PlotStyle {
        let mut style = PlotStyle::default();
        if self.inherit {
            style.series = SeriesStyle {
                line_width: Some(self.parent_width),
                ..Default::default()
            };
        }
        if self.override_background {
            style.background = Some(self.background);
        }
        style
    }

    pub fn configure(
        &self,
        mut plot: Plot,
        presentation: &Presentation,
        scene: Scene,
        l: Language,
    ) -> Plot {
        if let Some(theme) = self.resolved_theme(presentation) {
            plot = plot.theme(theme);
        }
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
            plot = plot.annotations(
                presentation.overlays[l.index()][usize::from(scene == Scene::PowerLaw)].clone(),
            );
        }
        if self.custom_tooltip {
            plot = plot.hover_formatter(move |info| {
                let point = info.hovered_point.map_or(info.cursor, |p| p.point);
                let units = if scene == Scene::Signal {
                    "s / V"
                } else {
                    tr(l, "x / power", "x / 幂", "x / べき")
                };
                let source = if info.hovered_point.is_some() {
                    info.series_name
                } else {
                    tr(l, "cursor", "游标", "カーソル")
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
        let l = lang(ui.ctx());
        egui::CollapsingHeader::new(tr(
            l,
            "Theme & inherited style",
            "主题与继承样式",
            "テーマと継承スタイル",
        ))
        .id_salt("appearance_collapsing")
        .default_open(true)
        .show(ui, |ui| {
            for (value, label) in [
                (
                    ThemeChoice::Host,
                    tr(
                        l,
                        "Inherit host egui style",
                        "继承宿主 egui 样式（默认）",
                        "ホスト egui スタイルを継承",
                    ),
                ),
                (
                    ThemeChoice::Scientific,
                    tr(l, "Scientific light", "科研浅色", "科学ライト"),
                ),
                (
                    ThemeChoice::Monitoring,
                    tr(l, "Monitoring dark", "监控深色", "監視ダーク"),
                ),
            ] {
                ui.selectable_value(&mut self.theme, value, label);
            }
            ui.checkbox(
                &mut self.inherit,
                tr(
                    l,
                    "Parent line-width default",
                    "启用父级默认线宽",
                    "親線幅の既定値",
                ),
            );
            ui.add_enabled(
                self.inherit,
                egui::Slider::new(&mut self.parent_width, 0.0..=5.0).text(tr(
                    l,
                    "Parent line width",
                    "父级线宽",
                    "親線幅",
                )),
            );
            ui.checkbox(
                &mut self.override_background,
                tr(
                    l,
                    "Canvas background override",
                    "画布背景色覆盖",
                    "背景色オーバーライド",
                ),
            );
            if self.override_background {
                ui.color_edit_button_srgba(&mut self.background);
            }
        });
    }

    pub fn show_scene_extras(&mut self, ui: &mut egui::Ui, signal_color: &mut egui::Color32) {
        let l = lang(ui.ctx());
        ui.checkbox(
            &mut self.override_signal,
            tr(
                l,
                "Override Signal palette color",
                "覆盖信号线调色盘",
                "信号色をカスタム指定",
            ),
        );
        if self.override_signal {
            ui.color_edit_button_srgba(signal_color);
        }
        ui.checkbox(
            &mut self.annotations,
            tr(
                l,
                "Text / reference lines / regions",
                "文本 / 参考线 / 区域注解",
                "注釈 / 基準線 / 領域",
            ),
        );
        ui.checkbox(
            &mut self.custom_tooltip,
            tr(
                l,
                "Custom data tooltip with units",
                "带单位自定义悬停提示",
                "単位付きカスタムツールチップ",
            ),
        );
    }
}
