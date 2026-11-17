use std::{f64::consts::TAU, sync::Arc};

use eframe::egui::{self, Color32, Stroke};
use egui_customplot::{
    AxisScale, ColorBar, ColorMap, ColorScale, Figure, GridData, Heatmap, Line, LinePattern,
    MarkerShape, Plot, PlotBounds, PlotData, PlotLayer, PlotResponse, PlotState, PlotStyle,
    PlotTheme, Scatter, SeriesStyle,
};

use crate::i18n::{lang, tr};

pub(super) struct CustomStylingDemo {
    pub theme_mode: usize,
    pub fig_style_on: bool,
    pub plot_style_on: bool,
    pub series_override_on: bool,
    pub colormap: ColorMap,
    pub log_colorbar: bool,
    pub columns: usize,
    pub spacing: f32,
    light_theme: Arc<PlotTheme>,
    dark_theme: Arc<PlotTheme>,
    base_wave: PlotData,
    over_wave: PlotData,
    scatter_pts: PlotData,
    grid: GridData,
    states: [PlotState; 4],
}

impl Default for CustomStylingDemo {
    fn default() -> Self {
        let base_wave = PlotData::new(
            (0..100)
                .map(|i| {
                    let x = i as f64 * 0.1;
                    [x, (1.2 * x).sin()]
                })
                .collect::<Vec<_>>(),
        );
        let over_wave = PlotData::new(
            (0..100)
                .map(|i| {
                    let x = i as f64 * 0.1;
                    [x, 0.7 * (1.2 * x + 0.7).cos()]
                })
                .collect::<Vec<_>>(),
        );
        let scatter_pts = PlotData::new(
            (0..16)
                .map(|i| {
                    let t = TAU * i as f64 / 15.0;
                    [i as f64 * 0.65, 0.8 * t.sin()]
                })
                .collect::<Vec<_>>(),
        );
        let vals = (0..32)
            .flat_map(|r| {
                (0..40).map(move |c| {
                    let dx = (c as f64 - 20.0) / 10.0;
                    let dy = (r as f64 - 16.0) / 8.0;
                    0.05 + (-0.9 * (dx * dx + dy * dy)).exp() * 8.0
                })
            })
            .collect::<Vec<_>>();
        let grid = GridData::uniform(
            40,
            32,
            PlotBounds::new([0.0, 0.0], [10.0, 8.0]).expect("grid bounds"),
            vals,
        )
        .expect("grid");

        Self {
            theme_mode: 0,
            fig_style_on: true,
            plot_style_on: true,
            series_override_on: true,
            colormap: ColorMap::Turbo,
            log_colorbar: false,
            columns: 2,
            spacing: 10.0,
            light_theme: Arc::new(PlotTheme::scientific_light()),
            dark_theme: Arc::new(PlotTheme::monitoring_dark()),
            base_wave,
            over_wave,
            scatter_pts,
            grid,
            states: Default::default(),
        }
    }
}

