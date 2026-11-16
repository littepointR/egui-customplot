//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    f64::consts::TAU,
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use egui::Color32;
use egui_customplot::{
    Bar3D, Bar3DData, Bar3DMode, Bar3DOrientation, Bubble3D, Bubble3DData, ColorMap, ColorScale,
    ConstantPlane3D, DepthMode, Figure, Fill3D, FunctionPlotOptions, Layer3D, Line3D,
    ParametricCurve3DData, Pie3D, Pie3DData, Plot3D, Plot3DState, PlotDiagnostics, Point3D,
    Point3DData, Polygon3DData, Quiver3D, Quiver3DData, Scatter3D, Stem3D, Stem3DData, Swarm3D,
    Swarm3DData,
};

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}

fn raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2024.0, 1124.0),
        )),
        ..Default::default()
    }
}

fn bench_preprocess_p50_ms<T>(mut f: impl FnMut() -> T) -> (f64, T) {
    let mut times = Vec::with_capacity(5);
    let mut last = None;
    for _ in 0..5 {
        let start = Instant::now();
        let val = black_box(f());
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        last = Some(val);
    }
    (percentile(&mut times, 0.5), last.unwrap())
}

fn plot3d_primitives(d: &PlotDiagnostics) -> usize {
    d.plot3d.output_triangles + d.plot3d.output_segments + d.plot3d.output_markers
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    // 1: plot3_helix_50k
    let n_helix = 50_000usize;
    let hx: Vec<f64> = (0..n_helix)
        .map(|i| (i as f64 * 0.015).cos() * (1.0 + (i as f64 / n_helix as f64) * 0.5))
        .collect();
    let hy: Vec<f64> = (0..n_helix)
        .map(|i| (i as f64 * 0.015).sin() * (1.0 + (i as f64 / n_helix as f64) * 0.5))
        .collect();
    let hz: Vec<f64> = (0..n_helix).map(|i| i as f64 * 0.001).collect();
    let (helix_ms, helix_50k) =
        bench_preprocess_p50_ms(|| Point3DData::from_xyz(&hx, &hy, &hz).unwrap());

    // 2: scatter3_bubble3_10k
    let n_sb = 5_000usize;
    let sb_pts: Vec<Point3D> = (0..n_sb)
        .map(|i| {
            let t = i as f64 * 0.07;
            [t.cos(), t.sin(), (t * 0.3).sin()]
        })
        .collect();
    let sb_sz: Vec<f64> = (0..n_sb).map(|i| 10.0 + ((i * 7) % 40) as f64).collect();
    let sb_col: Vec<f64> = (0..n_sb).map(|i| (i % 20) as f64).collect();
    let (sb_ms, (sc_5k, bub_5k)) = bench_preprocess_p50_ms(|| {
        let sc = Point3DData::new(sb_pts.clone())
            .with_scalars(&sb_col)
            .unwrap();
        let bb = Bubble3DData::new(&sb_pts, &sb_sz)
            .unwrap()
            .with_scalars(&sb_col)
            .unwrap();
        (sc, bb)
    });

    // 3: swarm3_5k
    let swarm_groups: Vec<Vec<f64>> = (0..5)
        .map(|g| {
            (0..1_000)
                .map(|i| ((i * 13 + g * 29) % 100) as f64 / 50.0 - 1.0)
                .collect()
        })
        .collect();
    let swarm_slices: Vec<&[f64]> = swarm_groups.iter().map(Vec::as_slice).collect();
    let centers = [[0.0, 0.0], [1.5, 0.0], [0.0, 1.5], [1.5, 1.5], [0.75, 0.75]];
    let (swarm_ms, swarm_5k) = bench_preprocess_p50_ms(|| {
        Swarm3DData::from_groups(&centers, &swarm_slices, 0.32).unwrap()
    });

    // 4: bar3_grouped_32x32
    let mat_vals: Vec<f64> = (0..(32 * 32))
        .map(|i| 0.5 + ((i * 17) % 30) as f64 * 0.1)
        .collect();
    let (bar_ms, bar_32x32) = bench_preprocess_p50_ms(|| {
        Bar3DData::from_matrix(
            &mat_vals,
            32,
            32,
            Bar3DMode::Grouped,
            Bar3DOrientation::Vertical,
            0.75,
        )
        .unwrap()
    });

    // 5: stem3_quiver3_2k
    let n_sq = 1_000usize;
    let stem_pts: Vec<Point3D> = (0..n_sq)
        .map(|i| {
            let a = i as f64 * 0.09;
            [a.cos(), a.sin(), (a * 0.25).cos() + 1.2]
        })
        .collect();
    let qx: Vec<f64> = (0..n_sq).map(|i| (i % 25) as f64 * 0.1 - 1.2).collect();
    let qy: Vec<f64> = (0..n_sq).map(|i| (i / 25) as f64 * 0.06 - 1.2).collect();
    let qz: Vec<f64> = (0..n_sq).map(|i| (i % 10) as f64 * 0.2 - 0.9).collect();
    let qu: Vec<f64> = qx.iter().map(|v| v * 0.4 + 0.1).collect();
    let qv: Vec<f64> = qy.iter().map(|v| -v * 0.35 + 0.15).collect();
    let qw: Vec<f64> = qz.iter().map(|v| 0.3 + v * 0.2).collect();
    let (sq_ms, (stem_1k, quiver_1k)) = bench_preprocess_p50_ms(|| {
        let s = Stem3DData::new(stem_pts.clone(), 0.0).unwrap();
        let q = Quiver3DData::from_components(&qx, &qy, &qz, &qu, &qv, &qw)
            .unwrap()
            .autoscale(0.9);
        (s, q)
    });

    // 6: fill3_planes_zbuffer
    let quads: Vec<[Point3D; 4]> = (0..64)
        .map(|i| {
            let r = (i / 8) as f64 * 0.22 - 0.8;
            let c = (i % 8) as f64 * 0.22 - 0.8;
            let z = (r * r - c * c) * 0.6;
            let dz = (i % 3) as f64 * 0.08 + 0.08;
            [
                [c, r, z - dz],
                [c + 0.19, r, z + dz],
                [c + 0.19, r + 0.19, z + dz],
                [c, r + 0.19, z - dz],
            ]
        })
        .collect();
    let quad_refs: Vec<&[Point3D]> = quads.iter().map(|q| q.as_slice()).collect();
    let (fill_ms, polys_64) =
        bench_preprocess_p50_ms(|| Polygon3DData::from_polygons(&quad_refs).unwrap());
    let planes = [
        ConstantPlane3D::x_plane(0.0),
        ConstantPlane3D::y_plane(0.0),
        ConstantPlane3D::new([1.0, 1.0, 1.0], 0.1),
    ];

    // 7: fplot3_pie3_combo
    let pie_vals = [25.0, 20.0, 18.0, 15.0, 12.0, 10.0];
    let (fp_ms, (fcurve, pie6)) = bench_preprocess_p50_ms(|| {
        let fp = ParametricCurve3DData::from_parametric(
            [0.0, 4.0 * TAU],
            FunctionPlotOptions::default(),
            |t| [t.cos(), t.sin(), t / (4.0 * TAU)],
        )
        .unwrap();
        let p = Pie3DData::with_options(
            &pie_vals,
            &[false, true, false, false, false, false],
            &[""; 6],
            0.32,
        )
        .unwrap();
        (fp, p)
    });

    println!(
        "# Stage 9A CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: plot3_helix_50k={helix_ms:.4}, scatter3_bubble3_10k={sb_ms:.4}, swarm3_5k={swarm_ms:.4}, bar3_grouped_32x32={bar_ms:.4}, stem3_quiver3_2k={sq_ms:.4}, fill3_planes_zbuffer={fill_ms:.4}, fplot3_pie3_combo={fp_ms:.4}; fcurve_pts={}",
        fcurve.points().len()
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    measure_plot3d(
        "plot3_helix_50k",
        n_helix,
        helix_50k.sample_bytes(),
        DepthMode::SortedPainter,
        &[],
        seconds,
        &[Layer3D::from(
            Line3D::new(&helix_50k).color(Color32::from_rgb(0, 114, 189)),
        )],
    );

    let sb_scale = ColorScale::new([0.0, 19.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    measure_plot3d(
        "scatter3_bubble3_10k",
        n_sb * 2,
        sc_5k.sample_bytes() + bub_5k.sample_bytes(),
        DepthMode::SortedPainter,
        &[],
        seconds,
        &[
            Layer3D::from(Scatter3D::new(&sc_5k).color_scale(sb_scale)),
            Layer3D::from(Bubble3D::new(&bub_5k).color_scale(sb_scale)),
        ],
    );

    measure_plot3d(
        "swarm3_5k",
        swarm_5k.len(),
        swarm_5k.sample_bytes(),
        DepthMode::SortedPainter,
        &[],
        seconds,
        &[Layer3D::from(Swarm3D::new(&swarm_5k))],
    );

    measure_plot3d(
        "bar3_grouped_32x32",
        bar_32x32.boxes().len(),
        bar_32x32.sample_bytes(),
        DepthMode::SortedPainter,
        &[],
        seconds,
        &[Layer3D::from(Bar3D::new(&bar_32x32))],
    );

    measure_plot3d(
        "stem3_quiver3_2k",
        n_sq * 2,
        stem_1k.sample_bytes() + quiver_1k.sample_bytes(),
        DepthMode::SortedPainter,
        &[],
        seconds,
        &[
            Layer3D::from(Stem3D::new(&stem_1k)),
            Layer3D::from(Quiver3D::new(&quiver_1k)),
        ],
    );

    measure_plot3d(
        "fill3_planes_zbuffer",
        polys_64.polygons().len() + planes.len(),
        polys_64.sample_bytes(),
        DepthMode::ZBuffer,
        &planes,
        seconds,
        &[Layer3D::from(Fill3D::new(&polys_64).alpha(1.0))],
    );

    measure_combo(
        "fplot3_pie3_combo",
        fcurve.points().len() + pie6.slices().len(),
        fcurve.points().sample_bytes() + pie6.sample_bytes() + size_of::<Pie3DData>(),
        seconds,
        &fcurve,
        &pie6,
    );
}

fn measure_plot3d(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    depth_mode: DepthMode,
    planes: &[ConstantPlane3D],
    seconds: f64,
    layers: &[Layer3D<'_>],
) {
    let planes_arc: std::sync::Arc<[ConstantPlane3D]> = planes.into();
    let ctx = egui::Context::default();
    let mut state = Plot3DState::default();
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut last_out_prims = 0usize;
    let mut last_visited = 0usize;
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let response = Plot3D::new("stage9a_bench")
                .height(1080.0)
                .depth_mode(depth_mode)
                .constant_planes(std::sync::Arc::clone(&planes_arc))
                .show_legend(false)
                .show_hover(false)
                .show(ui, &mut state, layers);
            assert!(response.plot_area.width() > 0.0 && response.plot_area.height() > 0.0);
            last_out_prims = plot3d_primitives(&response.diagnostics);
            last_visited = response.diagnostics.plot3d.visited_items;
        });
        let ui_elapsed = start.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        let total_elapsed = start.elapsed();
        output_meshes = meshes
            .iter()
            .filter(|m| matches!(m.primitive, egui::epaint::Primitive::Mesh(_)))
            .count();
        black_box(&meshes);
        out.drop_without_applying_deltas();
        if frame >= 5 {
            start_wall.get_or_insert_with(Instant::now);
            ui_times.push(ui_elapsed.as_secs_f64() * 1000.0);
            tess_times.push((total_elapsed - ui_elapsed).as_secs_f64() * 1000.0);
            total_times.push(total_elapsed.as_secs_f64() * 1000.0);
        }
        frame += 1;
    }

    print_row(
        scene,
        items,
        start_wall.map_or(0.0, |t| t.elapsed().as_secs_f64()),
        &mut ui_times,
        &mut tess_times,
        &mut total_times,
        last_out_prims,
        output_meshes,
        0,
        sample_bytes,
        last_visited,
    );
}

fn measure_combo(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    seconds: f64,
    fcurve: &ParametricCurve3DData,
    pie: &Pie3DData,
) {
    let ctx = egui::Context::default();
    let mut states = [Plot3DState::default(), Plot3DState::default()];
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut last_out_prims = 0usize;
    let mut last_visited = 0usize;
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let [s0, s1] = &mut states;
            let fig = Figure::new("stage9a_combo_bench")
                .columns(2)
                .cell_height(1080.0)
                .show(ui, 2, |cell| match cell.index() {
                    0 => cell
                        .show_plot3d(
                            Plot3D::new("fp").show_legend(false).show_hover(false),
                            s0,
                            &[Layer3D::from(Line3D::new(fcurve.points()))],
                        )
                        .into_plot_response(),
                    _ => cell
                        .show_plot3d(
                            Plot3D::new("pie").show_legend(false).show_hover(false),
                            s1,
                            &[Layer3D::from(Pie3D::new(pie))],
                        )
                        .into_plot_response(),
                })
                .unwrap();
            last_out_prims = fig
                .inner
                .iter()
                .map(|r| plot3d_primitives(&r.diagnostics))
                .sum();
            last_visited = fig
                .inner
                .iter()
                .map(|r| r.diagnostics.plot3d.visited_items)
                .sum();
        });
        let ui_elapsed = start.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        let total_elapsed = start.elapsed();
        output_meshes = meshes
            .iter()
            .filter(|m| matches!(m.primitive, egui::epaint::Primitive::Mesh(_)))
            .count();
        black_box(&meshes);
        out.drop_without_applying_deltas();
        if frame >= 5 {
            start_wall.get_or_insert_with(Instant::now);
            ui_times.push(ui_elapsed.as_secs_f64() * 1000.0);
            tess_times.push((total_elapsed - ui_elapsed).as_secs_f64() * 1000.0);
            total_times.push(total_elapsed.as_secs_f64() * 1000.0);
        }
        frame += 1;
    }

    print_row(
        scene,
        items,
        start_wall.map_or(0.0, |t| t.elapsed().as_secs_f64()),
        &mut ui_times,
        &mut tess_times,
        &mut total_times,
        last_out_prims,
        output_meshes,
        0,
        sample_bytes,
        last_visited,
    );
}

#[allow(clippy::too_many_arguments)]
fn print_row(
    scene: &str,
    items: usize,
    elapsed_secs: f64,
    ui_times: &mut [f64],
    tess_times: &mut [f64],
    total_times: &mut [f64],
    output_primitives: usize,
    output_meshes: usize,
    geometry_capacity_bytes: usize,
    sample_bytes: usize,
    visited_items: usize,
) {
    let u = [0.5, 0.95, 0.99].map(|q| percentile(ui_times, q));
    let t = [0.5, 0.95, 0.99].map(|q| percentile(tess_times, q));
    let a = [0.5, 0.95, 0.99].map(|q| percentile(total_times, q));
    println!(
        "{scene},{items},{},{elapsed_secs:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{output_primitives},{output_meshes},{geometry_capacity_bytes},{sample_bytes},{visited_items}",
        total_times.len(),
        u[0],
        u[1],
        u[2],
        t[0],
        t[1],
        t[2],
        a[0],
        a[1],
        a[2],
    );
}
