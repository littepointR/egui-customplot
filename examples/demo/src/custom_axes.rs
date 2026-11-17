use std::f64::consts::{PI, TAU};

use eframe::egui::{self, Color32};
use egui_customplot::{
    Axis, AxisLink, AxisScale, Figure, Line, Plot, PlotData, PlotResponse, PlotState, YAxis,
    civil_to_unix_secs,
};

use crate::i18n::{lang, tr};

pub(super) struct CustomAxesDemo {
    pub fmt_mode: usize,
    pub max_ticks: usize,
    pub minor_grid: bool,
    pub show_frame: bool,
    pub log_xy: [bool; 2],
    pub rev_xy: [bool; 2],
    pub lock_xy: [bool; 2],
    pub clamp_pan: bool,
    pub equal_aspect: bool,
    trig_curve: PlotData,
    trig_cos: PlotData,
    time_series: PlotData,
    dur_series: PlotData,
    dual_left: PlotData,
    dual_right: PlotData,
    circle_curve: PlotData,
    link: AxisLink,
    states: [PlotState; 4],
}

impl Default for CustomAxesDemo {
    fn default() -> Self {
        let n = 160usize;
        let trig_curve = PlotData::new(
            (0..=n)
                .map(|i| {
                    let x = TAU * 2.0 * (i as f64 / n as f64);
                    [x, x.sin() * 1200.0 + 300.0 * (2.0 * x).cos()]
                })
                .collect::<Vec<_>>(),
        );
        let trig_cos = PlotData::new(
            (0..=n)
                .map(|i| {
                    let x = TAU * 2.0 * (i as f64 / n as f64);
                    [x, 0.85 * (x - PI * 0.35).cos()]
                })
                .collect::<Vec<_>>(),
        );
        let t0 = civil_to_unix_secs(2026, 11, 17, 8, 0, 0.0).expect("valid utc date");
        let time_series = PlotData::new(
            (0..60)
                .map(|i| {
                    let dt = i as f64 * 120.0;
                    [t0 + dt, 22.0 + 4.5 * (i as f64 * 0.22).sin()]
                })
                .collect::<Vec<_>>(),
        );
        let dur_series = PlotData::new(
            (0..60)
                .map(|i| {
                    let dt = i as f64 * 120.0;
                    [t0 + dt, 180.0 + 95.0 * (i as f64 * 0.14).cos().abs()]
                })
                .collect::<Vec<_>>(),
        );
        let dual_left = PlotData::new(
            (0..120)
                .map(|i| {
                    let x = 0.5 + i as f64 * 0.1;
                    [x, 1.0 + 2.8 * (-0.18 * x).exp() * (x * 1.4).sin().abs()]
                })
                .collect::<Vec<_>>(),
        );
        let dual_right = PlotData::new(
            (0..120)
                .map(|i| {
                    let x = 0.5 + i as f64 * 0.1;
                    [x, 10.0 + 90.0 * (0.14 * x).exp().min(8.0)]
                })
                .collect::<Vec<_>>(),
        );
        let circle_curve = PlotData::new(
            (0..=180)
                .map(|i| {
                    let a = TAU * (i as f64 / 180.0);
                    [a.cos(), a.sin()]
                })
                .collect::<Vec<_>>(),
        );
        Self {
            fmt_mode: 0,
            max_ticks: 8,
            minor_grid: true,
            show_frame: true,
            log_xy: [false, false],
            rev_xy: [false, false],
            lock_xy: [false, false],
            clamp_pan: true,
            equal_aspect: false,
            trig_curve,
            trig_cos,
            time_series,
            dur_series,
            dual_left,
            dual_right,
            circle_curve,
            link: AxisLink::new([true, false]),
            states: Default::default(),
        }
    }
}

impl CustomAxesDemo {
    pub fn reset_views(&mut self) {
        self.states.iter_mut().for_each(PlotState::reset);
        self.link.reset();
    }

