//! Stage 16 completion gallery (`--stage16`, `EXT-23..26`):
//! volshow, pcshow, geoglobe, Radar, Sankey, Treemap, Sunburst, Gantt.
use std::f64::consts::TAU;

use eframe::egui::{self, Color32};
use egui_customplot::{
    Bounds3D, ColorMap, DepthMode, Figure, GanttChart, GanttData, GanttTask, GeoGlobeData,
    GeoGlobePlot, GeoGlobeState, GlobeMarker3D, Layer3D, Plot, Plot3D, Plot3DState, PlotLayer,
    PlotResponse, PlotState, PointCloud3D, PointCloud3DData, RadarChart, RadarData, SankeyChart,
    SankeyData, ScalarVolume3D, SunburstChart, SunburstData, SunburstInput, Surface3D,
    Surface3DData, TransferStop, TreemapChart, TreemapData, VolumeRender3D, VolumeRenderMode,
    VolumeTransferFunction,
};

use crate::i18n::{lang, tr};

pub(super) struct Stage16Demo {
    pub vol_mode: usize,
    pub show_globe_grid: bool,
    pub show_gantt_deps: bool,
    volume: ScalarVolume3D,
    occlusion_surf: Surface3DData,
    cloud: PointCloud3DData,
    globe: GeoGlobeData,
    radar: RadarData,
    sankey: SankeyData,
    treemap: TreemapData,
    sunburst: SunburstData,
    gantt: GanttData,
    vol_state: Plot3DState,
    cloud_state: Plot3DState,
    globe_state: GeoGlobeState,
    states: [PlotState; 5],
}

impl Default for Stage16Demo {
    fn default() -> Self {
        let bounds = Bounds3D::new([-1.2, -1.2, -1.2], [1.2, 1.2, 1.2]).expect("vol bounds");
        let volume = ScalarVolume3D::from_function(bounds, [28, 28, 28], 32_000, |x, y, z| {
            let r1 = (x * x + y * y + z * z).sqrt();
            let torus = ((x * x + y * y).sqrt() - 0.62).hypot(z);
            (-3.2 * r1 * r1).exp() + 0.85 * (-14.0 * torus * torus).exp()
        })
        .expect("volume");
        let xs: Vec<f64> = (-6..=6).map(|i| i as f64 * 0.2).collect();
        let ys: Vec<f64> = (-6..=6).map(|i| i as f64 * 0.2).collect();
        let mut zs = Vec::with_capacity(xs.len() * ys.len());
        for &y in &ys {
            for &x in &xs {
                zs.push(-0.65 + 0.12 * (2.0 * x).sin() * (2.0 * y).cos());
            }
        }
        let occlusion_surf = Surface3DData::from_grid(&xs, &ys, &zs).expect("surf");

        let n_pts = 40_000usize;
        let mut pts = Vec::with_capacity(n_pts);
        let mut rgb = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let t = i as f64 / n_pts as f64;
            let ang = 18.0 * TAU * t;
            let rad = 0.25 + 0.9 * t;
            let z = 1.6 * (t - 0.5) + 0.14 * (7.0 * ang).sin();
            pts.push([rad * ang.cos(), rad * ang.sin(), z]);
            let c = ColorMap::Turbo.sample(t);
            rgb.push([c.r(), c.g(), c.b()]);
        }
        let cloud = PointCloud3DData::from_xyz(pts)
            .expect("cloud")
            .with_rgb(rgb)
            .expect("rgb");

        let n_arc = 36usize;
        let mut arc_lat = Vec::with_capacity(n_arc);
        let mut arc_lon = Vec::with_capacity(n_arc);
        let mut arc_alt = Vec::with_capacity(n_arc);
        for i in 0..n_arc {
            let t = i as f64 / (n_arc - 1) as f64;
            arc_lat.push(31.2 + (51.5 - 31.2) * t + 11.0 * (std::f64::consts::PI * t).sin());
            arc_lon.push(121.5 - 121.6 * t);
            arc_alt.push(620_000.0 * (std::f64::consts::PI * t).sin());
        }
        let globe = GeoGlobeData::new()
            .alt_exaggeration(1.5)
            .with_extruded_track(
                "PVG-LHR LEO",
                &arc_lat,
                &arc_lon,
                &arc_alt,
                Color32::from_rgb(255, 180, 60),
            )
            .expect("track")
            .with_markers(vec![
                GlobeMarker3D {
                    label: "Shanghai".into(),
                    lat_deg: 31.23,
                    lon_deg: 121.47,
                    alt_m: 12_000.0,
                    radius_px: 5.5,
                    color: Color32::from_rgb(0, 210, 190),
                },
                GlobeMarker3D {
                    label: "Tokyo".into(),
                    lat_deg: 35.68,
                    lon_deg: 139.76,
                    alt_m: 12_000.0,
                    radius_px: 5.0,
                    color: Color32::from_rgb(90, 170, 255),
                },
                GlobeMarker3D {
                    label: "London".into(),
                    lat_deg: 51.51,
                    lon_deg: -0.13,
                    alt_m: 12_000.0,
                    radius_px: 5.0,
                    color: Color32::from_rgb(235, 95, 95),
                },
            ])
            .expect("markers");

