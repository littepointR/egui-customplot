//! Stage 13 completion gallery (`--stage13`, `EXT-06..12`):
//! confusionchart, skyplot, curvilinear pcolor/contour, adaptive fcontour/fsurf,
//! Patch3D, AlphaShape3D, and Graph3D.
use std::f64::consts::PI;

use eframe::egui;
use egui_customplot::{
    AlphaShape3D, AlphaShape3DData, ColorMap, ColorScale, ConfusionChart, ConfusionMatrixData,
    ConfusionNormalization, ContourData, ContourLevels, Contours, CurvilinearField2D,
    CurvilinearPcolor, DepthMode, Figure, FilledContourData, FilledContours, Graph3D, Graph3DData,
    GraphLayout3D, Layer3D, MarkerShape, Patch3D, Patch3DData, PcolorShading, Plot, Plot3D,
    Plot3DState, PlotLayer, PlotResponse, PlotState, Point3D, SkyPlot, SkyPlotData, SkyPlotState,
    SkyTrack, SurfaceShading, TriMesh3DData, TriSurf3D,
};

use crate::i18n::{lang, tr};

pub(super) struct Stage13Demo {
    pub norm: ConfusionNormalization,
    pub show_summaries: bool,
    pub mask_el_deg: f64,
    pub curv_interp: bool,
    pub zbuffer_3d: bool,
    pub alpha_radius: f64,
    pub crit_alpha: f64,
    confusion: ConfusionMatrixData,
    sky: SkyPlotData,
    curv: CurvilinearField2D,
    curv_contours: ContourData,
    adaptive_filled: FilledContourData,
    adaptive_lines: ContourData,
    adaptive_surf: TriMesh3DData,
    patch3: Patch3DData,
    alpha_sites: Vec<Point3D>,
    alpha_data: AlphaShape3DData,
    graph3: Graph3DData,
    plot_states: [PlotState; 3],
    sky_state: SkyPlotState,
    plot3d_states: [Plot3DState; 4],
}

impl Default for Stage13Demo {
    fn default() -> Self {
        let confusion = ConfusionMatrixData::from_counts(
            &["Normal", "Fault-A", "Fault-B", "Drift"],
            &[84, 3, 1, 2, 4, 62, 5, 1, 0, 6, 51, 3, 2, 1, 4, 47],
        )
        .expect("confusion");

        let sky = SkyPlotData::from_az_el(
            &[15.0, 82.0, 148.0, 224.0, 305.0, 52.0],
            &[74.0, 41.0, 22.0, 56.0, 11.0, 6.0],
        )
        .expect("sky")
        .with_labels(&["G02", "G14", "G29", "E07", "E19", "B04"])
        .expect("labels")
        .with_groups(&[0, 0, 0, 1, 1, 2])
        .expect("groups")
        .with_group_names(&["GPS", "Galileo", "BeiDou"])
        .with_tracks(vec![
            SkyTrack::new(
                "G14 Pass",
                &[42.0, 58.0, 74.0, 82.0, 96.0, 112.0],
                &[12.0, 26.0, 38.0, 41.0, 35.0, 20.0],
            )
            .expect("track")
            .with_scalars(&[32.0, 39.0, 44.0, 46.0, 43.0, 35.0])
            .expect("scalars")
            .group(0),
        ]);

        let (rows, cols) = (18usize, 26usize);
        let mut xs = Vec::with_capacity(rows * cols);
        let mut ys = Vec::with_capacity(rows * cols);
        let mut zs = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            let rad = 0.6 + 1.6 * (r as f64 / (rows - 1) as f64);
            for c in 0..cols {
                let th = -0.35 * PI + 0.70 * PI * (c as f64 / (cols - 1) as f64);
                let x = rad * th.cos();
                let y = rad * th.sin();
                xs.push(x);
                ys.push(y);
                zs.push((2.2 * rad).sin() * (1.8 * th).cos());
            }
        }
        let curv = CurvilinearField2D::from_nodes(rows, cols, xs, ys, zs).expect("curv");
        let curv_contours =
            ContourData::from_curvilinear(&curv, ContourLevels::Count(7)).expect("curv_c");

        let ridge = |x: f64, y: f64| {
            1.2 * (-18.0 * ((x - 0.3) * (x - 0.3) + (y + 0.25) * (y + 0.25))).exp()
                - 0.7 * (-16.0 * ((x + 0.4) * (x + 0.4) + (y - 0.35) * (y - 0.35))).exp()
        };
        let adaptive_filled = FilledContourData::from_function_adaptive(
            [-1.0, 1.0],
            [-1.0, 1.0],
            ContourLevels::Count(9),
            2_500,
            ridge,
        )
        .expect("adaptive_f");
        let adaptive_lines = ContourData::from_function_adaptive(
            [-1.0, 1.0],
            [-1.0, 1.0],
            ContourLevels::Count(9),
            2_500,
            ridge,
        )
        .expect("adaptive_c");
        let adaptive_surf =
            TriMesh3DData::from_explicit_fn_adaptive([-1.0, 1.0], [-1.0, 1.0], 1_800, ridge)
                .expect("adaptive_surf");

