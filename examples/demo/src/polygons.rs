//! Host-owned area bands, polygons, holed polyshapes, Gouraud patches, and rounded shapes.
use std::sync::Arc;

use eframe::egui::{self, Color32, Stroke};
use egui_customplot::{
    Area, AreaBandData, AreaStackMode, ColorMap, ColorScale, DataRect, Figure, Fill, Line, Patch,
    PatchData, Plot, PlotData, PlotResponse, PlotState, PolyRegion, PolyShape, PolyShapeData,
    PolygonData, RectData, Rectangles,
};

use crate::i18n::{lang, tr};

pub(super) struct PolygonDemo {
    pub enabled: bool,
    pub pos_neg_split: bool,
    pub gouraud_patch: bool,
    pub show_borders: bool,
    cum_bands: Vec<AreaBandData>,
    split_bands: Vec<AreaBandData>,
    reference: PlotData,
    concave: PolygonData,
    holed: PolyShapeData,
    patch: PatchData,
    patch_scalars: Vec<f64>,
    patch_face_colors: Vec<Color32>,
    rects: RectData,
    states: [PlotState; 2],
}

impl Default for PolygonDemo {
    fn default() -> Self {
        let n = 25usize;
        let x: Arc<[f64]> = (0..n).map(|i| i as f64 * 0.5).collect::<Vec<_>>().into();
        let mut s1 = Vec::with_capacity(n);
        let mut s2 = Vec::with_capacity(n);
        let mut s3 = Vec::with_capacity(n);
        for (i, &xv) in x.iter().enumerate() {
            if i == 12 {
                s1.push(f64::NAN);
                s2.push(f64::NAN);
                s3.push(f64::NAN);
            } else {
                s1.push(1.4 * (xv * 0.6).sin());
                s2.push(0.9 * (xv * 0.45 + 0.6).cos());
                s3.push(0.6 * (xv * 0.9 - 0.4).sin());
            }
        }
        let cum_bands = AreaBandData::stack(
            Arc::clone(&x),
            &[&s1, &s2, &s3],
            0.0,
            AreaStackMode::Cumulative,
        )
        .expect("finite baseline");
        let split_bands = AreaBandData::stack(x, &[&s1, &s2, &s3], 0.0, AreaStackMode::PosNegSplit)
            .expect("finite baseline");

        let concave = PolygonData::new(vec![
            [0.5, 0.5],
            [4.5, 0.5],
            [4.5, 4.5],
            [3.3, 4.5],
            [3.3, 1.8],
            [1.7, 1.8],
            [1.7, 4.5],
            [0.5, 4.5],
        ]);
        let holed = PolyShapeData::new(vec![
            PolyRegion::new(vec![[0.5, 5.5], [5.5, 5.5], [5.5, 9.5], [0.5, 9.5]]).hole(vec![
                [2.0, 6.7],
                [4.0, 6.7],
                [4.0, 8.3],
                [2.0, 8.3],
            ]),
        ]);

        let (cols, rows) = (4usize, 3usize);
        let mut verts = Vec::with_capacity((cols + 1) * (rows + 1));
        let mut scalars = Vec::with_capacity((cols + 1) * (rows + 1));
        for r in 0..=rows {
            for c in 0..=cols {
                let vx = 6.4 + c as f64 * 0.95;
                let vy = 0.8 + r as f64 * 1.45;
                verts.push([vx, vy]);
                scalars.push((c as f64 / cols as f64) * 0.6 + (r as f64 / rows as f64) * 0.4);
            }
        }
        let mut faces = Vec::with_capacity(cols * rows);
        let mut face_colors = Vec::with_capacity(cols * rows);
        for r in 0..rows {
            for c in 0..cols {
                let i0 = r * (cols + 1) + c;
                let i1 = i0 + 1;
                let i2 = i0 + (cols + 1) + 1;
                let i3 = i0 + (cols + 1);
                faces.push([i0, i1, i2, i3]);
                let shade = (c * 45 + r * 55) as u8;
                face_colors.push(Color32::from_rgb(70 + shade / 2, 130, 230 - shade / 2));
            }
        }
        let face_slices: Vec<&[usize]> = faces.iter().map(|f| f.as_slice()).collect();
        let patch = PatchData::new(verts, &face_slices).expect("valid quad faces");
        Self {
            enabled: false,
            pos_neg_split: false,
            gouraud_patch: true,
            show_borders: true,
            cum_bands,
            split_bands,
            reference: PlotData::new(vec![[0.0, 0.0], [12.0, 0.0]]),
            concave,
            holed,
            patch,
            patch_scalars: scalars,
            patch_face_colors: face_colors,
            rects: RectData::new(vec![
                DataRect::new([6.4, 5.8], [3.6, 1.9]).curvature([0.45, 0.45]),
                DataRect::ellipse([8.2, 8.9], [1.6, 0.85]),
            ]),
            states: Default::default(),
        }
    }
}

