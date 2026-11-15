//! Host-owned grouped/stacked bars, legacy hist, box/swarm/violin/raincloud, pie/donut, and clouds.
use std::f32::consts::FRAC_PI_2;

use eframe::egui::{self, Color32};
use egui_customplot::{
    Axis, BarData, Bars, BoxChart, BoxChartData, BubbleCloud, BubbleCloudData, Figure,
    HistogramNormalization, LegacyHistData, PieChart, PieData, Plot, PlotLayer, PlotResponse,
    PlotState, Raincloud, RaincloudData, SwarmChart, SwarmData, ViolinData, ViolinHalf,
    ViolinOptions, ViolinPlot, ViolinScale, WordCloud, WordCloudData,
};

const PALETTE: [Color32; 4] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(126, 47, 142),
];

pub(super) struct DistributionsDemo {
    pub enabled: bool,
    pub stacked_mode: bool,
    pub half_violin: bool,
    pub donut_mode: bool,
    grouped_bars: Vec<BarData>,
    stacked_bars: Vec<BarData>,
    legacy_hist: BarData,
    box_data: BoxChartData,
    swarm_data: SwarmData,
    violin_full: ViolinData,
    violin_half: ViolinData,
    raincloud: RaincloudData,
    pie_data: PieData,
    word_cloud: WordCloudData,
    bubble_cloud: BubbleCloudData,
    states: [PlotState; 8],
}

fn synthetic_group(mean: f64, spread: f64, phase: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let t = (i as f64 + 0.5) / n as f64;
            let z = (t * std::f64::consts::TAU * 3.0 + phase).sin() * 0.65
                + (t * std::f64::consts::TAU * 7.0 + phase * 1.7).cos() * 0.35;
            let bimod = if i % 5 == 0 { spread * 0.55 } else { 0.0 };
            mean + spread * z + bimod
        })
        .collect()
}

impl Default for DistributionsDemo {
    fn default() -> Self {
        let centers = [1.0, 2.0, 3.0, 4.0];
        let s0 = [4.2, 5.8, 3.9, 6.4];
        let s1 = [2.6, 3.4, 4.8, 3.1];
        let s2 = [1.9, 2.2, 2.7, 3.5];
        let grouped_bars = BarData::grouped(&centers, &[&s0, &s1, &s2], 0.8).expect("grouped");
        let stacked_bars = BarData::stacked(&centers, &[&s0, &s1, &s2], 0.65).expect("stacked");

        let hist_samples: Vec<f64> = (0..360)
            .map(|i| {
                let x = (i as f64) * 0.09;
                5.0 + 1.8 * x.sin() + 0.7 * (2.3 * x).cos()
            })
            .collect();
        let legacy_hist = LegacyHistData::new(&hist_samples, 12)
            .expect("legacy hist")
            .normalized_bars(HistogramNormalization::Count)
            .expect("hist bars");

        let g1 = synthetic_group(3.2, 1.1, 0.2, 48);
        let g2 = synthetic_group(5.0, 1.4, 1.1, 48);
        let g3 = synthetic_group(4.1, 0.9, 2.3, 48);
        let groups: [&[f64]; 3] = [&g1, &g2, &g3];
        let pos = [1.0, 2.0, 3.0];

        let box_data = BoxChartData::from_groups(&pos, &groups, 0.42).expect("box");
        let swarm_data = SwarmData::from_groups(&pos, &groups, 0.52).expect("swarm");
        let violin_full =
            ViolinData::from_groups(&pos, &groups, ViolinOptions::default()).expect("full violin");
        let violin_half = ViolinData::from_groups(
            &pos,
            &groups,
            ViolinOptions {
                scale: ViolinScale::Area,
                half: ViolinHalf::Right,
                show_box: false,
                ..Default::default()
            },
        )
        .expect("half violin");
        let raincloud = RaincloudData::from_groups(
            &pos,
            &groups,
            ViolinOptions {
                width: 0.76,
                ..Default::default()
            },
        )
        .expect("raincloud");

        let pie_data = PieData::with_options(
            &[34.0, 26.0, 22.0, 18.0],
            Some(&["Compute", "Memory", "IO", "Sync"]),
            Some(&[false, true, false, false]),
            -FRAC_PI_2,
        )
        .expect("pie");

        let words = [
            "egui",
            "violin",
            "kde",
            "raincloud",
            "swarm",
            "boxchart",
            "pie",
            "donut",
            "pareto",
            "mesh",
            "tess",
            "atlas",
            "hist",
            "bar",
        ];
        let weights = [
            96.0, 84.0, 76.0, 72.0, 65.0, 58.0, 54.0, 49.0, 45.0, 40.0, 36.0, 32.0, 28.0, 24.0,
        ];
        let word_cloud = WordCloudData::new(&words, &weights).expect("wordcloud");
        let bubble_cloud =
            BubbleCloudData::new(&words[..8], &weights[..8], Some(&[0, 1, 2, 0, 1, 2, 3, 0]))
                .expect("bubblecloud");

        Self {
            enabled: false,
            stacked_mode: false,
            half_violin: false,
            donut_mode: true,
            grouped_bars,
            stacked_bars,
            legacy_hist,
            box_data,
            swarm_data,
            violin_full,
            violin_half,
            raincloud,
            pie_data,
            word_cloud,
            bubble_cloud,
            states: Default::default(),
        }
    }
}

