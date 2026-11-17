//! Host-owned adaptive functions (`fplot`/`fimplicit`/`ezplot`), animation (`animatedline`/`comet`), and EX-027 annotations.
use std::{
    f64::consts::{PI, TAU},
    time::Duration,
};

use eframe::egui::{self, Color32, Stroke};
use egui_customplot::{
    AnimatedLine, AnimatedLine2D, Annotation, AnnotationCoord, Comet, CometPlayback, Figure,
    FunctionCurveData, FunctionPlot, FunctionPlotOptions, ImplicitCurve, ImplicitCurveData,
    ImplicitOptions, LinePattern, Plot, PlotBounds, PlotResponse, PlotState,
};

use crate::i18n::{lang, tr};

pub(super) struct FunctionsDemo {
    pub show_poles: bool,
    pub animate_live: bool,
    chirp: FunctionCurveData,
    tangent: FunctionCurveData,
    lissajous: FunctionCurveData,
    ez_circle: FunctionCurveData,
    cassini: ImplicitCurveData,
    anim: AnimatedLine2D,
    anim_rev: u64,
    phase: f64,
    comet_data: egui_customplot::PlotData,
    comet: CometPlayback,
    clock: crate::stream::FrameClock,
    step_accum: f64,
    last_step_frame: Option<u64>,
    next_batch_id: u64,
    states: [PlotState; 4],
}

impl Default for FunctionsDemo {
    fn default() -> Self {
        let chirp = FunctionCurveData::explicit([0.08, 3.2], FunctionPlotOptions::default(), |x| {
            (1.0 / (0.18 * x)).sin() * 0.85
        })
        .expect("finite chirp");
        let tangent = FunctionCurveData::explicit(
            [-1.35 * PI, 1.35 * PI],
            FunctionPlotOptions::default(),
            |x| 0.28 * x.tan(),
        )
        .expect("valid tan domain");
        let lissajous =
            FunctionCurveData::parametric([0.0, TAU], FunctionPlotOptions::default(), |t| {
                [(3.0 * t).sin() * 1.4, (2.0 * t).cos() * 1.1]
            })
            .expect("valid parametric curve");
        let ez_circle =
            FunctionCurveData::ezplot_parametric(2048, |t| [1.6 * t.cos(), 1.6 * t.sin()])
                .expect("valid ezplot");
        let cassini = ImplicitCurveData::new(
            PlotBounds::new([-2.2, -1.6], [2.2, 1.6]).expect("valid bounds"),
            ImplicitOptions::default(),
            |x, y| {
                let r2 = x * x + y * y;
                r2 * r2 - 2.0 * (x * x - y * y) - (0.95_f64.powi(4) - 1.0)
            },
        )
        .expect("valid implicit");

        let mut anim = AnimatedLine2D::new(240, 240 * 16).expect("valid capacity");
        let initial_pts: Vec<[f64; 2]> = (0..64)
            .map(|i| {
                let t = i as f64 * 0.08;
                [t, (t * 2.0).sin() * (-0.08 * t).exp()]
            })
            .collect();
        let _ = anim.add_points_once(1, &initial_pts);
        let anim_rev = anim.revision();

        let comet_pts: Vec<[f64; 2]> = (0..180)
            .map(|i| {
                let t = TAU * (i as f64 / 179.0);
                [t, (2.0 * t).sin() * (t * 0.5).cos()]
            })
            .collect();
        let comet_data = egui_customplot::PlotData::new(comet_pts);
        let mut comet = CometPlayback::new(45.0).tail_fraction(0.22).looping(true);
        comet.set_progress_points(60.0, comet_data.len());

        let mut states: [PlotState; 4] = Default::default();
        states[0].set_bounds(PlotBounds::new([-3.2, -2.5], [3.2, 2.5]).expect("valid bounds"));

        Self {
            show_poles: true,
            animate_live: true,
            chirp,
            tangent,
            lissajous,
            ez_circle,
            cassini,
            anim,
            anim_rev,
            phase: 64.0 * 0.08,
            comet_data,
            comet,
            clock: crate::stream::FrameClock::default(),
            step_accum: 0.0,
            last_step_frame: None,
            next_batch_id: 10,
            states,
        }
    }
}

