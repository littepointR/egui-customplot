//! Host-owned vector records; rendering never rescales or reconstructs the data.
use eframe::egui;
use egui_customplot::{
    Axis, AxisScale, Figure, Line, LinePattern, Plot, PlotData, PlotResponse, PlotState, Quiver,
    QuiverArrow, QuiverData, YAxis,
};

use crate::i18n::{lang, tr};

pub(super) struct QuiverDemo {
    pub enabled: bool,
    pub heads: bool,
    pub dashed: bool,
    pub reversed: bool,
    pub head_length: f32,
    field: QuiverData,
    positive: QuiverData,
    reference: PlotData,
    states: [PlotState; 2],
}

impl Default for QuiverDemo {
    fn default() -> Self {
        let mut arrows = Vec::new();
        for y in -3..=3 {
            for x in -5..=5 {
                let (x, y) = (f64::from(x), f64::from(y));
                arrows.push(QuiverArrow::new([x, y], [-0.18 * y, 0.18 * x]));
            }
        }
        arrows.push(QuiverArrow::new([f64::NAN, 1.0], [1.0; 2]));
        Self {
            enabled: false,
            heads: true,
            dashed: false,
            reversed: false,
            head_length: 10.0,
            field: QuiverData::new(arrows),
            positive: QuiverData::new(vec![
                QuiverArrow::new([1.0, 10.0], [1.0, 30.0]),
                QuiverArrow::new([3.0, 50.0], [1.0, -40.0]),
                QuiverArrow::new([5.0, 20.0], [1.0, 80.0]),
                QuiverArrow::new([7.0, 80.0], [1.0, -30.0]),
            ]),
            reference: PlotData::new(vec![[0.0, -1.0], [9.0, 1.0]]),
            states: Default::default(),
        }
    }
}

impl QuiverDemo {
    #[cfg(test)]
    pub fn sample_identities(&self) -> [usize; 2] {
        [
            self.field.arrows().as_ptr() as usize,
            self.positive.arrows().as_ptr() as usize,
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.separator();
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "Explicit 2D vector field / quiver",
                "显式二维矢量场 (Quiver)",
                "2D ベクトル場 / quiver",
            ),
        );
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(&mut self.heads, tr(l, "True-endpoint arrowheads", "真实端点箭头", "実終点矢印"));
            ui.checkbox(&mut self.dashed, tr(l, "Dashed shafts / solid heads", "虚线杆 / 实心箭头", "破線シャフト"));
            ui.checkbox(&mut self.reversed, tr(l, "Reverse X", "反向 X 轴", "X 軸反転"));
            ui.add(
                egui::Slider::new(&mut self.head_length, 2.0..=24.0)
                    .text(tr(l, "Head logical points", "箭头长度 (pt)", "矢印長 (pt)")),
            );
            if ui.button(tr(l, "Fit vector panels", "适配矢量面板", "ベクトル図をフィット")).clicked() {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small(tr(
                l,
                "Origin + displacement, no implicit autoscale. Hover original origins/tips, not clipped intersections or shafts. Zero vectors are not drawn. Exact scan; no vector LOD/index/cache.",
                "起点+位移表示（无隐式缩放），悬停拾取原始起点或终点。",
                "始点＋変位表示（自動スケーリングなし）。始点・終点をホバー対象とします。",
            ));
        });
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
        Figure::new("vector-panels")
            .cell_height(height)
            .show(ui, 2, |cell| {
                let index = cell.index();
                let plot = Plot::new("plot")
                    .height(height)
                    .allow_scroll_zoom(true)
                    .legend(true)
                    .x_axis(Axis::new().reversed(self.reversed));
                let quiver = |data| {
                    Quiver::new(data)
                        .show_heads(self.heads)
                        .head_size(self.head_length, self.head_length * 0.8)
                        .pattern(pattern)
                };
                if index == 0 {
                    cell.show_layers(
                        plot.title(tr(
                            l,
                            "Rotational field: explicit data-space vectors",
                            "旋转矢量场（数据坐标箭头）",
                            "回転ベクトル場",
                        ))
                        .data_aspect(1.0),
                        &mut self.states[0],
                        &[quiver(&self.field)
                            .id_salt("quiv-rot")
                            .name(tr(
                                l,
                                "Rotation / no autoscale",
                                "旋转场（无自动缩放）",
                                "回転場",
                            ))
                            .into()],
                    )
                } else {
                    cell.show_layers(
                        plot.title(tr(
                            l,
                            "Straight transformed-endpoint arrows on right log Y",
                            "右对数 Y 轴变点矢量箭头",
                            "右対数 Y 軸ベクトル",
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
                                .id_salt("quiv-ref")
                                .name(tr(l, "Left reference", "左参考线", "左基準線"))
                                .into(),
                            quiver(&self.positive)
                                .id_salt("quiv-pos")
                                .name(tr(l, "Right vectors", "右矢量序列", "右ベクトル"))
                                .y_axis(YAxis::Right)
                                .color(egui::Color32::LIGHT_RED)
                                .into(),
                        ],
                    )
                }
            })
            .unwrap()
            .inner
    }
}
