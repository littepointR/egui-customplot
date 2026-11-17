# 阶段 15：信号处理、控制系统与射频 Smith 圆图（`EXT-19..22`）

日期：2026-11-17。依据 [扩展规划](EXPANSION_PLAN.md)，Stage 15 在保持 [MATLAB 133 项冻结基线](MATLAB_PLOT_CHECKLIST.md) 不变的前提下，完整落地 4 项电子/控制/射频信号领域能力 `EXT-19..22`：RF Smith 圆图、Z 平面零极点图（含重数聚类）、Bode/Nyquist/Nichols/Evans 根轨迹控制分析套图，以及纯 Rust Radix-2 + Bluestein FFT / Welch / STFT / Morlet CWT 时频谱分析。

## 1. 实现与架构落位

| 扩展编号 | 能力范围 | 新增/扩展核心类型与方法 | 源码域落位 |
| --- | --- | --- | --- |
| `EXT-19` | `smithplot` RF Smith 圆图 | `SmithGridKind`（阻抗 / 导纳 / 双叠网格）、`SmithTransform`（共形莫比乌斯正反变换 `z_to_gamma`/`gamma_to_z`、VSWR、Return Loss、圆图平移与滚轮缩放）、`SmithPlotData`/`SmithTrace`、`SmithPlot`/`SmithPlotState`、`HoveredSmithPoint` | [axis/smith.rs](../crates/egui-customplot/src/axis/smith.rs)、[data/smith.rs](../crates/egui-customplot/src/data/smith.rs)、[interaction/smith.rs](../crates/egui-customplot/src/interaction/smith.rs)、[render/smith.rs](../crates/egui-customplot/src/render/smith.rs)、[widget/smith.rs](../crates/egui-customplot/src/widget/smith.rs) |
| `EXT-20` | `zplane` 零极点图 | `poly_roots_aberth`（Aberth-Ehrlich 全自动复根求解）、`PoleZeroData`（MATLAB `zplane(b,a)` 阶次零补齐 + `×N` 重根聚类 + `is_causal_stable`）、`ZPlanePlot`（`o`/`x` 与重数上标）、`Plot::zplane_axes` | [data/control/poly.rs](../crates/egui-customplot/src/data/control/poly.rs)、[data/zplane.rs](../crates/egui-customplot/src/data/zplane.rs)、[model/stage15.rs](../crates/egui-customplot/src/model/stage15.rs) |
| `EXT-21` | `bode` / `nyquist` / `nichols` / `rlocus` | `TransferFunction`、`FrequencyResponseData`（连续展开相位 + 自动稳定裕度 `GM`/`PM`/`w_pc`/`w_gc`）、`RootLocusData`（连续跟踪 Evans 根轨迹分支）、`BodePlot`/`BodePlotState`、`NyquistPlot`、解析等 M 轮廓 `NicholsPlot`、`RootLocus` | [data/control/freq_response.rs](../crates/egui-customplot/src/data/control/freq_response.rs)、[data/control/rlocus.rs](../crates/egui-customplot/src/data/control/rlocus.rs)、[widget/bode.rs](../crates/egui-customplot/src/widget/bode.rs)、[render/stage15.rs](../crates/egui-customplot/src/render/stage15.rs) |
| `EXT-22` | `pwelch` / `spectrogram` / `pspectrum` / `cwt` | `fft_radix2` + Bluestein `fft_arbitrary`（支持任意样本长度）、`WindowKind`、`PspectrumData::{periodogram, welch}`、`SpectrogramData::stft`、`CwtData::morlet_logspace`（含 `COI` 边界）与 `CwtPlot` | [data/dsp/fft.rs](../crates/egui-customplot/src/data/dsp/fft.rs)、[data/dsp/spectral.rs](../crates/egui-customplot/src/data/dsp/spectral.rs)、[data/dsp/cwt.rs](../crates/egui-customplot/src/data/dsp/cwt.rs)、[interaction/point_stage15.rs](../crates/egui-customplot/src/interaction/point_stage15.rs) |

## 2. 演示程序与多语言更新

- [stage15.rs](../examples/demo/src/stage15.rs) 与 [stage15_tests.rs](../examples/demo/src/stage15_tests.rs)：新增 `--stage15` 独立展厅（8 面板：RF Smith 圆图及导纳切换、Z 平面重根零极点、双轴对齐 Bode 与裕度线、Nyquist、Nichols 等 M 圆、Evans 根轨迹、Welch PSD 与 STFT/Morlet CWT 切换），中/英/日控件均已接入 [app.rs](../examples/demo/src/app.rs) 与 [i18n_tests.rs](../examples/demo/src/i18n_tests.rs)（扩展至 22 组预设）。

## 3. 验证结果与诚实边界

- **专项测试**：[stage15_rf_control_dsp.rs](../crates/egui-customplot/tests/stage15_rf_control_dsp.rs) 4 项（涵盖 Smith 共形精反演、重根计数、三阶系统解析 `GM = 12.04 dB` / `w_pc = sqrt(3)` 校验、Parseval/Bluestein FFT 与 Morlet COI）与 [export_stage15.rs](../crates/egui-customplot/tests/export_stage15.rs) 通过；工作区 `customplot-demo` 测试、架构护栏 `<= 400` 行检查与严格 `fmt`/`clippy` 同批验证。
- **保留边界**：跨平台视觉基准及 Stage 16（体渲染 / 3D 点云 / 3D 地球 / 现代图型与最终全收敛）留在下一阶段完成。
