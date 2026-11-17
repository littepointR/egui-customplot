//! Host-owned 3D volumetric slicing, tetrahedral, isosurface & flow gallery (`slice`, `contourslice`, `isosurface`, `isocaps`, `tetramesh`, `streamline` 3D, `streamslice` 3D, `streamparticles`, `streamribbon`, `streamtube`, `coneplot`).
use std::time::Duration;

use eframe::egui::{self, Color32};
use egui_customplot::{
    Bounds3D, ColorMap, ColorScale, ConePlot3D, ConePlot3DData, Contour3D, Contour3DData,
    ContourLevels, DepthMode, Figure, IsocapThreshold, Isocaps3D, Isocaps3DData, Isosurface3D,
    Isosurface3DData, Layer3D, Plot3D, Plot3DState, PlotResponse, Quiver3D, Quiver3DData,
    ScalarVolume3D, SlicePlane3D, Stream3DOptions, StreamParticles3D, StreamParticles3DData,
    StreamParticlesPlayback, StreamRibbon3D, StreamRibbon3DData, StreamTube3D, StreamTube3DData,
    Streamline3D, Streamline3DData, TetraMesh3D, TetraMesh3DData, VectorField3D, VolumeSlice3D,
    VolumeSliceData,
};

use crate::i18n::{lang, tr};

pub(super) struct Volumes3DDemo {
    pub zbuffer: bool,
    pub boundary_only: bool,
    pub animate_particles: bool,
    pub arbitrary_slice: bool,
    clock: crate::stream::FrameClock,
    slices_ortho: VolumeSliceData,
    slices_all: VolumeSliceData,
    cslice: Contour3DData,
    iso: Isosurface3DData,
    caps: Isocaps3DData,
    tetramesh: TetraMesh3DData,
    streamlines: Streamline3DData,
    slice_lines: Streamline3DData,
    slice_arrows: Quiver3DData,
    particles: StreamParticles3DData,
    playback: StreamParticlesPlayback,
    ribbon: StreamRibbon3DData,
    tube: StreamTube3DData,
    cones: ConePlot3DData,
    states: [Plot3DState; 6],
}

impl Default for Volumes3DDemo {
    fn default() -> Self {
        let bounds = Bounds3D::new([-0.75, -0.75, -0.75], [1.35, 1.35, 1.35]).expect("vol bounds");
        let vol = ScalarVolume3D::from_function(bounds, [18, 18, 18], 20_000, |x, y, z| {
            x * x + y * y + z * z
        })
        .expect("vol");
        let col_vol =
            ScalarVolume3D::from_function(bounds, [10, 10, 10], 5_000, |x, y, z| x + 0.5 * y + z)
                .expect("col_vol");

        let slices_ortho = VolumeSliceData::from_volume(
            &vol,
            &[
                SlicePlane3D::X(0.0),
                SlicePlane3D::Y(0.0),
                SlicePlane3D::Z(0.0),
            ],
        )
        .expect("slices_ortho");
        let slices_all = VolumeSliceData::from_volume(
            &vol,
            &[
                SlicePlane3D::X(0.2),
                SlicePlane3D::Z(0.2),
                SlicePlane3D::Arbitrary {
                    normal: [1.0, 1.0, 1.0],
                    offset: 0.4,
                    resolution: 18,
                },
            ],
        )
        .expect("slices_all");
        let cslice = Contour3DData::from_volume_slices(
            &vol,
            &[
                SlicePlane3D::X(0.25),
                SlicePlane3D::Y(-0.25),
                SlicePlane3D::Z(0.1),
            ],
            ContourLevels::Count(6),
        )
        .expect("contourslice");

        let iso = Isosurface3DData::from_volume(&vol, 1.0)
            .expect("iso")
            .with_colors_from_volume(&col_vol);
        let caps = Isocaps3DData::from_volume(&vol, 1.0, IsocapThreshold::Below).expect("caps");

        let tetramesh = build_cube_tetramesh();

        let flow_b = Bounds3D::new([-1.5; 3], [1.5; 3]).expect("flow bounds");
        let vf = VectorField3D::from_function(flow_b, [12, 12, 12], 5_000, |x, y, _| {
            [-y + 0.15 * x, x + 0.15 * y, 0.7]
        })
        .expect("vf3d");
        let opts = Stream3DOptions {
            step_size: 0.06,
            max_steps: 90,
            ..Default::default()
        };
        let streamlines = Streamline3DData::from_field(
            &vf,
            &[[0.55, 0.0, -1.2], [0.0, 0.85, -1.2], [-0.7, -0.3, -1.2]],
            opts,
        )
        .expect("sl3d");
        let (slice_lines, slice_arrows) =
            Streamline3DData::streamslice(&vf, SlicePlane3D::Z(0.0), 4).expect("streamslice3d");
        let particles = StreamParticles3DData::from_streamlines(&streamlines, 6);
        let mut playback = StreamParticlesPlayback::new(0.45);
        playback.set_phase(0.2);

        let ribbon = StreamRibbon3DData::from_field(&vf, &[[0.65, 0.0, -1.2]], 0.22, 1.0, opts)
            .expect("ribbon3d");
        let tube = StreamTube3DData::from_field(&vf, &[[-0.65, 0.0, -1.2]], 0.14, 10, opts)
            .expect("tube3d");
        let mut cone_pts = Vec::new();
        for &z in &[-0.7, 0.4] {
            for &y in &[-0.8, 0.8] {
                for &x in &[-0.8, 0.8] {
                    cone_pts.push([x, y, z]);
                }
            }
        }
        let cones = ConePlot3DData::from_field(&vf, &cone_pts, 0.22, 8).expect("cones");

        Self {
            zbuffer: true,
            boundary_only: true,
            animate_particles: false,
            arbitrary_slice: true,
            clock: crate::stream::FrameClock::default(),
            slices_ortho,
            slices_all,
            cslice,
            iso,
            caps,
            tetramesh,
            streamlines,
            slice_lines,
            slice_arrows,
            particles,
            playback,
            ribbon,
            tube,
            cones,
            states: Default::default(),
        }
    }
}

