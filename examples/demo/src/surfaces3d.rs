//! Host-owned 3D surface, wireframe mesh, analytic & implicit gallery (`surf`, `surfc`, `surfl`, `surfnorm`, `ribbon`, `contour3`, `mesh`, `meshc`, `meshz`, `waterfall`, `fsurf`, `fmesh`, `fimplicit3`, `ez*`, `sphere`, `cylinder`, `ellipsoid`, `trisurf`).
use std::f64::consts::TAU;

use eframe::egui::{self, Color32};
use egui_customplot::{
    Axis3D, AxisEdge3D, Bounds3D, ColorMap, ColorScale, Contour3D, Contour3DData, ContourLevels,
    DepthMode, Figure, ImplicitSurface3DData, Layer3D, MeshSurface3D, Plot3D, Plot3DState,
    PlotResponse, Point3D, Ribbon3D, Ribbon3DData, Surface3D, Surface3DData, SurfaceShading,
    TriMesh3DData, TriSurf3D,
};

use crate::i18n::{lang, tr};

pub(super) struct Surfaces3DDemo {
    pub shading: SurfaceShading,
    pub zbuffer: bool,
    pub hidden_line: bool,
    pub floor_contours: bool,
    pub show_normals: bool,
    pub colormap_lighting: bool,
    peaks: Surface3DData,
    saddle: Surface3DData,
    torus: Surface3DData,
    contours: Contour3DData,
    ribbon: Ribbon3DData,
    schwarz: ImplicitSurface3DData,
    sphere: Surface3DData,
    cylinder: Surface3DData,
    ellipsoid: Surface3DData,
    trimesh: TriMesh3DData,
    states: [Plot3DState; 6],
}

impl Default for Surfaces3DDemo {
    fn default() -> Self {
        let peaks =
            Surface3DData::from_explicit_fn([-2.4, 2.4], [-2.4, 2.4], [22, 22], 2_000, |x, y| {
                let r2 = x * x + y * y;
                1.4 * (-0.35 * r2).exp() * (x * 1.4).cos() + 0.5 * (y * 1.2).sin()
            })
            .expect("peaks")
            .with_contours(ContourLevels::Count(6))
            .expect("peaks contours");

        let saddle =
            Surface3DData::from_explicit_fn([-1.8, 1.8], [-1.8, 1.8], [16, 16], 1_000, |x, y| {
                0.35 * (x * x - y * y)
            })
            .expect("saddle")
            .with_contours(ContourLevels::Count(5))
            .expect("saddle contours");

        let torus =
            Surface3DData::from_parametric_fn([0.0, TAU], [0.0, TAU], [24, 16], 2_000, |u, v| {
                let r = 1.8 + 0.65 * v.cos();
                [r * u.cos(), r * u.sin(), 0.65 * v.sin()]
            })
            .expect("torus");

        let contours = Contour3DData::from_surface(&peaks, ContourLevels::Count(8)).expect("c3");

        let col_ys: Vec<Vec<f64>> = (0..5)
            .map(|c| {
                (0..20)
                    .map(|r| {
                        let t = r as f64 * 0.25;
                        0.4 + (t + c as f64 * 0.6).sin() * 0.65
                    })
                    .collect()
            })
            .collect();
        let col_refs: Vec<&[f64]> = col_ys.iter().map(Vec::as_slice).collect();
        let ribbon = Ribbon3DData::new(&col_refs, None, 0.58).expect("ribbon");

        let schwarz = ImplicitSurface3DData::from_function(
            Bounds3D::new([-2.8; 3], [2.8; 3]).expect("bounds"),
            [22, 22, 22],
            20_000,
            |x, y, z| x.cos() + y.cos() + z.cos(),
        )
        .expect("fimplicit3");

        let sphere =
            Surface3DData::ellipsoid([-1.8, -0.5, 0.5], [0.65, 0.65, 0.65], 14).expect("sphere");
        let cylinder = Surface3DData::cylinder(&[0.55, 0.28, 0.68, 0.42], 16).expect("cylinder");
        let ellipsoid =
            Surface3DData::ellipsoid([1.8, 0.55, 0.5], [0.85, 0.5, 0.8], 14).expect("ellipsoid");

        let mut dome_pts: Vec<Point3D> = Vec::with_capacity(36);
        for j in 0..6 {
            for i in 0..6 {
                let x = (i as f64 - 2.5) * 0.45;
                let y = (j as f64 - 2.5) * 0.45 + 1.5;
                let z = -0.35 - 0.18 * (x * x + (y - 1.5) * (y - 1.5));
                dome_pts.push([x, y, z]);
            }
        }
        let trimesh = TriMesh3DData::from_delaunay_xy(&dome_pts).expect("trimesh");

        Self {
            shading: SurfaceShading::Interp,
            zbuffer: true,
            hidden_line: true,
            floor_contours: true,
            show_normals: false,
            colormap_lighting: false,
            peaks,
            saddle,
            torus,
            contours,
            ribbon,
            schwarz,
            sphere,
            cylinder,
            ellipsoid,
            trimesh,
            states: Default::default(),
        }
    }
}

