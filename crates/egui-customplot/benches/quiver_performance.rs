//! Explicit 2D vectors; headless CPU only, no GPU/presentation measurements.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui_customplot::{Plot, PlotBounds, PlotState, Quiver, QuiverArrow, QuiverData};

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
        "# Exact quiver CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}; heads=10x8pt; solid; no_autoscale; no_index_or_LOD"
    );
    println!(
        "count,scene,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,pick_p95_ms,visited_arrows,shafts,heads,pick_visits,geometry_capacity_bytes,sample_bytes"
    );
    for count in [1000, 10_000, 100_000] {
        let data = QuiverData::new(
            (0..count)
                .map(|i| {
                    let x = (i % 100) as f64 + 1.0;
                    let y = (i / 100) as f64 + 1.0;
                    QuiverArrow::new([x, y], [0.4 * (y * 0.2).cos(), 0.4 * (x * 0.2).sin()])
                })
                .collect::<Vec<_>>(),
        );
        for scene in ["full", "narrow_2pct", "raw_hover_full"] {
            measure(&data, scene, seconds);
        }
    }
}

fn measure(data: &QuiverData, scene: &str, seconds: f64) {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    let x = if scene == "narrow_2pct" {
        [49.5, 51.5]
    } else {
        [0.0, 101.0]
    };
    state.set_bounds(PlotBounds::new([x[0], 0.0], [x[1], (data.len() / 100 + 1) as f64]).unwrap());
    let mut pointer = egui::Pos2::ZERO;
    let mut frame = || {
        let start = Instant::now();
        let mut diagnostics = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(2024.0, 1124.0),
                )),
                events: if scene == "raw_hover_full" {
                    vec![egui::Event::PointerMoved(pointer)]
                } else {
                    vec![]
                },
                ..Default::default()
            },
            |ui| {
                let response = Plot::new("quiver")
                    .height(1124.0)
                    .show_grid(false)
                    .show_hover(scene == "raw_hover_full")
                    .show_layers(ui, &mut state, &[Quiver::new(data).into()]);
                assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
                pointer = response
                    .transform
                    .position_from_point(data.arrows()[data.len() / 2 + 49].origin);
                diagnostics = Some(response.diagnostics);
            },
        );
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
    let mut pick = Vec::new();
    let mut last;
    loop {
        let (a, b, diagnostics) = frame();
        ui.push(a);
        tess.push(b);
        total.push(a + b);
        pick.push(diagnostics.pick_duration.as_secs_f64() * 1000.0);
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
        percentile(&mut ui, 0.99),
        percentile(&mut tess, 0.5),
        percentile(&mut tess, 0.95),
        percentile(&mut tess, 0.99),
        percentile(&mut total, 0.5),
        percentile(&mut total, 0.95),
        percentile(&mut total, 0.99),
    ];
    println!(
        "{},{},{frames},{elapsed:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{},{},{},{}",
        data.len(),
        scene,
        values[0],
        values[1],
        values[2],
        values[3],
        values[4],
        values[5],
        values[6],
        values[7],
        values[8],
        percentile(&mut pick, 0.95),
        last.visited_quiver_arrows,
        last.output_quiver_shafts,
        last.output_quiver_heads,
        last.visited_pick_quiver_arrows,
        last.geometry_capacity_bytes,
        data.sample_bytes()
    );
}
