//! Stage 14 completion gallery (`--stage14`, `EXT-13..18`):
//! probplot/qqplot, Kaplan-Meier ecdf, dendrogram & clustergram, PCA/Varimax biplot,
//! SPC controlchart, financial candlestick/OHLC & 2D bridge waterfall.
use eframe::egui;
use egui_customplot::{
    Axis, BarOrientation, Biplot, BiplotData, Candlestick, ClusterStandardize, Clustergram,
    ClustergramData, ClustergramState, ControlChart, ControlChartData, DendrogramData,
    DendrogramPlot, EcdfData, EcdfKind, EcdfPlot, Figure, LinkageMethod, OhlcBars, OhlcData, Plot,
    PlotLayer, PlotResponse, PlotState, ProbabilityPlot, ProbabilityPlotData, Waterfall2D,
    Waterfall2DData, civil_to_unix_secs,
};

use crate::i18n::{lang, tr};

pub(super) struct Stage14Demo {
    pub weibull_mode: bool,
    pub km_survival: bool,
    pub varimax: bool,
    pub nelson_rules: bool,
    pub ohlc_ticks: bool,
    pub dendro_threshold: f64,
    prob_normal: ProbabilityPlotData,
    prob_weibull: ProbabilityPlotData,
    ecdf_cdf: EcdfData,
    ecdf_surv: EcdfData,
    dendro: DendrogramData,
    clustergram: ClustergramData,
    biplot_pca: BiplotData,
    biplot_varimax: BiplotData,
    spc_rules: ControlChartData,
    spc_basic: ControlChartData,
    ohlc: OhlcData,
    waterfall: Waterfall2DData,
    states: [PlotState; 7],
    cg_state: ClustergramState,
}