impl DistributionsDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Distributions & clouds");
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(&mut self.stacked_mode, "Stacked bars");
            ui.checkbox(&mut self.half_violin, "Half-violin");
            ui.checkbox(&mut self.donut_mode, "Donut ring");
            if ui.button("Fit distribution panels").clicked() {
                self.states.iter_mut().for_each(PlotState::reset);
            }
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let group_names = ["Group A", "Group B", "Group C"];

        Figure::new("distribution-panels")
            .columns(4)
            .cell_height(height)
            .show(ui, 8, |cell| match cell.index() {
                0 => {
                    let bars = if self.stacked_mode {
                        &self.stacked_bars
                    } else {
                        &self.grouped_bars
                    };
                    let layers: Vec<PlotLayer<'_>> = bars
                        .iter()
                        .enumerate()
                        .map(|(i, b)| {
                            Bars::new(b)
                                .name(group_names[i])
                                .color(PALETTE[i % PALETTE.len()])
                                .into()
                        })
                        .collect();
                    let title = if self.stacked_mode {
                        "bar(..., stacked)"
                    } else {
                        "bar(..., grouped)"
                    };
                    cell.show_layers(
                        Plot::new("bars-grouped-stacked")
                            .height(height)
                            .x_axis(Axis::new().categories(["Q1", "Q2", "Q3", "Q4"]))
                            .legend(true)
                            .title(title),
                        &mut self.states[0],
                        &layers,
                    )
                }
                1 => cell.show_layers(
                    Plot::new("legacy-hist")
                        .height(height)
                        .title("Legacy hist(12 bins)"),
                    &mut self.states[1],
                    &[Bars::new(&self.legacy_hist)
                        .name("hist")
                        .color(PALETTE[0])
                        .into()],
                ),
                2 => cell.show_layers(
                    Plot::new("box-swarm")
                        .height(height)
                        .x_axis(Axis::new().categories(["Alpha", "Beta", "Gamma"]))
                        .legend(true)
                        .title("boxchart + swarmchart"),
                    &mut self.states[2],
                    &[
                        BoxChart::new(&self.box_data)
                            .name("boxchart")
                            .color(PALETTE[0])
                            .notched(true)
                            .show_mean(true)
                            .into(),
                        SwarmChart::new(&self.swarm_data)
                            .name("swarm")
                            .color(PALETTE[1])
                            .radius(2.8)
                            .into(),
                    ],
                ),
                3 => {
                    let vdata = if self.half_violin {
                        &self.violin_half
                    } else {
                        &self.violin_full
                    };
                    cell.show_layers(
                        Plot::new("violin")
                            .height(height)
                            .x_axis(Axis::new().categories(["Alpha", "Beta", "Gamma"]))
                            .title("violinplot (KDE)"),
                        &mut self.states[3],
                        &[ViolinPlot::new(vdata)
                            .name("violin")
                            .color(PALETTE[3])
                            .into()],
                    )
                }
                4 => cell.show_layers(
                    Plot::new("raincloud")
                        .height(height)
                        .x_axis(Axis::new().categories(["Alpha", "Beta", "Gamma"]))
                        .title("raincloudplot"),
                    &mut self.states[4],
                    &[Raincloud::new(&self.raincloud)
                        .name("raincloud")
                        .color(PALETTE[2])
                        .into()],
                ),
                5 => {
                    let layer = if self.donut_mode {
                        PieChart::donut(&self.pie_data)
                    } else {
                        PieChart::new(&self.pie_data)
                    };
                    cell.show_layers(
                        Plot::new("pie-donut")
                            .canvas_axes()
                            .data_aspect(1.0)
                            .height(height)
                            .title(if self.donut_mode {
                                "donutchart"
                            } else {
                                "piechart"
                            }),
                        &mut self.states[5],
                        &[layer.into()],
                    )
                }
                6 => cell.show_layers(
                    Plot::new("word-cloud")
                        .canvas_axes()
                        .height(height)
                        .title("wordcloud"),
                    &mut self.states[6],
                    &[WordCloud::new(&self.word_cloud).base_font_px(16.0).into()],
                ),
                _ => cell.show_layers(
                    Plot::new("bubble-cloud")
                        .canvas_axes()
                        .data_aspect(1.0)
                        .height(height)
                        .title("bubblecloud"),
                    &mut self.states[7],
                    &[BubbleCloud::new(&self.bubble_cloud).into()],
                ),
            })
            .expect("8 distribution cells")
            .inner
    }
}
