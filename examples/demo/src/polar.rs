//! Host-owned polar charts (`polarplot`, `polarscatter`, `polarbubblechart`, `polarhistogram`, `compassplot`, `fpolarplot`, `ezpolar`, `rose`, regions).
use std::f64::consts::TAU;

use eframe::egui::{self, Color32, Stroke};
use egui_customplot::{
    BubbleLegend, ColorMap, ColorScale, CompassData, CompassPlot, Figure, FunctionPlotOptions,
    MarkerShape, MarkerStyle, NegativeRadiusMode, PlotResponse, PolarBubble, PolarBubbleData,
    PolarCurveData, PolarHistNormalization, PolarHistogram, PolarHistogramData, PolarLayer,
    PolarLine, PolarPlot, PolarRegionAnnotation, PolarScatter, PolarState, RoseData,
    ThetaDirection, ThetaZeroLocation,
};

use crate::i18n::{lang, tr};

pub(super) struct PolarDemo {
    pub enabled: bool,
    pub clockwise_top: bool,
    pub neg_reflect: bool,
    curve_cardioid: PolarCurveData,
    curve_fpolar: PolarCurveData,
    curve_ezpolar: PolarCurveData,
    bubble: PolarBubbleData,
    scatter: PolarCurveData,
    hist: PolarHistogramData,
    rose_curve: PolarCurveData,
    compass_cart: CompassData,
    compass_cmplx: CompassData,
    neg_curve: PolarCurveData,
    states: [PolarState; 6],
}

impl Default for PolarDemo {
    fn default() -> Self {
        let mut th = Vec::with_capacity(180);
        let mut r = Vec::with_capacity(180);
        for i in 0..=180 {
            let t = i as f64 / 180.0 * TAU;
            th.push(t);
            r.push(1.0 + (t).cos());
        }
        let curve_cardioid = PolarCurveData::from_theta_r(&th, &r).expect("cardioid");
        let curve_fpolar =
            PolarCurveData::from_function([0.0, TAU], FunctionPlotOptions::default(), |t| {
                0.8 + 0.65 * (5.0 * t).cos().abs()
            })
            .expect("fpolar")
            .curve()
            .clone();
        let curve_ezpolar = PolarCurveData::ezpolar(1600, |t| 0.5 + 0.5 * (2.0 * t).sin())
            .expect("ezpolar")
            .curve()
            .clone();

        let s_th: Vec<f64> = (0..36).map(|i| i as f64 * TAU / 36.0).collect();
        let s_r: Vec<f64> = s_th
            .iter()
            .map(|t| 0.4 + 0.9 * (2.0 * t).cos().abs())
            .collect();
        let s_sz: Vec<f64> = s_th
            .iter()
            .enumerate()
            .map(|(i, _)| 12.0 + ((i * 5) % 9) as f64 * 10.0)
            .collect();
        let s_col: Vec<f64> = s_th.iter().map(|t| t.sin() * 0.5 + 0.5).collect();
        let scatter = PolarCurveData::from_theta_r(&s_th[..18], &s_r[..18]).expect("scatter");
        let bubble = PolarBubbleData::new(&s_th, &s_r, &s_sz)
            .expect("bubble")
            .with_scalars(&s_col)
            .expect("bubble colors")
            .size_range([4.0, 18.0]);

        let angles: Vec<f64> = (0..360)
            .map(|i| (i as f64 * 0.17).sin() * 1.1 + if i % 3 == 0 { 0.35 } else { 2.6 })
            .collect();
        let hist = PolarHistogramData::new(&angles, 18).expect("polar hist");
        let rose_curve = RoseData::new(&angles, 12).expect("rose").to_polar_curve();

        let c_u: Vec<f64> = (0..12)
            .map(|i| {
                let a = i as f64 * TAU / 12.0;
                a.cos() * (0.6 + 0.4 * (2.0 * a).sin())
            })
            .collect();
        let c_v: Vec<f64> = (0..12)
            .map(|i| {
                let a = i as f64 * TAU / 12.0;
                a.sin() * (0.6 + 0.4 * (2.0 * a).sin())
            })
            .collect();
        let compass_cart = CompassData::from_cartesian(&c_u, &c_v).expect("compass");
        let compass_cmplx = CompassData::from_complex(&[[0.9, 0.3], [-0.4, 0.8], [-0.7, -0.5]]);

        let n_th: Vec<f64> = (0..=120).map(|i| i as f64 / 120.0 * TAU).collect();
        let n_r: Vec<f64> = n_th.iter().map(|t| (3.0 * t).cos()).collect();
        let neg_curve = PolarCurveData::from_theta_r(&n_th, &n_r).expect("neg polar");

        Self {
            enabled: false,
            clockwise_top: false,
            neg_reflect: true,
            curve_cardioid,
            curve_fpolar,
            curve_ezpolar,
            bubble,
            scatter,
            hist,
            rose_curve,
            compass_cart,
            compass_cmplx,
            neg_curve,
            states: Default::default(),
        }
    }
}

