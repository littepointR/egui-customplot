use std::{
    collections::VecDeque,
    f64::consts::TAU,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::egui;
use egui_customplot::{
    IndexedStream, Plot, PlotDiagnostics, PlotPoint, PlotResponse, PlotState, RenderBackend,
    StreamLine, UniformWaveform, WaveformLine,
};

use crate::i18n::{Language, lang, tr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DynamicMode {
    SweepingWaveform,
    StreamingRing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pacing {
    Uncapped,
    Hz240,
    Hz120,
}

struct BenchPhaseStats {
    scene: &'static str,
    batch_points: usize,
    frame_deltas_ms: Vec<f64>,
    prepare_ms: Vec<f64>,
    query_ms: Vec<f64>,
    wgpu_series: usize,
    wgpu_uploaded_bytes: usize,
    wgpu_retained_bytes: usize,
    elapsed_secs: f64,
}

struct WindowBench {
    csv_path: PathBuf,
    phase: usize,
    in_warmup: bool,
    phase_start: Option<Instant>,
    prev_frame: Option<Instant>,
    completed: Vec<BenchPhaseStats>,
    current: BenchPhaseStats,
    done: bool,
}

pub(super) struct Dynamic10mDemo {
    pub enabled: bool,
    pub running: bool,
    pub mode: DynamicMode,
    pub backend: RenderBackend,
    pub batch_size: usize,
    pub pacing: Pacing,
    target_points: usize,
    wave: Option<UniformWaveform>,
    stream: Option<IndexedStream>,
    state: PlotState,
    wave_cursor: usize,
    stream_next_x: usize,
    batch_id: u64,
    last_applied_frame: Option<u64>,
    wave_scratch: Vec<f32>,
    stream_scratch: Vec<PlotPoint>,
    last_frame_at: Option<Instant>,
    frame_deltas_ms: VecDeque<f64>,
    adapter_info: String,
    wgpu_registered: bool,
    last_wgpu_active: usize,
    last_wgpu_fallback: usize,
    bench: Option<WindowBench>,
}

impl Default for Dynamic10mDemo {
    fn default() -> Self {
        Self {
            enabled: false,
            running: true,
            mode: DynamicMode::SweepingWaveform,
            backend: RenderBackend::Auto,
            batch_size: 32_768,
            pacing: Pacing::Uncapped,
            target_points: 10_000_000,
            wave: None,
            stream: None,
            state: PlotState::default(),
            wave_cursor: 0,
            stream_next_x: 0,
            batch_id: 1,
            last_applied_frame: None,
            wave_scratch: Vec::new(),
            stream_scratch: Vec::new(),
            last_frame_at: None,
            frame_deltas_ms: VecDeque::with_capacity(240),
            adapter_info: String::from("software/headless"),
            wgpu_registered: false,
            last_wgpu_active: 0,
            last_wgpu_fallback: 0,
            bench: None,
        }
    }
}

impl Dynamic10mDemo {
    #[cfg(test)]
    pub fn set_capacity_for_test(&mut self, points: usize) {
        self.target_points = points.max(128);
        self.batch_size = self.batch_size.min(self.target_points / 2).max(16);
        self.wave = None;
        self.stream = None;
        self.wave_cursor = 0;
        self.stream_next_x = 0;
    }

    pub fn set_wgpu_info(&mut self, wgpu_registered: bool, adapter_info: String) {
        self.wgpu_registered = wgpu_registered;
        self.adapter_info = adapter_info;
    }

    pub fn configure_window_bench(&mut self, csv_path: PathBuf) {
        assert!(
            !csv_path.exists(),
            "Refusing to overwrite window benchmark output: {}",
            csv_path.display()
        );
        self.enabled = true;
        self.running = true;
        self.mode = DynamicMode::SweepingWaveform;
        self.backend = RenderBackend::Auto;
        self.batch_size = 32_768;
        self.pacing = Pacing::Uncapped;
        self.bench = Some(WindowBench {
            csv_path,
            phase: 0,
            in_warmup: true,
            phase_start: None,
            prev_frame: None,
            completed: Vec::new(),
            current: BenchPhaseStats::new("wgpu_10m_waveform_live_32k", 32_768),
            done: false,
        });
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        let was_enabled = self.enabled;
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "Dynamic 10M @ 120Hz+ gallery",
                "动态 1,000 万点 @ 120Hz+ 性能展示",
                "動的 1,000 万点 @ 120Hz+ ギャラリー",
            ),
        );
        if self.enabled != was_enabled {
            self.state.reset();
        }
        if !self.enabled {
            return;
        }
        ui.separator();
        ui.horizontal(|ui| {
            let label = if self.running {
                tr(l, "Pause", "暂停", "一時停止")
            } else {
                tr(l, "Run", "继续运行", "再開")
            };
            if ui.button(label).clicked() {
                self.running = !self.running;
            }
            if ui
                .button(tr(l, "Reset view", "重置视图", "ビュー初期化"))
                .clicked()
            {
                self.state.reset();
            }
        });
        let old_mode = self.mode;
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.mode,
                DynamicMode::SweepingWaveform,
                tr(l, "10M Waveform", "1,000万点波形", "1,000万点波形"),
            );
            ui.selectable_value(
                &mut self.mode,
                DynamicMode::StreamingRing,
                tr(l, "10M Stream Ring", "1,000万点环形流", "1,000万点リング"),
            );
        });
        if self.mode != old_mode {
            self.state.reset();
        }
        ui.horizontal(|ui| {
            ui.label(tr(l, "Backend:", "渲染后端:", "バックエンド:"));
            ui.selectable_value(
                &mut self.backend,
                RenderBackend::Auto,
                tr(l, "Auto", "自动", "自動"),
            );
            ui.selectable_value(&mut self.backend, RenderBackend::Wgpu, "Wgpu");
            ui.selectable_value(&mut self.backend, RenderBackend::Painter, "Painter");
        });
        let batch_txt = |n: usize| match l {
            Language::English => format!("{n} pts/frame"),
            Language::ZhCn => format!("{n} 点/帧"),
            Language::Japanese => format!("{n} 点/フレーム"),
        };
        egui::ComboBox::from_id_salt("dyn10m_batch")
            .selected_text(batch_txt(self.batch_size))
            .show_ui(ui, |ui| {
                for b in [8_192, 32_768, 65_536] {
                    ui.selectable_value(
                        &mut self.batch_size,
                        b.min(self.target_points),
                        batch_txt(b),
                    );
                }
            });
        ui.horizontal(|ui| {
            ui.label(tr(l, "Pacing:", "帧率节流:", "ペーシング:"));
            ui.selectable_value(
                &mut self.pacing,
                Pacing::Uncapped,
                tr(l, "Uncapped", "不限速", "無制限"),
            );
            ui.selectable_value(&mut self.pacing, Pacing::Hz240, "240 Hz");
            ui.selectable_value(&mut self.pacing, Pacing::Hz120, "120 Hz");
        });
        self.hud(ui, l);
    }

    fn hud(&self, ui: &mut egui::Ui, l: Language) {
        let (fps, p50, p95, p99) = self.rolling_stats();
        let active_pts = match self.mode {
            DynamicMode::SweepingWaveform => self.wave.as_ref().map_or(0, UniformWaveform::len),
            DynamicMode::StreamingRing => self.stream.as_ref().map_or(0, IndexedStream::len),
        };
        ui.monospace(format!(
            "{}: {active_pts} | {}: {}",
            tr(l, "Points", "总点数", "総点数"),
            tr(l, "Batch", "每帧更新", "更新数"),
            self.batch_size
        ));
        ui.monospace(format!(
            "FPS: {fps:.1} | p50: {p50:.2} ms | p95: {p95:.2} ms | p99: {p99:.2} ms"
        ));
        ui.monospace(format!(
            "{}: {:.2} M/s | wgpu: {} (active={}, fb={})",
            tr(l, "Throughput", "吞吐速率", "更新速度"),
            fps * self.batch_size as f64 / 1_000_000.0,
            self.wgpu_registered,
            self.last_wgpu_active,
            self.last_wgpu_fallback
        ));
        ui.small(format!("GPU: {}", self.adapter_info));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let now = Instant::now();
        let frame_nr = ui.ctx().cumulative_frame_nr();
        let new_frame = self.last_applied_frame != Some(frame_nr);
        if new_frame {
            self.last_applied_frame = Some(frame_nr);
            if let Some(prev) = self.last_frame_at.replace(now) {
                let dt_ms = (now - prev).as_secs_f64() * 1000.0;
                if self.frame_deltas_ms.len() == 240 {
                    self.frame_deltas_ms.pop_front();
                }
                self.frame_deltas_ms.push_back(dt_ms);
            }
        }
        self.ensure_active_data();
        if self.running && new_frame {
            self.step_active_data();
        }
        let resp = match self.mode {
            DynamicMode::SweepingWaveform => {
                let wave = self.wave.as_ref().expect("wave initialized");
                Plot::new("dynamic_10m_wave")
                    .height(ui.available_height().max(320.0))
                    .title(tr(
                        l,
                        "Dynamic 10,000,000-Point Live UniformWaveform",
                        "动态 10,000,000 点实时扫频波形（UniformWaveform）",
                        "動的 10,000,000 点ライブ波形（UniformWaveform）",
                    ))
                    .show_waveform(
                        ui,
                        &mut self.state,
                        &[WaveformLine::new(wave)
                            .id_salt("dyn10m-wf")
                            .name(tr(
                                l,
                                "10M waveform (live sweep)",
                                "1000万点实时波形",
                                "1,000万点ライブ波形",
                            ))
                            .backend(self.backend)
                            .width(1.5)],
                    )
            }
            DynamicMode::StreamingRing => {
                let stream = self.stream.as_ref().expect("stream initialized");
                Plot::new("dynamic_10m_stream")
                    .height(ui.available_height().max(320.0))
                    .title(tr(
                        l,
                        "Dynamic 10,000,000-Point Live IndexedStream Ring",
                        "动态 10,000,000 点实时环形流（IndexedStream）",
                        "動的 10,000,000 点ライブリング（IndexedStream）",
                    ))
                    .show_stream(
                        ui,
                        &mut self.state,
                        &[StreamLine::new(stream)
                            .id_salt("dyn10m-stream")
                            .name(tr(
                                l,
                                "10M indexed ring stream",
                                "1000万点索引环形流",
                                "1,000万点索引リング",
                            ))
                            .width(1.5)],
                    )
            }
        };
        if new_frame {
            self.last_wgpu_active = resp.diagnostics.wgpu_series;
            self.last_wgpu_fallback = resp.diagnostics.wgpu_fallback_series;
            self.advance_bench(ui.ctx(), now, &resp.diagnostics);
        }
        if self.enabled && self.running {
            match self.pacing {
                Pacing::Uncapped => ui.ctx().request_repaint(),
                Pacing::Hz240 => ui.ctx().request_repaint_after(Duration::from_micros(4_166)),
                Pacing::Hz120 => ui.ctx().request_repaint_after(Duration::from_micros(8_333)),
            }
        }
        vec![resp]
    }

    fn ensure_active_data(&mut self) {
        let n = self.target_points;
        match self.mode {
            DynamicMode::SweepingWaveform => {
                if self.wave.is_none() {
                    let initial: Vec<f32> = (0..n)
                        .map(|i| {
                            let x = i as f64 * 0.001;
                            ((x * 0.15).sin() * 0.7 + (x * 1.9).cos() * 0.3) as f32
                        })
                        .collect();
                    self.wave = Some(
                        UniformWaveform::from_f32(0.0, 0.001, initial, 128 * 1024 * 1024)
                            .expect("10M waveform fits within 128 MiB budget"),
                    );
                }
            }
            DynamicMode::StreamingRing => {
                if self.stream.is_none() {
                    let mut stream = IndexedStream::new(n, 512 * 1024 * 1024)
                        .expect("10M indexed stream fits within 512 MiB budget");
                    let init: Vec<PlotPoint> = (0..n)
                        .map(|i| {
                            let x = i as f64 * 0.001;
                            [x, (x * 0.12).sin() * 0.8 + (x * 1.7).cos() * 0.25]
                        })
                        .collect();
                    let _ = stream.append_once(1, &init);
                    self.batch_id = 2;
                    self.stream_next_x = n;
                    self.stream = Some(stream);
                }
            }
        }
    }

    fn step_active_data(&mut self) {
        self.batch_id += 1;
        let count = self.batch_size.min(self.target_points);
        let phase = (self.batch_id as f64) * 0.09;
        match self.mode {
            DynamicMode::SweepingWaveform => {
                let Some(wave) = self.wave.as_mut() else {
                    return;
                };
                let end = (self.wave_cursor + count).min(wave.len());
                let slice_len = end - self.wave_cursor;
                self.wave_scratch.clear();
                for k in 0..slice_len {
                    let idx = self.wave_cursor + k;
                    let x = idx as f64 * 0.001;
                    let amp = 0.65 + 0.35 * (phase * 0.5).sin();
                    let v = ((x * 0.15 + phase).sin() * amp + (x * 2.1 - phase * 1.4).cos() * 0.25)
                        as f32;
                    self.wave_scratch.push(v);
                }
                let _ = wave.write_range_once(self.batch_id, self.wave_cursor, &self.wave_scratch);
                self.wave_cursor = if end >= wave.len() { 0 } else { end };
            }
            DynamicMode::StreamingRing => {
                let Some(stream) = self.stream.as_mut() else {
                    return;
                };
                self.stream_scratch.clear();
                for k in 0..count {
                    let idx = self.stream_next_x + k;
                    let x = idx as f64 * 0.001;
                    let y = (x * 0.12 + (phase % TAU)).sin() * 0.8 + (x * 1.7).cos() * 0.25;
                    self.stream_scratch.push([x, y]);
                }
                self.stream_next_x += count;
                let _ = stream.append_once(self.batch_id, &self.stream_scratch);
            }
        }
    }

    fn rolling_stats(&self) -> (f64, f64, f64, f64) {
        if self.frame_deltas_ms.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut sorted: Vec<f64> = self.frame_deltas_ms.iter().copied().collect();
        let sum: f64 = sorted.iter().sum();
        let fps = (sorted.len() as f64 * 1000.0) / sum.max(0.001);
        (
            fps,
            percentile(&mut sorted, 0.50),
            percentile(&mut sorted, 0.95),
            percentile(&mut sorted, 0.99),
        )
    }

    fn advance_bench(&mut self, ctx: &egui::Context, now: Instant, diag: &PlotDiagnostics) {
        let Some(bench) = self.bench.as_mut() else {
            return;
        };
        if bench.done {
            return;
        }
        let start = *bench.phase_start.get_or_insert(now);
        let prev = bench.prev_frame.replace(now);
        let elapsed = (now - start).as_secs_f64();
        if bench.in_warmup {
            if elapsed >= 1.0 {
                bench.in_warmup = false;
                bench.phase_start = Some(now);
                bench.prev_frame = Some(now);
            }
            return;
        }
        if let Some(prev_t) = prev {
            bench
                .current
                .frame_deltas_ms
                .push((now - prev_t).as_secs_f64() * 1000.0);
        }
        bench
            .current
            .prepare_ms
            .push(diag.prepare_duration.as_secs_f64() * 1000.0);
        bench
            .current
            .query_ms
            .push(diag.query_duration.as_secs_f64() * 1000.0);
        bench.current.wgpu_series = bench.current.wgpu_series.max(diag.wgpu_series);
        bench.current.wgpu_uploaded_bytes = bench
            .current
            .wgpu_uploaded_bytes
            .max(diag.wgpu_uploaded_bytes);
        bench.current.wgpu_retained_bytes = bench
            .current
            .wgpu_retained_bytes
            .max(diag.wgpu_retained_bytes);
        bench.current.elapsed_secs = elapsed;
        if elapsed >= 3.5 && bench.current.frame_deltas_ms.len() >= 10 {
            let finished = std::mem::replace(
                &mut bench.current,
                BenchPhaseStats::new("cpu_10m_stream_ring_live_32k", self.batch_size),
            );
            bench.completed.push(finished);
            if bench.phase == 0 {
                bench.phase = 1;
                bench.in_warmup = true;
                bench.phase_start = None;
                bench.prev_frame = None;
                self.mode = DynamicMode::StreamingRing;
                self.state.reset();
                self.wave = None;
            } else {
                bench.done = true;
                self.write_bench_csv();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn write_bench_csv(&self) {
        let Some(bench) = &self.bench else { return };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&bench.csv_path)
            .expect("create unique window benchmark CSV");
        writeln!(
            file,
            "# Real native window eframe+wgpu 10M swapchain benchmark; adapter={}",
            self.adapter_info
        )
        .unwrap();
        writeln!(file, "scene,points,batch_points,frames,fps,frame_p50_ms,frame_p95_ms,frame_p99_ms,prepare_p95_ms,query_p95_ms,wgpu_series,wgpu_uploaded_bytes,wgpu_retained_bytes").unwrap();
        for p in &bench.completed {
            let mut dt = p.frame_deltas_ms.clone();
            let mut prep = p.prepare_ms.clone();
            let mut qry = p.query_ms.clone();
            let frames = dt.len();
            let fps = frames as f64 / p.elapsed_secs.max(0.001);
            writeln!(
                file,
                "{},{},{},{},{:.2},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{}",
                p.scene,
                self.target_points,
                p.batch_points,
                frames,
                fps,
                percentile(&mut dt, 0.50),
                percentile(&mut dt, 0.95),
                percentile(&mut dt, 0.99),
                percentile(&mut prep, 0.95),
                percentile(&mut qry, 0.95),
                p.wgpu_series,
                p.wgpu_uploaded_bytes,
                p.wgpu_retained_bytes
            )
            .unwrap();
        }
    }
}

impl BenchPhaseStats {
    fn new(scene: &'static str, batch_points: usize) -> Self {
        Self {
            scene,
            batch_points,
            frame_deltas_ms: Vec::new(),
            prepare_ms: Vec::new(),
            query_ms: Vec::new(),
            wgpu_series: 0,
            wgpu_uploaded_bytes: 0,
            wgpu_retained_bytes: 0,
            elapsed_secs: 0.0,
        }
    }
}

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}
