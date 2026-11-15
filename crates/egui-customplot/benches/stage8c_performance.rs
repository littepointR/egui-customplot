//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use egui::Color32;
use egui_customplot::{
    BinScatter, BinScatterData, BinShape, BubbleChart, BubbleCloud, BubbleCloudData, BubbleData,
    ColorMap, ColorScale, Histogram2, Histogram2Data, ParallelData, ParallelNormalization,
    ParallelPlot, PieChart, PieData, Plot, PlotBounds, PlotDiagnostics, PlotLayer, PlotMatrix,
    PlotMatrixData, PlotPoint, PlotState, Raincloud, RaincloudData, SwarmChart, SwarmData,
    ViolinData, ViolinOptions, ViolinPlot, WordCloud, WordCloudData,
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

fn output_primitives(d: &PlotDiagnostics) -> usize {
    d.output_paths
        + d.output_markers.max(d.stats.output_bubbles)
        + d.output_bars
        + d.output_heatmap_cells.max(d.stats.output_bins)
        + d.stats.output_boxes
        + d.stats.output_violins
        + d.stats.output_slices
        + d.stats.output_clouds
        + d.stats.output_parallel_segments
}

fn visited_items(d: &PlotDiagnostics) -> usize {
    d.visited_segments
        + d.visited_marker_points.max(d.stats.input_bubbles)
        + d.visited_heatmap_cells.max(d.stats.input_bins)
        + d.stats.input_boxes
        + d.stats.input_violins
        + d.stats.input_slices
        + d.stats.input_clouds
        + d.stats.input_parallel_rows
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    // 1 & 2: 200k bivariate samples -> Histogram2 (64x64) + BinScatter Hexagon (48x48)
    let n_biv = 200_000usize;
    let mut xs = Vec::with_capacity(n_biv);
    let mut ys = Vec::with_capacity(n_biv);
    let mut pts: Vec<PlotPoint> = Vec::with_capacity(n_biv);
    for i in 0..n_biv {
        let a = i as f64 * 0.037;
        let r = ((i % 256) as f64 / 256.0) * 3.0;
        let x = r * a.cos() + (i % 5) as f64 * 0.35;
        let y = r * a.sin() * 0.8 + 0.5 * x;
        xs.push(x);
        ys.push(y);
        pts.push([x, y]);
    }
    let (h2_ms, hist2) = bench_preprocess_p50_ms(|| Histogram2Data::new(&xs, &ys, 64, 64).unwrap());
    let (bs_ms, binscatter) =
        bench_preprocess_p50_ms(|| BinScatterData::new(&pts, 48, 48, BinShape::Hexagon).unwrap());

    // 3: 50k samples across 4 groups (2 violin + 2 raincloud) exercising binned KDE (n > 1024)
    let mut groups_raw: Vec<Vec<f64>> = Vec::with_capacity(4);
    for g in 0..4 {
        let mut col = Vec::with_capacity(12_500);
        for i in 0..12_500 {
            let t = i as f64 * 0.013 + g as f64 * 0.7;
            col.push(4.0 + g as f64 * 1.4 + t.sin() * 1.3 + (t * 2.7).cos() * 0.6);
        }
        groups_raw.push(col);
    }
    let g_refs: Vec<&[f64]> = groups_raw.iter().map(|v| v.as_slice()).collect();
    let (violin_ms, (violin, raincloud)) = bench_preprocess_p50_ms(|| {
        let v =
            ViolinData::from_groups(&[1.0, 2.0], &g_refs[..2], ViolinOptions::default()).unwrap();
        let r = RaincloudData::from_groups(&[3.0, 4.0], &g_refs[2..], ViolinOptions::default())
            .unwrap();
        (v, r)
    });

    // 4: 5k swarm + bubble points (2500 swarm + 2500 bubble)
    let mut sx = Vec::with_capacity(2_500);
    let mut sy = Vec::with_capacity(2_500);
    let mut b_pts = Vec::with_capacity(2_500);
    let mut b_sizes = Vec::with_capacity(2_500);
    let mut b_colors = Vec::with_capacity(2_500);
    for i in 0..2_500 {
        sx.push((i % 5) as f64 + 1.0);
        sy.push(2.0 + ((i as f64) * 0.09).sin() * 1.8);
        b_pts.push([0.5 + (i % 50) as f64 * 0.1, (i / 50) as f64 * 0.1]);
        b_sizes.push(5.0 + (i % 40) as f64 * 2.0);
        b_colors.push((i % 20) as f64);
    }
    let (sb_ms, (swarm, bubble)) = bench_preprocess_p50_ms(|| {
        let sw = SwarmData::new(&sx, &sy, 0.6).unwrap();
        let bb = BubbleData::new(&b_pts, &b_sizes)
            .unwrap()
            .with_colors(&b_colors)
            .unwrap();
        (sw, bb)
    });

    // 5: parallel_1k_x_8 (1000 rows x 8 variables)
    let par_names = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7"];
    let par_rows_owned: Vec<Vec<f64>> = (0..1_000)
        .map(|r| {
            (0..8)
                .map(|c| (r as f64 * 0.05 + c as f64 * 0.7).sin() * (c + 1) as f64)
                .collect()
        })
        .collect();
    let par_refs: Vec<&[f64]> = par_rows_owned.iter().map(|r| r.as_slice()).collect();
    let par_groups: Vec<usize> = (0..1_000).map(|r| r % 4).collect();
    let (par_ms, parallel) = bench_preprocess_p50_ms(|| {
        ParallelData::new(&par_names, &par_refs, ParallelNormalization::ZScore)
            .unwrap()
            .with_groups(&par_groups, &["G0", "G1", "G2", "G3"])
            .unwrap()
    });

    // 6: pie_clouds_200 (100 words + 88 bubbles + 12 slices)
    let word_labels: Vec<String> = (0..100).map(|i| format!("term_{i:02}")).collect();
    let word_refs: Vec<&str> = word_labels.iter().map(String::as_str).collect();
    let word_wts: Vec<f64> = (0..100).map(|i| 120.0 - i as f64).collect();
    let pie_vals = [10.0; 12];
    let pie_labels = [
        "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8", "S9", "S10", "S11",
    ];
    let bc_groups: Vec<usize> = (0..88).map(|i| i % 5).collect();
    let (clouds_ms, (wc, bc, pie)) = bench_preprocess_p50_ms(|| {
        let w = WordCloudData::new(&word_refs, &word_wts).unwrap();
        let b = BubbleCloudData::new(&word_refs[..88], &word_wts[..88], Some(&bc_groups)).unwrap();
        let p = PieData::with_labels(&pie_vals, &pie_labels).unwrap();
        (w, b, p)
    });

    // 7: plotmatrix_4x4 (4 vars x 1000 rows)
    let pm_cols: Vec<Vec<f64>> = (0..4)
        .map(|c| {
            (0..1_000)
                .map(|r| (r as f64 * 0.03 + c as f64 * 0.8).sin() + (r % 7) as f64 * 0.15)
                .collect()
        })
        .collect();
    let (pm_ms, plot_matrix) = bench_preprocess_p50_ms(|| {
        PlotMatrixData::new(
            &["V0", "V1", "V2", "V3"],
            &[&pm_cols[0], &pm_cols[1], &pm_cols[2], &pm_cols[3]],
            Some(&par_groups),
            12,
        )
        .unwrap()
    });

    println!(
        "# Stage 8C CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: histogram2_200k={h2_ms:.4}, binscatter_hex_200k={bs_ms:.4}, violin_raincloud_50k={violin_ms:.4}, swarm_bubble_5k={sb_ms:.4}, parallel_1k_x_8={par_ms:.4}, pie_clouds_200={clouds_ms:.4}, plotmatrix_4x4={pm_ms:.4}; h2_bins={}, bs_cells={}",
        hist2.counts().len(),
        binscatter.cells().len(),
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    let biv_scale = ColorScale::new([0.0, hist2.max_count().max(1) as f64])
        .unwrap()
        .color_map(ColorMap::Viridis);
    measure_plot(
        "histogram2_200k",
        n_biv,
        n_biv * size_of::<PlotPoint>(),
        hist2.bounds([Default::default(); 2]).unwrap(),
        false,
        seconds,
        &[Histogram2::new(&hist2, biv_scale).into()],
    );

    let bin_scale = ColorScale::new([0.0, binscatter.max_count().max(1) as f64])
        .unwrap()
        .color_map(ColorMap::Viridis);
    measure_plot(
        "binscatter_hex_200k",
        n_biv,
        n_biv * size_of::<PlotPoint>(),
        binscatter.bounds([Default::default(); 2]).unwrap(),
        false,
        seconds,
        &[BinScatter::new(&binscatter, bin_scale).into()],
    );

    let vr_bounds = PlotBounds::new([0.2, 0.0], [4.8, 8.5]).unwrap();
    measure_plot(
        "violin_raincloud_50k",
        50_000,
        50_000 * size_of::<f64>(),
        vr_bounds,
        false,
        seconds,
        &[
            ViolinPlot::new(&violin)
                .color(Color32::from_rgb(126, 47, 142))
                .into(),
            Raincloud::new(&raincloud)
                .color(Color32::from_rgb(119, 172, 48))
                .into(),
        ],
    );

    let sb_scale = ColorScale::new([0.0, 20.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    let sb_bounds = PlotBounds::new([0.0, -0.5], [6.0, 5.5]).unwrap();
    measure_plot(
        "swarm_bubble_5k",
        5_000,
        2_500 * size_of::<[f64; 2]>() + 2_500 * size_of::<[f64; 3]>(),
        sb_bounds,
        false,
        seconds,
        &[
            BubbleChart::new(&bubble).color_scale(sb_scale).into(),
            SwarmChart::new(&swarm)
                .color(Color32::from_rgb(217, 83, 25))
                .into(),
        ],
    );

    measure_plot(
        "parallel_1k_x_8",
        1_000 * 7,
        1_000 * 8 * size_of::<f64>(),
        parallel.bounds([Default::default(); 2]).unwrap(),
        false,
        seconds,
        &[ParallelPlot::new(&parallel).into()],
    );

    measure_plot(
        "pie_clouds_200",
        200,
        200 * size_of::<[f64; 4]>(),
        PlotBounds::new([-4.0, -4.0], [4.0, 4.0]).unwrap(),
        true,
        seconds,
        &[
            PieChart::donut(&pie).show_labels(false).into(),
            WordCloud::new(&wc).into(),
            BubbleCloud::new(&bc).show_labels(false).into(),
        ],
    );

    measure_plotmatrix(
        "plotmatrix_4x4",
        16 * 1_000,
        4 * 1_000 * size_of::<f64>(),
        seconds,
        &plot_matrix,
    );
}

fn measure_plot(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    bounds: PlotBounds,
    canvas_axes: bool,
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
            let mut plot = Plot::new("stage8c_bench")
                .height(1124.0)
                .show_grid(false)
                .show_hover(false);
            if canvas_axes {
                plot = plot.canvas_axes();
            }
            let response = plot.show_layers(ui, &mut state, layers);
            if !canvas_axes {
                assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
            } else {
                let sz = response.transform.rect().size();
                assert!(sz.x > 0.0 && sz.y > 0.0);
            }
            last_out_prims = output_primitives(&response.diagnostics);
            last_visited = visited_items(&response.diagnostics);
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

fn measure_plotmatrix(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    seconds: f64,
    pm_data: &PlotMatrixData,
) {
    let ctx = egui::Context::default();
    let mut states = Vec::new();
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
            let fig = PlotMatrix::new("stage8c_pm", pm_data)
                .cell_height(260.0)
                .show(ui, &mut states)
                .expect("valid plotmatrix");
            assert_eq!(fig.inner.len(), 16);
            assert!(fig.inner[0].transform.rect().width() > 0.0);
            last_out_prims = fig
                .inner
                .iter()
                .map(|r| output_primitives(&r.diagnostics))
                .sum();
            last_visited = fig
                .inner
                .iter()
                .map(|r| visited_items(&r.diagnostics))
                .sum();
            last_geom_bytes = fig
                .inner
                .iter()
                .map(|r| r.diagnostics.geometry_capacity_bytes)
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
