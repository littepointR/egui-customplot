//! Headless CPU + tessellation benchmark for 10M-point live & interactive paths
//! (`egui-customplot` L1 smart defaults + L3 IndexedStream / UniformWaveform).
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui_customplot::{
    IndexedStream, Line, Plot, PlotBounds, PlotData, PlotDiagnostics, PlotLayer, PlotPoint,
    PlotResponse, PlotState, Scatter, Series, StreamLine, UniformWaveform, WaveformLine,
};

#[derive(Debug)]
struct ReferenceLevel;
impl egui_customplot::CustomPlotLayer for ReferenceLevel {
    fn id(&self) -> egui::IdSalt {
        egui::IdSalt::new("reference")
    }
    fn extent(&self, _: [egui_customplot::AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        None
    }
    fn paint(&self, c: &egui_customplot::LayerPaintContext<'_>) {
        let y = c.transform.position_from_point([0., 0.5]).y;
        let rect = c.transform.rect();
        c.painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            c.stroke,
        );
    }
}

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

#[allow(clippy::too_many_arguments)]
fn measure_scene<S>(
    missed: &mut Vec<String>,
    bench_mode: bool,
    seconds: f64,
    points: usize,
    batch_points: usize,
    scene: &str,
    mut state: S,
    mut build_frame: impl FnMut(&mut S, &egui::Context, &mut egui::Ui, usize) -> PlotResponse,
    mut step_events: impl FnMut(&mut S, usize) -> Vec<egui::Event>,
    mut update_host: impl FnMut(&mut S, usize),
) {
    let ctx = egui::Context::default();
    let mut build = Vec::new();
    let mut tess = Vec::new();
    let mut total = Vec::new();
    let mut query = Vec::new();
    let mut pick = Vec::new();
    let mut counters = PlotDiagnostics::default();
    let mut vertices = 0usize;
    let mut measurement_start = None;
    let mut frame = 0usize;
    let warmup = if bench_mode { 5 } else { 2 };
    let target_frames = if bench_mode { usize::MAX } else { 8 };

    loop {
        if frame >= warmup {
            let start = measurement_start.get_or_insert_with(Instant::now);
            if total.len() >= target_frames
                || (bench_mode
                    && start.elapsed() >= Duration::from_secs_f64(seconds)
                    && total.len() >= 20)
            {
                break;
            }
        }
        let events = step_events(&mut state, frame);
        let start = Instant::now();
        update_host(&mut state, frame);
        let mut output = ctx.run_ui(input(events), |ui| {
            let response = build_frame(&mut state, &ctx, ui, frame);
            assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
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
        if frame >= warmup {
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
    let b = [0.5, 0.95].map(|q| percentile(&mut build, q));
    let t = [0.5, 0.95].map(|q| percentile(&mut tess, q));
    let all = [0.5, 0.95, 0.99].map(|q| percentile(&mut total, q));
    let q95 = percentile(&mut query, 0.95);
    let p95 = percentile(&mut pick, 0.95);
    println!(
        "{points},{scene},{batch_points},{frames},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{q95:.4},{p95:.4},{},{},{vertices},{},{}",
        b[0],
        b[1],
        t[0],
        t[1],
        all[0],
        all[1],
        all[2],
        counters.output_points,
        counters.output_markers,
        counters.auto_lod_series,
        counters.marker_lod_series,
    );
    if bench_mode && all[1] >= 8.33 {
        missed.push(format!("{scene} ({:.4} ms)", all[1]));
    }
}

fn main() {
    let mut missed = Vec::new();
    let bench_mode = std::env::args().any(|a| a == "--bench");
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(2.0);
    let line_count = if bench_mode { 10_000_000 } else { 50_000 };
    let scatter_count = if bench_mode { 1_000_000 } else { 20_000 };
    let batch_size = if bench_mode { 32_768 } else { 1_024 };

    println!(
        "# CPU+tessellation 10M benchmark; bench_mode={bench_mode}; canvas=2024x1124; plot=1920x1080; dpi=1; seconds_per_scene={seconds}"
    );
    println!(
        "# No GPU swapchain/presentation in this binary; see customplot-demo --bench-window-10m for real wgpu swapchain FPS."
    );
    println!(
        "points,scene,batch_points,frames,build_p50_ms,build_p95_ms,tess_p50_ms,tess_p95_ms,total_p50_ms,total_p95_ms,total_p99_ms,query_p95_ms,pick_p95_ms,output_points,output_markers,mesh_vertices,auto_lod_series,marker_lod_series"
    );

    // 1. IndexedStream 10M ring + incremental 32k append/frame
    let mut stream = IndexedStream::new(line_count, 512 * 1024 * 1024).unwrap();
    let init_pts: Vec<PlotPoint> = (0..line_count)
        .map(|i| {
            let x = i as f64 * 0.001;
            [x, (x * 0.12).sin() * 0.8]
        })
        .collect();
    stream.append_once(1, &init_pts).unwrap();
    drop(init_pts);
    let batch_pts: Vec<PlotPoint> = (0..batch_size)
        .map(|k| [k as f64 * 0.001, (k as f64 * 0.01).sin()])
        .collect();
    let stream_batch_buf = batch_pts;
    let s_stream = PlotState::default();
    let next_x = line_count;
    measure_scene(
        &mut missed,
        bench_mode,
        seconds,
        line_count,
        batch_size,
        "indexed_stream_10m_live_32k",
        (stream, next_x, stream_batch_buf, s_stream),
        |(stream, _, _, s_stream), _, ui, _| {
            Plot::new("s10m")
                .height(1124.0)
                .show_hover(false)
                .show_stream(ui, s_stream, &[StreamLine::new(stream)])
        },
        |_, _| vec![],
        |(stream, next_x, stream_batch_buf, _), f| {
            for (k, p) in stream_batch_buf.iter_mut().enumerate() {
                p[0] = (*next_x + k) as f64 * 0.001;
            }
            *next_x += batch_size;
            stream.append_once(f as u64 + 2, stream_batch_buf).unwrap();
        },
    );

    // 2. UniformWaveform 10M + incremental 32k sweep write/frame
    let ys: Vec<f32> = (0..line_count)
        .map(|i| ((i as f64 * 0.0002).sin() * 0.8) as f32)
        .collect();
    let wave = UniformWaveform::from_f32(0.0, 0.001, ys, 128 * 1024 * 1024).unwrap();
    let wave_patch: Vec<f32> = (0..batch_size)
        .map(|k| ((k as f64 * 0.01).cos() * 0.8) as f32)
        .collect();
    let s_wave = PlotState::default();
    let wave_cursor = 0usize;
    measure_scene(
        &mut missed,
        bench_mode,
        seconds,
        line_count,
        batch_size,
        "uniform_waveform_10m_live_32k",
        (wave, wave_cursor, wave_patch, s_wave),
        |(wave, _, _, s_wave), _, ui, _| {
            Plot::new("w10m")
                .height(1124.0)
                .show_hover(false)
                .show_waveform(ui, s_wave, &[WaveformLine::new(wave)])
        },
        |_, _| vec![],
        |(wave, wave_cursor, wave_patch, _), f| {
            let count = batch_size.min(line_count - *wave_cursor);
            wave.write_range_once(f as u64 + 1, *wave_cursor, &wave_patch[..count])
                .unwrap();
            *wave_cursor = if *wave_cursor + count >= line_count {
                0
            } else {
                *wave_cursor + count
            };
        },
    );

    // 3. Default PlotData::new(10M) + Line::new auto LOD continuous pan + hover pick
    let mono = PlotData::new(
        (0..line_count)
            .map(|i| {
                let x = i as f64 / (line_count - 1) as f64 * 100.0;
                [x, (x * 0.6).sin()]
            })
            .collect::<Vec<_>>(),
    );
    for with_reference in [false, true] {
        let s_line = PlotState::default();
        let hover_pos = egui::pos2(960.0, 560.0);
        measure_scene(
            &mut missed,
            bench_mode,
            seconds,
            line_count,
            0,
            if with_reference {
                "plotdata_auto_10m_pan_hover_custom_reference"
            } else {
                "plotdata_auto_10m_pan_hover"
            },
            (hover_pos, s_line),
            |(pos, s_line), _, ui, f| {
                let x = 25.0 + (f as f64 * 0.03).sin() * 20.0;
                s_line.set_bounds(PlotBounds::new([x, -1.3], [x + 30.0, 1.3]).unwrap());
                let layers = [Line::new(&mono).into(), PlotLayer::custom(&ReferenceLevel)];
                let r = Plot::new("l10m")
                    .height(1124.)
                    .show_hover(true)
                    .show_layers(
                        ui,
                        s_line,
                        if with_reference {
                            &layers
                        } else {
                            &layers[..1]
                        },
                    );
                *pos = r.transform.rect().center();
                r
            },
            |(pos, _), _| vec![egui::Event::PointerMoved(*pos)],
            |_, _| {},
        );
    }
    drop(mono);

    // 4. Default PlotData::new(1M) + Scatter::new PixelGrid auto LOD continuous pan
    let scatter_data = PlotData::new(
        (0..scatter_count)
            .map(|i| {
                let x = i as f64 / (scatter_count - 1) as f64 * 100.0;
                [x, (x * 0.9).cos()]
            })
            .collect::<Vec<_>>(),
    );
    let s_scatter = PlotState::default();
    measure_scene(
        &mut missed,
        bench_mode,
        seconds,
        scatter_count,
        0,
        "scatter_auto_1m_pan",
        s_scatter,
        |s_scatter, _, ui, f| {
            let x = 20.0 + (f as f64 * 0.04).sin() * 15.0;
            s_scatter.set_bounds(PlotBounds::new([x, -1.3], [x + 40.0, 1.3]).unwrap());
            Plot::new("sc1m")
                .height(1124.0)
                .show_hover(false)
                .show_series(ui, s_scatter, &[Series::from(Scatter::new(&scatter_data))])
        },
        |_, _| vec![],
        |_, _| {},
    );

    // 5. 1M Scatter + deep zoom-out sweep (10x .. 10^8x) + active hover on cluster
    let s_sc_zoom = PlotState::default();
    let sc_hover = egui::pos2(960.0, 560.0);
    measure_scene(
        &mut missed,
        bench_mode,
        seconds,
        scatter_count,
        0,
        "scatter_auto_1m_zoom_out_hover",
        (sc_hover, s_sc_zoom),
        |(pos, s_sc_zoom), _, ui, f| {
            let t = ((f as f64 * 0.03).sin() + 1.0) * 0.5;
            let half = 50.0 * 10.0_f64.powf(1.0 + 7.0 * t);
            s_sc_zoom.set_bounds(
                PlotBounds::new([50.0 - half, -half * 0.05], [50.0 + half, half * 0.05]).unwrap(),
            );
            let r = Plot::new("sc1mz")
                .height(1124.0)
                .show_hover(true)
                .show_series(ui, s_sc_zoom, &[Series::from(Scatter::new(&scatter_data))]);
            *pos = r.transform.position_from_point([50.0, 0.0]);
            r
        },
        |(pos, _), _| vec![egui::Event::PointerMoved(*pos)],
        |_, _| {},
    );
    drop(scatter_data);

    // 6. 10M PlotData + 10M live 32k UniformWaveform + 10M live 32k IndexedStream under deep zoom-out + active hover
    let z_pts: Vec<PlotPoint> = (0..line_count)
        .map(|i| {
            let x = i as f64 * 0.001;
            [x, ((x * 0.12).sin() * 0.8) as f32 as f64]
        })
        .collect();
    let z_mono = PlotData::new(z_pts.clone());
    let z_ys: Vec<f32> = z_pts.iter().map(|p| p[1] as f32).collect();
    let z_wave = UniformWaveform::from_f32(0.0, 0.001, z_ys, 128 * 1024 * 1024).unwrap();
    let mut z_stream = IndexedStream::new(line_count, 512 * 1024 * 1024).unwrap();
    z_stream.append_once(1, &z_pts).unwrap();
    drop(z_pts);
    let z_wave_patch: Vec<f32> = (0..batch_size)
        .map(|k| ((k as f64 * 0.01).cos() * 0.8) as f32)
        .collect();
    let z_stream_batch: Vec<PlotPoint> = (0..batch_size)
        .map(|k| [k as f64 * 0.001, (k as f64 * 0.01).sin()])
        .collect();
    let s_zoom = PlotState::default();
    let z_hover = egui::pos2(960.0, 560.0);
    let z_anchor_x = (line_count as f64 * 0.001) * 0.5;
    measure_scene(
        &mut missed,
        bench_mode,
        seconds,
        line_count,
        batch_size,
        "zoom_out_deep_10m_hover",
        (
            z_hover,
            z_mono,
            z_wave,
            0usize,
            z_wave_patch,
            z_stream,
            line_count,
            z_stream_batch,
            s_zoom,
        ),
        |(pos, z_mono, z_wave, _, _, z_stream, _, _, s_zoom), _, ui, f| {
            let t = ((f as f64 * 0.03).sin() + 1.0) * 0.5;
            let half = z_anchor_x * 10.0_f64.powf(1.3 + 10.7 * t);
            s_zoom.set_bounds(
                PlotBounds::new(
                    [z_anchor_x - half, -half * 0.05],
                    [z_anchor_x + half, half * 0.05],
                )
                .unwrap(),
            );
            let r = Plot::new("z10m")
                .height(1124.0)
                .show_hover(true)
                .show_layers(
                    ui,
                    s_zoom,
                    &[
                        PlotLayer::from(Line::new(z_mono)),
                        PlotLayer::from(WaveformLine::new(z_wave)),
                        PlotLayer::from(StreamLine::new(z_stream)),
                    ],
                );
            *pos = r.transform.position_from_point([z_anchor_x, 0.0]);
            r
        },
        |(pos, _, _, _, _, _, _, _, _), _| vec![egui::Event::PointerMoved(*pos)],
        |(_, _, z_wave, wave_cursor, z_wave_patch, z_stream, next_x, z_stream_batch, _), f| {
            let count = batch_size.min(line_count - *wave_cursor);
            z_wave
                .write_range_once(f as u64 + 1, *wave_cursor, &z_wave_patch[..count])
                .unwrap();
            *wave_cursor = if *wave_cursor + count >= line_count {
                0
            } else {
                *wave_cursor + count
            };
            for (k, p) in z_stream_batch.iter_mut().enumerate() {
                p[0] = (*next_x + k) as f64 * 0.001;
            }
            *next_x += batch_size;
            z_stream.append_once(f as u64 + 2, z_stream_batch).unwrap();
        },
    );
    assert!(
        missed.is_empty(),
        "CPU + tessellation p95 exceeded 8.33 ms: {}",
        missed.join(", ")
    );
}