fn build_cube_tetramesh() -> TetraMesh3DData {
    let mut verts = Vec::with_capacity(27);
    for kz in 0..3 {
        for ky in 0..3 {
            for kx in 0..3 {
                verts.push([kx as f64 - 1.0, ky as f64 - 1.0, kz as f64 - 1.0]);
            }
        }
    }
    let idx = |x: u32, y: u32, z: u32| (z * 3 + y) * 3 + x;
    let mut tets = Vec::with_capacity(48);
    let kuhn = [
        [0, 1, 3, 7],
        [0, 3, 2, 7],
        [0, 2, 6, 7],
        [0, 6, 4, 7],
        [0, 4, 5, 7],
        [0, 5, 1, 7],
    ];
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let c = [
                    idx(x, y, z),
                    idx(x + 1, y, z),
                    idx(x, y + 1, z),
                    idx(x + 1, y + 1, z),
                    idx(x, y, z + 1),
                    idx(x + 1, y, z + 1),
                    idx(x, y + 1, z + 1),
                    idx(x + 1, y + 1, z + 1),
                ];
                for kt in kuhn {
                    tets.push(kt.map(|k| c[k]));
                }
            }
        }
    }
    let scals: Vec<f64> = (0..tets.len()).map(|i| (i % 6) as f64 / 5.0).collect();
    TetraMesh3DData::new(verts, tets)
        .expect("cube tetramesh")
        .with_cell_scalars(&scals)
        .expect("tet scalars")
}