impl FunctionsDemo {
    pub fn reset_views(&mut self) {
        self.comet.reset();
        self.comet.set_playing(self.animate_live);
        self.clock.reset();
        self.step_accum = 0.0;
        self.states.iter_mut().for_each(PlotState::reset);
        self.states[0].set_bounds(PlotBounds::new([-3.2, -2.5], [3.2, 2.5]).expect("valid bounds"));
    }

    #[cfg(test)]
    pub fn comet_progress(&self) -> f64 {
        self.comet.progress_points()
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Functions, animation & EX-027",
            "自适应函数、动态动画与 EX-027 注解",
            "関数・アニメーション・EX-027 注釈",
        ));
        ui.checkbox(
            &mut self.show_poles,
            tr(
                l,
                "Show split tan(x) poles",
                "显示 tan(x) 极点断枝",
                "tan(x) 特異点分割",
            ),
        );
        if ui
            .checkbox(
                &mut self.animate_live,
                tr(
                    l,
                    "Live animation / comet",
                    "实时运行 / 彗星追踪动画",
                    "ライブアニメーション",
                ),
            )
            .changed()
        {
            self.anim.set_paused(!self.animate_live);
            self.comet.set_playing(self.animate_live);
            if !self.animate_live {
                self.clock.reset();
            }
        }
        if ui
            .button(tr(
                l,
                "Reset animation & views",
                "重置动画与视图",
                "アニメ・表示を初期化",
            ))
            .clicked()
        {
            self.comet.reset();
            self.comet.set_playing(self.animate_live);
            self.clock.reset();
            self.step_accum = 0.0;
            self.states.iter_mut().for_each(PlotState::reset);
            self.states[0]
                .set_bounds(PlotBounds::new([-3.2, -2.5], [3.2, 2.5]).expect("valid bounds"));
        }
        ui.small(tr(
            l,
            "Adaptive bisection / quadtree sampling on host; multi-pass safe comet & animatedline.",
            "自适应二分 / 四叉树采样；多遍安全的动态彗星与动画线。",
            "適応二分・四分木サンプリング；マルチパス安全な動的ライン。",
        ));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let frame_id = ui.ctx().cumulative_frame_nr();
        let dt = self.clock.tick(ui.ctx(), self.animate_live);
        if self.animate_live {
            if dt > 0.0 && self.last_step_frame != Some(frame_id) {
                self.last_step_frame = Some(frame_id);
                self.step_accum += dt;
                while self.step_accum >= 1.0 / 30.0 {
                    self.step_accum -= 1.0 / 30.0;
                    let step = (self.phase + 0.08).min(24.0);
                    let next_pt = [step, (step * 2.0).sin() * (-0.08 * step).exp()];
                    self.next_batch_id += 1;
                    if let Ok(rep) = self.anim.add_points_once(self.next_batch_id, &[next_pt])
                        && rep.applied
                    {
                        self.phase = if step >= 24.0 {
                            let _ = self.anim.clear();
                            0.0
                        } else {
                            step
                        };
                    }
                }
                let _ = self.comet.advance_once(frame_id, dt, self.comet_data.len());
            }
            let _ = self.anim.sync_repaint(ui.ctx(), &mut self.anim_rev);
            let _ =
                self.comet
                    .sync_repaint(ui.ctx(), self.comet_data.len(), Duration::from_millis(16));
        }

        let height = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let expl_annots = [
            Annotation::arrow(AnnotationCoord::Data, [0.4, 1.3], [0.22, 0.86])
                .with_stroke(Stroke::new(1.5, Color32::from_rgb(200, 60, 40))),
            Annotation::text_arrow(
                AnnotationCoord::Normalized,
                [0.62, 0.85],
                [0.75, 0.68],
                "chirp",
            )
            .with_stroke(Stroke::new(1.2, Color32::from_rgb(40, 90, 180))),
            Annotation::rectangle(AnnotationCoord::Data, [[0.08, -1.0], [0.8, 1.0]])
                .with_fill(Color32::from_rgba_unmultiplied(60, 140, 220, 28))
                .with_stroke(Stroke::new(1.0, Color32::from_rgb(60, 140, 220))),
        ];
        let flow_annots = [
            Annotation::textbox(
                AnnotationCoord::Normalized,
                [[0.04, 0.78], [0.42, 0.94]],
                "comet + animatedline",
            )
            .with_fill(Color32::from_rgba_unmultiplied(245, 245, 220, 180))
            .with_stroke(Stroke::new(1.0, Color32::GRAY)),
            Annotation::ellipse(AnnotationCoord::Normalized, [[0.68, 0.14], [0.92, 0.40]])
                .with_stroke(Stroke::new(1.2, Color32::from_rgb(180, 90, 40))),
        ];
        let fig_annots = [
            Annotation::line(AnnotationCoord::Normalized, [0.02, 0.50], [0.98, 0.50]).with_stroke(
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 120, 120, 80)),
            ),
            Annotation::double_arrow(AnnotationCoord::Normalized, [0.35, 0.985], [0.65, 0.985])
                .with_stroke(Stroke::new(1.2, Color32::from_rgb(70, 120, 180))),
        ];

        Figure::new("functions-panels")
            .columns(2)
            .cell_height(height)
            .annotations(fig_annots)
            .show(ui, 4, |cell| match cell.index() {
                0 => {
                    let mut layers =
                        vec![FunctionPlot::new(&self.chirp).name("sin(1/0.18x)").into()];
                    if self.show_poles {
                        layers.push(
                            FunctionPlot::new(&self.tangent)
                                .name("0.28 tan(x)")
                                .pattern(LinePattern::Dashed {
                                    length: 6.0,
                                    gap: 3.0,
                                })
                                .into(),
                        );
                    }
                    cell.show_layers(
                        Plot::new("fplot-explicit")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .legend(true)
                            .annotations(expl_annots.as_slice())
                            .title(tr(
                                l,
                                "fplot Explicit + Pole Split + EX-027",
                                "fplot 显式自适应 + 极点分裂 + EX-027",
                                "fplot 陽関数＋極分割＋EX-027",
                            )),
                        &mut self.states[0],
                        &layers,
                    )
                }
                1 => cell.show_layers(
                    Plot::new("fplot-parametric")
                        .height(height)
                        .data_aspect(1.0)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "fplot Parametric Lissajous & ezplot Circle",
                            "fplot 参数利萨茹 + ezplot 圆",
                            "fplot 媒介変数リサジュー＆円",
                        )),
                    &mut self.states[1],
                    &[
                        FunctionPlot::new(&self.lissajous)
                            .id_salt("liss")
                            .name("Lissajous")
                            .into(),
                        FunctionPlot::new(&self.ez_circle)
                            .id_salt("ezcirc")
                            .name(tr(l, "ezplot circle", "ezplot 圆", "ezplot 円"))
                            .into(),
                    ],
                ),
                2 => cell.show_layers(
                    Plot::new("fimplicit-cassini")
                        .height(height)
                        .data_aspect(1.0)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .title(tr(
                            l,
                            "fimplicit Cassini Oval Zero-Level Branches",
                            "fimplicit 卡西尼卵形隐式零等值支",
                            "fimplicit カッシーニ卵形陰関数",
                        )),
                    &mut self.states[2],
                    &[ImplicitCurve::new(&self.cassini).name("Cassini F=0").into()],
                ),
                _ => cell.show_layers(
                    Plot::new("anim-comet")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .annotations(flow_annots.as_slice())
                        .title(tr(
                            l,
                            "animatedline + comet + EX-027 Annotations",
                            "动画折线 + 彗星轨迹 + EX-027 注解",
                            "animatedline＋comet＋EX-027 注釈",
                        )),
                    &mut self.states[3],
                    &[
                        AnimatedLine::new(&self.anim).name("animatedline").into(),
                        Comet::new(&self.comet_data, &self.comet)
                            .name("comet")
                            .into(),
                    ],
                ),
            })
            .expect("4 cells within limit")
            .inner
    }
}
