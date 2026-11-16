//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    f64::consts::TAU,
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use egui::Color32;
use egui_customplot::{
    ColorMap, ColorScale, CompassData, CompassPlot, EliminationTree, FunctionPlotOptions,
    GraphLayout, GraphPlot, Plot, PlotBounds, PlotDiagnostics, PlotLayer, PlotState, PolarBubble,
    PolarBubbleData, PolarCurveData, PolarHistNormalization, PolarHistogram, PolarHistogramData,
    PolarLayer, PolarLine, PolarPlot, PolarRegionAnnotation, PolarScatter, PolarState, RoseData,
    SpyData,
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

fn polar_primitives(d: &PlotDiagnostics) -> usize {
    d.polar.output_paths
        + d.polar.output_markers
        + d.polar.output_sectors
        + d.polar.output_arrows
        + d.polar.output_regions
}

fn graph_primitives(d: &PlotDiagnostics) -> usize {
    d.network.output_edges + d.network.output_arrows + d.network.output_nodes
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    // 1: polarplot_50k
    let n_curve = 50_000usize;
    let th_50k: Vec<f64> = (0..n_curve)
        .map(|i| i as f64 / n_curve as f64 * 12.0 * TAU)
        .collect();
    let r_50k: Vec<f64> = th_50k
        .iter()
        .map(|t| 1.2 + 0.65 * (5.0 * t).sin() + 0.25 * (t / 6.0))
        .collect();
    let (p50k_ms, curve_50k) =
        bench_preprocess_p50_ms(|| PolarCurveData::from_theta_r(&th_50k, &r_50k).unwrap());

    // 2: polarscatter_bubble_5k (2,500 scatter + 2,500 colored bubbles)
    let n_sb = 2_500usize;
    let sb_th: Vec<f64> = (0..n_sb).map(|i| (i as f64 * 0.13) % TAU).collect();
    let sb_r: Vec<f64> = (0..n_sb)
        .map(|i| 0.3 + ((i % 100) as f64 / 100.0) * 1.6)
        .collect();
    let sb_sz: Vec<f64> = (0..n_sb).map(|i| 10.0 + ((i * 7) % 40) as f64).collect();
    let sb_col: Vec<f64> = (0..n_sb).map(|i| (i % 20) as f64).collect();
    let (sb_ms, (sc_data, bub_data)) = bench_preprocess_p50_ms(|| {
        let sc = PolarCurveData::from_theta_r(&sb_th, &sb_r).unwrap();
        let bb = PolarBubbleData::new(&sb_th, &sb_r, &sb_sz)
            .unwrap()
            .with_scalars(&sb_col)
            .unwrap();
        (sc, bb)
    });

    // 3: polarhist_100k_72bins (+ RoseData petal extraction)
    let n_hist = 100_000usize;
    let hist_angles: Vec<f64> = (0..n_hist)
        .map(|i| (i as f64 * 0.037).sin() * 1.4 + (i % 7) as f64 * 0.85)
        .collect();
    let (phist_ms, (hist_72, rose_72)) = bench_preprocess_p50_ms(|| {
        let h = PolarHistogramData::new(&hist_angles, 72).unwrap();
        let rose = RoseData::new(&hist_angles, 72).unwrap().to_polar_curve();
        (h, rose)
    });

    // 4: fpolar_compass_1k (adaptive fpolarplot + 1,000 compass arrows + 3 regions)
    let c_th: Vec<f64> = (0..1_000).map(|i| i as f64 * TAU / 1_000.0).collect();
    let c_rho: Vec<f64> = c_th
        .iter()
        .map(|t| 0.4 + 0.8 * (3.0 * t).cos().abs())
        .collect();
    let (fc_ms, (fpolar, compass_1k)) = bench_preprocess_p50_ms(|| {
        let fp = PolarCurveData::from_function([0.0, TAU], FunctionPlotOptions::default(), |t| {
            1.1 + 0.5 * (7.0 * t).cos() + 0.2 * (13.0 * t).sin()
        })
        .unwrap();
        let cp = CompassData::from_polar(&c_th, &c_rho).unwrap();
        (fp, cp)
    });

    // 5: graph_force_1k_nodes_3k_edges
    let n_g = 1_000usize;
    let mut g_edges = Vec::with_capacity(3_000);
    for i in 0..n_g {
        g_edges.push([i, (i + 1) % n_g]);
        g_edges.push([i, (i + 17) % n_g]);
        g_edges.push([i, (i * 37 + 13) % n_g]);
    }
    let (gforce_ms, force_graph) = bench_preprocess_p50_ms(|| {
        egui_customplot::GraphData::undirected(n_g, &g_edges, GraphLayout::force_default()).unwrap()
    });

    // 6: digraph_layered_500_nodes_1500_edges
    let n_dg = 500usize;
    let mut dg_edges = Vec::with_capacity(1_500);
    for i in 0..n_dg {
        for stride in [1usize, 3, 7] {
            let tgt = (i + stride).min(n_dg - 1);
            if tgt != i {
                dg_edges.push([i, tgt]);
            }
        }
    }
    let (dglay_ms, layered_digraph) = bench_preprocess_p50_ms(|| {
        egui_customplot::GraphData::directed(n_dg, &dg_edges, GraphLayout::Layered).unwrap()
    });

    // 7: etree_sparse_10k_n_40k_nnz (banded symmetric matrix n=10,000, ~40k nonzeros)
    let n_et = 10_000usize;
    let mut spy_entries = Vec::with_capacity(50_000);
    for i in 0..n_et {
        spy_entries.push((i, i));
        for off in [1usize, 64] {
            if i + off < n_et {
                spy_entries.push((i, i + off));
                spy_entries.push((i + off, i));
            }
        }
    }
    let spy_10k = SpyData::from_coords(n_et, n_et, &spy_entries).unwrap();
    let (etree_ms, etree_graph) = bench_preprocess_p50_ms(|| {
        let et = EliminationTree::from_spy(&spy_10k).unwrap();
        et.to_tree_data().to_graph()
    });

    println!(
        "# Stage 8D CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: polarplot_50k={p50k_ms:.4}, polarscatter_bubble_5k={sb_ms:.4}, polarhist_100k_72bins={phist_ms:.4}, fpolar_compass_1k={fc_ms:.4}, graph_force_1k_nodes_3k_edges={gforce_ms:.4}, digraph_layered_500_nodes_1500_edges={dglay_ms:.4}, etree_sparse_10k_n_40k_nnz={etree_ms:.4}; spy_nnz={}, etree_edges={}",
        spy_entries.len(),
        etree_graph.edges().len()
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    measure_polar(
        "polarplot_50k",
        n_curve,
        curve_50k.sample_bytes(),
        std::sync::Arc::from([]),
        seconds,
        &[PolarLayer::from(
            PolarLine::new(&curve_50k).color(Color32::from_rgb(0, 114, 189)),
        )],
    );

    let sb_scale = ColorScale::new([0.0, 19.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    measure_polar(
        "polarscatter_bubble_5k",
        n_sb * 2,
        sc_data.sample_bytes() + bub_data.sample_bytes(),
        std::sync::Arc::from([]),
        seconds,
        &[
            PolarLayer::from(PolarScatter::new(&sc_data).color(Color32::from_rgb(217, 83, 25))),
            PolarLayer::from(PolarBubble::new(&bub_data).color_scale(sb_scale)),
        ],
    );

    measure_polar(
        "polarhist_100k_72bins",
        n_hist,
        n_hist * size_of::<f64>(),
        std::sync::Arc::from([]),
        seconds,
        &[
            PolarLayer::from(
                PolarHistogram::new(&hist_72).normalization(PolarHistNormalization::Count),
            ),
            PolarLayer::from(PolarLine::new(&rose_72)),
        ],
    );

    let regions: std::sync::Arc<[PolarRegionAnnotation]> = std::sync::Arc::from([
        PolarRegionAnnotation::theta_region(5.75, 0.55),
        PolarRegionAnnotation::radius_region(0.35, 0.95),
        PolarRegionAnnotation::polar_region([1.2, 2.4], [0.5, 1.3]),
    ]);
    measure_polar(
        "fpolar_compass_1k",
        1_000 + fpolar.curve().len(),
        fpolar.curve().sample_bytes() + compass_1k.sample_bytes(),
        regions,
        seconds,
        &[
            PolarLayer::from(PolarLine::new(fpolar.curve())),
            PolarLayer::from(CompassPlot::new(&compass_1k)),
        ],
    );

    measure_graph(
        "graph_force_1k_nodes_3k_edges",
        n_g + g_edges.len(),
        force_graph.sample_bytes(),
        PlotBounds::new([-0.15, -0.15], [1.15, 1.15]).unwrap(),
        seconds,
        &[PlotLayer::from(GraphPlot::new(&force_graph).node_size(3.5))],
    );

    let dg_items = n_dg + dg_edges.len();
    measure_graph(
        "digraph_layered_500_nodes_1500_edges",
        dg_items,
        layered_digraph.sample_bytes(),
        PlotBounds::new([-0.15, -0.15], [1.15, 1.15]).unwrap(),
        seconds,
        &[PlotLayer::from(
            GraphPlot::new(&layered_digraph).node_size(4.5),
        )],
    );

    measure_graph(
        "etree_sparse_10k_n_40k_nnz",
        n_et + etree_graph.edges().len(),
        etree_graph.sample_bytes(),
        PlotBounds::new([-0.15, -0.15], [1.15, 1.15]).unwrap(),
        seconds,
        &[PlotLayer::from(GraphPlot::new(&etree_graph).node_size(2.0))],
    );
}

fn measure_polar(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    regions: std::sync::Arc<[PolarRegionAnnotation]>,
    seconds: f64,
    layers: &[PolarLayer<'_>],
) {
    let ctx = egui::Context::default();
    let mut state = PolarState::default();
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut last_out_prims = 0usize;
    let mut last_visited = 0usize;
    let mut last_geom_bytes = 0usize;
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let response = PolarPlot::new("stage8d_polar_bench")
                .height(1080.0)
                .show_grid(false)
                .show_legend(false)
                .show_hover(false)
                .regions(std::sync::Arc::clone(&regions))
                .show(ui, &mut state, layers);
            assert!(response.plot_area.width() > 0.0 && response.plot_area.height() > 0.0);
            last_out_prims = polar_primitives(&response.diagnostics);
            last_visited = response.diagnostics.polar.visited_items;
            last_geom_bytes = response.diagnostics.geometry_capacity_bytes;
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
        last_geom_bytes,
        sample_bytes,
        last_visited,
    );
}

fn measure_graph(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    bounds: PlotBounds,
    seconds: f64,
    layers: &[PlotLayer<'_>],
) {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut last_out_prims = 0usize;
    let mut last_visited = 0usize;
    let mut last_geom_bytes = 0usize;
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        state.set_bounds(bounds);
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let response = Plot::new("stage8d_graph_bench")
                .height(1124.0)
                .graph_axes()
                .show_grid(false)
                .show_hover(false)
                .show_layers(ui, &mut state, layers);
            assert!(response.transform.rect().width() > 0.0);
            last_out_prims = graph_primitives(&response.diagnostics);
            last_visited = response.diagnostics.network.visited_nodes
                + response.diagnostics.network.visited_edges;
            last_geom_bytes = response.diagnostics.geometry_capacity_bytes;
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
        last_geom_bytes,
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