    #[cfg(test)]
    pub fn state_bounds(&self, idx: usize) -> Option<egui_customplot::PlotBounds> {
        self.states.get(idx).and_then(PlotState::bounds)
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Axes, Ticks, Time & Pan Limits",
            "数轴、自定义刻度、时间轴与限位",
            "軸・目盛・時刻・範囲制限",
        ));
        let fmt_label = |m: usize| match m {
            0 => tr(l, "π Radians", "π 弧度刻度", "π ラジアン"),
            1 => tr(l, "SI (k / M)", "SI 工程单位", "SI 接頭辞"),
            2 => tr(l, "dB Scale", "dB 对数分贝", "dB 表示"),
            _ => tr(l, "Explicit Ticks", "显式固定刻度", "明示固定目盛"),
        };
        egui::ComboBox::from_id_salt("custom-axis-fmt")
            .selected_text(fmt_label(self.fmt_mode))
            .show_ui(ui, |ui| {
                for mode in 0..4 {
                    ui.selectable_value(&mut self.fmt_mode, mode, fmt_label(mode));
                }
            });
        ui.add(egui::Slider::new(&mut self.max_ticks, 3..=14).text(tr(
            l,
            "Max ticks",
            "最大刻度数",
            "最大目盛数",
        )));
        ui.checkbox(
            &mut self.minor_grid,
            tr(l, "Minor grid lines", "显示次网格线", "補助グリッド"),
        );
        ui.checkbox(
            &mut self.show_frame,
            tr(l, "Plot border frame", "显示坐标框线", "枠線表示"),
        );
        ui.separator();
        let old_log = self.log_xy;
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.log_xy[0], "Log X");
            ui.checkbox(&mut self.log_xy[1], "Log Y");
        });
        if self.log_xy != old_log {
            self.states[2].reset();
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.rev_xy[0], tr(l, "Rev X", "反向 X", "反転 X"));
            ui.checkbox(&mut self.rev_xy[1], tr(l, "Rev Y", "反向 Y", "反転 Y"));
        });
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.lock_xy[0],
                tr(l, "Lock X", "锁定 X 轴", "X ロック"),
            );
            ui.checkbox(
                &mut self.lock_xy[1],
                tr(l, "Lock Y", "锁定 Y 轴", "Y ロック"),
            );
        });
        ui.checkbox(
            &mut self.clamp_pan,
            tr(
                l,
                "Stop-at-boundary pan limits",
                "平滑边界限位 (无回弹抖动)",
                "境界停止パン制限",
            ),
        );
        if ui
            .checkbox(
                &mut self.equal_aspect,
                tr(
                    l,
                    "Cell 4: Equal aspect (else AxisLink)",
                    "子图4：等比例 1:1（否则联动 X）",
                    "セル4：等倍アスペクト（無効時は X 連動）",
                ),
            )
            .changed()
        {
            self.states[0].clear_axis_links();
            self.states[3].clear_axis_links();
            self.link.reset();
        }
        if ui
            .button(tr(l, "Reset axes views", "重置数轴视图", "軸ビュー初期化"))
            .clicked()
        {
            self.reset_views();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 96.0) * 0.5).clamp(160.0, 320.0);

        let mut x0 = Axis::new()
            .label("Phase / Input")
            .max_ticks(self.max_ticks)
            .minor_grid(self.minor_grid);
        let mut y0 = Axis::new()
            .label("Magnitude")
            .max_ticks(self.max_ticks)
            .minor_grid(self.minor_grid);
        match self.fmt_mode {
            0 => {
                x0 = x0.formatter(|v| format!("{:.2}π", v / PI));
            }
            1 => {
                y0 = y0.formatter(|v| {
                    if v.abs() >= 1_000.0 {
                        format!("{:.1}k", v / 1_000.0)
                    } else {
                        format!("{v:.0}")
                    }
                });
            }
            2 => {
                y0 = y0.formatter(|v| {
                    let db = 20.0 * v.abs().max(1.0).log10();
                    format!("{db:.1} dB")
                });
            }
            _ => {
                x0 = x0.explicit_ticks([
                    (0.0, "0".into()),
                    (PI, "π".into()),
                    (TAU, "2π".into()),
                    (3.0 * PI, "3π".into()),
                    (4.0 * PI, "4π".into()),
                ]);
            }
        }

        let linked = !self.equal_aspect;
        let [s0, s1, s2, s3] = &mut self.states;
        let resp = Figure::new("custom-axes-fig")
            .columns(2)
            .cell_height(height)
            .show(ui, 4, |cell| match cell.index() {
                0 => cell.show_layers(
                    Plot::new("cax-fmt")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .show_frame(self.show_frame)
                        .x_axis(x0.clone())
                        .y_axis(y0.clone())
                        .legend(true)
                        .title(tr(
                            l,
                            "Custom Tick Formatter & Max Ticks",
                            "自定义刻度格式化与次网格",
                            "カスタム目盛フォーマッタ",
                        )),
                    s0,
                    &[Line::new(&self.trig_curve)
                        .name("Harmonic")
                        .color(Color32::from_rgb(0, 114, 189))
                        .into()],
                ),
                1 => cell.show_layers(
                    Plot::new("cax-time")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .show_frame(self.show_frame)
                        .x_axis(Axis::new().datetime().label("UTC Time"))
                        .y_axis(Axis::new().label("Temp (°C)"))
                        .right_y_axis(Axis::new().duration().label("Latency (Dur)"))
                        .legend(true)
                        .title(tr(
                            l,
                            "UTC DateTime & Elapsed Duration Axes",
                            "UTC 日期时间轴 + 耗时 Duration 右轴",
                            "UTC 日時＆経過時間軸",
                        )),
                    s1,
                    &[
                        Line::new(&self.time_series)
                            .name("Temp (°C)")
                            .color(Color32::from_rgb(217, 83, 25))
                            .into(),
                        Line::new(&self.dur_series)
                            .name("Duration")
                            .y_axis(YAxis::Right)
                            .color(Color32::from_rgb(46, 160, 67))
                            .into(),
                    ],
                ),
                2 => {
                    let scale = |log| {
                        if log {
                            AxisScale::Log10
                        } else {
                            AxisScale::Linear
                        }
                    };
                    let mut p = Plot::new("cax-clamp")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .show_frame(self.show_frame)
                        .drag_axes([!self.lock_xy[0], !self.lock_xy[1]])
                        .zoom_axes([!self.lock_xy[0], !self.lock_xy[1]])
                        .x_axis(
                            Axis::new()
                                .scale(scale(self.log_xy[0]))
                                .reversed(self.rev_xy[0])
                                .minor_grid(self.minor_grid)
                                .label("x"),
                        )
                        .y_axis(
                            Axis::new()
                                .scale(scale(self.log_xy[1]))
                                .reversed(self.rev_xy[1])
                                .minor_grid(self.minor_grid)
                                .label("Left Gain"),
                        )
                        .right_y_axis(Axis::new().scale(AxisScale::Log10).label("Right Growth"))
                        .legend(true)
                        .title(tr(
                            l,
                            "Dual-Y + Axis Locks + Pan Limits",
                            "双 Y 对数轴 + 轴锁 + 无抖动限位",
                            "2軸対数＋ロック＋境界停止",
                        ));
                    if self.clamp_pan && !self.log_xy[0] && !self.log_xy[1] {
                        p = p.pan_limits(Some([-1.0, 15.0]), Some([-2.5, 4.5]));
                    }
                    cell.show_layers(
                        p,
                        s2,
                        &[
                            Line::new(&self.dual_left).name("Damped").into(),
                            Line::new(&self.dual_right)
                                .name("Growth")
                                .y_axis(YAxis::Right)
                                .into(),
                        ],
                    )
                }
                _ => {
                    let mut p = Plot::new("cax-link")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .show_frame(self.show_frame)
                        .legend(true);
                    if self.equal_aspect {
                        p = p.data_aspect(1.0).title(tr(
                            l,
                            "Locked 1:1 Data Aspect Circle",
                            "1:1 等比例数据圆",
                            "1:1 等倍アスペクト円",
                        ));
                        cell.show_layers(
                            p,
                            s3,
                            &[Line::new(&self.circle_curve).name("r = 1").into()],
                        )
                    } else {
                        p = p.title(tr(
                            l,
                            "X-Linked to Panel 1 (Pan / Zoom Synced)",
                            "与子图 1 双向联动 X 轴（拖拽不回弹）",
                            "パネル1と X 軸双方向連動",
                        ));
                        cell.show_layers(
                            p,
                            s3,
                            &[Line::new(&self.trig_cos)
                                .name("Shifted Cos")
                                .color(Color32::from_rgb(126, 47, 142))
                                .into()],
                        )
                    }
                }
            })
            .expect("custom axes figure")
            .inner;

        if linked {
            let [s0, _, _, s3] = &mut self.states;
            let mut refs = [s0, s3];
            let _ = self.link.synchronize(ui.ctx(), &mut refs);
        }
        resp
    }
}
