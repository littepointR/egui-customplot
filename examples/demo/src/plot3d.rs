//! Host-owned 3D gallery (`plot3`, `fplot3`, `ezplot3`, `comet3`, `scatter3`, `bubblechart3`, `swarmchart3`, `bar3`, `bar3h`, `stem3`, `quiver3`, `fill3`, `constantplane`, `pie3`).
use std::{f64::consts::TAU, time::Duration};

use eframe::egui::{self, Color32};
use egui_customplot::{
    Bar3D, Bar3DData, Bar3DMode, Bar3DOrientation, BoxStyle3D, Bubble3D, Bubble3DData,
    BubbleLegend, Camera3D, ColorMap, ColorScale, Comet3D, Comet3DPlayback, ConstantPlane3D,
    DepthMode, Figure, Fill3D, FunctionPlotOptions, Layer3D, Line3D, ParametricCurve3DData, Pie3D,
    Pie3DData, Plot3D, Plot3DState, PlotResponse, Point3D, Point3DData, Polygon3DData,
    Projection3D, Quiver3D, Quiver3DData, Scatter3D, Stem3D, Stem3DData, Swarm3D, Swarm3DData,
    Text3D,
};

use crate::i18n::{lang, tr};

pub(super) struct Plot3DDemo {
    pub perspective: bool,
    pub zbuffer: bool,
    pub bar3h: bool,
    pub animate_comet: bool,
    clock: crate::stream::FrameClock,
    helix: Point3DData,
    fcurve: ParametricCurve3DData,
    ezcurve: ParametricCurve3DData,
    comet_pb: Comet3DPlayback,
    scatter: Point3DData,
    bubble: Bubble3DData,
    swarm: Swarm3DData,
    stem: Stem3DData,
    bars_vert: Bar3DData,
    bars_horiz: Bar3DData,
    quiver: Quiver3DData,
    polys: Polygon3DData,
    pie: Pie3DData,
    states: [Plot3DState; 6],
}

impl Default for Plot3DDemo {
    fn default() -> Self {
        let helix_pts: Vec<Point3D> = (0..=160)
            .map(|i| {
                let t = i as f64 / 160.0 * 3.0 * TAU;
                [t.cos(), t.sin(), t / (3.0 * TAU) * 2.0 - 1.0]
            })
            .collect();
        let mut comet_pb = Comet3DPlayback::new(45.0).tail_fraction(0.28).looping(true);
        comet_pb.set_progress_points(112.0, helix_pts.len());
        let helix = Point3DData::new(helix_pts);

        let fcurve = ParametricCurve3DData::from_parametric(
            [0.0, 2.0 * TAU],
            FunctionPlotOptions::default(),
            |t| {
                [
                    0.65 * (2.0 * t).cos(),
                    0.65 * (2.0 * t).sin(),
                    t / TAU - 1.0,
                ]
            },
        )
        .expect("fplot3");
        let ezcurve = ParametricCurve3DData::ezplot3(800, |t| {
            [
                0.45 * t.cos(),
                0.45 * (2.0 * t).sin(),
                (t / TAU) * 1.4 - 0.7,
            ]
        })
        .expect("ezplot3");

        let s_pts: Vec<Point3D> = (0..48)
            .map(|i| {
                let t = i as f64 * 0.25;
                [t.cos() * 0.85, t.sin() * 0.85, (t * 0.4).sin()]
            })
            .collect();
        let s_scal: Vec<f64> = (0..48).map(|i| i as f64 / 47.0).collect();
        let scatter = Point3DData::new(s_pts.clone())
            .with_scalars(&s_scal)
            .expect("scatter3 scalars");
        let bub_sz: Vec<f64> = (0..24).map(|i| 10.0 + ((i * 7) % 9) as f64 * 9.0).collect();
        let bubble = Bubble3DData::new(&s_pts[..24], &bub_sz)
            .expect("bubble3")
            .with_scalars(&s_scal[..24])
            .expect("bubble3 scalars")
            .size_range([4.0, 18.0]);

        let g0: Vec<f64> = (0..28).map(|i| (i as f64 * 0.37).sin()).collect();
        let g1: Vec<f64> = (0..28)
            .map(|i| 0.35 + (i as f64 * 0.51).cos() * 0.8)
            .collect();
        let swarm = Swarm3DData::from_groups(&[[-0.7, 0.0], [0.7, 0.4]], &[&g0, &g1], 0.36)
            .expect("swarm3");
        let stem_pts: Vec<Point3D> = (0..12)
            .map(|i| {
                let a = i as f64 * TAU / 12.0;
                [0.65 * a.cos(), 0.65 * a.sin(), 0.4 + 0.55 * (2.0 * a).sin()]
            })
            .collect();
        let stem = Stem3DData::new(stem_pts, -0.4).expect("stem3");

        let mat = [0.8, 1.4, 2.1, 1.6, 2.5, 1.2, 2.2, 1.8, 2.8, 1.1, 2.0, 1.7];
        let bars_vert = Bar3DData::from_matrix(
            &mat,
            4,
            3,
            Bar3DMode::Grouped,
            Bar3DOrientation::Vertical,
            0.72,
        )
        .expect("bar3");
        let bars_horiz = Bar3DData::from_matrix(
            &mat,
            4,
            3,
            Bar3DMode::Grouped,
            Bar3DOrientation::Horizontal,
            0.72,
        )
        .expect("bar3h");

        let qx = [-0.6, 0.0, 0.6, -0.6, 0.0, 0.6];
        let qy = [-0.5, -0.5, -0.5, 0.5, 0.5, 0.5];
        let qz = [-0.4, 0.0, 0.4, -0.2, 0.2, 0.5];
        let qu = [0.3, -0.4, 0.3, 0.4, 0.2, -0.3];
        let qv = [0.5, 0.4, -0.3, -0.3, 0.5, 0.2];
        let qw = [0.6, 0.5, 0.4, 0.5, 0.3, 0.6];
        let quiver = Quiver3DData::from_components(&qx, &qy, &qz, &qu, &qv, &qw)
            .expect("quiver3")
            .autoscale(0.95);

        let quad_a = [
            [-0.75, -0.75, -0.75],
            [0.75, -0.75, 0.75],
            [0.75, 0.75, 0.75],
            [-0.75, 0.75, -0.75],
        ];
        let quad_b = [
            [-0.75, -0.75, 0.75],
            [0.75, -0.75, -0.75],
            [0.75, 0.75, -0.75],
            [-0.75, 0.75, 0.75],
        ];
        let polys = Polygon3DData::from_polygons(&[&quad_a, &quad_b]).expect("fill3");
        let pie = Pie3DData::with_options(
            &[32.0, 26.0, 22.0, 20.0],
            &[false, true, false, false],
            &["North", "East", "South", "West"],
            0.34,
        )
        .expect("pie3");

        Self {
            perspective: false,
            zbuffer: true,
            bar3h: false,
            animate_comet: false,
            clock: crate::stream::FrameClock::default(),
            helix,
            fcurve,
            ezcurve,
            comet_pb,
            scatter,
            bubble,
            swarm,
            stem,
            bars_vert,
            bars_horiz,
            quiver,
            polys,
            pie,
            states: Default::default(),
        }
    }
}

