//! Host-owned scalar field contours, pcolor shading, and RGBA images.
use eframe::egui;
use egui_customplot::{
    ColorBar, ColorMap, ColorScale, ContourData, ContourLevels, Contours, Figure,
    FilledContourData, FilledContours, ImageData, Pcolor, PcolorShading, Plot, PlotBounds,
    PlotImage, PlotResponse, PlotState, ScalarField2D,
};

use crate::i18n::{lang, tr};

pub(super) struct ContourDemo {
    pub show_pcolor: bool,
    pub interp_shading: bool,
    pub show_labels: bool,
    pub colors: ColorMap,
    field: ScalarField2D,
    lines: ContourData,
    filled: FilledContourData,
    image: ImageData,
    states: [PlotState; 2],
}

impl Default for ContourDemo {
    fn default() -> Self {
        let bounds = PlotBounds::new([-3.0, -3.0], [3.0, 3.0]).expect("valid bounds");
        let field = ScalarField2D::sample_function(bounds, [40, 36], 10_000, |x, y| {
            x.sin() * y.cos() + 0.35 * (-(x * x + y * y) * 0.25).exp()
        })
        .expect("within evaluation budget");
        let levels = ContourLevels::Count(8);
        let lines = ContourData::from_field(&field, levels.clone()).expect("valid levels");
        let filled = FilledContourData::from_field(&field, levels).expect("valid levels");

        let (w, h) = (32usize, 24usize);
        let mut rgba = Vec::with_capacity(w * h * 4);
        for r in 0..h {
            for c in 0..w {
                if (c < 4 && r < 4) || (c >= w - 4 && r >= h - 4) {
                    rgba.extend_from_slice(&[0, 0, 0, 0]);
                } else {
                    let fx = c as f32 / (w - 1) as f32;
                    let fy = r as f32 / (h - 1) as f32;
                    let checker = if ((c / 4) + (r / 4)) % 2 == 0 {
                        220.0
                    } else {
                        70.0
                    };
                    let red = (fx * checker) as u8;
                    let green = (fy * 210.0) as u8;
                    let blue = ((1.0 - fx) * 200.0) as u8;
                    rgba.extend_from_slice(&[red, green, blue, 255]);
                }
            }
        }
        let image = ImageData::rgba8(
            w,
            h,
            PlotBounds::new([0.0, 0.0], [w as f64, h as f64]).expect("valid image bounds"),
            rgba,
        )
        .expect("matching RGBA length");

        Self {
            show_pcolor: false,
            interp_shading: false,
            show_labels: true,
            colors: ColorMap::Viridis,
            field,
            lines,
            filled,
            image,
            states: Default::default(),
        }
    }
}

