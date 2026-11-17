use std::f64::consts::TAU;

use eframe::egui::{self, Color32, Stroke};
use egui_customplot::{
    Annotation, AnnotationCoord, BubbleChart, BubbleData, BubbleLegend, Figure, Line, LinePattern,
    MarkerShape, Plot, PlotData, PlotPoint, PlotResponse, PlotState, Scatter,
};

use crate::i18n::{lang, tr};

pub(super) struct CustomAnnotationsDemo {
    pub upper_limit: f64,
    pub lower_limit: f64,
    pub win_end: f64,
    pub dash_len: f32,
    pub gap_len: f32,
    pub pinned: Vec<PlotPoint>,
    last_pin_frame: Option<u64>,
    signal: PlotData,
    signal_peak: PlotPoint,
    signal_valley: PlotPoint,
    marker_rows: Vec<(MarkerShape, PlotData)>,
    dash_line: PlotData,
    bubble_data: BubbleData,
    brush_cloud: PlotData,
    states: [PlotState; 4],
}

impl Default for CustomAnnotationsDemo {
    fn default() -> Self {
        let mut pts = Vec::with_capacity(140);
        let mut peak = [0.0, f64::NEG_INFINITY];
        let mut valley = [0.0, f64::INFINITY];
        for i in 0..140 {
            let x = i as f64 * 0.08;
            let y = 1.65 * (-0.12 * x).exp() * (1.7 * x).sin();
            pts.push([x, y]);
            if y > peak[1] {
                peak = [x, y];
            }
            if y < valley[1] {
                valley = [x, y];
            }
        }
        let signal = PlotData::new(pts);

        let marker_rows = MarkerShape::ALL
            .iter()
            .enumerate()
            .map(|(row, &shape)| {
                let y = MarkerShape::ALL.len() as f64 - row as f64;
                let row_pts = (0..6).map(|k| [k as f64 + 1.0, y]).collect::<Vec<_>>();
                (shape, PlotData::new(row_pts))
            })
            .collect();
        let dash_line = PlotData::new(
            (0..60)
                .map(|i| {
                    let x = 0.5 + i as f64 * 0.1;
                    [x, 0.8 * (x * 1.4).sin()]
                })
                .collect::<Vec<_>>(),
        );
        let b_pts: Vec<PlotPoint> = (0..12)
            .map(|i| [1.0 + i as f64 * 0.45, -2.5 + 1.4 * (i as f64 * 0.6).cos()])
            .collect();
        let b_sizes: Vec<f64> = (0..12).map(|i| 12.0 + (i % 5) as f64 * 18.0).collect();
        let bubble_data = BubbleData::new(&b_pts, &b_sizes)
            .expect("bubble")
            .radius_range(4.0, 14.0)
            .expect("radius range");

        let brush_cloud = PlotData::new(
            (0..240)
                .map(|i| {
                    let t = TAU * (i as f64 / 60.0);
                    let r = 0.35 + 1.4 * (i as f64 / 240.0);
                    [
                        r * t.cos() + 0.12 * ((i * 7) as f64).sin(),
                        r * t.sin() + 0.12 * ((i * 11) as f64).cos(),
                    ]
                })
                .collect::<Vec<_>>(),
        );

        Self {
            upper_limit: 1.0,
            lower_limit: -0.8,
            win_end: 5.5,
            dash_len: 9.0,
            gap_len: 4.5,
            pinned: vec![peak],
            last_pin_frame: None,
            signal,
            signal_peak: peak,
            signal_valley: valley,
            marker_rows,
            dash_line,
            bubble_data,
            brush_cloud,
            states: Default::default(),
        }
    }
}

