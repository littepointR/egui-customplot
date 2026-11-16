//! Host-owned Delaunay triangulation, Voronoi, AlphaShape, streamline/streamslice, feather, and spy matrix.
use std::f64::consts::TAU;

use eframe::egui::{self, Color32, Stroke};
use egui_customplot::{
    AlphaShape, AlphaShapeData, ColorMap, ColorScale, Feather, FeatherData, Figure, Plot,
    PlotBounds, PlotResponse, PlotState, Spy, SpyData, StreamlineData, Streamlines, TriMesh2D,
    TriPlot, VectorField2D, Voronoi, VoronoiData,
};

use crate::i18n::{lang, tr};

pub(super) struct GeometryDemo {
    pub enabled: bool,
    pub show_voronoi: bool,
    pub alpha_radius: f32,
    built_alpha: f32,
    tri_mesh: TriMesh2D,
    tri_scalars: Vec<f64>,
    voronoi: VoronoiData,
    alpha_ring: Vec<[f64; 2]>,
    alpha_shape: AlphaShapeData,
    streamslice: StreamlineData,
    feather: FeatherData,
    spy: SpyData,
    states: [PlotState; 5],
}

impl Default for GeometryDemo {
    fn default() -> Self {
        let mut sites = Vec::with_capacity(48);
        for iy in 0..6 {
            for ix in 0..8 {
                let x = ix as f64 + if iy % 2 == 1 { 0.35 } else { 0.0 } + 0.15 * (iy as f64).sin();
                let y = iy as f64 + 0.15 * (ix as f64).cos();
                sites.push([x, y]);
            }
        }
        let tri_mesh = TriMesh2D::delaunay(&sites).expect("valid triangulation sites");
        let tri_scalars: Vec<f64> = tri_mesh
            .vertices()
            .iter()
            .map(|p| (p[0] * 0.6).sin() + (p[1] * 0.7).cos())
            .collect();
        let vor_bounds = PlotBounds::new([-0.5, -0.5], [8.0, 5.8]).expect("valid bounds");
        let voronoi = VoronoiData::from_sites(&sites, vor_bounds).expect("valid voronoi");

        let mut alpha_ring = Vec::with_capacity(72);
        for &r in &[1.6_f64, 3.2_f64] {
            for k in 0..36 {
                let a = TAU * (k as f64 / 36.0);
                alpha_ring.push([4.0 + r * a.cos(), 4.0 + r * a.sin()]);
            }
        }
        let alpha_radius = 1.05_f32;
        let alpha_shape =
            AlphaShapeData::from_sites(&alpha_ring, f64::from(alpha_radius)).expect("valid alpha");

        let n = 28usize;
        let fbounds = PlotBounds::new([-2.0, -2.0], [2.0, 2.0]).expect("valid flow bounds");
        let mut u = Vec::with_capacity(n * n);
        let mut v = Vec::with_capacity(n * n);
        for iy in 0..n {
            let y = -2.0 + 4.0 * (iy as f64 / (n - 1) as f64);
            for ix in 0..n {
                let x = -2.0 + 4.0 * (ix as f64 / (n - 1) as f64);
                u.push(-y - 0.15 * x);
                v.push(x - 0.15 * y);
            }
        }
        let field = VectorField2D::uniform(n, n, fbounds, u, v).expect("valid flow field");
        let streamslice = StreamlineData::streamslice(&field, 0.85).expect("valid streamslice");

        let theta: Vec<f64> = (0..18).map(|i| i as f64 * 0.32).collect();
        let rho: Vec<f64> = (0..18)
            .map(|i| 0.4 + 0.8 * ((i as f64 * 0.25).sin().abs()))
            .collect();
        let feather = FeatherData::from_polar(&theta, &rho)
            .expect("matching lengths")
            .with_baseline(1.0, 1.0, 0.0);

        let size = 64usize;
        let mut entries = Vec::with_capacity(size * 4);
        for i in 0..size {
            entries.push((i, i));
            if i + 1 < size {
                entries.push((i, i + 1));
                entries.push((i + 1, i));
            }
            if i + 12 < size {
                entries.push((i, i + 12));
                entries.push((i + 12, i));
            }
        }
        let spy = SpyData::from_coords(size, size, &entries).expect("valid sparse pattern");

        Self {
            enabled: false,
            show_voronoi: true,
            alpha_radius,
            built_alpha: alpha_radius,
            tri_mesh,
            tri_scalars,
            voronoi,
            alpha_ring,
            alpha_shape,
            streamslice,
            feather,
            spy,
            states: Default::default(),
        }
    }
}