impl ContourDemo {
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(PlotState::reset);
    }

    #[cfg(test)]
    pub fn sample_identities(&self) -> [usize; 2] {
        [
            self.field.values().as_ptr() as usize,
            self.image.rgba_bytes().as_ptr() as usize,
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Contours, pcolor & images",
            "等高线、伪彩与图像",
            "等高線・疑似カラー・画像",
        ));
        egui::ComboBox::from_id_salt("contour-color-map")
            .selected_text(format!("{:?}", self.colors))
            .show_ui(ui, |ui| {
                for map in ColorMap::ALL {
                    ui.selectable_value(&mut self.colors, map, format!("{map:?}"));
                }
            });
        ui.checkbox(
            &mut self.show_pcolor,
            tr(
                l,
                "Pcolor instead of contourf",
                "显示 Pcolor 替换填充等高线",
                "contourf の代わりに Pcolor",
            ),
        );
        ui.add_enabled(
            self.show_pcolor,
            egui::Checkbox::new(
                &mut self.interp_shading,
                tr(
                    l,
                    "Interpolated pcolor shading",
                    "Pcolor 平滑插值着色",
                    "Pcolor 補間シェーディング",
                ),
            ),
        );
        ui.checkbox(
            &mut self.show_labels,
            tr(
                l,
                "Contour level labels",
                "显示等高线层级标签",
                "等高線ラベル表示",
            ),
        );
        if ui
            .button(tr(
                l,
                "Fit contour/image panels",
                "适配等高线/图像面板",
                "等高線・画像フィット",
            ))
            .clicked()
        {
            self.states.iter_mut().for_each(PlotState::reset);
        }
        ui.small(tr(
            l,
            "Marching Squares contours extracted once on host; imshow_axes reverses Y with 1:1 aspect.",
            "Marching Squares 等高线在宿主一次提取；imshow_axes 自动反转 Y 轴并锁定 1:1 比例。",
            "等高線はホストで一度抽出；imshow_axes は Y 反転・等倍アスペクトを自動設定します。",
        ));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let total_h = (ui.available_height() - 100.0).max(120.0);
        let scale = ColorScale::new([-1.0, 1.35])
            .expect("valid range")
            .color_map(self.colors);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ColorBar::new(scale)
                .label(tr(l, "Scalar z", "标量 z", "スカラー z"))
                .height(total_h)
                .show(ui);
            Figure::new("contour-panels")
                .cell_height(height)
                .show(ui, 2, |cell| {
                    if cell.index() == 0 {
                        let plot = Plot::new("contour-plot")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .legend(true);
                        if self.show_pcolor {
                            let shading = if self.interp_shading {
                                PcolorShading::Interp
                            } else {
                                PcolorShading::Flat
                            };
                            cell.show_layers(
                                plot.title(tr(
                                    l,
                                    "Pcolor Rectilinear Field + Contour Isolines",
                                    "Pcolor 伪彩网格 + 等值线",
                                    "Pcolor 場＋等値線",
                                )),
                                &mut self.states[0],
                                &[
                                    Pcolor::new(&self.field, scale)
                                        .id_salt("pcolor")
                                        .name("Pcolor")
                                        .shading(shading)
                                        .into(),
                                    Contours::new(&self.lines)
                                        .id_salt("isolines")
                                        .name(tr(l, "Isolines", "等值线", "等値線"))
                                        .color(egui::Color32::WHITE)
                                        .show_labels(self.show_labels)
                                        .into(),
                                ],
                            )
                        } else {
                            cell.show_layers(
                                plot.title(tr(
                                    l,
                                    "Filled Contours (contourf) + Labeled Isolines",
                                    "填充等高线 (contourf) + 带标签等值线",
                                    "塗りつぶし等高線 (contourf)＋等値線",
                                )),
                                &mut self.states[0],
                                &[
                                    FilledContours::new(&self.filled, scale)
                                        .id_salt("contourf")
                                        .name(tr(
                                            l,
                                            "Filled contours",
                                            "填充等高线",
                                            "塗りつぶし等高線",
                                        ))
                                        .into(),
                                    Contours::new(&self.lines)
                                        .id_salt("isolines")
                                        .name(tr(l, "Isolines", "等值线", "等値線"))
                                        .color(egui::Color32::from_gray(235))
                                        .show_labels(self.show_labels)
                                        .into(),
                                ],
                            )
                        }
                    } else {
                        cell.show_layers(
                            Plot::new("imshow-plot")
                                .imshow_axes()
                                .height(height)
                                .allow_scroll_zoom(true)
                                .legend(true)
                                .title(tr(
                                    l,
                                    "RGBA Image (imshow_axes: top-down Y, 1:1 aspect)",
                                    "RGBA 图像（imshow_axes：Y 自上而下，1:1 比例）",
                                    "RGBA 画像（imshow_axes：上向き原点・1:1 比率）",
                                )),
                            &mut self.states[1],
                            &[PlotImage::new(&self.image)
                                .id_salt("rgba-img")
                                .name(tr(l, "RGBA image", "RGBA 图像", "RGBA 画像"))
                                .into()],
                        )
                    }
                })
                .unwrap()
                .inner
        })
        .inner
    }
}
