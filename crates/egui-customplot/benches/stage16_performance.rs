//! Headless CPU frame + tessellation benchmark for Stage 16 (`EXT-23..26`):
//! volshow 64^3 DirectAlpha raycasting, 200k pcshow octree LOD, WGS84 GeoGlobe, and modern charts.
use std::{
    f64::consts::TAU,
    time::{Duration, Instant},
};

use egui::{Color32, Context, Pos2, RawInput, Rect};
use egui_customplot::{
    Bounds3D, ColorMap, DepthMode, Figure, GanttChart, GanttData, GanttTask, GeoGlobeData,
    GeoGlobePlot, GeoGlobeState, GlobeMarker3D, Layer3D, Plot, Plot3D, Plot3DState, PlotLayer,
    PlotState, PointCloud3D, PointCloud3DData, RadarChart, RadarData, SankeyChart, SankeyData,
    ScalarVolume3D, SunburstChart, SunburstData, SunburstInput, TreemapChart, TreemapData,
    VolumeRender3D, VolumeRenderMode, VolumeTransferFunction,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 800.0))),
        ..Default::default()
    }
}

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}

fn measure_cpu(scenario: &str, seconds: f64, mut frame_fn: impl FnMut(&mut egui::Ui)) {
    let ctx = Context::default();
    let mut build_ms = Vec::new();
    let mut total_ms = Vec::new();
    let mut last_meshes: usize;
    let mut last_verts: usize;

    for _ in 0..3 {
        let mut out = ctx.run_ui(raw(), |ui| frame_fn(ui));
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
    }

    let bench_start = Instant::now();
    loop {
        let t0 = Instant::now();
        let mut out = ctx.run_ui(raw(), |ui| frame_fn(ui));
        let dt_build = t0.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        let dt_total = t0.elapsed();
        last_meshes = meshes.len();
        last_verts = meshes
            .iter()
            .map(|m| match &m.primitive {
                egui::epaint::Primitive::Mesh(mesh) => mesh.vertices.len(),
                _ => 0,
            })
            .sum();
        build_ms.push(dt_build.as_secs_f64() * 1000.0);
        total_ms.push(dt_total.as_secs_f64() * 1000.0);
        if total_ms.len() >= 15 && bench_start.elapsed() >= Duration::from_secs_f64(seconds) {
            break;
        }
    }

    let b50 = percentile(&mut build_ms, 0.50);
    let b95 = percentile(&mut build_ms, 0.95);
    let t50 = percentile(&mut total_ms, 0.50);
    let t95 = percentile(&mut total_ms, 0.95);
    let tmax = percentile(&mut total_ms, 1.0);
    println!(
        "{scenario},{samples},{b50:.3},{b95:.3},{t50:.3},{t95:.3},{tmax:.3},{last_meshes},{last_verts}",
        samples = total_ms.len()
    );
}