        let radar = RadarData::new(
            &["FPS", "Memory", "Pick", "Export", "3D Depth", "Geo"],
            &[100.0; 6],
            &[
                ("Engine A", &[88.0, 82.0, 94.0, 91.0, 86.0, 84.0]),
                ("Baseline", &[62.0, 70.0, 58.0, 65.0, 52.0, 60.0]),
            ],
        )
        .expect("radar");

        let sankey = SankeyData::new(
            &["Solar", "Wind", "Hydro", "Grid", "Industry", "Home", "EV"],
            &[
                (0, 3, 42.0),
                (1, 3, 36.0),
                (2, 3, 22.0),
                (3, 4, 48.0),
                (3, 5, 32.0),
                (3, 6, 20.0),
            ],
        )
        .expect("sankey");

        let treemap = TreemapData::squarified(&[
            ("Compute", 0, 42.0),
            ("Render", 0, 28.0),
            ("Data", 1, 22.0),
            ("Spatial", 1, 18.0),
            ("Control", 2, 16.0),
            ("DSP", 2, 14.0),
            ("Modern", 3, 12.0),
        ])
        .expect("treemap");

        let sunburst = SunburstData::new(&[
            SunburstInput {
                label: "Plot".into(),
                parent: None,
                value: 0.0,
                group: 0,
            },
            SunburstInput {
                label: "2D".into(),
                parent: Some(0),
                value: 0.0,
                group: 1,
            },
            SunburstInput {
                label: "3D".into(),
                parent: Some(0),
                value: 0.0,
                group: 2,
            },
            SunburstInput {
                label: "Stats".into(),
                parent: Some(1),
                value: 38.0,
                group: 1,
            },
            SunburstInput {
                label: "DSP/RF".into(),
                parent: Some(1),
                value: 30.0,
                group: 1,
            },
            SunburstInput {
                label: "Volume".into(),
                parent: Some(2),
                value: 22.0,
                group: 2,
            },
            SunburstInput {
                label: "Globe".into(),
                parent: Some(2),
                value: 18.0,
                group: 2,
            },
        ])
        .expect("sunburst");

        let gantt = GanttData::new(
            vec![
                GanttTask {
                    label: "Spec".into(),
                    start: 0.0,
                    end: 3.0,
                    progress: 1.0,
                    group: 0,
                    milestone: false,
                },
                GanttTask {
                    label: "Vol/PC3D".into(),
                    start: 2.5,
                    end: 7.0,
                    progress: 1.0,
                    group: 1,
                    milestone: false,
                },
                GanttTask {
                    label: "Globe".into(),
                    start: 5.0,
                    end: 9.0,
                    progress: 0.9,
                    group: 2,
                    milestone: false,
                },
                GanttTask {
                    label: "Modern".into(),
                    start: 6.5,
                    end: 11.0,
                    progress: 0.85,
                    group: 3,
                    milestone: false,
                },
                GanttTask {
                    label: "v1.0".into(),
                    start: 11.0,
                    end: 11.0,
                    progress: 1.0,
                    group: 4,
                    milestone: true,
                },
            ],
            vec![[0, 1], [1, 2], [2, 3], [3, 4]],
        )
        .expect("gantt");

        Self {
            vol_mode: 0,
            show_globe_grid: true,
            show_gantt_deps: true,
            volume,
            occlusion_surf,
            cloud,
            globe,
            radar,
            sankey,
            treemap,
            sunburst,
            gantt,
            vol_state: Plot3DState::default(),
            cloud_state: Plot3DState::default(),
            globe_state: GeoGlobeState::default(),
            states: std::array::from_fn(|_| PlotState::default()),
        }
    }
}