        let l_verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.45, 0.0],
            [0.45, 0.45, 0.0],
            [0.45, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.85],
            [1.0, 0.0, 0.85],
            [1.0, 0.45, 0.85],
            [0.45, 0.45, 0.85],
            [0.45, 1.0, 0.85],
            [0.0, 1.0, 0.85],
        ];
        let f_bot: &[u32] = &[0, 5, 4, 3, 2, 1];
        let f_top: &[u32] = &[6, 7, 8, 9, 10, 11];
        let mut faces: Vec<&[u32]> = vec![f_bot, f_top];
        let sides: [[u32; 4]; 6] = [
            [0, 1, 7, 6],
            [1, 2, 8, 7],
            [2, 3, 9, 8],
            [3, 4, 10, 9],
            [4, 5, 11, 10],
            [5, 0, 6, 11],
        ];
        for s in &sides {
            faces.push(s);
        }
        let v_scals: Vec<f64> = l_verts.iter().map(|p| p[0] + 0.5 * p[1] + p[2]).collect();
        let patch3 = Patch3DData::new(l_verts, &faces)
            .expect("patch3")
            .with_vertex_scalars(&v_scals)
            .expect("v_scals");

        let mut alpha_sites = Vec::new();
        for ix in 0..3 {
            for iy in 0..3 {
                for iz in 0..3 {
                    let p = [ix as f64 * 0.5, iy as f64 * 0.5, iz as f64 * 0.5];
                    if ix == 1 && iy == 1 && iz == 1 {
                        continue;
                    }
                    alpha_sites.push(p);
                }
            }
        }
        let crit_alpha = AlphaShape3DData::critical_alpha(&alpha_sites).expect("crit_alpha");
        let alpha_radius = crit_alpha * 0.72;
        let alpha_data =
            AlphaShape3DData::from_sites(&alpha_sites, alpha_radius).expect("alpha_data");

        let graph3 = Graph3DData::directed(
            7,
            &[
                [0, 1],
                [1, 2],
                [2, 3],
                [3, 0],
                [1, 0],
                [2, 4],
                [4, 5],
                [5, 6],
                [6, 2],
                [0, 0],
            ],
            GraphLayout3D::default(),
        )
        .expect("graph3")
        .with_node_labels(&["Hub", "RX1", "Core", "TX1", "EdgeA", "EdgeB", "Sink"])
        .expect("nlabels")
        .with_node_scalars(&[1.0, 0.7, 0.9, 0.4, 0.5, 0.3, 0.8])
        .expect("nscals");

        Self {
            norm: ConfusionNormalization::Absolute,
            show_summaries: true,
            mask_el_deg: 15.0,
            curv_interp: true,
            zbuffer_3d: true,
            alpha_radius,
            crit_alpha,
            confusion,
            sky,
            curv,
            curv_contours,
            adaptive_filled,
            adaptive_lines,
            adaptive_surf,
            patch3,
            alpha_sites,
            alpha_data,
            graph3,
            plot_states: Default::default(),
            sky_state: SkyPlotState::default(),
            plot3d_states: Default::default(),
        }
    }
}