impl Default for Stage14Demo {
    fn default() -> Self {
        let normal_samples = [
            1.8, 2.1, 2.4, 2.5, 2.7, 2.9, 3.0, 3.1, 3.3, 3.4, 3.5, 3.6, 3.8, 4.0, 4.2, 4.5, 4.9,
        ];
        let weibull_samples = [
            12.0, 18.0, 24.0, 29.0, 35.0, 42.0, 51.0, 63.0, 74.0, 88.0, 105.0, 128.0, 152.0, 190.0,
        ];
        let prob_normal = ProbabilityPlotData::normal(&normal_samples).expect("normal probplot");
        let prob_weibull = ProbabilityPlotData::weibull(&weibull_samples).expect("weibull");

        let times = [
            1.0, 2.0, 3.0, 3.0, 5.0, 6.0, 8.0, 10.0, 11.0, 14.0, 16.0, 20.0,
        ];
        let cens = [
            false, true, false, false, false, true, false, false, true, false, false, true,
        ];
        let ecdf_cdf = EcdfData::kaplan_meier(&times, &cens, EcdfKind::Cdf, 0.95).expect("cdf");
        let ecdf_surv =
            EcdfData::kaplan_meier(&times, &cens, EcdfKind::Survival, 0.95).expect("surv");

        let obs_rows: [&[f64]; 8] = [
            &[1.0, 1.2, 0.2],
            &[1.3, 1.0, 0.4],
            &[0.9, 1.4, 0.1],
            &[5.1, 5.4, 1.8],
            &[5.3, 4.9, 2.1],
            &[4.8, 5.2, 1.9],
            &[2.7, 3.0, 4.5],
            &[3.1, 2.8, 4.8],
        ];
        let dendro = DendrogramData::from_observations(&obs_rows, LinkageMethod::Ward)
            .expect("dendrogram")
            .with_labels(
                ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8"]
                    .map(str::to_owned)
                    .to_vec(),
            );

        let cg_matrix: [&[f64]; 6] = [
            &[2.4, 2.1, -1.2, -1.5, 0.3],
            &[2.0, 2.5, -1.0, -1.4, 0.1],
            &[-1.3, -1.1, 2.2, 1.9, -0.4],
            &[-1.5, -1.3, 2.4, 2.1, -0.2],
            &[0.2, 0.1, -0.3, 0.2, 2.6],
            &[-0.1, 0.3, -0.2, 0.1, 2.3],
        ];
        let clustergram = ClustergramData::new(
            &cg_matrix,
            LinkageMethod::Average,
            ClusterStandardize::RowZScore,
        )
        .expect("clustergram")
        .with_row_labels(["G1", "G2", "G3", "G4", "G5", "G6"].map(str::to_owned))
        .with_col_labels(["T1", "T2", "T3", "T4", "T5"].map(str::to_owned));

        let pca_rows: [&[f64]; 10] = [
            &[2.5, 2.4, 1.2, 0.5],
            &[0.5, 0.7, 0.2, 0.1],
            &[2.2, 2.9, 1.4, 0.8],
            &[1.9, 2.2, 1.0, 0.4],
            &[3.1, 3.0, 1.8, 1.1],
            &[2.3, 2.7, 1.3, 0.7],
            &[2.0, 1.6, 0.8, 0.3],
            &[1.0, 1.1, 0.4, 0.2],
            &[1.5, 1.6, 0.9, 0.5],
            &[1.1, 0.9, 0.3, 0.1],
        ];
        let biplot_pca = BiplotData::from_pca(&pca_rows)
            .expect("biplot")
            .with_var_labels(["Temp", "Press", "Flow", "Load"].map(str::to_owned));
        let biplot_varimax = biplot_pca.clone().with_varimax();

        let spc_vals = [
            10.1, 10.3, 9.9, 10.2, 10.4, 10.1, 10.5, 10.2, 10.6, 10.7, 10.8, 10.9, 11.1, 11.3,
            13.6, 10.2, 9.8, 10.1, 9.7, 9.6,
        ];
        let spc_rules = ControlChartData::imr(&spc_vals)
            .expect("imr")
            .with_nelson_rules(true);
        let spc_basic = spc_rules.clone().with_nelson_rules(false);

        let base_t = civil_to_unix_secs(2026, 3, 2, 0, 0, 0.0).expect("date");
        let dates: Vec<f64> = (0..12).map(|d| base_t + d as f64 * 86_400.0).collect();
        let open = [
            102.0, 104.5, 103.8, 106.2, 108.0, 107.1, 105.4, 104.0, 106.8, 109.5, 108.4, 111.0,
        ];
        let close = [
            104.5, 103.8, 106.2, 108.0, 107.1, 105.4, 104.0, 106.8, 109.5, 108.4, 111.0, 113.2,
        ];
        let high = [
            105.8, 106.0, 107.1, 109.4, 109.0, 108.2, 106.9, 107.8, 110.4, 111.2, 112.1, 114.6,
        ];
        let low = [
            101.1, 102.6, 103.2, 105.1, 106.3, 104.6, 103.0, 103.4, 106.0, 107.9, 107.5, 110.2,
        ];
        let ohlc = OhlcData::from_slices(&dates, &open, &high, &low, &close).expect("ohlc");

        let waterfall = Waterfall2DData::from_deltas(
            &[120.0, 42.0, -28.0, 0.0, 35.0, -19.0, 0.0],
            Some(&[false, false, false, true, false, false, true]),
            Some(&["Base", "Sales", "COGS", "Gross", "Fx", "Tax", "Net"]),
        )
        .expect("waterfall");

        Self {
            weibull_mode: false,
            km_survival: false,
            varimax: false,
            nelson_rules: true,
            ohlc_ticks: false,
            dendro_threshold: 2.2,
            prob_normal,
            prob_weibull,
            ecdf_cdf,
            ecdf_surv,
            dendro,
            clustergram,
            biplot_pca,
            biplot_varimax,
            spc_rules,
            spc_basic,
            ohlc,
            waterfall,
            states: std::array::from_fn(|_| PlotState::default()),
            cg_state: ClustergramState::default(),
        }
    }
}

impl Stage14Demo {
    pub(super) fn reset_views(&mut self) {
        self.states.iter_mut().for_each(PlotState::reset);
        self.cg_state = ClustergramState::default();
    }