impl CustomAnnotationsDemo {
    pub fn reset_views(&mut self) {
        self.pinned.clear();
        self.pinned.push(self.signal_peak);
        self.states.iter_mut().for_each(PlotState::reset);
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "DataTips, Annotations, Markers & Brush",
            "数据游标 (DataTip)、注解、标记与刷选",
            "データヒント・注釈・マーカー・選択",
        ));
        ui.add(egui::Slider::new(&mut self.upper_limit, 0.2..=1.8).text(tr(
            l,
            "Upper threshold",
            "上告警阈值",
            "上限しきい値",
        )));
        ui.add(
            egui::Slider::new(&mut self.lower_limit, -1.6..=-0.1).text(tr(
                l,
                "Lower threshold",
                "下告警阈值",
                "下限しきい値",
            )),
        );
        ui.add(egui::Slider::new(&mut self.win_end, 2.5..=9.5).text(tr(
            l,
            "ROI window end",
            "ROI 区域右界",
            "ROI 領域終端",
        )));
        ui.add(egui::Slider::new(&mut self.dash_len, 2.0..=20.0).text(tr(
            l,
            "Dash length",
            "虚线实段长",
            "破線長",
        )));
        ui.add(egui::Slider::new(&mut self.gap_len, 1.0..=16.0).text(tr(
            l,
            "Dash gap",
            "虚线间隔",
            "破線間隔",
        )));
        if ui
            .button(tr(
                l,
                "Clear pinned DataTips",
                "清空固定 DataTip",
                "ピン留め消去",
            ))
            .clicked()
        {
            self.pinned.clear();
        }
        if ui
            .button(tr(l, "Reset annotation views", "重置视图", "ビュー初期化"))
            .clicked()
        {
            self.reset_views();
        }
        ui.small(tr(
            l,
            "Cell 1: click curve point to pin DataTip (up to 4, shows delta between last 2).\nCell 4: Alt+Drag to brush points and view live stats.",
            "子图1：点击曲线任意点固定 DataTip（最多4个，自动算两点斜率）。\n子图4：Alt+拖拽刷选数据并统计 N / 均值 / 标准差。",
            "セル1：曲線クリックでデータヒント固定。\nセル4：Alt+ドラッグで統計表示。",
        ));
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 126.0) * 0.5).clamp(155.0, 310.0);
        let fig_ann = [Annotation::text(
            [0.02, 0.965],
            tr(
                l,
                "Figure Header Annotation (Normalized [0, 1]^2)",
                "图窗级跨面板归一化注解 (Figure::annotations)",
                "図全体正規化注釈",
            ),
        )];

        let mut tip_ann = vec![
            Annotation::text_arrow(
                AnnotationCoord::Data,
                [self.signal_peak[0] + 0.9, self.signal_peak[1] + 0.35],
                self.signal_peak,
                format!(
                    "Peak ({:.2}, {:.2})",
                    self.signal_peak[0], self.signal_peak[1]
                ),
            )
            .with_stroke(Stroke::new(1.3, Color32::from_rgb(200, 50, 40))),
            Annotation::text_arrow(
                AnnotationCoord::Data,
                [self.signal_valley[0] + 0.9, self.signal_valley[1] - 0.35],
                self.signal_valley,
                format!(
                    "Min ({:.2}, {:.2})",
                    self.signal_valley[0], self.signal_valley[1]
                ),
            )
            .with_stroke(Stroke::new(1.3, Color32::from_rgb(35, 115, 200))),
        ];
        for (i, &p) in self.pinned.iter().enumerate() {
            tip_ann.push(
                Annotation::text_arrow(
                    AnnotationCoord::Data,
                    [p[0] - 0.65, p[1] + 0.42],
                    p,
                    format!("#{} ({:.2}, {:.2})", i + 1, p[0], p[1]),
                )
                .with_stroke(Stroke::new(1.4, Color32::from_rgb(215, 120, 20))),
            );
        }
        if self.pinned.len() >= 2 {
            let a = self.pinned[self.pinned.len() - 2];
            let b = self.pinned[self.pinned.len() - 1];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let slope = if dx.abs() > 1e-9 { dy / dx } else { 0.0 };
            tip_ann.push(
                Annotation::double_arrow(AnnotationCoord::Data, a, b)
                    .with_stroke(Stroke::new(1.2, Color32::from_rgb(130, 50, 180))),
            );
            tip_ann.push(
                Annotation::text(
                    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5],
                    format!("Δx={dx:.2} Δy={dy:.2} m={slope:.2}"),
                )
                .with_color(Color32::from_rgb(130, 50, 180)),
            );
        }

        let thresh_ann = [
            Annotation::y_region(
                [self.lower_limit, self.upper_limit],
                Color32::from_rgba_unmultiplied(60, 175, 110, 32),
            ),
            Annotation::x_region(
                [2.0, self.win_end],
                Color32::from_rgba_unmultiplied(70, 145, 235, 34),
            ),
            Annotation::y_line(self.upper_limit, "Upper")
                .with_stroke(Stroke::new(1.2, Color32::from_rgb(215, 70, 55))),
            Annotation::y_line(self.lower_limit, "Lower")
                .with_stroke(Stroke::new(1.2, Color32::from_rgb(215, 140, 40))),
            Annotation::double_arrow(
                AnnotationCoord::Data,
                [2.0, self.upper_limit + 0.12],
                [self.win_end, self.upper_limit + 0.12],
            )
            .with_stroke(Stroke::new(1.2, Color32::from_rgb(45, 120, 210))),
            Annotation::ellipse(AnnotationCoord::Normalized, [[0.66, 0.14], [0.94, 0.42]])
                .with_stroke(Stroke::new(1.2, Color32::from_rgb(180, 90, 45))),
            Annotation::textbox(
                AnnotationCoord::Normalized,
                [[0.04, 0.78], [0.38, 0.94]],
                "Safe Operating Band",
            )
            .with_fill(Color32::from_rgba_unmultiplied(245, 240, 210, 175))
            .with_stroke(Stroke::new(1.0, Color32::GRAY)),
        ];

        let bubble_leg = BubbleLegend::new(
            self.bubble_data.size_range(),
            self.bubble_data.radius_limits(),
        )
        .steps(3)
        .title("Size");

        let frame_nr = ui.ctx().cumulative_frame_nr();
        let [s0, s1, s2, s3] = &mut self.states;
        let responses = Figure::new("custom-annotations-fig")
            .columns(2)
            .cell_height(height)
            .annotations(fig_ann)
            .show(ui, 4, |cell| match cell.index() {
                0 => {
                    let resp = cell.show_layers(
                        Plot::new("ca-datatips")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .annotations(tip_ann.as_slice())
                            .title(tr(
                                l,
                                "Click-to-Pin DataTips & Peak/Valley Callouts",
                                "点击固定 DataTip 游标 + 极值标注",
                                "クリック DataTip 固定＆極値注釈",
                            )),
                        s0,
                        &[Line::new(&self.signal).name("Damped").into()],
                    );
                    if resp.response.clicked()
                        && self.last_pin_frame != Some(frame_nr)
                        && let Some(hp) = resp.hovered_point.as_ref()
                    {
                        self.last_pin_frame = Some(frame_nr);
                        if self.pinned.len() >= 4 {
                            self.pinned.remove(0);
                        }
                        self.pinned.push(hp.point);
                    }
                    resp
                }
                1 => cell.show_layers(
                    Plot::new("ca-thresholds")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .annotations(thresh_ann.as_slice())
                        .title(tr(
                            l,
                            "Threshold Lines, Regions, Shapes & TextBox",
                            "交互阈值线 / 区间 / 形状与文本框",
                            "しきい値・領域・図形注釈",
                        )),
                    s1,
                    &[Line::new(&self.signal).name("Signal").into()],
                ),
                2 => {
                    let mut layers: Vec<egui_customplot::PlotLayer<'_>> = Vec::new();
                    for (idx, (shape, pts)) in self.marker_rows.iter().enumerate() {
                        let col = egui::Color32::from_rgb(
                            ((idx * 53) % 190 + 35) as u8,
                            ((idx * 97) % 160 + 50) as u8,
                            ((idx * 139) % 180 + 40) as u8,
                        );
                        layers.push(
                            Scatter::new(pts)
                                .shape(*shape)
                                .radius(5.0)
                                .color(col)
                                .into(),
                        );
                    }
                    layers.push(
                        Line::new(&self.dash_line)
                            .name("Dashed")
                            .pattern(LinePattern::Dashed {
                                length: self.dash_len,
                                gap: self.gap_len,
                            })
                            .into(),
                    );
                    layers.push(BubbleChart::new(&self.bubble_data).name("Bubbles").into());
                    cell.show_layers(
                        Plot::new("ca-markers")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .bubble_legend(bubble_leg.clone())
                            .title(tr(
                                l,
                                "MarkerShape Palette, Dash Pattern & BubbleLegend",
                                "全标记形状 / 动态虚线与气泡图例",
                                "マーカー種別・破線・バブル凡例",
                            )),
                        s2,
                        &layers,
                    )
                }
                _ => cell.show_layers(
                    Plot::new("ca-brush")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .allow_brush(true)
                        .hover_formatter(|info| {
                            let pt = info.hovered_point.map_or(info.cursor, |p| p.point);
                            format!("Sample unit (mV / ms)\nx = {:.3}, y = {:.3}", pt[0], pt[1])
                        })
                        .title(tr(
                            l,
                            "Alt+Drag Brush Selection & Unit Tooltip",
                            "Alt+拖拽数据刷选与定制单位悬停提示",
                            "Alt+ドラッグ選択＆カスタムツールチップ",
                        )),
                    s3,
                    &[Scatter::new(&self.brush_cloud)
                        .radius(3.5)
                        .shape(MarkerShape::CircleHollow)
                        .name("Cloud")
                        .into()],
                ),
            })
            .expect("custom annotations figure")
            .inner;

        if let Some(brushed) = self.states[3].brushed_bounds() {
            let idxs = self.brush_cloud.brushed_indices(brushed);
            let count = idxs.len();
            if count > 0 {
                let pts = self.brush_cloud.points();
                let mut sum = 0.0;
                let mut ymin = f64::INFINITY;
                let mut ymax = f64::NEG_INFINITY;
                for &i in &idxs {
                    let y = pts[i][1];
                    sum += y;
                    ymin = ymin.min(y);
                    ymax = ymax.max(y);
                }
                let mean = sum / count as f64;
                let var = idxs
                    .iter()
                    .map(|&i| (pts[i][1] - mean).powi(2))
                    .sum::<f64>()
                    / count as f64;
                ui.horizontal(|ui| {
                    ui.small(format!(
                        "Brush stats: N={count}, mean={mean:.3}, std={:.3}, min={ymin:.3}, max={ymax:.3}",
                        var.sqrt()
                    ));
                    if ui.small_button(tr(l, "Clear", "清除", "解除")).clicked() {
                        self.states[3].clear_brush();
                    }
                });
            }
        }
        responses
    }
}
