# 阶段 14：统计 / 生信 / SPC / 金融绘图（`EXT-13..18`）

日期：2026-11-17。依据 [扩展规划](EXPANSION_PLAN.md)，Stage 14 在保持 [MATLAB 133 项冻结基线](MATLAB_PLOT_CHECKLIST.md) 不变的前提下，完整落地 6 项统计/生物信息/工业过程/金融能力 `EXT-13..18`：`AxisScale::{Probit, Weibull}` 概率刻度、Kaplan-Meier ECDF 与 Greenwood 置信带、四种凝聚层次聚类树与双向聚类热图、PCA/Varimax 双标图、SPC 控制图与 Nelson 判异规则、金融 K 线/OHLC 竹线与二维增减桥接瀑布图。

## 1. 实现与架构落位

| 扩展编号 | 能力范围 | 新增/扩展核心类型与方法 | 源码域落位 |
| --- | --- | --- | --- |
| `EXT-13` | `probplot` / `normplot` / `wblplot` / `qqplot` | `AxisScale::{Probit, Weibull}`、`norm_inv`（Acklam 高精度反函数）/ `norm_cdf`（Cody `erfc`）、`ProbabilityPlotData::{normal, weibull, qq_normal, qq_two_sample}`、`ProbabilityPlot`、`Plot::probplot_axes` | [axis/probability.rs](../crates/egui-customplot/src/axis/probability.rs)、[axis/scale.rs](../crates/egui-customplot/src/axis/scale.rs)、[data/probplot.rs](../crates/egui-customplot/src/data/probplot.rs) |
| `EXT-14` | `ecdf` / `cdfplot` | `EcdfData::{from_samples, kaplan_meier}`（右删失同点时序处理 + Greenwood 置信带）、`EcdfKind::{Cdf, Survival, CumulativeHazard}`、`EcdfPlot` | [data/ecdf.rs](../crates/egui-customplot/src/data/ecdf.rs)、[render/stage14_stats.rs](../crates/egui-customplot/src/render/stage14_stats.rs) |
| `EXT-15` | `dendrogram` & `clustergram` | `DendrogramData::{from_observations, from_condensed_distances}`（`Single`/`Complete`/`Average`/`Ward` Lance-Williams 凝聚）、`DendrogramPlot`、`RegionKind::DendrogramBranch`、`ClustergramData` 与双树联锁复合控件 `Clustergram` / `FigureCell::show_clustergram` | [data/clustering.rs](../crates/egui-customplot/src/data/clustering.rs)、[data/clustergram.rs](../crates/egui-customplot/src/data/clustergram.rs)、[widget/clustergram.rs](../crates/egui-customplot/src/widget/clustergram.rs) |
| `EXT-16` | `biplot` | `BiplotData::{from_pca, from_loadings_scores, with_varimax}`（Jacobi 实对称特征求解、MATLAB 主分量符号约定与正交 Varimax 旋转）、`Biplot` 与 `HoveredVector` + 得分点联合拾取 | [data/biplot.rs](../crates/egui-customplot/src/data/biplot.rs)、[interaction/point_stage14.rs](../crates/egui-customplot/src/interaction/point_stage14.rs)、[interaction/hover.rs](../crates/egui-customplot/src/interaction/hover.rs) |
| `EXT-17` | `controlchart` | `ControlChartData::{imr, xbar_r, p_chart, u_chart}`、`SpcViolationKind::{BeyondThreeSigma, NineOneSide, SixTrend}`、`ControlChart`（红菱失控点高亮 + UCL/CL/LCL 标签与拾取） | [data/spc.rs](../crates/egui-customplot/src/data/spc.rs)、[model/stage14_spc_fin.rs](../crates/egui-customplot/src/model/stage14_spc_fin.rs)、[render/stage14_spc_fin.rs](../crates/egui-customplot/src/render/stage14_spc_fin.rs) |
| `EXT-18` | `candle` / `highlow` / 2D `waterfall` | `OhlcData::{new, from_slices}`、`Candlestick`（`hollow_up` 实/空心切换 + `Axis::datetime` 对接）、`OhlcBars`、`Waterfall2DData`、`Waterfall2D`、`Plot::waterfall_axes`、`RegionKind::{OhlcCandle, WaterfallBar}` | [data/financial.rs](../crates/egui-customplot/src/data/financial.rs)、[render/stage14_spc_fin.rs](../crates/egui-customplot/src/render/stage14_spc_fin.rs)、[interaction/region/stage14.rs](../crates/egui-customplot/src/interaction/region/stage14.rs) |

## 2. 演示程序与多语言更新

- [stage14.rs](../examples/demo/src/stage14.rs) 与 [stage14_tests.rs](../examples/demo/src/stage14_tests.rs)：新增 `--stage14` 独立展厅（8 面板 `Figure`：Normal/Weibull `ProbabilityPlot`、Kaplan-Meier `EcdfPlot`、Ward `DendrogramPlot`、`Clustergram`、PCA/Varimax `Biplot`、SPC `ControlChart`、`Candlestick`/`OhlcBars` 与 `Waterfall2D`），中/英/日交互控制均已接入 [app.rs](../examples/demo/src/app.rs) 与 [i18n_tests.rs](../examples/demo/src/i18n_tests.rs)（扩展至 21 组预设）。

## 3. 验证结果与诚实边界

- **专项测试**：[stage14_stats_bio_finance.rs](../crates/egui-customplot/tests/stage14_stats_bio_finance.rs) 与 [export_stage14.rs](../crates/egui-customplot/tests/export_stage14.rs) 全通过；[architecture.rs](../crates/egui-customplot/tests/architecture.rs)（七域结构 + 400 行源码红线）、[checklist.rs](../crates/egui-customplot/tests/checklist.rs)、工作区 `customplot-demo` 测试、`cargo fmt --all -- --check`、`cargo check --no-default-features` 与严格 `clippy --all-targets --all-features -- -D warnings` 同批通过。
- **保留边界**：跨平台实机视觉回归及 Stage 15–16 图种仍在后续计划推进中。
