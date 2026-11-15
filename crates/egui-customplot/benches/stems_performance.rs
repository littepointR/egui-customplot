//! Exact rods and raw markers: headless CPU only, never GPU or presentation.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui_customplot::{Plot, PlotBounds, PlotData, PlotState, Stems};

fn input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2024.0, 1124.0),
        )),
        ..Default::default()
    }
}

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
        "# Exact stems CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}; markers=true; baseline=true; hover=false; no_index"
    );
    println!(
        "count,scene,frames,seconds,ui_p50_ms,ui_p95_ms,tess_p50_ms,tess_p95_ms,total_p50_ms,total_p95_ms,visited_stems,rod_paths,markers,geometry_capacity_bytes,sample_bytes"
    );
    for count in [1000, 10_000, 100_000] {
        let data = PlotData::new(
            (0..count)
                .map(|i| [i as f64, (i as f64 * 0.13).sin()])
                .collect::<Vec<_>>(),
        );
        for narrow in [false, true] {
            measure(&data, narrow, seconds);
        }
    }
}

fn measure(data: &PlotData, narrow: bool, seconds: f64) {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    let max_x = data.len() as f64 - 1.0;
    let x = if narrow {
        [max_x * 0.49, max_x * 0.51]
    } else {
        [0.0, max_x]
    };
    state.set_bounds(PlotBounds::new([x[0], -1.1], [x[1], 1.1]).unwrap());
    let mut frame = || {
        let start = Instant::now();
        let mut diagnostics = None;
        let mut output = ctx.run_ui(input(), |ui| {
            let response = Plot::new("stems")
                .height(1124.0)
                .show_grid(false)
                .show_hover(false)
                .show_layers(ui, &mut state, &[Stems::new(data).into()]);
            assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
            diagnostics = Some(response.diagnostics);
        });
        let ui_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let meshes = ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
        let tess_ms = start.elapsed().as_secs_f64() * 1000.0;
        black_box(meshes);
        output.drop_without_applying_deltas();
        (ui_ms, tess_ms, diagnostics.unwrap())
    };
    for _ in 0..5 {
        black_box(frame());
    }
    let start = Instant::now();
    let mut ui = Vec::new();
    let mut tess = Vec::new();
    let mut total = Vec::new();
    let mut last;
    loop {
        let (a, b, diagnostics) = frame();
        ui.push(a);
        tess.push(b);
        total.push(a + b);
        last = diagnostics;
        if start.elapsed() >= Duration::from_secs_f64(seconds) {
            break;
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let frames = ui.len();
    let values = [
        percentile(&mut ui, 0.5),
        percentile(&mut ui, 0.95),
        percentile(&mut tess, 0.5),
        percentile(&mut tess, 0.95),
        percentile(&mut total, 0.5),
        percentile(&mut total, 0.95),
    ];
    println!(
        "{},{},{frames},{elapsed:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{},{}",
        data.len(),
        if narrow { "narrow_2pct" } else { "full" },
        values[0],
        values[1],
        values[2],
        values[3],
        values[4],
        values[5],
        last.visited_stems,
        last.output_stem_segments,
        last.output_markers,
        last.geometry_capacity_bytes,
        data.sample_bytes()
    );
}