impl CustomStylingDemo {
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(PlotState::reset);
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "5-Layer Style Cascade & ColorMaps",
            "五层样式继承与色条布局",
            "5層スタイル継承・カラーマップ",
        ));
        let theme_lbl = |m: usize| match m {
            0 => tr(
                l,
                "L1/L2: Context Default",
                "L1/L2：跟随全局控制面板",
                "グローバル継承",
            ),
            1 => tr(
                l,
                "L2: Scientific Light",
                "L2：局部科研浅色主题",
                "科学ライト",
            ),
            _ => tr(
                l,
                "L2: Monitoring Dark",
                "L2：局部监控深色主题",
                "監視ダーク",
            ),
        };
        egui::ComboBox::from_id_salt("cstyle-theme")
            .selected_text(theme_lbl(self.theme_mode))
            .show_ui(ui, |ui| {
                for m in 0..3 {
                    ui.selectable_value(&mut self.theme_mode, m, theme_lbl(m));
                }
            });
        ui.checkbox(
            &mut self.fig_style_on,
            tr(
                l,
                "L3: Figure::style line width=2.8",
                "L3：Figure::style 默认线宽 2.8",
                "Figure::style 適用",
            ),
        );
        ui.checkbox(
            &mut self.plot_style_on,
            tr(
                l,
                "L4: Plot::style accent border/grid",
                "L4：Plot::style 局部边框网格覆写",
                "Plot::style 上書き",
            ),
        );
        ui.checkbox(
            &mut self.series_override_on,
            tr(
                l,
                "L5: Series explicit color & dash",
                "L5：Series 显式色/虚线/空心点",
                "Series 固有上書き",
            ),
        );
        ui.separator();
        egui::ComboBox::from_id_salt("cstyle-cmap")
            .selected_text(format!("{:?}", self.colormap))
            .show_ui(ui, |ui| {
                for map in ColorMap::ALL {
                    ui.selectable_value(&mut self.colormap, map, format!("{map:?}"));
                }
            });
        ui.checkbox(
            &mut self.log_colorbar,
            tr(
                l,
                "Log10 ColorScale",
                "对数 ColorScale 色条",
                "対数 ColorScale",
            ),
        );
        ui.add(egui::Slider::new(&mut self.columns, 1..=4).text(tr(
            l,
            "Figure columns",
            "Figure 列数",
            "列数",
        )));
        ui.add(egui::Slider::new(&mut self.spacing, 0.0..=20.0).text(tr(
            l,
            "Figure spacing",
            "子图间距",
            "セル間隔",
        )));
        if ui
            .button(tr(l, "Reset style views", "重置样式视图", "ビュー初期化"))
            .clicked()
        {
            self.reset_views();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let rows = 4usize.div_ceil(self.columns);
        let height = ((ui.available_height() - 90.0) / rows as f32).clamp(130.0, 290.0);
        let mut fig = Figure::new("custom-styling-fig")
            .columns(self.columns)
            .cell_height(height)
            .spacing(egui::vec2(self.spacing, self.spacing));
        if self.theme_mode == 1 {
            fig = fig.theme(self.light_theme.clone());
        } else if self.theme_mode == 2 {
            fig = fig.theme(self.dark_theme.clone());
        }
        if self.fig_style_on {
            fig = fig.style(PlotStyle {
                series: SeriesStyle {
                    line_width: Some(2.8),
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        let plot_override = PlotStyle {
            border_stroke: Some(Stroke::new(2.0, Color32::from_rgb(215, 110, 40))),
            grid_stroke: Some(Stroke::new(1.0, Color32::from_rgb(95, 145, 200))),
            crosshair_stroke: Some(Stroke::new(1.5, Color32::from_rgb(220, 60, 120))),
            ..Default::default()
        };

        let cscale = ColorScale::with_scale(
            [0.05, 8.05],
            if self.log_colorbar {
                AxisScale::Log10
            } else {
                AxisScale::Linear
            },
        )
        .expect("color scale")
        .color_map(self.colormap);

        let [s0, s1, s2, s3] = &mut self.states;
        fig.show(ui, 4, |cell| match cell.index() {
            0 => cell.show_layers(
                Plot::new("cst-fig")
                    .height(height)
                    .allow_scroll_zoom(true)
                    .legend(true)
                    .title(tr(
                        l,
                        "L1/L2/L3: Figure Theme & Style Inherit",
                        "L1/L2/L3：图窗主题与样式继承",
                        "Figure スタイル継承",
                    )),
                s0,
                &[
                    Line::new(&self.base_wave).name("Inherited").into(),
                    Line::new(&self.over_wave).name("Series #2").into(),
                ],
            ),
            1 => {
                let mut p = Plot::new("cst-plot")
                    .height(height)
                    .allow_scroll_zoom(true)
                    .legend(true)
                    .title(tr(
                        l,
                        "L4: Plot::style Frame/Grid/Crosshair",
                        "L4：Plot::style 边框网格十字线定制",
                        "Plot::style 個別設定",
                    ));
                if self.plot_style_on {
                    p = p.style(plot_override);
                }
                cell.show_layers(p, s1, &[Line::new(&self.base_wave).name("Wave").into()])
            }
            2 => {
                let mut l5 = Line::new(&self.over_wave).name("L5 Explicit");
                let mut sc = Scatter::new(&self.scatter_pts).name("Hollow Diamond");
                if self.series_override_on {
                    l5 = l5
                        .color(Color32::from_rgb(220, 50, 110))
                        .width(3.2)
                        .pattern(LinePattern::Dashed {
                            length: 4.0,
                            gap: 4.0,
                        });
                    sc = sc
                        .color(Color32::from_rgb(40, 165, 95))
                        .shape(MarkerShape::DiamondHollow)
                        .radius(6.5);
                }
                cell.show_layers(
                    Plot::new("cst-series")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "L5: Series Color, Dash & Hollow Marker",
                            "L5：Series 线型点型精确覆写",
                            "Series スタイル上書き",
                        )),
                    s2,
                    &[l5.into(), sc.into()],
                )
            }
            _ => {
                let hm_plot = cell.plot(
                    Plot::new("cst-cmap")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .title(tr(
                            l,
                            "ColorMap & Linear/Log10 ColorBar",
                            "动态调色板与线性/对数 ColorBar",
                            "カラーマップ・ColorBar",
                        )),
                );
                cell.ui()
                    .with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ColorBar::new(cscale)
                            .label(format!("{:?}", self.colormap))
                            .height(height)
                            .show(ui);
                        hm_plot.show_layers(
                            ui,
                            s3,
                            &[PlotLayer::from(Heatmap::new(&self.grid, cscale))],
                        )
                    })
                    .inner
            }
        })
        .expect("custom styling figure")
        .inner
    }
}