impl PolarDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.separator();
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "Polar plots & histograms",
                "极坐标绘图与直方图",
                "極座標プロット",
            ),
        );
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(
                &mut self.clockwise_top,
                tr(l, "Zero=Top, Clockwise", "零点朝上 / 顺时针", "上基準・時計回り"),
            );
            ui.checkbox(
                &mut self.neg_reflect,
                tr(
                    l,
                    "Reflect negative r through origin",
                    "负半径过原点对称映射",
                    "負半径を原点対称反転",
                ),
            );
            if ui.button(tr(l, "Reset polar views", "重置极坐标视图", "極座標ビュー初期化")).clicked() {
                self.states.iter_mut().for_each(PolarState::reset);
            }
            ui.small(tr(
                l,
                "polarplot, fpolarplot, ezpolar, polarscatter, polarbubblechart, polarhistogram, rose, compass & regions.",
                "包含 polarplot/fpolarplot/ezpolar、极坐标散点气泡直方图、风玫瑰与罗盘图。",
                "polarplot・散布図・バブル・極座標ヒストグラム・rose・compass 対応。",
            ));
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 120.0) * 0.5).max(110.0);
        let zero = if self.clockwise_top {
            ThetaZeroLocation::Top
        } else {
            ThetaZeroLocation::Right
        };
        let dir = if self.clockwise_top {
            ThetaDirection::Clockwise
        } else {
            ThetaDirection::CounterClockwise
        };
        let neg_mode = if self.neg_reflect {
            NegativeRadiusMode::ReflectOrigin
        } else {
            NegativeRadiusMode::ShiftMinRadius
        };
        let cscale = ColorScale::new([0.0, 1.0])
            .expect("valid color scale")
            .color_map(ColorMap::Viridis);
        let b_legend = BubbleLegend::new(self.bubble.size_extent(), self.bubble.radius_range())
            .steps(3)
            .title(tr(l, "Magnitude", "幅值", "強度"));
        let reg_theta =
            PolarRegionAnnotation::theta_region(330.0_f64.to_radians(), 30.0_f64.to_radians())
                .label(tr(l, "Wrap Sector", "跨零扇区", "境界セクタ"));
        let reg_rad =
            PolarRegionAnnotation::radius_region(0.4, 0.95).label(tr(l, "Band", "环带", "帯域"));
        let reg_pol = PolarRegionAnnotation::polar_region([1.2, 2.3], [0.6, 1.5])
            .border(Stroke::new(1.5, Color32::from_rgb(217, 95, 2)))
            .label(tr(l, "Polar ROI", "极区 ROI", "極 ROI"));

        let [s0, s1, s2, s3, s4, s5] = &mut self.states;
        Figure::new("polar-gallery")
            .columns(3)
            .cell_height(height)
            .show(ui, 6, |cell| match cell.index() {
                0 => cell
                    .show_polar(
                        PolarPlot::new("polar-curves")
                            .height(height)
                            .theta_zero(zero)
                            .theta_direction(dir)
                            .title("polarplot / fpolarplot / ezpolar"),
                        s0,
                        &[
                            PolarLayer::from(
                                PolarLine::new(&self.curve_cardioid)
                                    .name("polarplot")
                                    .color(Color32::from_rgb(0, 114, 189))
                                    .width(2.0),
                            ),
                            PolarLayer::from(
                                PolarLine::new(&self.curve_fpolar)
                                    .name("fpolarplot")
                                    .color(Color32::from_rgb(217, 83, 25)),
                            ),
                            PolarLayer::from(
                                PolarLine::new(&self.curve_ezpolar)
                                    .name("ezpolar")
                                    .color(Color32::from_rgb(126, 47, 142)),
                            ),
                        ],
                    )
                    .into_plot_response(),
                1 => cell
                    .show_polar(
                        PolarPlot::new("polar-scatter-bubble")
                            .height(height)
                            .theta_zero(zero)
                            .theta_direction(dir)
                            .bubble_legend(b_legend.clone())
                            .title("polarscatter & polarbubblechart"),
                        s1,
                        &[
                            PolarLayer::from(
                                PolarBubble::new(&self.bubble)
                                    .name("bubble")
                                    .color_scale(cscale),
                            ),
                            PolarLayer::from(
                                PolarScatter::new(&self.scatter)
                                    .name("scatter")
                                    .color(Color32::from_rgb(220, 40, 40))
                                    .marker(MarkerStyle {
                                        shape: MarkerShape::Diamond,
                                        radius: 4.5,
                                    }),
                            ),
                        ],
                    )
                    .into_plot_response(),
                2 => cell
                    .show_polar(
                        PolarPlot::new("polar-hist-rose")
                            .height(height)
                            .theta_zero(zero)
                            .theta_direction(dir)
                            .title("polarhistogram (PDF) & rose"),
                        s2,
                        &[
                            PolarLayer::from(
                                PolarHistogram::new(&self.hist)
                                    .name("polarhistogram")
                                    .normalization(PolarHistNormalization::Count)
                                    .color(Color32::from_rgb(70, 130, 180)),
                            ),
                            PolarLayer::from(
                                PolarLine::new(&self.rose_curve)
                                    .name("rose petals")
                                    .color(Color32::from_rgb(217, 83, 25))
                                    .width(1.6),
                            ),
                        ],
                    )
                    .into_plot_response(),
                3 => cell
                    .show_polar(
                        PolarPlot::new("polar-compass")
                            .height(height)
                            .theta_zero(zero)
                            .theta_direction(dir)
                            .title("compassplot & compass(Z)"),
                        s3,
                        &[
                            PolarLayer::from(
                                CompassPlot::new(&self.compass_cart)
                                    .name("compassplot")
                                    .color(Color32::from_rgb(0, 114, 189)),
                            ),
                            PolarLayer::from(
                                CompassPlot::new(&self.compass_cmplx)
                                    .name("complex")
                                    .color(Color32::from_rgb(217, 83, 25)),
                            ),
                        ],
                    )
                    .into_plot_response(),
                4 => cell
                    .show_polar(
                        PolarPlot::new("polar-regions")
                            .height(height)
                            .theta_zero(zero)
                            .theta_direction(dir)
                            .regions([reg_theta.clone(), reg_rad.clone(), reg_pol.clone()])
                            .title("thetaregion / radiusregion / polarregion"),
                        s4,
                        &[PolarLayer::from(
                            PolarLine::new(&self.curve_cardioid)
                                .name("signal")
                                .color(Color32::from_rgb(0, 114, 189)),
                        )],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_polar(
                        PolarPlot::new("polar-neg-mode")
                            .height(height)
                            .theta_zero(ThetaZeroLocation::Top)
                            .theta_direction(ThetaDirection::Clockwise)
                            .negative_radius_mode(neg_mode)
                            .title("r = cos(3θ) Negative Radius Mode"),
                        s5,
                        &[PolarLayer::from(
                            PolarLine::new(&self.neg_curve)
                                .name("rose(3θ)")
                                .color(Color32::from_rgb(119, 172, 48))
                                .width(2.0),
                        )],
                    )
                    .into_plot_response(),
            })
            .expect("polar gallery figure")
            .inner
    }
}
