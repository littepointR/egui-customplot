//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui::{Color32, Stroke};
use egui_customplot::{
    AlphaShape, AlphaShapeData, FunctionCurveData, FunctionPlot, FunctionPlotOptions,
    ImplicitCurve, ImplicitCurveData, ImplicitOptions, Plot, PlotBounds, PlotDiagnostics,
    PlotLayer, PlotState, Spy, SpyData, StreamlineData, Streamlines, TriMesh2D, TriPlot,
    VectorField2D, Voronoi, VoronoiData,
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

fn annulus_sites(count: usize) -> Vec<[f64; 2]> {
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    let r_in = 15.0_f64;
    let r_out = 48.0_f64;
    (0..count)
        .map(|i| {
            let t = (i as f64 + 0.5) / count as f64;
            let r = (r_in * r_in + t * (r_out * r_out - r_in * r_in)).sqrt();
            let theta = i as f64 * golden_angle;
            [50.0 + r * theta.cos(), 50.0 + r * theta.sin()]
        })
        .collect()
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    let (fplot_ms, chirp) = bench_preprocess_p50_ms(|| {
        FunctionCurveData::explicit([0.05, 5.0], FunctionPlotOptions::default(), |x| {
            (1.0 / (0.15 * x)).sin()
        })
        .unwrap()
    });
    let cassini_bounds = PlotBounds::new([-2.2, -1.6], [2.2, 1.6]).unwrap();
    let (fimplicit_ms, cassini) = bench_preprocess_p50_ms(|| {
        ImplicitCurveData::new(cassini_bounds, ImplicitOptions::default(), |x, y| {
            let r2 = x * x + y * y;
            r2 * r2 - 2.0 * (x * x - y * y) - (0.95_f64.powi(4) - 1.0)
        })
        .unwrap()
    });

    let sites = annulus_sites(5_000);
    let sites_bounds = PlotBounds::new([0.0, 0.0], [100.0, 100.0]).unwrap();
    let (delaunay_ms, tri) = bench_preprocess_p50_ms(|| TriMesh2D::delaunay(&sites).unwrap());
    let (voronoi_ms, vor) =
        bench_preprocess_p50_ms(|| VoronoiData::from_sites(&sites, sites_bounds).unwrap());
    let (alpha_ms, alpha) =
        bench_preprocess_p50_ms(|| AlphaShapeData::from_sites(&sites, 1.8).unwrap());
    assert!(
        alpha.patch().face_count() > 2_000,
        "unexpectedly sparse alpha shape: faces={}",
        alpha.patch().face_count()
    );

    let grid_n = 128usize;
    let flow_bounds = PlotBounds::new([-2.0, -2.0], [2.0, 2.0]).unwrap();
    let mut u = Vec::with_capacity(grid_n * grid_n);
    let mut v = Vec::with_capacity(grid_n * grid_n);
    for iy in 0..grid_n {
        let y = -2.0 + 4.0 * (iy as f64 / (grid_n - 1) as f64);
        for ix in 0..grid_n {
            let x = -2.0 + 4.0 * (ix as f64 / (grid_n - 1) as f64);
            u.push(-y - 0.12 * x);
            v.push(x - 0.12 * y);
        }
    }
    let field = VectorField2D::uniform(grid_n, grid_n, flow_bounds, u, v).unwrap();
    let (streamslice_ms, slice) =
        bench_preprocess_p50_ms(|| StreamlineData::streamslice(&field, 1.0).unwrap());

    let spy_dim = 10_000usize;
    let mut entries = Vec::with_capacity(spy_dim * 10);
    for r in 0..spy_dim {
        for k in 0..10 {
            entries.push((r, (r + k * 7) % spy_dim));
        }
    }
    let (spy_ms, spy) =
        bench_preprocess_p50_ms(|| SpyData::from_coords(spy_dim, spy_dim, &entries).unwrap());
    assert_eq!(spy.nnz(), 100_000);

    println!(
        "# Stage 8B CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: fplot_chirp={fplot_ms:.4}, fimplicit_cassini={fimplicit_ms:.4}, delaunay_5k={delaunay_ms:.4}, voronoi_5k={voronoi_ms:.4}, alphashape_5k={alpha_ms:.4}, streamslice_128x128={streamslice_ms:.4}, spy_100k_nnz={spy_ms:.4}; chirp_pts={}, cassini_lines={}, tri_edges={}, vor_edges={}, alpha_faces={}, slice_lines={}, slice_arrows={}",
        chirp.data().len(),
        cassini.contours().lines().len(),
        tri.edges().len(),
        vor.edges().len(),
        alpha.patch().face_count(),
        slice.lines().len(),
        slice.arrows().len(),
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    measure(
        "fplot_adaptive_chirp",
        chirp.data().len(),
        chirp.data().sample_bytes(),
        PlotBounds::new([0.05, -1.2], [5.0, 1.2]).unwrap(),
        false,
        seconds,
        &[FunctionPlot::new(&chirp).into()],
    );
    measure(
        "fimplicit_cassini_branches",
        cassini.contours().lines().len(),
        cassini.contours().sample_bytes(),
        cassini_bounds,
        false,
        seconds,
        &[ImplicitCurve::new(&cassini).into()],
    );
    measure(
        "triplot_voronoi_5k_sites",
        tri.edges().len() + vor.edges().len(),
        tri.sample_bytes() + vor.sample_bytes(),
        sites_bounds,
        false,
        seconds,
        &[TriPlot::new(&tri).into(), Voronoi::new(&vor).into()],
    );
    measure(
        "alphashape_5k_sites",
        alpha.patch().face_count(),
        alpha.sample_bytes(),
        sites_bounds,
        false,
        seconds,
        &[AlphaShape::new(&alpha)
            .fill_color(Color32::from_rgba_unmultiplied(40, 140, 220, 140))
            .border(Stroke::new(1.2, Color32::BLACK))
            .into()],
    );
    measure(
        "streamslice_128x128",
        slice.lines().len() + slice.arrows().len(),
        slice.sample_bytes(),
        flow_bounds,
        false,
        seconds,
        &[Streamlines::new(&slice).into()],
    );
    measure(
        "spy_100k_nnz_full",
        spy.nnz(),
        spy.sample_bytes(),
        PlotBounds::new([0.5, 0.5], [10_000.5, 10_000.5]).unwrap(),
        true,
        seconds,
        &[Spy::new(&spy).into()],
    );
    measure(
        "spy_100k_nnz_narrow_2pct",
        spy.nnz(),
        spy.sample_bytes(),
        PlotBounds::new([4900.0, 4900.0], [5100.0, 5100.0]).unwrap(),
        true,
        seconds,
        &[Spy::new(&spy).into()],
    );
}

fn measure(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    bounds: PlotBounds,
    spy_axes: bool,
    seconds: f64,
    layers: &[PlotLayer<'_>],
) {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut counters = PlotDiagnostics::default();
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;
    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        state.set_bounds(bounds);
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let mut plot = Plot::new("stage8b_bench")
                .height(1124.0)
                .show_grid(false)
                .show_hover(false);
            if spy_axes {
                plot = plot.spy_axes();
            }
            let response = plot.show_layers(ui, &mut state, layers);
            assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
            counters = response.diagnostics;
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
    let elapsed_secs = start_wall.map_or(0.0, |t| t.elapsed().as_secs_f64());
    let output_primitives = counters.output_paths
        + counters.output_markers
        + counters.contours.output_paths
        + counters.geometry.output_paths
        + counters.geometry.output_markers
        + counters.patches.output_triangles
        + counters.flow.output_paths
        + counters.flow.output_heads;
    let visited_items = counters.visited_segments
        + counters.contours.visited_items
        + counters.geometry.visited_items
        + counters.patches.visited_items
        + counters.flow.visited_items;
    let u = [0.5, 0.95, 0.99].map(|q| percentile(&mut ui_times, q));
    let t = [0.5, 0.95, 0.99].map(|q| percentile(&mut tess_times, q));
    let a = [0.5, 0.95, 0.99].map(|q| percentile(&mut total_times, q));
    println!(
        "{scene},{items},{},{elapsed_secs:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{output_primitives},{output_meshes},{},{sample_bytes},{visited_items}",
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
        counters.geometry_capacity_bytes,
    );
}
