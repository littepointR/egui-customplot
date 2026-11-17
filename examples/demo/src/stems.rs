//! Fixed host-owned discrete signals; no data construction inside plot passes.
use eframe::egui;
use egui_customplot::{
    Axis, AxisScale, Figure, Line, LinePattern, Plot, PlotData, PlotResponse, PlotState,
    StemOrientation, Stems, YAxis,
};

use crate::i18n::{lang, tr};

pub(super) struct StemDemo {
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
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(PlotState::reset);
    }

    #[cfg(test)]
    pub fn data_versions(&self) -> [u64; 3] {
        [
            self.signal.version(),
            self.positive.version(),
            self.reference.version(),
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Discrete stems / baseline / markers",
            "离散火柴杆 / 基线 / 端点标记",
            "離散ステム図 / 基準線",
        ));
        if ui
            .checkbox(
                &mut self.horizontal,
                tr(
                    l,
                    "Horizontal top-panel rods (raw XY unchanged)",
                    "顶部面板水平杆（原始 XY 不变）",
                    "上段パネル横向き（元 XY 不変）",
                ),
            )
            .changed()
        {
            self.states[0].reset();
        }
        ui.checkbox(
            &mut self.markers,
            tr(
                l,
                "Raw endpoint markers",
                "原始端点标记",
                "端点マーカー表示",
            ),
        );
        ui.checkbox(
            &mut self.baseline_visible,
            tr(l, "Show baseline", "显示基线", "ベースライン表示"),
        );
        ui.checkbox(
            &mut self.dashed,
            tr(
                l,
                "Dashed rods (baseline stays solid)",
                "虚线杆（基线保持实线）",
                "破線ステム",
            ),
        );
        ui.add(egui::Slider::new(&mut self.baseline, -2.0..=2.0).text(tr(
            l,
            "Top baseline",
            "上方基线",
            "上段基準線",
        )));
        if ui
            .button(tr(
                l,
                "Fit stem panels",
                "适配火柴杆面板",
                "ステム図をフィット",
            ))
            .clicked()
        {
            self.states.iter_mut().for_each(PlotState::reset);
        }
        ui.small(tr(
            l,
            "Independent rods; NaN omits only its record. Hover raw endpoints, not rods or baseline. Rendering and markers scan all points; no stem LOD/cache.",
            "独立离散杆；NaN 仅忽略单条记录，悬停拾取原始端点。",
            "独立ロッド；NaN のみ除外し、ホバーは生端点を対象とします。",
        ));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
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
                        plot.title(tr(
                            l,
                            "Independent signed stems",
                            "独立有向火柴杆",
                            "符号付きステム図",
                        )),
                        &mut self.states[0],
                        &[Stems::new(&self.signal)
                            .id_salt("stem-main")
                            .name(tr(l, "Discrete signal", "离散信号", "離散信号"))
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
                        plot.title(tr(
                            l,
                            "Raw endpoints on a right logarithmic Y axis",
                            "右对数 Y 轴离散端点",
                            "右対数 Y 軸ステム図",
                        ))
                        .right_y_axis(
                            Axis::new().scale(AxisScale::Log10).label(tr(
                                l,
                                "Amplitude (log)",
                                "幅值（对数）",
                                "振幅（対数）",
                            )),
                        ),
                        &mut self.states[1],
                        &[
                            Line::new(&self.reference)
                                .id_salt("stem-ref")
                                .name(tr(l, "Left reference", "左参考线", "左基準線"))
                                .into(),
                            Stems::new(&self.positive)
                                .id_salt("stem-pos")
                                .name(tr(l, "Right discrete signal", "右离散信号", "右離散信号"))
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
