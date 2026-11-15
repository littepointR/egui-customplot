//! CPU-only run_ui + tessellation baseline. No window, renderer, VSync or GPU.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui_customplot::{Line, Plot, PlotBounds, PlotData, PlotState};

fn percentile(samples: &mut [f64], percentile: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * percentile).round() as usize]
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);
    let start = Instant::now();
    let data = PlotData::new(
        (0..100_000)
            .map(|i| {
                let x = i as f64 / 99_999.0 * 100.0;
                [x, x.sin()]
            })
            .collect::<Vec<_>>(),
    );
    println!(
        "# customplot CPU baseline; profile=bench; egui=0.37; canvas=2024x1124; plot=1920x1080; pixels_per_point=1"
    );
    println!(
        "# data construction_ms={:.3}; sample_bytes={}; seconds_per_scene={seconds}",
        start.elapsed().as_secs_f64() * 1000.0,
        data.sample_bytes()
    );
    println!(
        "scene,frames,build_p50_ms,build_p95_ms,build_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,visited_segments,output_points,geometry_capacity_bytes,mesh_vertices"
    );
    for scene in ["empty_egui", "empty_plot", "100k_static", "100k_pan"] {
        let ctx = egui::Context::default();
        let mut state = PlotState::default();
        let mut build = Vec::new();
        let mut tess = Vec::new();
        let mut total = Vec::new();
        let mut counters = (0, 0, 0, 0);
        let sampling_start = Instant::now();
        let mut frame = 0;
        let mut measurement_start = None;
        while measurement_start
            .is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds))
        {
            if scene == "100k_pan" {
                let offset = (frame as f64 * 0.02).sin() * 10.0;
                state.set_bounds(PlotBounds::new([offset, -1.1], [offset + 100.0, 1.1]).unwrap());
            }
            let start = Instant::now();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(2024.0, 1124.0),
                    )),
                    time: Some(sampling_start.elapsed().as_secs_f64()),
                    ..Default::default()
                },
                |ui| {
                    if scene != "empty_egui" {
                        let lines = [Line::new(&data)];
                        let lines = if scene == "empty_plot" {
                            &[][..]
                        } else {
                            &lines[..]
                        };
                        let response = Plot::new("benchmark")
                            .height(1124.0)
                            .show(ui, &mut state, lines);
                        assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
                        let d = response.diagnostics;
                        counters.0 = d.visited_segments;
                        counters.1 = d.output_points;
                        counters.2 = d.geometry_capacity_bytes;
                    }
                },
            );
            let built = start.elapsed();
            let meshes =
                ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
            let complete = start.elapsed();
            counters.3 = meshes
                .iter()
                .map(|mesh| match &mesh.primitive {
                    egui::epaint::Primitive::Mesh(mesh) => mesh.vertices.len(),
                    egui::epaint::Primitive::Callback(_) => 0,
                })
                .sum::<usize>();
            black_box(&meshes);
            output.drop_without_applying_deltas();
            if frame >= 30 {
                measurement_start.get_or_insert_with(Instant::now);
                build.push(built.as_secs_f64() * 1000.0);
                tess.push((complete - built).as_secs_f64() * 1000.0);
                total.push(complete.as_secs_f64() * 1000.0);
            }
            frame += 1;
        }
        let b = [
            percentile(&mut build, 0.5),
            percentile(&mut build, 0.95),
            percentile(&mut build, 0.99),
        ];
        let t = [
            percentile(&mut tess, 0.5),
            percentile(&mut tess, 0.95),
            percentile(&mut tess, 0.99),
        ];
        let all = [
            percentile(&mut total, 0.5),
            percentile(&mut total, 0.95),
            percentile(&mut total, 0.99),
        ];
        println!(
            "{scene},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{}",
            total.len(),
            b[0],
            b[1],
            b[2],
            t[0],
            t[1],
            t[2],
            all[0],
            all[1],
            all[2],
            counters.0,
            counters.1,
            counters.2,
            counters.3
        );
    }
}
