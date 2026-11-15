//! Headless CPU measurements only: no window, GPU, renderer or presentation.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui_customplot::{
    Line, LineInterpolation, LineSampling, Plot, PlotBounds, PlotCache, PlotData, PlotDiagnostics,
    PlotState, PlotStream,
};

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}

fn input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2024.0, 1124.0),
        )),
        events,
        ..Default::default()
    }
}

fn main() {
    let interpolation = match std::env::var("CUSTOMPLOT_BENCH_INTERPOLATION").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("linear") => LineInterpolation::Linear,
        Ok("after") => LineInterpolation::StepAfter,
        Ok("before") => LineInterpolation::StepBefore,
        other => panic!("invalid CUSTOMPLOT_BENCH_INTERPOLATION: {other:?}"),
    };
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);
    println!(
        "# CPU data benchmark; interpolation={interpolation:?}; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup_frames=5; seconds_per_scene={seconds}"
    );
    println!(
        "# No GPU/FPS/process-memory conclusions; cache budget=8MiB; index budget=64MiB; input=ordered sine+spikes+3 NaN breaks; no markers"
    );
    println!(
        "points,scene,frames,build_p50_ms,build_p95_ms,build_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,query_p95_ms,pick_p95_ms,visited_segments,selected_points,query_runs,query_comparisons,query_nodes,query_scans,pick_visits,sample_bytes,index_bytes,cache_bytes,query_capacity_bytes,geometry_capacity_bytes,mesh_vertices"
    );
    for count in [100_000, 1_000_000, 10_000_000] {
        let start = Instant::now();
        let mut data = PlotData::new(
            (0..count)
                .map(|i| {
                    let x = i as f64 / (count - 1) as f64 * 100.0;
                    let y = if [count / 4, count / 2, count * 3 / 4].contains(&i) {
                        f64::NAN
                    } else if i % 10_007 == 0 {
                        1.2
                    } else {
                        x.sin()
                    };
                    [x, y]
                })
                .collect::<Vec<_>>(),
        );
        let construction_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        data.build_line_index(64 * 1024 * 1024).unwrap();
        println!(
            "# points={count}; construction_ms={construction_ms:.4}; index_ms={:.4}; sample_bytes={}; index_bytes={}",
            start.elapsed().as_secs_f64() * 1000.0,
            data.sample_bytes(),
            data.line_index_bytes()
        );
        for scene in [
            "exact_zoom",
            "visible_zoom",
            "minmax_full",
            "minmax_pan",
            "cached_full",
            "indexed_hover",
        ] {
            measure(&data, scene, seconds, interpolation);
        }
    }
    stream_measure(seconds);
}

fn measure(data: &PlotData, scene: &str, seconds: f64, interpolation: LineInterpolation) {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(8 * 1024 * 1024);
    let sampling = match scene {
        "exact_zoom" => LineSampling::Exact,
        "visible_zoom" => LineSampling::Visible,
        _ => LineSampling::MinMax,
    };
    let mut build = Vec::new();
    let mut tess = Vec::new();
    let mut total = Vec::new();
    let mut query = Vec::new();
    let mut pick = Vec::new();
    let mut counters = PlotDiagnostics::default();
    let mut vertices = 0;
    let mut measurement_start = None;
    let mut hover_pos = egui::Pos2::ZERO;
    let mut frame = 0usize;
    while measurement_start.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds))
    {
        let x = if scene.ends_with("zoom") {
            [40.0, 40.1]
        } else if scene == "minmax_pan" {
            let x = 40.0 + (frame as f64 * 0.02).sin() * 20.0;
            [x, x + 20.0]
        } else {
            [0.0, 100.0]
        };
        state.set_bounds(PlotBounds::new([x[0], -1.3], [x[1], 1.3]).unwrap());
        let events = if scene == "indexed_hover" {
            vec![egui::Event::PointerMoved(hover_pos)]
        } else {
            vec![]
        };
        let start = Instant::now();
        let mut output = ctx.run_ui(input(events), |ui| {
            let plot = Plot::new("benchmark")
                .height(1124.0)
                .show_hover(scene == "indexed_hover");
            let lines = [Line::new(data)
                .sampling(sampling)
                .interpolation(interpolation)];
            let response = if scene == "cached_full" {
                plot.show_cached(ui, &mut state, &lines, &mut cache)
            } else {
                plot.show(ui, &mut state, &lines)
            };
            assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
            hover_pos = response.transform.rect().center();
            counters = response.diagnostics;
        });
        let built = start.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
        let complete = start.elapsed();
        vertices = meshes
            .iter()
            .map(|m| match &m.primitive {
                egui::epaint::Primitive::Mesh(m) => m.vertices.len(),
                _ => 0,
            })
            .sum::<usize>();
        black_box(&meshes);
        output.drop_without_applying_deltas();
        if frame >= 5 {
            measurement_start.get_or_insert_with(Instant::now);
            build.push(built.as_secs_f64() * 1000.0);
            tess.push((complete - built).as_secs_f64() * 1000.0);
            total.push(complete.as_secs_f64() * 1000.0);
            query.push(counters.query_duration.as_secs_f64() * 1000.0);
            pick.push(counters.pick_duration.as_secs_f64() * 1000.0);
        }
        frame += 1;
    }
    let frames = total.len();
    let b = [0.5, 0.95, 0.99].map(|q| percentile(&mut build, q));
    let t = [0.5, 0.95, 0.99].map(|q| percentile(&mut tess, q));
    let all = [0.5, 0.95, 0.99].map(|q| percentile(&mut total, q));
    println!(
        "{},{scene},{frames},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        data.len(),
        b[0],
        b[1],
        b[2],
        t[0],
        t[1],
        t[2],
        all[0],
        all[1],
        all[2],
        percentile(&mut query, 0.95),
        percentile(&mut pick, 0.95),
        counters.visited_segments,
        counters.query_selected_points,
        counters.query_runs,
        counters.query_comparisons,
        counters.query_index_nodes,
        counters.query_scanned_points,
        counters.visited_pick_points,
        counters.unique_sample_bytes,
        counters.unique_index_bytes,
        cache.retained_bytes(),
        counters.query_capacity_bytes,
        counters.geometry_capacity_bytes,
        vertices
    );
}

fn stream_measure(seconds: f64) {
    let mut stream = PlotStream::new(65_536, 65_536 * 16).unwrap();
    let mut batch = [[0.0; 2]; 256];
    let mut times = Vec::new();
    let start = Instant::now();
    let mut id = 0u64;
    while start.elapsed().as_secs_f64() < seconds {
        for (i, p) in batch.iter_mut().enumerate() {
            let x = (id * 256 + i as u64) as f64 * 0.001;
            *p = [x, x.sin()];
        }
        let start = Instant::now();
        let report = stream.append_once(id, &batch).unwrap();
        assert!(report.applied);
        assert!(!stream.append_once(id, &batch).unwrap().applied);
        let mut snapshot = stream.snapshot();
        snapshot.build_line_index(1024 * 1024).unwrap();
        black_box(snapshot);
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(stream.allocated_bytes(), 65_536 * 16);
        id += 1;
    }
    println!(
        "# stream capacity=65536; batch=256; batches={id}; retained={}; ring_bytes={}; append+snapshot+index_p50_ms={:.4}; p95_ms={:.4}; p99_ms={:.4}; snapshots_dropped_each_batch=true",
        stream.len(),
        stream.allocated_bytes(),
        percentile(&mut times, 0.5),
        percentile(&mut times, 0.95),
        percentile(&mut times, 0.99)
    );
}
