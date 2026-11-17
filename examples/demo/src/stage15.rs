//! Stage 15 completion gallery (`--stage15`, `EXT-19..22`):
//! smithplot, zplane, bode, nyquist, nichols, rlocus, pwelch/spectrogram/Morlet CWT.
use std::f64::consts::TAU;

use eframe::egui;
use egui_customplot::{
    Axis, BodePlot, BodePlotState, ColorBar, ColorMap, ColorScale, CwtData, CwtPlot, Figure,
    FrequencyResponseData, Heatmap, Line, NicholsPlot, NyquistPlot, Plot, PlotLayer, PlotResponse,
    PlotState, PoleZeroData, PspectrumData, RootLocus, RootLocusData, SmithGridKind, SmithPlot,
    SmithPlotData, SmithPlotState, SmithTrace, SpectrogramData, TransferFunction, WindowKind,
    ZPlanePlot,
};

use crate::i18n::{lang, tr};

pub(super) struct Stage15Demo {
    pub smith_admittance: bool,
    pub nyquist_mirror: bool,
    pub nichols_grid: bool,
    pub use_cwt: bool,
    smith: SmithPlotData,
    zplane: PoleZeroData,
    freq_resp: FrequencyResponseData,
    rlocus: RootLocusData,
    welch: PspectrumData,
    periodo: PspectrumData,
    spec: SpectrogramData,
    cwt: CwtData,
    smith_state: SmithPlotState,
    bode_state: BodePlotState,
    states: [PlotState; 6],
}

impl Default for Stage15Demo {
    fn default() -> Self {
        let freqs: Vec<f64> = (0..48).map(|i| 1e9 + i as f64 * 6e7).collect();
        let z_series: Vec<[f64; 2]> = freqs
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let u = (i as f64 / 47.0) * 2.0 - 1.0;
                [50.0 / (1.0 + 0.55 * u * u), 38.0 * u]
            })
            .collect();
        let stub_gamma: Vec<[f64; 2]> = freqs
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let th = -0.4 - (i as f64 / 47.0) * 2.1;
                let mag = 0.25 + 0.22 * (i as f64 / 10.0).cos();
                [mag * th.cos(), mag * th.sin()]
            })
            .collect();
        let smith = SmithPlotData::new(vec![
            SmithTrace::from_impedance("RLC Load", &freqs, &z_series, 50.0).expect("s11"),
            SmithTrace::from_s_params("Stub Match", &freqs, &stub_gamma, 50.0).expect("s22"),
        ])
        .expect("smith")
        .with_vswr_circles(&[2.0]);

        let zplane = PoleZeroData::from_zeros_poles(
            &[[-1.0, 0.0], [0.0, 0.72], [0.0, -0.72]],
            &[[0.55, 0.52], [0.55, -0.52], [-0.45, 0.0], [-0.45, 0.0]],
        )
        .expect("zplane");

        let plant = TransferFunction::new(&[1.6], &[1.0, 1.8, 3.05, 2.25]).expect("tf");
        let freq_resp =
            FrequencyResponseData::from_transfer_fn(&plant, 0.08, 25.0, 220).expect("freq_resp");
        let rlocus = RootLocusData::auto_gains(&plant, 16.0, 120).expect("rlocus");

        let fs = 256.0;
        let n = 512usize;
        let sig: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / fs;
                let chirp_f = 8.0 + 48.0 * (t / 2.0);
                (TAU * 22.0 * t).sin()
                    + 0.7 * (TAU * 58.0 * t).cos()
                    + 0.8 * (TAU * chirp_f * t).sin()
                    + 0.15 * ((i as f64 * 1.7).sin())
            })
            .collect();
        let welch = PspectrumData::welch(&sig, fs, 128, 64, WindowKind::Hann).expect("welch");
        let periodo =
            PspectrumData::periodogram(&sig, fs, WindowKind::Blackman).expect("periodogram");
        let spec = SpectrogramData::stft(&sig, fs, 64, 48, 128, WindowKind::Hann).expect("spec");
        let cwt = CwtData::morlet_logspace(&sig[..256], fs, 6.0, 96.0, 28).expect("cwt");

        Self {
            smith_admittance: false,
            nyquist_mirror: true,
            nichols_grid: true,
            use_cwt: false,
            smith,
            zplane,
            freq_resp,
            rlocus,
            welch,
            periodo,
            spec,
            cwt,
            smith_state: SmithPlotState::default(),
            bode_state: BodePlotState::default(),
            states: std::array::from_fn(|_| PlotState::default()),
        }
    }
}

impl Stage15Demo {
    pub(super) fn reset_views(&mut self) {
        self.smith_state.reset();
        self.bode_state = BodePlotState::default();
        self.states.iter_mut().for_each(PlotState::reset);
    }