impl Stage16Demo {
    pub fn reset_views(&mut self) {
        self.vol_state.reset();
        self.cloud_state.reset();
        self.globe_state.reset();
        self.states.iter_mut().for_each(PlotState::reset);
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Volume, PointCloud, Globe & Modern",
            "体渲染、三维点云、地球仪与现代图",
            "体描画・点群・地球儀・モダン図",
        ));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.vol_mode, 0, "Alpha");
            ui.selectable_value(&mut self.vol_mode, 1, "MIP");
            ui.selectable_value(&mut self.vol_mode, 2, "IsoRay");
        });
        ui.checkbox(
            &mut self.show_globe_grid,
            tr(
                l,
                "Globe Graticule & Coast",
                "地球经纬与海岸线",
                "地球グリッド線",
            ),
        );
        ui.checkbox(
            &mut self.show_gantt_deps,
            tr(l, "Gantt Dependencies", "甘特依赖箭头", "ガント依存線"),
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 96.0) * 0.5).clamp(165.0, 270.0);
        let tf = VolumeTransferFunction::from_stops(&[
            TransferStop {
                position: 0.08,
                color: [20, 40, 120],
                alpha: 0.0,
            },
            TransferStop {
                position: 0.38,
                color: [0, 185, 220],
                alpha: 0.25,
            },
            TransferStop {
                position: 0.72,
                color: [255, 190, 40],
                alpha: 0.55,
            },
            TransferStop {
                position: 1.0,
                color: [255, 255, 240],
                alpha: 0.85,
            },
        ]);
        let mode = match self.vol_mode {
            1 => VolumeRenderMode::MaximumIntensity,
            2 => VolumeRenderMode::IsosurfaceRay { threshold: 0.48 },
            _ => VolumeRenderMode::DirectAlpha,
        };

        let fig = Figure::new("stage16_fig")
            .columns(4)
            .cell_height(height)
            .show(ui, 8, |cell| match cell.index() {
                0 => cell
                    .show_plot3d(
                        Plot3D::new("s16_vol")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .depth_mode(DepthMode::ZBuffer)
                            .title(tr(
                                l,
                                "EXT-23 Volume Raycast",
                                "EXT-23 体渲染光线投射",
                                "EXT-23 ボリューム描画",
                            )),
                        &mut self.vol_state,
                        &[
                            Layer3D::from(Surface3D::new(&self.occlusion_surf).name("Base")),
                            Layer3D::from(
                                VolumeRender3D::new(&self.volume)
                                    .transfer(tf)
                                    .mode(mode)
                                    .max_resolution(96)
                                    .sample_steps(48)
                                    .name("Volume"),
                            ),
                        ],
                    )
                    .into_plot_response(),
                1 => cell
                    .show_plot3d(
                        Plot3D::new("s16_pc")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .depth_mode(DepthMode::ZBuffer)
                            .title(tr(
                                l,
                                "EXT-24 PointCloud 40k",
                                "EXT-24 四万点三维点云",
                                "EXT-24 4万点クラウド",
                            )),
                        &mut self.cloud_state,
                        &[Layer3D::from(
                            PointCloud3D::new(&self.cloud)
                                .max_points(12_000)
                                .name("LiDAR Spiral"),
                        )],
                    )
                    .into_plot_response(),
                2 => cell
                    .show_geoglobe(
                        GeoGlobePlot::new("s16_globe", &self.globe)
                            .height(height)
                            .allow_scroll_zoom(true)
                            .show_graticule(self.show_globe_grid)
                            .show_coastlines(self.show_globe_grid)
                            .title(tr(
                                l,
                                "EXT-25 WGS84 GeoGlobe",
                                "EXT-25 三维地球仪",
                                "EXT-25 3D 地球儀",
                            )),
                        &mut self.globe_state,
                    )
                    .into_plot_response(),
                3 => cell.show_layers(
                    Plot::new("s16_radar")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .canvas_axes()
                        .legend(true)
                        .title(tr(
                            l,
                            "EXT-26 RadarChart",
                            "EXT-26 雷达图",
                            "EXT-26 レーダーチャート",
                        )),
                    &mut self.states[0],
                    &[PlotLayer::from(RadarChart::new(&self.radar).name("Score"))],
                ),
                4 => cell.show_layers(
                    Plot::new("s16_sankey")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .canvas_axes()
                        .title(tr(
                            l,
                            "EXT-26 SankeyChart",
                            "EXT-26 桑基图",
                            "EXT-26 サンキー図",
                        )),
                    &mut self.states[1],
                    &[PlotLayer::from(
                        SankeyChart::new(&self.sankey).name("Energy"),
                    )],
                ),
                5 => cell.show_layers(
                    Plot::new("s16_treemap")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .canvas_axes()
                        .title(tr(
                            l,
                            "EXT-26 TreemapChart",
                            "EXT-26 矩形树图",
                            "EXT-26 ツリーマップ",
                        )),
                    &mut self.states[2],
                    &[PlotLayer::from(
                        TreemapChart::new(&self.treemap).name("Modules"),
                    )],
                ),
                6 => cell.show_layers(
                    Plot::new("s16_sunburst")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .canvas_axes()
                        .title(tr(
                            l,
                            "EXT-26 SunburstChart",
                            "EXT-26 旭日图",
                            "EXT-26 サンバースト",
                        )),
                    &mut self.states[3],
                    &[PlotLayer::from(
                        SunburstChart::new(&self.sunburst).name("Catalog"),
                    )],
                ),
                _ => cell.show_layers(
                    Plot::new("s16_gantt")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .gantt_axes(&self.gantt)
                        .title(tr(
                            l,
                            "EXT-26 GanttChart",
                            "EXT-26 甘特图",
                            "EXT-26 ガントチャート",
                        )),
                    &mut self.states[4],
                    &[PlotLayer::from(
                        GanttChart::new(&self.gantt)
                            .show_dependencies(self.show_gantt_deps)
                            .name("Roadmap"),
                    )],
                ),
            });
        fig.map_or_else(|_| Vec::new(), |r| r.inner)
    }
}