    pub(super) fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "Stats, Clustering, SPC & Finance",
            "概率分位、聚类、SPC 与金融 K 线",
            "確率・クラスタリング・SPC・金融",
        ));
        ui.checkbox(
            &mut self.weibull_mode,
            tr(
                l,
                "Weibull Scale (wblplot)",
                "Weibull 概率尺",
                "Weibull 確率軸",
            ),
        );
        ui.checkbox(
            &mut self.km_survival,
            tr(
                l,
                "KM Survival S(t)",
                "KM 生存曲线 S(t)",
                "KM 生存曲線 S(t)",
            ),
        );
        ui.checkbox(
            &mut self.varimax,
            tr(
                l,
                "Biplot Varimax Rotation",
                "双标图 Varimax 旋转",
                "Varimax 回転",
            ),
        );
        ui.checkbox(
            &mut self.nelson_rules,
            tr(
                l,
                "SPC Nelson Rules",
                "SPC Nelson 判异规则",
                "SPC Nelson ルール",
            ),
        );
        ui.checkbox(
            &mut self.ohlc_ticks,
            tr(l, "OHLC Tick Bars", "OHLC 竹线模式", "OHLC バー表示"),
        );
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 96.0) * 0.5).clamp(165.0, 255.0);
        let fig = Figure::new("stage14_fig")
            .columns(4)
            .cell_height(height)
            .spacing(egui::vec2(8.0, 8.0))
            .show(ui, 8, |cell| match cell.index() {
                0 => {
                    let data = if self.weibull_mode {
                        &self.prob_weibull
                    } else {
                        &self.prob_normal
                    };
                    let plot = Plot::new("s14_prob")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .probplot_axes(data)
                        .title(tr(
                            l,
                            "EXT-13 Probability Plot",
                            "EXT-13 概率图",
                            "EXT-13 確率プロット",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[0],
                        &[ProbabilityPlot::new(data).name("Samples").into()],
                    )
                }
                1 => {
                    let data = if self.km_survival {
                        &self.ecdf_surv
                    } else {
                        &self.ecdf_cdf
                    };
                    let plot = Plot::new("s14_ecdf")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .title(tr(
                            l,
                            "EXT-14 Kaplan-Meier ECDF",
                            "EXT-14 Kaplan-Meier ECDF",
                            "EXT-14 Kaplan-Meier ECDF",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[1],
                        &[EcdfPlot::new(data).name("KM + 95% CI").into()],
                    )
                }
                2 => {
                    let plot = Plot::new("s14_dendro")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .x_axis(Axis::new().explicit_ticks(self.dendro.leaf_ticks()))
                        .title(tr(
                            l,
                            "EXT-15 Ward Dendrogram",
                            "EXT-15 Ward 聚类树",
                            "EXT-15 Ward デンドログラム",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[2],
                        &[DendrogramPlot::new(&self.dendro)
                            .orientation(BarOrientation::Vertical)
                            .color_threshold(self.dendro_threshold)
                            .into()],
                    )
                }
                3 => {
                    let cg = Clustergram::new("s14_clustergram", &self.clustergram)
                        .height(height)
                        .allow_scroll_zoom(true)
                        .title(tr(
                            l,
                            "EXT-15 Hierarchical Clustergram",
                            "EXT-15 双向聚类热图",
                            "EXT-15 クラスターグラム",
                        ));
                    cell.show_clustergram(cg, &mut self.cg_state)
                        .into_plot_response()
                }
                4 => {
                    let data = if self.varimax {
                        &self.biplot_varimax
                    } else {
                        &self.biplot_pca
                    };
                    let plot = Plot::new("s14_biplot")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .data_aspect(1.0)
                        .title(tr(
                            l,
                            "EXT-16 PCA / Varimax Biplot",
                            "EXT-16 PCA / Varimax 双标图",
                            "EXT-16 PCA / Varimax バイプロット",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[3],
                        &[Biplot::new(data).name("Biplot").into()],
                    )
                }
                5 => {
                    let data = if self.nelson_rules {
                        &self.spc_rules
                    } else {
                        &self.spc_basic
                    };
                    let plot = Plot::new("s14_spc")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .title(tr(
                            l,
                            "EXT-17 SPC I-MR Control Chart",
                            "EXT-17 SPC I-MR 控制图",
                            "EXT-17 SPC I-MR 管理図",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[4],
                        &[ControlChart::new(data).name("I-MR").into()],
                    )
                }
                6 => {
                    let plot = Plot::new("s14_ohlc")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .x_axis(Axis::new().datetime())
                        .title(tr(
                            l,
                            "EXT-18 Candlestick / OHLC",
                            "EXT-18 K线 / OHLC",
                            "EXT-18 ローソク足 / OHLC",
                        ));
                    let layer: PlotLayer<'_> = if self.ohlc_ticks {
                        OhlcBars::new(&self.ohlc).name("OHLC").into()
                    } else {
                        Candlestick::new(&self.ohlc).name("Candle").into()
                    };
                    cell.show_layers(plot, &mut self.states[5], &[layer])
                }
                _ => {
                    let plot = Plot::new("s14_wf")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .waterfall_axes(&self.waterfall)
                        .title(tr(
                            l,
                            "EXT-18 2D Waterfall Bridge",
                            "EXT-18 二维增减瀑布图",
                            "EXT-18 2D ウォーターフォール",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[6],
                        &[Waterfall2D::new(&self.waterfall).name("Bridge").into()],
                    )
                }
            });
        fig.map_or_else(|_| Vec::new(), |r| r.inner)
    }
}
