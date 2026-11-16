//! Host-owned bubble + BubbleLegend, histogram2, binscatter, categorical heatmap, parallelplot, and composites.
use eframe::egui::{self, Color32};
use egui_customplot::{
    Axis, BinScatter, BinScatterData, BinShape, BubbleChart, BubbleData, BubbleLegend,
    CategoricalHeatmap, CategoricalHeatmapData, ColorMap, ColorScale, Figure, Histogram2,
    Histogram2Data, Line, MarginalKind, ParallelData, ParallelNormalization, ParallelPlot,
    ParetoChart, ParetoData, Plot, PlotData, PlotLayer, PlotMatrix, PlotMatrixData, PlotPoint,
    PlotResponse, PlotState, PlotYy, ScatterHistogram, ScatterHistogramData, ScatterHistogramState,
    SortOrder, StackedPlot, TableAggregation,
};

use crate::i18n::{lang, tr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MultivariateView {
    Overview,
    Stacked,
    ScatterHistogram,
    PlotMatrix,
}

pub(super) struct MultivariateDemo {
    pub enabled: bool,
    pub view: MultivariateView,
    pub hex_bins: bool,
    bubble_data: BubbleData,
    hist2: Histogram2Data,
    bin_hex: BinScatterData,
    bin_rect: BinScatterData,
    cat_heatmap: CategoricalHeatmapData,
    parallel: ParallelData,
    pareto: ParetoData,
    yy_left: PlotData,
    yy_right: PlotData,
    stacked_signals: [PlotData; 3],
    scatter_hist: ScatterHistogramData,
    plot_matrix: PlotMatrixData,
    overview_states: [PlotState; 7],
    stacked_states: Vec<PlotState>,
    scatter_hist_state: ScatterHistogramState,
    matrix_states: Vec<PlotState>,
}

impl Default for MultivariateDemo {
    fn default() -> Self {
        let mut b_pts = Vec::with_capacity(28);
        let mut b_sizes = Vec::with_capacity(28);
        let mut b_colors = Vec::with_capacity(28);
        for i in 0..28 {
            let t = i as f64 * 0.35;
            b_pts.push([1.0 + t * 0.9, 2.0 + (t * 1.4).sin() * 2.2 + t * 0.25]);
            b_sizes.push(15.0 + ((i * 7) % 19) as f64 * 6.0);
            b_colors.push(i as f64);
        }
        let bubble_data = BubbleData::new(&b_pts, &b_sizes)
            .expect("bubble")
            .with_colors(&b_colors)
            .expect("bubble colors")
            .radius_range(4.0, 18.0)
            .expect("radius range");

        let mut xs = Vec::with_capacity(900);
        let mut ys = Vec::with_capacity(900);
        let mut pts = Vec::with_capacity(900);
        for i in 0..900 {
            let a = (i as f64) * 0.173;
            let r = ((i % 45) as f64 / 45.0) * 2.4;
            let x = r * a.cos() + if i % 3 == 0 { 1.2 } else { -0.8 };
            let y = 0.65 * x + r * a.sin() * 0.7;
            xs.push(x);
            ys.push(y);
            pts.push([x, y]);
        }
        let hist2 = Histogram2Data::new(&xs, &ys, 14, 14).expect("histogram2");
        let bin_hex = BinScatterData::new(&pts, 14, 14, BinShape::Hexagon).expect("hex");
        let bin_rect = BinScatterData::new(&pts, 14, 14, BinShape::Rectangle).expect("rect");

        let cat_heatmap = CategoricalHeatmapData::from_table(
            &[
                "US", "US", "EU", "EU", "APAC", "APAC", "US", "EU", "APAC", "US", "EU", "APAC",
            ],
            &[
                "Q1", "Q2", "Q1", "Q2", "Q1", "Q2", "Q3", "Q3", "Q3", "Q4", "Q4", "Q4",
            ],
            Some(&[
                42.0, 55.0, 38.0, 61.0, 49.0, 72.0, 64.0, 58.0, 81.0, 76.0, 69.0, 88.0,
            ]),
            TableAggregation::Mean,
            SortOrder::None,
        )
        .expect("cat heatmap");

        let mut par_storage = Vec::new();
        let mut par_groups = Vec::new();
        for i in 0..18 {
            let g = i % 3;
            let shift = g as f64 * 1.4;
            par_storage.push([
                2.0 + shift + (i as f64 * 0.4).sin(),
                18.0 - shift * 2.0 + (i as f64 * 0.7).cos() * 2.5,
                120.0 + shift * 22.0 + (i as f64 * 1.3),
                0.4 + g as f64 * 0.25 + (i as f64 * 0.3).sin() * 0.1,
            ]);
            par_groups.push(g);
        }
        let par_refs: Vec<&[f64]> = par_storage.iter().map(|r| r.as_slice()).collect();
        let parallel = ParallelData::new(
            &["Latency", "Throughput", "Memory", "Ratio"],
            &par_refs,
            ParallelNormalization::ZScore,
        )
        .expect("parallel")
        .with_groups(&par_groups, &["Cluster A", "Cluster B", "Cluster C"])
        .expect("groups");

        let pareto = ParetoData::new(
            &["Network", "Compute", "Disk", "Locking", "Parsing", "Other"],
            &[48.0, 31.0, 14.0, 9.0, 5.0, 3.0],
        )
        .expect("pareto");

        let y1: Vec<PlotPoint> = (0..120)
            .map(|i| {
                let x = i as f64 * 0.08;
                [x, (1.5 * x).sin() * 4.0 + 5.0]
            })
            .collect();
        let y2: Vec<PlotPoint> = (0..120)
            .map(|i| {
                let x = i as f64 * 0.08;
                [x, 100.0 * (-0.18 * x).exp() * (0.9 * x).cos() + 120.0]
            })
            .collect();

        let s0 = PlotData::new(
            (0..160)
                .map(|i| {
                    let x = i as f64 * 0.05;
                    [x, (2.0 * x).sin() + 0.3 * (5.0 * x).cos()]
                })
                .collect::<Vec<_>>(),
        );
        let s1 = PlotData::new(
            (0..160)
                .map(|i| {
                    let x = i as f64 * 0.05;
                    [x, 25.0 + 12.0 * (-0.25 * x).exp() * (3.0 * x).cos()]
                })
                .collect::<Vec<_>>(),
        );
        let s2 = PlotData::new(
            (0..160)
                .map(|i| {
                    let x = i as f64 * 0.05;
                    [x, x * 0.6 + 0.4 * (4.0 * x).sin()]
                })
                .collect::<Vec<_>>(),
        );

        let sh_x: Vec<f64> = (0..120)
            .map(|i| ((i % 3) as f64) * 1.8 + ((i as f64) * 0.53).sin() * 0.9)
            .collect();
        let sh_y: Vec<f64> = (0..120)
            .map(|i| sh_x[i] * 0.7 + ((i as f64) * 0.87).cos() * 0.8)
            .collect();
        let sh_grp: Vec<usize> = (0..120).map(|i| i % 3).collect();
        let scatter_hist =
            ScatterHistogramData::new(&sh_x, &sh_y, Some(&sh_grp), 10, MarginalKind::Kernel)
                .expect("scatterhistogram");

        let pm_z: Vec<f64> = (0..120)
            .map(|i| 0.5 * sh_x[i] - 0.6 * sh_y[i] + ((i as f64) * 0.41).sin() * 0.7)
            .collect();
        let plot_matrix = PlotMatrixData::new(
            &["Feature X", "Feature Y", "Feature Z"],
            &[&sh_x, &sh_y, &pm_z],
            Some(&sh_grp),
            8,
        )
        .expect("plotmatrix");

        Self {
            enabled: false,
            view: MultivariateView::Overview,
            hex_bins: true,
            bubble_data,
            hist2,
            bin_hex,
            bin_rect,
            cat_heatmap,
            parallel,
            pareto,
            yy_left: PlotData::new(y1),
            yy_right: PlotData::new(y2),
            stacked_signals: [s0, s1, s2],
            scatter_hist,
            plot_matrix,
            overview_states: Default::default(),
            stacked_states: vec![PlotState::default(); 3],
            scatter_hist_state: ScatterHistogramState::default(),
            matrix_states: Vec::new(),
        }
    }
}

impl MultivariateDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.separator();
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "Multivariate & composite",
                "多变量与复合子图",
                "多変量・複合プロット",
            ),
        );
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.selectable_value(
                &mut self.view,
                MultivariateView::Overview,
                tr(l, "Overview", "总览面板", "概要"),
            );
            ui.selectable_value(&mut self.view, MultivariateView::Stacked, "stackedplot");
            ui.selectable_value(
                &mut self.view,
                MultivariateView::ScatterHistogram,
                "scatterhistogram",
            );
            ui.selectable_value(&mut self.view, MultivariateView::PlotMatrix, "plotmatrix");
            if self.view == MultivariateView::Overview {
                ui.checkbox(
                    &mut self.hex_bins,
                    tr(l, "Hex binscatter", "六边形分箱散点", "六角ビニング"),
                );
            }
            if ui
                .button(tr(
                    l,
                    "Fit multivariate",
                    "适配多变量视图",
                    "多変量フィット",
                ))
                .clicked()
            {
                self.overview_states.iter_mut().for_each(PlotState::reset);
                self.stacked_states.iter_mut().for_each(PlotState::reset);
                self.scatter_hist_state = ScatterHistogramState::default();
                self.matrix_states.iter_mut().for_each(PlotState::reset);
            }
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        match self.view {
            MultivariateView::Overview => self.show_overview(ui),
            MultivariateView::Stacked => {
                let row_h = ((ui.available_height() - 120.0) / 3.0).max(96.0);
                StackedPlot::new(
                    "mv-stackedplot",
                    vec![
                        (
                            tr(l, "Signal 1", "信号 1", "信号 1"),
                            vec![PlotLayer::from(
                                Line::new(&self.stacked_signals[0])
                                    .color(Color32::from_rgb(0, 114, 189)),
                            )],
                        ),
                        (
                            tr(l, "Envelope", "包络线", "包絡線"),
                            vec![PlotLayer::from(
                                Line::new(&self.stacked_signals[1])
                                    .color(Color32::from_rgb(217, 83, 25)),
                            )],
                        ),
                        (
                            tr(l, "Trend", "趋势项", "トレンド"),
                            vec![PlotLayer::from(
                                Line::new(&self.stacked_signals[2])
                                    .color(Color32::from_rgb(119, 172, 48)),
                            )],
                        ),
                    ],
                )
                .row_height(row_h)
                .x_label(tr(l, "Time (s)", "时间 (s)", "時間 (s)"))
                .title(tr(
                    l,
                    "stackedplot (Synchronized X)",
                    "stackedplot（共享 X 轴堆叠图）",
                    "stackedplot（同期 X 軸）",
                ))
                .show(ui, &mut self.stacked_states)
                .expect("stackedplot")
                .figure
                .inner
            }
            MultivariateView::ScatterHistogram => {
                let cell_h = ((ui.available_height() - 120.0) * 0.48).max(120.0);
                ScatterHistogram::new("mv-scatterhist", &self.scatter_hist)
                    .cell_height(cell_h)
                    .labels(
                        tr(l, "X Value", "X 数值", "X 値"),
                        tr(l, "Y Value", "Y 数值", "Y 値"),
                    )
                    .title(tr(
                        l,
                        "scatterhistogram + Marginal KDE",
                        "散点边际直方图 + KDE",
                        "scatterhistogram＋周辺 KDE",
                    ))
                    .show(ui, &mut self.scatter_hist_state)
                    .expect("scatterhistogram")
                    .inner
            }
            MultivariateView::PlotMatrix => {
                let cell_h = ((ui.available_height() - 120.0) / 3.0).max(96.0);
                PlotMatrix::new("mv-plotmatrix", &self.plot_matrix)
                    .cell_height(cell_h)
                    .show(ui, &mut self.matrix_states)
                    .expect("plotmatrix")
                    .inner
            }
        }
    }

    fn show_overview(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let row_h = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let c_bubble = ColorScale::new([0.0, 27.0])
            .expect("bubble scale")
            .color_map(ColorMap::Viridis);
        let c_biv = ColorScale::new([0.0, 28.0])
            .expect("biv scale")
            .color_map(ColorMap::Viridis);
        let c_cat = ColorScale::new([35.0, 90.0])
            .expect("cat scale")
            .color_map(ColorMap::BlueRed);
        let bubble_legend = BubbleLegend::new(
            self.bubble_data.size_range(),
            self.bubble_data.radius_limits(),
        )
        .title("Weight");

        let mut all = Figure::new("mv-top")
            .columns(4)
            .cell_height(row_h)
            .show(ui, 4, |cell| match cell.index() {
                0 => cell.show_layers(
                    Plot::new("mv-bubble")
                        .height(row_h)
                        .bubble_legend(bubble_legend.clone())
                        .title("bubblechart + bubblelegend"),
                    &mut self.overview_states[0],
                    &[PlotLayer::from(
                        BubbleChart::new(&self.bubble_data).color_scale(c_bubble),
                    )],
                ),
                1 => cell.show_layers(
                    Plot::new("mv-hist2").height(row_h).title("histogram2"),
                    &mut self.overview_states[1],
                    &[PlotLayer::from(Histogram2::new(&self.hist2, c_biv))],
                ),
                2 => {
                    let bins = if self.hex_bins {
                        &self.bin_hex
                    } else {
                        &self.bin_rect
                    };
                    cell.show_layers(
                        Plot::new("mv-binscatter")
                            .height(row_h)
                            .title(if self.hex_bins {
                                "binscatter (Hex)"
                            } else {
                                "binscatter (Rect)"
                            }),
                        &mut self.overview_states[2],
                        &[PlotLayer::from(BinScatter::new(bins, c_biv))],
                    )
                }
                _ => cell.show_layers(
                    Plot::new("mv-cathm")
                        .categorical_heatmap_axes(&self.cat_heatmap)
                        .height(row_h)
                        .title("heatmap(tbl)"),
                    &mut self.overview_states[3],
                    &[PlotLayer::from(
                        CategoricalHeatmap::new(&self.cat_heatmap, c_cat).show_cell_labels(true),
                    )],
                ),
            })
            .expect("4 overview cells")
            .inner;

        ui.add_space(6.0);
        ui.columns(3, |cols| {
            all.push(
                Plot::new("mv-parallel")
                    .height(row_h)
                    .x_axis(Axis::new().explicit_ticks(self.parallel.x_ticks()))
                    .title("parallelplot (ZScore)")
                    .show_layers(
                        &mut cols[0],
                        &mut self.overview_states[4],
                        &[PlotLayer::from(
                            ParallelPlot::new(&self.parallel).show_markers(true),
                        )],
                    ),
            );
            all.push(
                ParetoChart::new("mv-pareto", &self.pareto)
                    .height(row_h)
                    .title("pareto")
                    .show(&mut cols[1], &mut self.overview_states[5]),
            );
            all.push(
                PlotYy::lines(
                    "mv-plotyy",
                    Line::new(&self.yy_left)
                        .name("Amplitude")
                        .color(Color32::from_rgb(0, 114, 189))
                        .width(2.0),
                    Line::new(&self.yy_right)
                        .name("Decay")
                        .color(Color32::from_rgb(217, 83, 25))
                        .width(2.0),
                )
                .height(row_h)
                .title("plotyy")
                .show_legend(true)
                .show(&mut cols[2], &mut self.overview_states[6]),
            );
        });
        all
    }
}