impl GeometryDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.separator();
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "Meshes, flow & spy",
                "三角网、流场与稀疏矩阵 (Spy)",
                "メッシュ・流線・スパイ",
            ),
        );
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(
                &mut self.show_voronoi,
                tr(
                    l,
                    "Overlay Voronoi bisectors",
                    "叠加 Voronoi 垂直平分线",
                    "ボロノイ図を重ね表示",
                ),
            );
            ui.add(
                egui::Slider::new(&mut self.alpha_radius, 0.45..=4.0).text(tr(
                    l,
                    "alpha radius",
                    "alpha 半径",
                    "alpha 半径",
                )),
            );
            if (self.alpha_radius - self.built_alpha).abs() > 1e-4
                && let Ok(rebuilt) =
                    AlphaShapeData::from_sites(&self.alpha_ring, f64::from(self.alpha_radius))
            {
                self.alpha_shape = rebuilt;
                self.built_alpha = self.alpha_radius;
            }
            if ui
                .button(tr(
                    l,
                    "Fit geometry panels",
                    "适配几何面板",
                    "幾何パネルをフィット",
                ))
                .clicked()
            {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small(tr(
                l,
                "Delaunay/Voronoi, alphaShape holes, RK4 streamslice, feather baseline & CSR spy.",
                "Delaunay/Voronoi、带孔 alphaShape、RK4 流线切片、羽状矢量与 CSR 稀疏模式。",
                "ドロネー/ボロノイ・穴あき alphaShape・RK4 流線・フェザー・CSR スパイ。",
            ));
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let tri_scale = ColorScale::new([-2.0, 2.0])
            .expect("valid scale")
            .color_map(ColorMap::Viridis);

        Figure::new("geometry-panels")
            .columns(3)
            .cell_height(height)
            .show(ui, 5, |cell| match cell.index() {
                0 => {
                    let mut layers = vec![
                        TriPlot::new(&self.tri_mesh)
                            .name("trimesh")
                            .vertex_scalars(&self.tri_scalars, tri_scale)
                            .face_fill(Color32::from_rgba_unmultiplied(60, 120, 200, 32))
                            .markers(true)
                            .into(),
                    ];
                    if self.show_voronoi {
                        layers.push(
                            Voronoi::new(&self.voronoi)
                                .name("voronoi")
                                .color(Color32::from_rgb(220, 90, 50))
                                .show_sites(false)
                                .into(),
                        );
                    }
                    cell.show_layers(
                        Plot::new("tri-voronoi")
                            .height(height)
                            .data_aspect(1.0)
                            .allow_scroll_zoom(true)
                            .legend(true)
                            .title(tr(
                                l,
                                "Delaunay trimesh + Voronoi",
                                "Delaunay 三角网 + Voronoi",
                                "ドロネー網＋ボロノイ",
                            )),
                        &mut self.states[0],
                        &layers,
                    )
                }
                1 => cell.show_layers(
                    Plot::new("alpha-shape")
                        .height(height)
                        .data_aspect(1.0)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "alphaShape with Hole",
                            "带孔 alphaShape 轮廓",
                            "穴あき alphaShape",
                        )),
                    &mut self.states[1],
                    &[AlphaShape::new(&self.alpha_shape)
                        .name("alphaShape")
                        .fill_color(Color32::from_rgba_unmultiplied(50, 160, 220, 130))
                        .border(Stroke::new(1.5, Color32::from_rgb(20, 60, 120)))
                        .into()],
                ),
                2 => cell.show_layers(
                    Plot::new("streamslice")
                        .height(height)
                        .data_aspect(1.0)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "streamslice RK4 + Arrows",
                            "RK4 流线与流向箭头",
                            "streamslice RK4＋矢印",
                        )),
                    &mut self.states[2],
                    &[Streamlines::new(&self.streamslice)
                        .name("streamslice")
                        .into()],
                ),
                3 => cell.show_layers(
                    Plot::new("feather")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "feather Vectors",
                            "feather 羽状矢量",
                            "feather ベクトル",
                        )),
                    &mut self.states[3],
                    &[Feather::new(&self.feather).name("feather").into()],
                ),
                _ => cell.show_layers(
                    Plot::new("spy-matrix")
                        .spy_axes()
                        .height(height)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "spy CSR Pattern (64x64)",
                            "spy 稀疏矩阵非零分布 (64x64)",
                            "spy CSR パターン (64x64)",
                        )),
                    &mut self.states[4],
                    &[Spy::new(&self.spy).name("spy(A)").into()],
                ),
            })
            .expect("5 cells within limit")
            .inner
    }
}
