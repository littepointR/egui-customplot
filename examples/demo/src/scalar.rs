use eframe::egui;
use egui_customplot::{
    Axis, AxisScale, ColorBar, ColorMap, ColorScale, GridData, Heatmap, Line, Plot, PlotBounds,
    PlotData, PlotResponse, PlotState,
};

use crate::i18n::{lang, tr};

/// Independent scalar-cell case; samples and navigation are owned by the host.
pub(super) struct ScalarDemo {
    pub colors: ColorMap,
    pub log_colors: bool,
    pub log_axes: bool,
    pub reversed_y: bool,
    grid: GridData,
    diagonal: PlotData,
    state: PlotState,
}

impl Default for ScalarDemo {
    fn default() -> Self {
        let values = (0..48)
            .flat_map(|row| {
                (0..64).map(move |column| {
                    if (20..25).contains(&column) && (12..30).contains(&row) {
                        f64::NAN
                    } else {
                        let x = (column as f64 - 32.0) / 14.0;
                        let y = (row as f64 - 24.0) / 10.0;
                        0.01 + (-(x * x + y * y)).exp()
                    }
                })
            })
            .collect::<Vec<_>>();
        Self {
            colors: ColorMap::Viridis,
            log_colors: false,
            log_axes: false,
            reversed_y: false,
            grid: GridData::uniform(
                64,
                48,
                PlotBounds::new([1.0; 2], [10.0; 2]).unwrap(),
                values,
            )
            .unwrap(),
            diagonal: PlotData::new(vec![[1.0; 2], [10.0; 2]]),
            state: PlotState::default(),
        }
    }
}

impl ScalarDemo {
    pub fn reset_views(&mut self) {
        self.state.reset();
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Heatmap + shared color bar",
            "热力图 + 共享色条",
            "ヒートマップ＋カラーバー",
        ));
        egui::ComboBox::from_id_salt("color-map")
            .selected_text(format!("{:?}", self.colors))
            .show_ui(ui, |ui| {
                for map in ColorMap::ALL {
                    ui.selectable_value(&mut self.colors, map, format!("{map:?}"));
                }
            });
        ui.checkbox(
            &mut self.log_colors,
            tr(l, "Log color limits", "对数颜色映射", "対数カラースケール"),
        );
        if ui
            .checkbox(
                &mut self.log_axes,
                tr(l, "Log X / Y", "对数 X / Y 轴", "対数 X / Y 軸"),
            )
            .changed()
        {
            self.state.reset();
        }
        ui.checkbox(
            &mut self.reversed_y,
            tr(
                l,
                "Image-style row direction",
                "图像行方向（Y 反转）",
                "画像行方向（Y 反転）",
            ),
        );
        if ui
            .button(tr(l, "Fit heatmap", "适配热力图", "ヒートマップに合わせる"))
            .clicked()
        {
            self.state.reset();
        }
        ui.small(tr(
            l,
            "NaN cells are holes. Hover reports row, column and raw value. No interpolation or automatic cell LOD.",
            "NaN 网格显示为空洞；悬停报告行列与原始值。",
            "NaN セルは空洞表示。ホバーで行・列・生値を取得。",
        ));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> PlotResponse {
        let l = lang(ui.ctx());
        let scale = |log| {
            if log {
                AxisScale::Log10
            } else {
                AxisScale::Linear
            }
        };
        let colors = ColorScale::with_scale([0.01, 1.01], scale(self.log_colors))
            .unwrap()
            .color_map(self.colors);
        let height = (ui.available_height() - 120.0).max(100.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ColorBar::new(colors)
                .label(tr(l, "Intensity", "强度", "強度"))
                .height(height)
                .show(ui);
            Plot::new("scalar")
                .height(height)
                .allow_scroll_zoom(true)
                .title(tr(
                    l,
                    "Scalar cells + line overlay",
                    "标量网格 + 折线叠加",
                    "スカラーセル＋ライン",
                ))
                .x_axis(Axis::new().label("x").scale(scale(self.log_axes)))
                .y_axis(
                    Axis::new()
                        .label("y")
                        .scale(scale(self.log_axes))
                        .reversed(self.reversed_y),
                )
                .show_layers(
                    ui,
                    &mut self.state,
                    &[
                        Heatmap::new(&self.grid, colors).id_salt("cells").into(),
                        Line::new(&self.diagonal)
                            .id_salt("diagonal")
                            .name(tr(l, "Diagonal", "对角线", "対角線"))
                            .color(egui::Color32::WHITE)
                            .into(),
                    ],
                )
        })
        .inner
    }
}