impl Stage13Demo {
    pub fn reset_views(&mut self) {
        self.plot_states.iter_mut().for_each(PlotState::reset);
        self.sky_state.reset();
        self.plot3d_states.iter_mut().for_each(Plot3DState::reset);
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Confusion, Sky, Curvilinear & 3D Mesh",
            "混淆矩阵、天球、曲线网格与三维面片",
            "混同行列・天球・曲線格子・3D メッシュ",
        ));
        ui.checkbox(
            &mut self.show_summaries,
            tr(l, "Confusion summaries", "混淆汇总条", "混同集計バー"),
        );
        ui.horizontal(|ui| {
            ui.label(tr(l, "Norm:", "归一化:", "正規化:"));
            for (mode, label) in [
                (ConfusionNormalization::Absolute, "#"),
                (ConfusionNormalization::RowNormalized, "Row%"),
                (ConfusionNormalization::ColumnNormalized, "Col%"),
                (ConfusionNormalization::TotalNormalized, "Tot%"),
            ] {
                ui.selectable_value(&mut self.norm, mode, label);
            }
        });
        ui.add(
            egui::Slider::new(&mut self.mask_el_deg, 0.0..=45.0).text(tr(
                l,
                "Mask el°",
                "仰角掩膜°",
                "マスク仰角°",
            )),
        );
        ui.checkbox(
            &mut self.curv_interp,
            tr(l, "Curvilinear interp", "曲线插值着色", "曲線補間"),
        );
        ui.checkbox(
            &mut self.zbuffer_3d,
            tr(l, "3D Z-buffer", "3D Z 缓冲", "3D Zバッファ"),
        );
        let prev_alpha = self.alpha_radius;
        ui.add(
            egui::Slider::new(
                &mut self.alpha_radius,
                0.0..=(self.crit_alpha * 1.4).max(0.5),
            )
            .text("Alpha 3D"),
        );
        if ui.button(tr(l, "Critical α", "临界 α", "臨界 α")).clicked() {
            self.alpha_radius = self.crit_alpha;
        }
        if (self.alpha_radius - prev_alpha).abs() > 1e-6
            && let Ok(updated) = AlphaShape3DData::from_sites(&self.alpha_sites, self.alpha_radius)
        {
            self.alpha_data = updated;
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let depth = if self.zbuffer_3d {
            DepthMode::ZBuffer
        } else {
            DepthMode::SortedPainter
        };
        let scale_curv = ColorScale::new([-1.0, 1.0])
            .expect("range")
            .color_map(ColorMap::Parula);
        let scale_ridge = ColorScale::new([-0.7, 1.2])
            .expect("range")
            .color_map(ColorMap::Turbo);
        let scale_sky = ColorScale::new([30.0, 48.0])
            .expect("range")
            .color_map(ColorMap::Plasma);

        let height = ((ui.available_height() - 96.0) * 0.5).clamp(165.0, 255.0);
        Figure::new("stage13_fig")
            .columns(4)
            .cell_height(height)
            .show(ui, 8, |cell| match cell.index() {
                0 => cell.show_layers(
                    Plot::new("s13_cm")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .confusion_axes(&self.confusion, self.show_summaries)
                        .title("confusionchart"),
                    &mut self.plot_states[0],
                    &[PlotLayer::from(
                        ConfusionChart::new(&self.confusion)
                            .normalization(self.norm)
                            .show_summaries(self.show_summaries),
                    )],
                ),
                1 => cell
                    .show_skyplot(
                        SkyPlot::new("s13_sky", &self.sky)
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title(tr(l, "skyplot", "天空图", "スカイプロット"))
                            .mask_elevation(self.mask_el_deg)
                            .color_scale(scale_sky)
                            .marker_shape(MarkerShape::Pentagram),
                        &mut self.sky_state,
                    )
                    .into_plot_response(),
                2 => cell.show_layers(
                    Plot::new("s13_curv")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .data_aspect(1.0)
                        .title("Curvilinear pcolor + contour"),
                    &mut self.plot_states[1],
                    &[
                        PlotLayer::from(CurvilinearPcolor::new(&self.curv, scale_curv).shading(
                            if self.curv_interp {
                                PcolorShading::Interp
                            } else {
                                PcolorShading::Flat
                            },
                        )),
                        PlotLayer::from(Contours::new(&self.curv_contours)),
                    ],
                ),
                3 => cell.show_layers(
                    Plot::new("s13_adapt2")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .data_aspect(1.0)
                        .title("Adaptive fcontour"),
                    &mut self.plot_states[2],
                    &[
                        PlotLayer::from(FilledContours::new(&self.adaptive_filled, scale_ridge)),
                        PlotLayer::from(Contours::new(&self.adaptive_lines)),
                    ],
                ),
                4 => cell
                    .show_plot3d(
                        Plot3D::new("s13_adapt3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("Adaptive fsurf")
                            .depth_mode(depth),
                        &mut self.plot3d_states[0],
                        &[Layer3D::from(
                            TriSurf3D::new(&self.adaptive_surf)
                                .color_scale(scale_ridge)
                                .shading(SurfaceShading::Interp),
                        )],
                    )
                    .into_plot_response(),
                5 => cell
                    .show_plot3d(
                        Plot3D::new("s13_patch3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("Patch3D Concave + Specular")
                            .depth_mode(depth),
                        &mut self.plot3d_states[1],
                        &[Layer3D::from(
                            Patch3D::new(&self.patch3)
                                .color_scale(scale_curv)
                                .shading(SurfaceShading::Interp)
                                .specular(28.0, 0.35),
                        )],
                    )
                    .into_plot_response(),
                6 => cell
                    .show_plot3d(
                        Plot3D::new("s13_alpha3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title(format!("AlphaShape3D (α={:.2})", self.alpha_radius))
                            .depth_mode(depth),
                        &mut self.plot3d_states[2],
                        &[Layer3D::from(
                            AlphaShape3D::new(&self.alpha_data)
                                .color_scale(scale_curv)
                                .shading(SurfaceShading::Faceted),
                        )],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("s13_graph3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("Graph3D Digraph")
                            .depth_mode(depth),
                        &mut self.plot3d_states[3],
                        &[Layer3D::from(
                            Graph3D::new(&self.graph3)
                                .color_scale(scale_curv)
                                .show_node_labels(true),
                        )],
                    )
                    .into_plot_response(),
            })
            .expect("valid stage13 figure")
            .inner
    }
}