    pub(super) fn controls(&mut self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        ui.strong(tr(
            l,
            "RF Smith, Control & DSP/CWT",
            "射频 Smith 圆图、控制系统与时频分析",
            "RF Smith・制御解析・DSP/CWT",
        ));
        ui.checkbox(
            &mut self.smith_admittance,
            tr(
                l,
                "Smith Admittance Overlay",
                "Smith 导纳网格叠加",
                "アドミタンス重ね描き",
            ),
        );
        ui.checkbox(
            &mut self.nyquist_mirror,
            tr(
                l,
                "Nyquist Negative Freq Mirror",
                "Nyquist 负频共轭轨迹",
                "Nyquist 負周波数鏡像",
            ),
        );
        ui.checkbox(
            &mut self.nichols_grid,
            tr(
                l,
                "Nichols Closed-Loop M-Grid",
                "Nichols 等 M 圆轮廓",
                "Nichols 等 M 線図",
            ),
        );
        ui.checkbox(
            &mut self.use_cwt,
            tr(
                l,
                "Morlet CWT + COI Mode",
                "Morlet 小波 + COI 模式",
                "Morlet CWT + COI",
            ),
        );
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
        let height = ((ui.available_height() - 96.0) * 0.5).clamp(165.0, 255.0);
        let fig = Figure::new("stage15_fig")
            .columns(4)
            .cell_height(height)
            .spacing(egui::vec2(8.0, 8.0))
            .show(ui, 8, |cell| match cell.index() {
                0 => {
                    let kind = if self.smith_admittance {
                        SmithGridKind::ImpedanceAdmittance
                    } else {
                        SmithGridKind::Impedance
                    };
                    cell.show_smithplot(
                        SmithPlot::new("s15_smith", &self.smith)
                            .height(height)
                            .allow_scroll_zoom(true)
                            .grid_kind(kind)
                            .title(tr(
                                l,
                                "EXT-19 RF Smith Chart",
                                "EXT-19 RF 史密斯圆图",
                                "EXT-19 スミスチャート",
                            )),
                        &mut self.smith_state,
                    )
                    .into_plot_response()
                }
                1 => {
                    let plot = Plot::new("s15_zplane")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .zplane_axes()
                        .title(tr(
                            l,
                            "EXT-20 Z-Plane Pole/Zero",
                            "EXT-20 Z 平面零极点图",
                            "EXT-20 Z 平面極零点",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[0],
                        &[ZPlanePlot::new(&self.zplane).name("H(z)").into()],
                    )
                }
                2 => cell
                    .show_bode(
                        BodePlot::new("s15_bode", &self.freq_resp)
                            .height(height)
                            .allow_scroll_zoom(true)
                            .title(tr(
                                l,
                                "EXT-21 Bode & Stability Margins",
                                "EXT-21 波特图与稳定裕度",
                                "EXT-21 ボード線図",
                            )),
                        &mut self.bode_state,
                    )
                    .into_plot_response(),
                3 => {
                    let plot = Plot::new("s15_nyq")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .data_aspect(1.0)
                        .title(tr(
                            l,
                            "EXT-21 Nyquist Locus",
                            "EXT-21 奈奎斯特轨迹",
                            "EXT-21 ナイキスト線図",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[1],
                        &[NyquistPlot::new(&self.freq_resp)
                            .show_mirror(self.nyquist_mirror)
                            .name("G(jw)")
                            .into()],
                    )
                }
                4 => {
                    let plot = Plot::new("s15_nich")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .x_axis(Axis::new().label("Open-Loop Phase (deg)"))
                        .y_axis(Axis::new().label("Magnitude (dB)"))
                        .title(tr(
                            l,
                            "EXT-21 Nichols Chart",
                            "EXT-21 尼科尔斯图",
                            "EXT-21 ニコルス線図",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[2],
                        &[NicholsPlot::new(&self.freq_resp)
                            .show_grid(self.nichols_grid)
                            .name("Nichols")
                            .into()],
                    )
                }
                5 => {
                    let plot = Plot::new("s15_rl")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .title(tr(
                            l,
                            "EXT-21 Evans Root Locus",
                            "EXT-21 根轨迹图",
                            "EXT-21 根軌跡",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[3],
                        &[RootLocus::new(&self.rlocus).name("rlocus").into()],
                    )
                }
                6 => {
                    let plot = Plot::new("s15_welch")
                        .height(height)
                        .allow_scroll_zoom(true)
                        .legend(true)
                        .x_axis(Axis::new().label("Frequency (Hz)"))
                        .y_axis(Axis::new().label("PSD (dB/Hz)"))
                        .title(tr(
                            l,
                            "EXT-22 Welch / Periodogram PSD",
                            "EXT-22 Welch / 周期图功率谱",
                            "EXT-22 Welch PSD",
                        ));
                    cell.show_layers(
                        plot,
                        &mut self.states[4],
                        &[
                            Line::new(self.periodo.psd_db_curve())
                                .color(egui::Color32::from_gray(145))
                                .name("Periodogram")
                                .into(),
                            Line::new(self.welch.psd_db_curve())
                                .width(1.8)
                                .name("Welch PSD")
                                .into(),
                        ],
                    )
                }
                _ => {
                    if self.use_cwt {
                        let scale = ColorScale::new(self.cwt.value_range())
                            .expect("cwt range")
                            .color_map(ColorMap::Viridis);
                        let plot = Plot::new("s15_tf")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .x_axis(Axis::new().label("Time (s)"))
                            .y_axis(Axis::new().label("Freq (Hz)"))
                            .title(tr(
                                l,
                                "EXT-22 Morlet CWT + COI",
                                "EXT-22 Morlet 小波时频",
                                "EXT-22 Morlet CWT",
                            ));
                        cell.show_layers(
                            plot,
                            &mut self.states[5],
                            &[CwtPlot::new(&self.cwt, scale).into()],
                        )
                    } else {
                        let scale = ColorScale::new(self.spec.value_range())
                            .expect("spec range")
                            .color_map(ColorMap::Inferno);
                        let _ = ColorBar::new(scale);
                        let plot = Plot::new("s15_tf")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .x_axis(Axis::new().label("Time (s)"))
                            .y_axis(Axis::new().label("Freq (Hz)"))
                            .title(tr(
                                l,
                                "EXT-22 STFT Spectrogram (dB)",
                                "EXT-22 短时傅里叶时频谱",
                                "EXT-22 STFT スペクトログラム",
                            ));
                        cell.show_layers(
                            plot,
                            &mut self.states[5],
                            &[PlotLayer::from(Heatmap::new(self.spec.grid(), scale))],
                        )
                    }
                }
            });
        fig.map_or_else(|_| Vec::new(), |r| r.inner)
    }
}
