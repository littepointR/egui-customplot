//! Headless CPU measurements, not GPU, presentation or process memory.
use egui_customplot::{ColorScale, GridData, Heatmap, Plot, PlotBounds, PlotState};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);
    println!(
        "# heatmap CPU benchmark; profile=bench; egui=0.37.0; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# Cells are exact piecewise constant quads, not texture LOD. No window, GPU, FPS, process-memory or idle-wakeup conclusions."
    );
    println!(
        "cells,scene,frames,construction_ms,sample_bytes,build_p50_ms,build_p95_ms,build_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,visited_cells,output_cells,output_meshes,geometry_capacity_bytes"
    );
    for (side, zoom) in [(256, false), (1024, true), (1024, false)] {
        let start = Instant::now();
        let bounds = PlotBounds::new([0.0; 2], [side as f64; 2]).unwrap();
        let values = (0..side * side)
            .map(|i| ((i % side) as f64 * 0.01).sin() * ((i / side) as f64 * 0.01).cos())
            .collect::<Vec<_>>();
        let data = GridData::uniform(side, side, bounds, values).unwrap();
        let construction = start.elapsed().as_secs_f64() * 1000.0;
        let ctx = egui::Context::default();
        let mut state = PlotState::default();
        state.set_bounds(if zoom {
            PlotBounds::new([400.25; 2], [416.75; 2]).unwrap()
        } else {
            bounds
        });
        let colors = ColorScale::new([-1.0, 1.0]).unwrap();
        let mut build = Vec::new();
        let mut tess = Vec::new();
        let mut total = Vec::new();
        let mut frames = 0;
        let mut stats = None;
        let mut measurement = Instant::now();
        loop {
            let start = Instant::now();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(2024.0, 1124.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let out = Plot::new("heatmap-bench")
                        .height(1124.0)
                        .show_hover(false)
                        .show_layers(ui, &mut state, &[Heatmap::new(&data, colors).into()]);
                    assert_eq!(out.transform.rect().size(), egui::vec2(1920.0, 1080.0));
                    stats = Some(out.diagnostics);
                },
            );
            let build_ms = start.elapsed().as_secs_f64() * 1000.0;
            let tess_start = Instant::now();
            black_box(ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point));
            let tess_ms = tess_start.elapsed().as_secs_f64() * 1000.0;
            output.drop_without_applying_deltas();
            frames += 1;
            if frames == 5 {
                measurement = Instant::now();
            }
            if frames > 5 {
                build.push(build_ms);
                tess.push(tess_ms);
                total.push(build_ms + tess_ms);
            }
            if frames > 5 && measurement.elapsed() >= Duration::from_secs_f64(seconds) {
                break;
            }
        }
        let d = stats.unwrap();
        let scene = if zoom { "zoom_17x17" } else { "full" };
        println!(
            "{},{scene},{},{construction:.4},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{}",
            side * side,
            frames - 5,
            data.sample_bytes(),
            percentile(&mut build, 0.50),
            percentile(&mut build, 0.95),
            percentile(&mut build, 0.99),
            percentile(&mut tess, 0.50),
            percentile(&mut tess, 0.95),
            percentile(&mut tess, 0.99),
            percentile(&mut total, 0.50),
            percentile(&mut total, 0.95),
            percentile(&mut total, 0.99),
            d.visited_heatmap_cells,
            d.output_heatmap_cells,
            d.output_heatmap_meshes,
            d.geometry_capacity_bytes
        );
    }
}