impl Volumes3DDemo {
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(Plot3DState::reset);
    }

    #[cfg(test)]
    pub fn particle_phase(&self) -> f64 {
        self.playback.phase()
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "3D volumes, isosurfaces & flow",
            "三维体切片、等值面与体流场",
            "3D ボリューム・等値面・流れ",
        ));
        ui.checkbox(
            &mut self.zbuffer,
            tr(
                l,
                "Software Z-buffer (cell 2 & 5)",
                "软件 Z-Buffer（子图 2/5）",
                "Z バッファ（2・5）",
            ),
        );
        ui.checkbox(
            &mut self.arbitrary_slice,
            tr(
                l,
                "Tilted arbitrary slice (cell 0)",
                "倾斜任意体切片（子图 0）",
                "任意傾斜スライス（セル 0）",
            ),
        );
        ui.checkbox(
            &mut self.boundary_only,
            tr(
                l,
                "Boundary-only tetramesh (cell 3)",
                "四面体仅绘外表面（子图 3）",
                "外表面のみ表示（セル 3）",
            ),
        );
        ui.checkbox(
            &mut self.animate_particles,
            tr(
                l,
                "Animate streamparticles (cell 4)",
                "流场粒子动画（子图 4）",
                "流れ粒子アニメ（セル 4）",
            ),
        );
        if ui
            .button(tr(
                l,
                "Reset volume views",
                "重置体视图",
                "ボリューム初期化",
            ))
            .clicked()
        {
            self.states.iter_mut().for_each(Plot3DState::reset);
        }
        ui.small(
            "slice, contourslice, isosurface, isocaps, tetramesh, streamline/streamslice 3D, streamparticles, streamribbon, streamtube, coneplot.",
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let dt = self.clock.tick(ui.ctx(), self.animate_particles);
        if self.animate_particles {
            self.playback.set_playing(true);
            if dt > 0.0 {
                self.playback
                    .advance_once(ui.ctx().cumulative_frame_nr(), dt);
            }
            self.playback
                .sync_repaint(ui.ctx(), Duration::from_millis(16));
        }
        let height = ((ui.available_height() - 120.0) * 0.5).max(120.0);
        let v_scale = ColorScale::new([0.0, 4.0])
            .expect("v scale")
            .color_map(ColorMap::Viridis);
        let zbuf = if self.zbuffer {
            DepthMode::ZBuffer
        } else {
            DepthMode::SortedPainter
        };
        let slice_data = if self.arbitrary_slice {
            &self.slices_all
        } else {
            &self.slices_ortho
        };
        let [s0, s1, s2, s3, s4, s5] = &mut self.states;

        Figure::new("volumes3d-gallery")
            .columns(3)
            .cell_height(height)
            .show(ui, 6, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("s9c-slice")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .color_bar(v_scale, "V")
                            .title("slice (orthogonal + arbitrary)"),
                        s0,
                        &[Layer3D::from(
                            VolumeSlice3D::new(slice_data)
                                .name("slice")
                                .color_scale(v_scale),
                        )],
                    )
                    .into_plot_response(),
                1 => cell
                    .show_plot3d(
                        Plot3D::new("s9c-contourslice")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("contourslice"),
                        s1,
                        &[Layer3D::from(
                            Contour3D::new(&self.cslice)
                                .name("contourslice")
                                .color_scale(v_scale),
                        )],
                    )
                    .into_plot_response(),
                2 => cell
                    .show_plot3d(
                        Plot3D::new("s9c-iso-caps")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .depth_mode(zbuf)
                            .title("isosurface & isocaps"),
                        s2,
                        &[
                            Layer3D::from(
                                Isosurface3D::new(&self.iso)
                                    .name("isosurface")
                                    .color_scale(ColorScale::new([-1.5, 2.5]).expect("cs")),
                            ),
                            Layer3D::from(
                                Isocaps3D::new(&self.caps)
                                    .name("isocaps")
                                    .color_scale(ColorScale::new([0.0, 1.0]).expect("cs2")),
                            ),
                        ],
                    )
                    .into_plot_response(),
                3 => cell
                    .show_plot3d(
                        Plot3D::new("s9c-tetramesh")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("tetramesh (Kuhn 6-tet cube)"),
                        s3,
                        &[Layer3D::from(
                            TetraMesh3D::new(&self.tetramesh)
                                .name("tetramesh")
                                .boundary_only(self.boundary_only),
                        )],
                    )
                    .into_plot_response(),
                4 => cell
                    .show_plot3d(
                        Plot3D::new("s9c-stream-particles")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title("streamline / streamparticles / streamslice"),
                        s4,
                        &[
                            Layer3D::from(Streamline3D::new(&self.streamlines).name("streamline")),
                            Layer3D::from(
                                Streamline3D::new(&self.slice_lines)
                                    .name("streamslice")
                                    .color(Color32::from_rgb(130, 130, 145)),
                            ),
                            Layer3D::from(Quiver3D::new(&self.slice_arrows).name("slice arrows")),
                            Layer3D::from(
                                StreamParticles3D::new(&self.particles, &self.playback)
                                    .name("streamparticles"),
                            ),
                        ],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_plot3d(
                        Plot3D::new("s9c-ribbon-tube-cone")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .depth_mode(zbuf)
                            .title("streamribbon / streamtube / coneplot"),
                        s5,
                        &[
                            Layer3D::from(StreamRibbon3D::new(&self.ribbon).name("streamribbon")),
                            Layer3D::from(StreamTube3D::new(&self.tube).name("streamtube")),
                            Layer3D::from(ConePlot3D::new(&self.cones).name("coneplot")),
                        ],
                    )
                    .into_plot_response(),
            })
            .expect("volumes3d gallery figure")
            .inner
    }
}