fn main() {
    let seconds = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|&v| v.is_finite() && v > 0.0)
        .unwrap_or(1.5);

    println!(
        "scenario,frames,cpu_build_p50_ms,cpu_build_p95_ms,cpu_total_p50_ms,cpu_total_p95_ms,cpu_total_max_ms,meshes,vertices"
    );

    let bounds = Bounds3D::new([-1.2, -1.2, -1.2], [1.2, 1.2, 1.2]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [64, 64, 64], 300_000, |x, y, z| {
        (-(x * x + y * y + z * z) * 2.2).exp()
    })
    .unwrap();
    let tf = VolumeTransferFunction::ramp(ColorMap::Inferno, 0.0, 0.85);
    let mut vol_state = Plot3DState::default();
    measure_cpu("volshow_64x64x64_direct_alpha", seconds, |ui| {
        let _ = Plot3D::new("b_vol")
            .height(620.0)
            .depth_mode(DepthMode::ZBuffer)
            .show(
                ui,
                &mut vol_state,
                &[Layer3D::from(
                    VolumeRender3D::new(&vol)
                        .transfer(tf)
                        .mode(VolumeRenderMode::DirectAlpha)
                        .max_resolution(120),
                )],
            );
    });

    let mut orbit_state = Plot3DState::default();
    let mut orbit_az = -38.0;
    measure_cpu("volshow_64x64x64_orbiting_uncached", seconds, |ui| {
        orbit_az += 0.5;
        orbit_state.set_camera(egui_customplot::Camera3D::new(orbit_az, 28.0));
        let _ = Plot3D::new("b_vol_orbit")
            .height(620.0)
            .depth_mode(DepthMode::ZBuffer)
            .show(
                ui,
                &mut orbit_state,
                &[Layer3D::from(
                    VolumeRender3D::new(&vol)
                        .transfer(tf)
                        .mode(VolumeRenderMode::DirectAlpha)
                        .max_resolution(96)
                        .sample_steps(48),
                )],
            );
    });

    let pts: Vec<[f64; 3]> = (0..200_000)
        .map(|i| {
            let t = i as f64 / 200_000.0;
            let a = 32.0 * TAU * t;
            [
                (0.15 + 0.85 * t) * a.cos(),
                (0.15 + 0.85 * t) * a.sin(),
                2.0 * t - 1.0,
            ]
        })
        .collect();
    let cloud = PointCloud3DData::from_xyz(pts).unwrap();
    let mut pc_state = Plot3DState::default();
    measure_cpu("pcshow_200k_octree_lod_zbuffer", seconds, |ui| {
        let _ = Plot3D::new("b_pc")
            .height(620.0)
            .depth_mode(DepthMode::ZBuffer)
            .show(
                ui,
                &mut pc_state,
                &[Layer3D::from(PointCloud3D::new(&cloud).max_points(12_000))],
            );
    });

    let globe = GeoGlobeData::new()
        .with_extruded_track(
            "Orbit",
            &[20.0, 31.2, 42.0, 51.5],
            &[90.0, 110.0, 45.0, -0.1],
            &[120_000.0, 450_000.0, 580_000.0, 120_000.0],
            Color32::from_rgb(255, 175, 50),
        )
        .unwrap()
        .with_markers(vec![GlobeMarker3D {
            label: "Hub".into(),
            lat_deg: 31.23,
            lon_deg: 121.47,
            alt_m: 10_000.0,
            radius_px: 5.0,
            color: Color32::LIGHT_BLUE,
        }])
        .unwrap();
    let mut globe_state = GeoGlobeState::default();
    measure_cpu("geoglobe_wgs84_coastlines_track", seconds, |ui| {
        let _ = GeoGlobePlot::new("b_globe", &globe)
            .height(620.0)
            .show(ui, &mut globe_state);
    });

    let radar = RadarData::new(
        &["A", "B", "C", "D", "E", "F"],
        &[100.0; 6],
        &[("S1", &[88.0, 76.0, 92.0, 81.0, 74.0, 89.0])],
    )
    .unwrap();
    let sankey = SankeyData::new(
        &["Solar", "Wind", "Grid", "Ind", "Home"],
        &[(0, 2, 42.0), (1, 2, 36.0), (2, 3, 48.0), (2, 4, 30.0)],
    )
    .unwrap();
    let treemap = TreemapData::squarified(&[
        ("A", 0, 40.0),
        ("B", 1, 25.0),
        ("C", 2, 20.0),
        ("D", 3, 15.0),
    ])
    .unwrap();
    let sunburst = SunburstData::new(&[
        SunburstInput {
            label: "Root".into(),
            parent: None,
            value: 0.0,
            group: 0,
        },
        SunburstInput {
            label: "2D".into(),
            parent: Some(0),
            value: 60.0,
            group: 1,
        },
        SunburstInput {
            label: "3D".into(),
            parent: Some(0),
            value: 40.0,
            group: 2,
        },
    ])
    .unwrap();
    let gantt = GanttData::new(
        vec![
            GanttTask {
                label: "Spec".into(),
                start: 0.0,
                end: 4.0,
                progress: 1.0,
                group: 0,
                milestone: false,
            },
            GanttTask {
                label: "Impl".into(),
                start: 3.0,
                end: 8.0,
                progress: 0.8,
                group: 1,
                milestone: false,
            },
            GanttTask {
                label: "Ship".into(),
                start: 8.0,
                end: 8.0,
                progress: 1.0,
                group: 2,
                milestone: true,
            },
        ],
        vec![[0, 1], [1, 2]],
    )
    .unwrap();
    let mut ps: [PlotState; 5] = std::array::from_fn(|_| PlotState::default());
    measure_cpu("modern_suite_5charts_figure", seconds, |ui| {
        let _ = Figure::new("b_mod")
            .columns(3)
            .cell_height(260.0)
            .show(ui, 5, |c| match c.index() {
                0 => c.show_layers(
                    Plot::new("br").canvas_axes(),
                    &mut ps[0],
                    &[PlotLayer::from(RadarChart::new(&radar))],
                ),
                1 => c.show_layers(
                    Plot::new("bs").canvas_axes(),
                    &mut ps[1],
                    &[PlotLayer::from(SankeyChart::new(&sankey))],
                ),
                2 => c.show_layers(
                    Plot::new("bt").canvas_axes(),
                    &mut ps[2],
                    &[PlotLayer::from(TreemapChart::new(&treemap))],
                ),
                3 => c.show_layers(
                    Plot::new("bu").canvas_axes(),
                    &mut ps[3],
                    &[PlotLayer::from(SunburstChart::new(&sunburst))],
                ),
                _ => c.show_layers(
                    Plot::new("bg").gantt_axes(&gantt),
                    &mut ps[4],
                    &[PlotLayer::from(GanttChart::new(&gantt))],
                ),
            });
    });
}