impl Surfaces3DDemo {
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(Plot3DState::reset);
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "3D surfaces, meshes & implicit",
            "三维曲面、网格与隐式面",
            "3D 曲面・メッシュ・陰関数",
        ));
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.shading,
                SurfaceShading::Faceted,
                tr(l, "Faceted", "网格面", "ファセット"),
            );
            ui.selectable_value(
                &mut self.shading,
                SurfaceShading::Flat,
                tr(l, "Flat", "平面色", "フラット"),
            );
            ui.selectable_value(
                &mut self.shading,
                SurfaceShading::Interp,
                tr(l, "Interp", "平滑插值", "補間"),
            );
        });
        ui.checkbox(
            &mut self.zbuffer,
            tr(
                l,
                "Software Z-buffer (cell 4 & 5)",
                "软件 Z-Buffer（子图 4/5）",
                "Z バッファ（4・5）",
            ),
        );
        ui.checkbox(
            &mut self.hidden_line,
            tr(
                l,
                "Hidden-line mesh occlusion",
                "线框隐藏线遮挡剔除",
                "陰線消去",
            ),
        );
        ui.checkbox(
            &mut self.floor_contours,
            tr(
                l,
                "Floor contours (surfc / meshc)",
                "底面等高投影 (surfc/meshc)",
                "底面等高線 (surfc/meshc)",
            ),
        );
        ui.checkbox(
            &mut self.show_normals,
            tr(
                l,
                "Surface normals (surfnorm)",
                "曲面法向矢量 (surfnorm)",
                "法線表示 (surfnorm)",
            ),
        );
        ui.checkbox(
            &mut self.colormap_lighting,
            tr(
                l,
                "Colormap lighting (surfl)",
                "光照明暗着色 (surfl)",
                "照明効果 (surfl)",
            ),
        );
        if ui
            .button(tr(
                l,
                "Reset surface views",
                "重置曲面视角",
                "曲面ビュー初期化",
            ))
            .clicked()
        {
            self.states.iter_mut().for_each(Plot3DState::reset);
        }
        ui.small(
            "surf, surfc, surfl, surfnorm, mesh, meshc, meshz, waterfall, contour3, ribbon, fsurf, fmesh, fimplicit3, sphere, cylinder, ellipsoid, trisurf.",
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 120.0) * 0.5).max(120.0);
        let z_scale = ColorScale::new([-1.5, 1.5])
            .expect("z scale")
            .color_map(ColorMap::Viridis);
        let zbuffer_mode = if self.zbuffer {
            DepthMode::ZBuffer
        } else {
            DepthMode::SortedPainter
        };
        let [s0, s1, s2, s3, s4, s5] = &mut self.states;

        Figure::new("surfaces3d-gallery")
            .columns(3)
            .cell_height(height)
            .show(ui, 6, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("s9b-surf")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .z_axis(
                                Axis3D::new()
                                    .label("Height")
                                    .label_edge(AxisEdge3D::new(false, true)),
                            )
                            .color_bar(z_scale, "Z")
                            .title("surf / surfc / surfl / surfnorm"),
                        s0,
                        &[Layer3D::from(
                            Surface3D::new(&self.peaks)
                                .name("peaks")
                                .shading(self.shading)
                                .color_scale(z_scale)
                                .floor_contours(self.floor_contours)
                                .show_normals(self.show_normals, 0.3)
                                .colormap_lighting(self.colormap_lighting),
                        )],
                    )
                    .into_plot_response(),
                1 => cell
                    .show_plot3d(
                        Plot3D::new("s9b-mesh")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("mesh / meshc / meshz / waterfall"),
                        s1,
                        &[
                            Layer3D::from(
                                MeshSurface3D::curtain(&self.peaks)
                                    .name("meshz/c")
                                    .hidden_line(self.hidden_line)
                                    .floor_contours(self.floor_contours)
                                    .color_scale(z_scale),
                            ),
                            Layer3D::from(
                                MeshSurface3D::waterfall(&self.saddle)
                                    .name("waterfall")
                                    .hidden_line(self.hidden_line)
                                    .color(Color32::from_rgb(217, 83, 25)),
                            ),
                        ],
                    )
                    .into_plot_response(),
                2 => cell
                    .show_plot3d(
                        Plot3D::new("s9b-contour-ribbon")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("contour3 & ribbon"),
                        s2,
                        &[
                            Layer3D::from(
                                Contour3D::new(&self.contours)
                                    .name("contour3")
                                    .color_scale(z_scale),
                            ),
                            Layer3D::from(
                                Ribbon3D::new(&self.ribbon)
                                    .name("ribbon")
                                    .shading(self.shading),
                            ),
                        ],
                    )
                    .into_plot_response(),
                3 => cell
                    .show_plot3d(
                        Plot3D::new("s9b-fsurf-fmesh")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("fsurf (torus) & fmesh (saddle)"),
                        s3,
                        &[
                            Layer3D::from(
                                Surface3D::new(&self.torus)
                                    .name("fsurf")
                                    .shading(self.shading)
                                    .color_scale(z_scale),
                            ),
                            Layer3D::from(
                                MeshSurface3D::new(&self.saddle)
                                    .name("fmesh")
                                    .hidden_line(self.hidden_line),
                            ),
                        ],
                    )
                    .into_plot_response(),
                4 => cell
                    .show_plot3d(
                        Plot3D::new("s9b-fimplicit3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .depth_mode(zbuffer_mode)
                            .title("fimplicit3 (cos x + cos y + cos z = 0)"),
                        s4,
                        &[Layer3D::from(
                            TriSurf3D::from_implicit(&self.schwarz).name("schwarz-p"),
                        )],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("s9b-analytic")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .depth_mode(zbuffer_mode)
                            .title("sphere / cylinder / ellipsoid / trisurf"),
                        s5,
                        &[
                            Layer3D::from(
                                Surface3D::new(&self.sphere)
                                    .name("sphere")
                                    .shading(self.shading)
                                    .color(Color32::from_rgb(65, 145, 220)),
                            ),
                            Layer3D::from(
                                Surface3D::new(&self.cylinder)
                                    .name("cylinder")
                                    .shading(self.shading)
                                    .color(Color32::from_rgb(225, 115, 55)),
                            ),
                            Layer3D::from(
                                Surface3D::new(&self.ellipsoid)
                                    .name("ellipsoid")
                                    .shading(self.shading)
                                    .color(Color32::from_rgb(65, 175, 115)),
                            ),
                            Layer3D::from(
                                TriSurf3D::new(&self.trimesh)
                                    .name("trisurf")
                                    .shading(self.shading),
                            ),
                        ],
                    )
                    .into_plot_response(),
            })
            .expect("surfaces3d gallery figure")
            .inner
    }
}