impl PolygonDemo {
    #[cfg(test)]
    pub fn sample_identities(&self) -> [usize; 3] {
        [
            self.cum_bands[0].x().as_ptr() as usize,
            self.concave.vertices().as_ptr() as usize,
            self.patch.vertices().as_ptr() as usize,
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.separator();
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "2D areas, polygons & patches",
                "二维面积、多边形与面片",
                "2D 領域・多角形・パッチ",
            ),
        );
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(
                &mut self.pos_neg_split,
                tr(l, "PosNegSplit area stacking", "正负分离面积堆叠", "正負分離エリア積上げ"),
            );
            ui.checkbox(
                &mut self.gouraud_patch,
                tr(l, "Gouraud vertex-colored patch", "Gouraud 顶点插值着色面片", "グーロー補間パッチ"),
            );
            ui.checkbox(&mut self.show_borders, tr(l, "Show shape borders", "显示轮廓边框", "境界線を表示"));
            if ui.button(tr(l, "Fit polygon panels", "适配多边形面板", "多角形ビューフィット")).clicked() {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small(tr(
                l,
                "Ear-clipped polygons/holes & Gouraud patches; SVG uses raster fallback for Gouraud meshes.",
                "支持带孔多边形耳切三角化与 Gouraud 面片（SVG 中平滑渐变网走栅格回退）。",
                "穴あき多角形・グーローパッチ対応（SVG 出力はラスターフォールバック）。",
            ));
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let border = if self.show_borders {
            Stroke::new(1.2, Color32::WHITE)
        } else {
            Stroke::NONE
        };
        let scale = ColorScale::new([0.0, 1.0])
            .expect("valid scale")
            .color_map(ColorMap::Viridis);
        let patch_layer = if self.gouraud_patch {
            Patch::new(&self.patch)
                .id_salt("poly-patch")
                .name(tr(l, "Gouraud patch", "Gouraud 面片", "グーローパッチ"))
                .vertex_scalars(&self.patch_scalars, scale)
                .expect("matching vertex count")
                .border(border)
        } else {
            Patch::new(&self.patch)
                .id_salt("poly-patch")
                .name(tr(l, "Flat patch", "纯色面片", "フラットパッチ"))
                .face_colors(&self.patch_face_colors)
                .expect("matching face count")
                .border(border)
        };
        let bands = if self.pos_neg_split {
            &self.split_bands
        } else {
            &self.cum_bands
        };
        let band_fills = [
            Color32::from_rgba_unmultiplied(70, 160, 235, 160),
            Color32::from_rgba_unmultiplied(235, 145, 65, 160),
            Color32::from_rgba_unmultiplied(90, 190, 130, 160),
        ];

        Figure::new("polygon-panels")
            .cell_height(height)
            .show(ui, 2, |cell| {
                let plot = Plot::new("plot")
                    .height(height)
                    .allow_scroll_zoom(true)
                    .legend(true);
                if cell.index() == 0 {
                    cell.show_layers(
                        plot.title(tr(
                            l,
                            "Stacked Area Bands + Data-Space Line Primitive",
                            "堆叠面积带 + 原始折线图元",
                            "積上げエリア＋基本ライン",
                        )),
                        &mut self.states[0],
                        &[
                            Area::from_band(&bands[0])
                                .id_salt("band-a")
                                .name(tr(l, "Band A", "面积带 A", "帯 A"))
                                .fill_color(band_fills[0])
                                .into(),
                            Area::from_band(&bands[1])
                                .id_salt("band-b")
                                .name(tr(l, "Band B", "面积带 B", "帯 B"))
                                .fill_color(band_fills[1])
                                .into(),
                            Area::from_band(&bands[2])
                                .id_salt("band-c")
                                .name(tr(l, "Band C", "面积带 C", "帯 C"))
                                .fill_color(band_fills[2])
                                .into(),
                            Line::primitive(&self.reference)
                                .id_salt("band-ref")
                                .name(tr(
                                    l,
                                    "Zero baseline (primitive)",
                                    "零基线 (primitive)",
                                    "ゼロ基準線",
                                ))
                                .color(Color32::WHITE)
                                .into(),
                        ],
                    )
                } else {
                    cell.show_layers(
                        plot.title(tr(
                            l,
                            "Concave Fill, Holed PolyShape, Patch & Rectangles",
                            "凹多边形、带孔 PolyShape、Patch 与圆角矩形",
                            "凹多角形・穴あき PolyShape・Patch・角丸矩形",
                        ))
                        .data_aspect(1.0),
                        &mut self.states[1],
                        &[
                            Fill::new(&self.concave)
                                .id_salt("poly-concave")
                                .name(tr(l, "Concave fill", "凹多边形填充", "凹多角形"))
                                .color(Color32::from_rgba_unmultiplied(75, 155, 235, 190))
                                .border(border)
                                .into(),
                            PolyShape::new(&self.holed)
                                .id_salt("poly-holed")
                                .name(tr(
                                    l,
                                    "PolyShape w/ hole",
                                    "带孔 PolyShape",
                                    "穴あき PolyShape",
                                ))
                                .color(Color32::from_rgba_unmultiplied(235, 165, 60, 190))
                                .border(border)
                                .into(),
                            patch_layer.into(),
                            Rectangles::new(&self.rects)
                                .id_salt("poly-rects")
                                .name(tr(
                                    l,
                                    "Rounded & ellipse",
                                    "圆角矩形与椭圆",
                                    "角丸矩形・楕円",
                                ))
                                .color(Color32::from_rgba_unmultiplied(110, 200, 140, 180))
                                .border(border)
                                .into(),
                        ],
                    )
                }
            })
            .unwrap()
            .inner
    }
}
