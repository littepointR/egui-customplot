use std::{f64::consts::TAU, time::Duration};

use eframe::egui::{self, Color32};
use egui_customplot::{
    AnimatedLine, AnimatedLine2D, Bounds3D, ColorMap, ColorScale, Comet, Comet3D, Comet3DPlayback,
    CometPlayback, Figure, IndexedStream, Layer3D, Plot, Plot3D, Plot3DState, PlotBounds, PlotData,
    PlotPoint, PlotResponse, PlotState, Point3D, Point3DData, Stream3DOptions, StreamLine,
    StreamParticles3D, StreamParticles3DData, StreamParticlesPlayback, Streamline3DData,
    VectorField3D,
};

use crate::i18n::{lang, tr};

pub(super) struct CustomAnimationDemo {
    pub playing: bool,
    pub speed_mult: f64,
    pub tail_frac: f64,
    pub auto_follow: bool,
    clock: crate::stream::FrameClock,
    last_step_frame: Option<u64>,
    lissajous: PlotData,
    comet2d: CometPlayback,
    anim_line: AnimatedLine2D,
    anim_rev: u64,
    pen_phase: f64,
    pen_accum: f64,
    pen_batch_id: u64,
    osc_stream: IndexedStream,
    osc_x: usize,
    osc_accum: f64,
    osc_batch_id: u64,
    helix3d: Point3DData,
    comet3d: Comet3DPlayback,
    particles: StreamParticles3DData,
    part_pb: StreamParticlesPlayback,
    states: [PlotState; 3],
    state3d: Plot3DState,
}

impl Default for CustomAnimationDemo {
    fn default() -> Self {
        let liss_pts: Vec<PlotPoint> = (0..220)
            .map(|i| {
                let t = TAU * (i as f64 / 219.0);
                [(3.0 * t).sin(), (2.0 * t).cos()]
            })
            .collect();
        let lissajous = PlotData::new(liss_pts);
        let comet2d = CometPlayback::new(45.0).tail_fraction(0.22).looping(true);

        let mut anim_line = AnimatedLine2D::new(300, 300 * 16).expect("anim");
        let init_pen: Vec<PlotPoint> = (0..40)
            .map(|i| {
                let x = i as f64 * 0.08;
                [x, (2.2 * x).sin() * (-0.06 * x).exp()]
            })
            .collect();
        let _ = anim_line.add_points_once(1, &init_pen);
        let anim_rev = anim_line.revision();

        let osc_stream = new_osc_stream();

        let helix_pts: Vec<Point3D> = (0..=180)
            .map(|i| {
                let t = i as f64 / 180.0 * 3.0 * TAU;
                [0.9 * t.cos(), 0.9 * t.sin(), 2.0 * (i as f64 / 180.0) - 1.0]
            })
            .collect();
        let helix3d = Point3DData::new(helix_pts);
        let comet3d = Comet3DPlayback::new(45.0).tail_fraction(0.25).looping(true);

        let vbounds = Bounds3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]).expect("bounds");
        let field = VectorField3D::from_function(vbounds, [8, 8, 8], 2_000, |x, y, z| {
            [-y - 0.15 * x, x - 0.15 * y, 0.25 + 0.1 * z]
        })
        .expect("vf3d");
        let streamlines = Streamline3DData::from_field(
            &field,
            &[[0.55, 0.0, -0.75], [-0.55, 0.25, -0.6], [0.1, -0.6, -0.5]],
            Stream3DOptions {
                step_size: 0.06,
                max_steps: 64,
                ..Default::default()
            },
        )
        .expect("streamlines");
        let particles = StreamParticles3DData::from_streamlines(&streamlines, 8);

        Self {
            playing: true,
            speed_mult: 1.0,
            tail_frac: 0.22,
            auto_follow: true,
            clock: crate::stream::FrameClock::default(),
            last_step_frame: None,
            lissajous,
            comet2d,
            anim_line,
            anim_rev,
            pen_phase: 40.0 * 0.08,
            pen_accum: 0.0,
            pen_batch_id: 2,
            osc_stream,
            osc_x: 120,
            osc_accum: 0.0,
            osc_batch_id: 2,
            helix3d,
            comet3d,
            particles,
            part_pb: StreamParticlesPlayback::new(0.45),
            states: Default::default(),
            state3d: Plot3DState::default(),
        }
    }
}

