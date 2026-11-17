//! Plot descriptors and multi-panel composition, separate from host lifecycle.
use eframe::egui;
use egui_customplot::{
    Axis, AxisLink, AxisScale, Figure, Line, LinePattern, MarkerStyle, PlotResponse, PlotState,
    Scatter, Series, YAxis,
};

use crate::{
    appearance::Presentation,
    controls::Options,
    data::DemoData,
    i18n::{Language, lang, tr},
    performance::Performance,
};

#[derive(Default)]
pub(super) struct MultiViewOptions {
    pub side_by_side: bool,
    pub link_x: bool,
    pub right_y: bool,
    pub right_log: bool,
    pub right_reversed: bool,
}

impl MultiViewOptions {
    pub fn show(&mut self, ui: &mut egui::Ui, second_view: &mut bool) {
        let l = lang(ui.ctx());
        ui.checkbox(
            second_view,
            tr(
                l,
                "Second Figure cell",
                "第二幅 Figure 子图",
                "2番目の Figure セル",
            ),
        );
        ui.add_enabled_ui(*second_view, |ui| {
            ui.checkbox(
                &mut self.side_by_side,
                tr(l, "Side-by-side cells", "并排子图", "並べて表示"),
            );
            ui.checkbox(
                &mut self.link_x,
                tr(
                    l,
                    "Link X (incompatible with equal units)",
                    "联动 X 轴（与等比例互斥）",
                    "X 軸連動（等倍単位とは非互換）",
                ),
            );
        });
        ui.checkbox(
            &mut self.right_y,
            tr(
                l,
                "Secondary signal on right Y",
                "右 Y 轴次级信号",
                "右 Y 軸に第2信号",
            ),
        );
        ui.add_enabled_ui(self.right_y, |ui| {
            ui.checkbox(
                &mut self.right_log,
                tr(l, "Right log Y", "右对数 Y", "右対数 Y"),
            );
            ui.checkbox(
                &mut self.right_reversed,
                tr(l, "Right reversed Y", "右反向 Y", "右反転 Y"),
            );
            ui.small(tr(
                l,
                "Static scene: independent pressure signal; Live mode omits it.",
                "静态模式显示独立压力信号；实时模式省略。",
                "静的シーンでは独立圧力信号を表示（ライブ時は省略）。",
            ));
        });
    }
}

pub(super) struct Charts {
    pub main: PlotState,
    pub detail: PlotState,
    link: AxisLink,
}

impl Default for Charts {
    fn default() -> Self {
        Self {
            main: PlotState::default(),
            detail: PlotState::default(),
            link: AxisLink::new([true, false]),
        }
    }
}

impl Charts {
    pub fn reset(&mut self) {
        self.main.reset();
        self.detail.reset();
        self.link.reset();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        options: &Options,
        presentation: &Presentation,
        data: &DemoData,
        performance: &mut Performance,
    ) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let series = series(options, data, &performance.live.data, l);
        let count: usize = if options.second_view { 2 } else { 1 };
        let columns = if options.multi.side_by_side { 2 } else { 1 };
        let rows = count.div_ceil(columns);
        let height = ((ui.available_height() - 190.0) / rows as f32).max(80.0);
        let linked = options.second_view && options.multi.link_x && !options.equal_units;
        if !linked {
            self.main.clear_axis_links();
            self.detail.clear_axis_links();
            self.link.reset();
        }
        let responses = Figure::new("demo-figure")
            .columns(columns)
            .cell_height(height)
            .show(ui, count, |cell| {
                let mut plot = options.appearance.configure(
                    options.plot("plot", height, l),
                    presentation,
                    options.scene,
                    l,
                );
                if options.multi.right_y {
                    plot = plot.right_y_axis(
                        Axis::new()
                            .label(tr(l, "Pressure", "压力", "圧力"))
                            .scale(if options.multi.right_log {
                                AxisScale::Log10
                            } else {
                                AxisScale::Linear
                            })
                            .reversed(options.multi.right_reversed),
                    );
                }
                let state = if cell.index() == 0 {
                    &mut self.main
                } else {
                    &mut self.detail
                };
                if options.performance.cached {
                    cell.show_series_cached(plot, state, &series, &mut performance.cache)
                } else {
                    cell.show_series(plot, state, &series)
                }
            })
            .expect("finite, fixed cell count")
            .inner;
        if linked
            && let Err(error) = self
                .link
                .synchronize(ui.ctx(), &mut [&mut self.main, &mut self.detail])
        {
            ui.colored_label(egui::Color32::YELLOW, error.to_string());
        }
        if let Some(brushed) = self.main.brushed_bounds() {
            let active = if options.performance.streaming {
                &performance.live.data
            } else {
                &data.signal
            };
            let picked = active.brushed_indices(brushed).len();
            ui.horizontal(|ui| {
                ui.small(match l {
                    Language::English => format!("Brushed signal points: {picked}"),
                    Language::ZhCn => format!("刷选命中信号点数：{picked}"),
                    Language::Japanese => format!("選択中の信号点数：{picked}"),
                });
                if ui
                    .small_button(tr(l, "Clear brush", "清除刷选", "選択解除"))
                    .clicked()
                {
                    self.main.clear_brush();
                }
            });
        }
        responses
    }
}

fn series<'a>(
    options: &Options,
    data: &'a DemoData,
    live: &'a egui_customplot::PlotData,
    l: Language,
) -> Vec<Series<'a>> {
    let settings = &options.performance;
    let signal = if settings.streaming {
        live
    } else {
        &data.signal
    };
    let mut line = Line::new(signal)
        .id_salt("signal")
        .name(tr(l, "Signal", "信号", "信号"))
        .sampling(settings.sampling)
        .interpolation(options.interpolation)
        .width(options.width);
    if options.appearance.override_signal {
        line = line.color(options.color);
    }
    if options.markers {
        line = line.markers(MarkerStyle {
            shape: options.shape,
            radius: options.radius,
        });
    }
    if options.dashed {
        line = line.pattern(LinePattern::Dashed {
            length: 8.0,
            gap: 5.0,
        });
    }
    let mut series = vec![line.into()];
    if !settings.streaming {
        series.push(
            Line::new(&data.reference)
                .id_salt("reference")
                .name(tr(l, "Reference", "参考线", "基準線"))
                .sampling(settings.sampling)
                .interpolation(options.interpolation)
                .into(),
        );
        if options.observations {
            series.push(
                Scatter::new(&data.observations)
                    .id_salt("observations")
                    .name(tr(l, "Observations", "观测点", "観測値"))
                    .shape(options.shape)
                    .colors(&data.colors)
                    .expect("matching colors")
                    .radii(&data.radii)
                    .expect("matching radii")
                    .into(),
            );
        }
        if options.multi.right_y {
            series.push(
                Line::new(&data.secondary)
                    .id_salt("pressure")
                    .name(tr(
                        l,
                        "Pressure (right Y)",
                        "压力（右 Y 轴）",
                        "圧力（右 Y 軸）",
                    ))
                    .y_axis(YAxis::Right)
                    .sampling(settings.sampling)
                    .interpolation(options.interpolation)
                    .into(),
            );
        }
    }
    series
}