impl Plot3DDemo {
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(Plot3DState::reset);
    }

    #[cfg(test)]
    pub fn comet_progress(&self) -> f64 {
        self.comet_pb.progress_points()
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "3D core, discrete & planes",
            "三维基础、离散柱杆与平面",
            "3D 基本・離散・平面",
        ));
        ui.checkbox(
            &mut self.perspective,
            tr(l, "Perspective projection", "透视投影", "透視投影"),
        );
        ui.checkbox(
            &mut self.zbuffer,
            tr(
                l,
                "Software Z-buffer (cell 5)",
                "软件深度 Z-Buffer（子图 5）",
                "ソフト Z バッファ（セル 5）",
            ),
        );
        ui.checkbox(
            &mut self.bar3h,
            tr(
                l,
                "Horizontal bars (bar3h)",
                "水平三维柱 (bar3h)",
                "横向き棒 (bar3h)",
            ),
        );
        ui.checkbox(
            &mut self.animate_comet,
            tr(
                l,
                "Animate comet3",
                "三维彗星轨迹动画 (comet3)",
                "comet3 アニメーション",
            ),
        );
        if ui
            .button(tr(
                l,
                "Reset 3D cameras",
                "重置 3D 相机视角",
                "3D カメラ初期化",
            ))
            .clicked()
        {
            self.states.iter_mut().for_each(Plot3DState::reset);
        }
        ui.small(tr(
            l,
            "plot3, fplot3, ezplot3, comet3, scatter3, bubblechart3, swarmchart3, bar3/bar3h, stem3, quiver3, fill3, constantplane, pie3.",
            "覆盖三维曲线、散点气泡、蜂群杆、柱、矢量、交叉面片与 3D 饼图。",
            "3D 線・散布・バブル・棒・ベクトル・平面・3D 円グラフ。",
        ));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let dt = self.clock.tick(ui.ctx(), self.animate_comet);
        if self.animate_comet {
            self.comet_pb.set_playing(true);
            let n = self.helix.len();
            if dt > 0.0 {
                self.comet_pb
                    .advance_once(ui.ctx().cumulative_frame_nr(), dt, n);
            }
            self.comet_pb
                .sync_repaint(ui.ctx(), n, Duration::from_millis(16));
        }
        let height = ((ui.available_height() - 120.0) * 0.5).max(120.0);
        let proj = if self.perspective {
            Projection3D::perspective_default()
        } else {
            Projection3D::Orthographic
        };
        let cscale = ColorScale::new([0.0, 1.0])
            .expect("unit color scale")
            .color_map(ColorMap::Viridis);
        let bar_scale = ColorScale::new([0.8, 2.8])
            .expect("bar color scale")
            .color_map(ColorMap::Viridis);
        let b_legend = BubbleLegend::new(self.bubble.size_extent(), self.bubble.radius_range())
            .steps(3)
            .title(tr(l, "Bubble", "气泡尺寸", "バブル"));

        let bars = if self.bar3h {
            &self.bars_horiz
        } else {
            &self.bars_vert
        };
        let depth_mode = if self.zbuffer {
            DepthMode::ZBuffer
        } else {
            DepthMode::SortedPainter
        };

        let [s0, s1, s2, s3, s4, s5] = &mut self.states;
        Figure::new("plot3d-gallery")
            .columns(3)
            .cell_height(height)
            .show(ui, 6, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("3d-lines")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .projection(proj)
                            .x_label("X")
                            .y_label("Y")
                            .z_label("Z")
                            .texts([Text3D::new([0.0, 0.0, 1.0], "Top")])
                            .title("plot3 / fplot3 / ezplot3 / comet3"),
                        s0,
                        &[
                            Layer3D::from(
                                Line3D::new(&self.helix)
                                    .name("plot3")
                                    .color(Color32::from_rgb(0, 114, 189)),
                            ),
                            Layer3D::from(
                                Line3D::new(self.fcurve.points())
                                    .name("fplot3")
                                    .color(Color32::from_rgb(217, 83, 25)),
                            ),
                            Layer3D::from(
                                Line3D::new(self.ezcurve.points())
                                    .name("ezplot3")
                                    .color(Color32::from_rgb(126, 47, 142)),
                            ),
                            Layer3D::from(
                                Comet3D::new(&self.helix, &self.comet_pb)
                                    .name("comet3")
                                    .tail_color(Color32::from_rgb(237, 177, 32)),
                            ),
                        ],
                    )
                    .into_plot_response(),
                1 => cell
                    .show_plot3d(
                        Plot3D::new("3d-scatter-bubble")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .projection(proj)
                            .bubble_legend(b_legend.clone())
                            .title("scatter3 & bubblechart3"),
                        s1,
                        &[
                            Layer3D::from(
                                Scatter3D::new(&self.scatter)
                                    .color_scale(cscale)
                                    .name("scatter3"),
                            ),
                            Layer3D::from(
                                Bubble3D::new(&self.bubble)
                                    .color_scale(cscale)
                                    .name("bubblechart3"),
                            ),
                        ],
                    )
                    .into_plot_response(),
                2 => cell
                    .show_plot3d(
                        Plot3D::new("3d-swarm-stem")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .projection(proj)
                            .title("swarmchart3 & stem3"),
                        s2,
                        &[
                            Layer3D::from(Swarm3D::new(&self.swarm).name("swarmchart3")),
                            Layer3D::from(
                                Stem3D::new(&self.stem)
                                    .name("stem3")
                                    .color(Color32::from_rgb(217, 83, 25)),
                            ),
                        ],
                    )
                    .into_plot_response(),
                3 => cell
                    .show_plot3d(
                        Plot3D::new("3d-bar3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .projection(proj)
                            .title(if self.bar3h {
                                "bar3h (Horizontal)"
                            } else {
                                "bar3 (Grouped)"
                            }),
                        s3,
                        &[Layer3D::from(
                            Bar3D::new(bars).color_scale(bar_scale).name(if self.bar3h {
                                "bar3h"
                            } else {
                                "bar3"
                            }),
                        )],
                    )
                    .into_plot_response(),
                4 => cell
                    .show_plot3d(
                        Plot3D::new("3d-quiver-fill-planes")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .projection(proj)
                            .depth_mode(depth_mode)
                            .constant_planes([
                                ConstantPlane3D::z_plane(0.0).label("z=0"),
                                ConstantPlane3D::new([1.0, 1.0, 1.0], 0.2).label("x+y+z=0.2"),
                            ])
                            .title("quiver3 / fill3 / constantplane"),
                        s4,
                        &[
                            Layer3D::from(
                                Fill3D::new(&self.polys)
                                    .face_colors(&[
                                        Color32::from_rgb(220, 60, 50),
                                        Color32::from_rgb(45, 110, 225),
                                    ])
                                    .alpha(1.0)
                                    .name("fill3"),
                            ),
                            Layer3D::from(
                                Quiver3D::new(&self.quiver)
                                    .name("quiver3")
                                    .color(Color32::from_rgb(40, 140, 75)),
                            ),
                        ],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("3d-pie3")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .projection(proj)
                            .camera(Camera3D::new(-40.0, 34.0))
                            .box_style(BoxStyle3D::AxesOnly)
                            .title("pie3 (Extruded Sectors)"),
                        s5,
                        &[Layer3D::from(Pie3D::new(&self.pie).name("pie3"))],
                    )
                    .into_plot_response(),
            })
            .expect("3d gallery figure")
            .inner
    }
}