fn new_osc_stream() -> IndexedStream {
    let mut osc_stream = IndexedStream::new(512, 512 * 1024).expect("osc stream");
    let osc_init: Vec<PlotPoint> = (0..120)
        .map(|i| {
            let x = i as f64 * 0.06;
            [x, (x * 1.9).sin() + 0.35 * (x * 4.3).cos()]
        })
        .collect();
    let _ = osc_stream.append_once(1, &osc_init);
    osc_stream
}

impl CustomAnimationDemo {
    pub fn reset_views(&mut self) {
        self.clock.reset();
        self.comet2d.reset();
        self.comet3d.reset();
        self.part_pb.set_phase(0.0);
        let _ = self.anim_line.clear();
        self.pen_phase = 0.0;
        self.osc_stream = new_osc_stream();
        self.osc_x = 120;
        self.osc_batch_id = 2;
        self.states.iter_mut().for_each(PlotState::reset);
        self.state3d.reset();
        self.auto_follow = true;
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Animation Transport & Playback",
            "动画时间轴与播放控制器",
            "アニメーション制御",
        ));
        ui.horizontal(|ui| {
            let play_lbl = if self.playing {
                tr(l, "Pause", "暂停", "一時停止")
            } else {
                tr(l, "Play", "播放", "再生")
            };
            if ui.button(play_lbl).clicked() {
                self.playing = !self.playing;
                self.anim_line.set_paused(!self.playing);
                self.comet2d.set_playing(self.playing);
                self.comet3d.set_playing(self.playing);
                self.part_pb.set_playing(self.playing);
                if !self.playing {
                    self.clock.reset();
                }
            }
            if ui.button(tr(l, "Reset", "重置", "初期化")).clicked() {
                self.reset_views();
            }
        });
        ui.add(egui::Slider::new(&mut self.speed_mult, 0.25..=4.0).text(tr(
            l,
            "Speed (x)",
            "倍速 (x)",
            "再生速度",
        )));
        let mut progress =
            self.comet2d.progress_points() / (self.lissajous.len().max(2) - 1) as f64;
        if ui
            .add(egui::Slider::new(&mut progress, 0.0..=1.0).text(tr(
                l,
                "Timeline scrub",
                "时间轴进度拖拽",
                "タイムライン",
            )))
            .changed()
        {
            self.comet2d.set_progress_points(
                progress * (self.lissajous.len() - 1) as f64,
                self.lissajous.len(),
            );
            self.comet3d.set_progress_points(
                progress * (self.helix3d.len() - 1) as f64,
                self.helix3d.len(),
            );
            self.part_pb.set_phase(progress);
        }
        if ui
            .add(egui::Slider::new(&mut self.tail_frac, 0.05..=0.60).text(tr(
                l,
                "Comet tail fraction",
                "彗星尾迹长度",
                "テール長比率",
            )))
            .changed()
        {
            let p2 = self.comet2d.progress_points();
            let mut next2 = CometPlayback::new(45.0)
                .tail_fraction(self.tail_frac)
                .looping(true);
            next2.set_progress_points(p2, self.lissajous.len());
            next2.set_playing(self.playing);
            self.comet2d = next2;

            let p3 = self.comet3d.progress_points();
            let mut next3 = Comet3DPlayback::new(45.0)
                .tail_fraction(self.tail_frac)
                .looping(true);
            next3.set_progress_points(p3, self.helix3d.len());
            next3.set_playing(self.playing);
            self.comet3d = next3;
        }
        ui.checkbox(
            &mut self.auto_follow,
            tr(
                l,
                "Oscilloscope auto-follow window",
                "示波器窗口自动跟随",
                "オシロ自動追従",
            ),
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let frame_id = ui.ctx().cumulative_frame_nr();
        let dt = self.clock.tick(ui.ctx(), self.playing) * self.speed_mult;
        if self.playing && dt > 0.0 && self.last_step_frame != Some(frame_id) {
            self.last_step_frame = Some(frame_id);
            let _ = self
                .comet2d
                .advance_once(frame_id, dt, self.lissajous.len());
            let _ = self.comet3d.advance_once(frame_id, dt, self.helix3d.len());
            let _ = self.part_pb.advance_once(frame_id, dt);

            self.pen_accum += dt;
            while self.pen_accum >= 1.0 / 30.0 {
                self.pen_accum -= 1.0 / 30.0;
                self.pen_phase += 0.08;
                if self.pen_phase > 24.0 {
                    let _ = self.anim_line.clear();
                    self.pen_phase = 0.0;
                }
                let pt = [
                    self.pen_phase,
                    (2.2 * self.pen_phase).sin() * (-0.06 * self.pen_phase).exp(),
                ];
                self.pen_batch_id += 1;
                let _ = self.anim_line.add_points_once(self.pen_batch_id, &[pt]);
            }

            self.osc_accum += dt;
            let mut osc_pts = Vec::new();
            while self.osc_accum >= 1.0 / 30.0 {
                self.osc_accum -= 1.0 / 30.0;
                let x = self.osc_x as f64 * 0.06;
                osc_pts.push([x, (x * 1.9).sin() + 0.35 * (x * 4.3).cos()]);
                self.osc_x += 1;
            }
            if !osc_pts.is_empty() {
                self.osc_batch_id += 1;
                let _ = self.osc_stream.append_once(self.osc_batch_id, &osc_pts);
            }
        }
        if self.playing {
            let _ = self.anim_line.sync_repaint(ui.ctx(), &mut self.anim_rev);
            let _ = self.comet2d.sync_repaint(
                ui.ctx(),
                self.lissajous.len(),
                Duration::from_millis(16),
            );
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }

        if self.auto_follow {
            let max_x = (self.osc_x as f64 * 0.06).max(8.0);
            if let Ok(b) = PlotBounds::new([(max_x - 8.0).max(0.0), -1.6], [max_x, 1.6]) {
                self.states[2].set_bounds(b);
            }
        }

        let height = ((ui.available_height() - 96.0) * 0.5).clamp(160.0, 320.0);
        let [s0, s1, s2] = &mut self.states;
        Figure::new("custom-anim-fig")
            .columns(2)
            .cell_height(height)
            .show(ui, 4, |cell| match cell.index() {
                0 => cell.show_layers(
                    Plot::new("canim-comet")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .data_aspect(1.0)
                        .title(tr(
                            l,
                            "2D Comet Lissajous (Wall-Clock Synced)",
                            "2D 彗星追踪动画（墙钟速率同步）",
                            "2D Comet（実時間同期）",
                        )),
                    s0,
                    &[Comet::new(&self.lissajous, &self.comet2d)
                        .name("Comet")
                        .into()],
                ),
                1 => cell.show_layers(
                    Plot::new("canim-pen")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .title(tr(
                            l,
                            "AnimatedLine2D Pen Recorder",
                            "AnimatedLine2D 实时记录笔",
                            "AnimatedLine2D レコーダ",
                        )),
                    s1,
                    &[AnimatedLine::new(&self.anim_line).name("Pen").into()],
                ),
                2 => {
                    let osc_plot = cell.plot(
                        Plot::new("canim-osc")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title(tr(
                                l,
                                "IndexedStream Oscilloscope (Drag Stops Follow)",
                                "实时滚动示波器（手动拖拽自动暂停跟随，不回弹）",
                                "リアルタイムオシロ（手動移動で追従停止）",
                            )),
                    );
                    let resp = osc_plot.show_stream(
                        cell.ui(),
                        s2,
                        &[StreamLine::new(&self.osc_stream)
                            .name("Ch1")
                            .color(Color32::from_rgb(0, 175, 130))],
                    );
                    if resp.response.dragged() {
                        self.auto_follow = false;
                    }
                    resp
                }
                _ => {
                    let cscale = ColorScale::new([-1.0, 1.0])
                        .expect("cs")
                        .color_map(ColorMap::Viridis);
                    let _ = cscale;
                    cell.show_plot3d(
                        Plot3D::new("canim-3d")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title(tr(
                                l,
                                "3D Comet3 + StreamParticles3D",
                                "3D 彗星与三维流场粒子动画",
                                "3D Comet3＋流線粒子",
                            )),
                        &mut self.state3d,
                        &[
                            Layer3D::from(
                                Comet3D::new(&self.helix3d, &self.comet3d)
                                    .name("comet3")
                                    .tail_color(Color32::from_rgb(237, 177, 32)),
                            ),
                            Layer3D::from(
                                StreamParticles3D::new(&self.particles, &self.part_pb)
                                    .name("particles"),
                            ),
                        ],
                    )
                    .into_plot_response()
                }
            })
            .expect("custom animation figure")
            .inner
    }
}
