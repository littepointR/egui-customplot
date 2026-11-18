[English](README.md) | 简体中文

# egui-customplot

面向 [`egui`](https://github.com/emilk/egui) 的原生嵌入式高性能科学计算与工程绘图库：以 MATLAB 全系绘图体验为参考，坚持宿主持有不可变数据、调用方显式持有视图状态、`egui` 多遍布局严格幂等，且绝不劫持窗口与渲染上下文。

> **项目状态（`0.1.0`）**
>
> - **已实现并持续验证**：阶段 1～16 全部落地——覆盖 [MATLAB 绘图类型对照表](docs/MATLAB_PLOT_CHECKLIST.md) 全部 133/133 项（`82 ML + 35 EX + 16 LG`）、[Stages 12–16 扩展规划](docs/EXPANSION_PLAN.md) `EXT-01..26`、三层千万点 `L1/L2/L3` 引擎、可选 `feature = "wgpu"` 硬件 Compute/实例化加速、可选 `feature = "export"` PNG/混合 SVG 离线导出、支持交互式 `EN | 简体中文` 切换的 `rustdoc` 文档，以及涵盖 27 组展厅的交互演示程序。
> - **开放边界（诚实交付口径）**：Linux / macOS / WebAssembly 实机显示回归、三维曲面/体渲染与大型分布图的硬件 GPU 管线、独立印刷排版纯矢量文字导出，以及长时全应用资源回归仍保留为开放边界。因此 133 项对照均保持 **“开发中”（0 已交付）**。
> - **安全与宿主隔离**：全 workspace 禁用 `unsafe`（`[workspace.lints.rust] unsafe_code = "forbid"`），绝不在核心库内另建窗口、事件循环、`wgpu::Device`、隐式后台线程或发起网络请求（含完全离线的二维地图与三维地球仪）。

---

## 核心特性

- **架构与宿主契约**
  - **宿主持有数据（`Arc`）**：`PlotData`、`GridData`、`ContourData`、`Surface3DData`、`ScalarVolume3D`、`PointCloud3DData` 等不可变/按版本共享的数据源留在业务模型中；`PlotState` 与 `Plot3DState` 仅保存视图范围与交互状态，不滞留样本或 GPU 纹理。
  - **多遍布局严格幂等**：在 `egui::Context::request_discard` 触发重复布局时，平移/缩放、框选/刷选、`AxisLink` 联动、`PlotStream` / 墙钟动画推进以及延迟文件导出均严格按帧去重。
  - **七域分层架构**：库源码按 `axis / data / interaction / layout / model / render / widget`（及 `diagnostics`）划分职责边界，由 `tests/architecture.rs` 强制校验目录规范与单文件 `<= 400` 行上限。
- **三层性能架构（`L1` / `L2` / `L3`）**
  - **L1 零配置智能默认**：`PlotData::new` 对 `>= 8,192` 点的单调非降 X 序列在 64 MiB 预算内自动建立索引；`LineSampling::Auto` 与 `MarkerSampling::Auto` 缩小自动峰值保真、放大自动切换精确绘制；`PlotState` 内置 2 MiB 自动缓存；原始点拾取采用 2D 屏幕包围盒分支定界剪枝。
  - **L2 全量客制化控制**：显式 `LineSampling::{Auto, Exact, Visible, MinMax}`、`MarkerSampling::{Auto, Exact, PixelGrid}`、`RenderBackend::{Auto, Painter, Wgpu}`、五层样式继承（`defaults -> Theme -> 继承样式/Figure -> Plot -> Series`）、`PlotDefaults`、`PlotCache`、`Plot::pan_limits`、双 Y 轴、线性/对数/Probit/Weibull/时间/时长/分类坐标轴，以及自包含 PNG/SVG 导出（`feature = "export"`）。
  - **L3 千万点流式与 `wgpu` 加速**：零拷贝环形 `IndexedStream`（`StreamLine`）、`O(1)` 切片 `UniformWaveform`（`WaveformLine`），以及可选 `feature = "wgpu"`（`register_wgpu`）复用宿主 `egui_wgpu::RenderState` 执行 WGSL Compute 列极值规约与实例化渲染；无硬件后端或导出快照时自动回退 CPU `Painter`。
- **全坐标系与图种覆盖**
  - **二维笛卡尔与计算几何**：折线 / 阶梯（`stairs`）/ 火柴杆（`stem`）/ 散点 / 误差棒 / 柱状图（分组、堆积、横向）/ 面积带 / 多边形 / 面片（Flat / Gouraud）/ 带孔 `PolyShape` / 矩形与椭圆 / 热力图 / `pcolor` / 曲线网格 / 等高线与填充等高线 / `imshow` / 自适应 `fplot` 与 `fimplicit` / 矢量场（`quiver` / `feather` / `streamline`）/ Delaunay `triplot` / Voronoi / `alphaShape` / CSR `spy` / `animatedline` & `comet` / `EX-027` 注解。
  - **统计分布、生信、SPC、金融与复合布局**：`histogram` / `histogram2` / `binscatter` / `boxchart` / `violinplot` / `raincloud` / `swarmchart` / `bubblechart` / 饼图与圆环 / 词云与气泡云 / 分类热力图 / `confusionchart` / 平行坐标 / PCA `biplot` / Kaplan-Meier `ecdf` / `probplot`（`Probit` / `Weibull` / `qqplot`）/ `dendrogram` / `clustergram` / `controlchart`（SPC + Nelson 规则）/ K 线与 OHLC / 瀑布图，以及 `Figure`（最多 256 格）、`StackedPlot`、`ScatterHistogram`、`PlotMatrix`、`PlotYy`、`ParetoChart`。
  - **极坐标、天球、射频、控制与信号谱**：`PolarPlot`（`polarplot` / `polarscatter` / `polarbubblechart` / `polarhistogram` / `rose` / `compass` / 极坐标高亮扇区）、`SkyPlot`、`SmithPlot`（共形阻抗/导纳圆图）、`ZPlanePlot`、`BodePlot`、`NyquistPlot`、`NicholsPlot`、`RootLocus`，以及纯 Rust Radix-2 + Bluestein FFT / Welch / STFT / Morlet CWT（`CwtPlot`）。
  - **三维与体数据**：`Plot3D`（正交/透视投影、`SortedPainter` / 有界 `ZBuffer` 含静止帧 RLE span 缓存）、三维点线柱杆矢饼平面、`surf`/`surfc`/`surfl`/`surfnorm`、`mesh`/`meshc`/`meshz`/`waterfall`、`contour3`、`ribbon`、`fsurf`/`fmesh`/`fimplicit3`、`sphere`/`cylinder`/`ellipsoid`/`trisurf`、`Patch3D`、`AlphaShape3D`、`Graph3D`、`slice`/`contourslice`、`isosurface`/`isocaps`、`tetramesh`、三维流可视化、CPU 光线投射 `VolumeRender3D`（`volshow`）与八叉树 LOD `PointCloud3D`（`pcshow`）。
  - **地理可视化与现代图表**：纯离线二维 `GeoPlot`（跨日期变更线切分、`GeoTileCache` 与离线回退策略）、WGS84 三维 `GeoGlobePlot`，以及雷达图 `RadarChart`、桑基图 `SankeyChart`、矩形树图 `TreemapChart`、旭日图 `SunburstChart` 与甘特图 `GanttChart`。

---

## 环境要求与 Cargo 特性

- **Rust**：**1.95+**（Edition `2024`；实测 Rust `1.100.0` / Windows x64 / `egui 0.37.0`）
- **Cargo Feature（`crates/egui-customplot/Cargo.toml`）**：
  - `default = []`：核心 CPU + `egui::Painter` 绘图（仅依赖 `egui`）。
  - `export`：启用 `PlotSnapshot`、`ExportOptions` 与 `CaptureLimits` 自包含 PNG / 混合矢量 SVG 导出（引入 `png`）。
  - `wgpu`：启用 `register_wgpu`、`register_wgpu_with_config`、`set_wgpu_budget` 与 `WgpuPlotConfig`（引入 `egui-wgpu`、`wgpu`、`bytemuck`）。

---

## 快速开始

在 `Cargo.toml` 中引入 `egui-customplot`：

```toml
[dependencies]
egui = "0.37.0"
egui-customplot = "0.1.0"
# 按需开启特性：
# egui-customplot = { version = "0.1.0", features = ["export", "wgpu"] }
```

### 最小二维绘图示例

跨帧在宿主模型中保留样本数据与 `PlotState`（不要每帧重建 `PlotData`）：

```rust
use egui_customplot::{Line, Plot, PlotData, PlotState};

// 初始化时：
let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
let mut state = PlotState::default();

// 在宿主的 egui UI 闭包中（ui: &mut egui::Ui）：
Plot::new("signal")
    .height(320.0)
    .allow_scroll_zoom(true) // 库默认关闭滚轮缩放以免抢占父 ScrollArea
    .allow_box_zoom(true)
    .show(ui, &mut state, &[Line::new(&data).name("信号")]);
```

### 异构图层叠加（`PlotLayer`）

```rust
use egui_customplot::{Bar, BarData, Bars, ErrorBar, ErrorBarData, ErrorBars, Plot, PlotState};

let bars = BarData::new(vec![Bar::new(1.0, 2.0), Bar::new(2.0, -1.0)]);
let errors = ErrorBarData::new(vec![ErrorBar::new([1.0, 2.0]).y_error([0.2, 0.4])]);
let mut state = PlotState::default();

Plot::new("measurements").show_layers(
    ui,
    &mut state,
    &[Bars::new(&bars).into(), ErrorBars::new(&errors).into()],
);
```

完整使用说明可阅读 [使用手册](crates/egui-customplot/docs/guide.md) 或本地构建支持页内 `EN | 简体中文` 一键切换的 `rustdoc` 文档：

```sh
cargo doc --workspace --no-deps --all-features --open
```

---

## 交互式演示程序（`customplot-demo`）

演示程序内置 **27 组互斥画廊**、全局 `PlotDefaults` 控制面板、系统深浅主题实时同步及 PNG/SVG 导出：

```sh
cargo run -p customplot-demo --release --locked
cargo run -p customplot-demo --release --locked -- --dynamic-10m
cargo run -p customplot-demo --release --locked -- --glow --plot3d
```

常用启动参数：

- **渲染后端与基准**：`--glow`（使用 `eframe::Renderer::Glow` 替代默认 `Wgpu`）、`--uncapped`、`--bench-window-10m=<路径.csv>`
- **基础二维与场**：默认二维信号/幂律、`--scalar`、`--statistical`、`--stairs`、`--stems`、`--quiver`、`--polygons`、`--contours`、`--functions`、`--geometry`
- **统计、极坐标与网络**：`--distributions`、`--multivariate`、`--polar`、`--networks`
- **三维、体数据与地理**：`--plot3d`、`--surfaces3d`、`--volumes3d`、`--geo`
- **扩展专题与千万点**：`--stage13`、`--stage14`、`--stage15`、`--stage16`、`--dynamic-10m`
- **客制化专题**：`--custom-axes`、`--custom-annotations`、`--custom-animation`、`--custom-styling`

交互操作：左键拖拽平移；悬停滚轮缩放；右键或 `Shift`+左键框选放大；`Alt`+拖拽刷选数据（`Esc` 清除）；双击或聚焦按 `Home` 复位；三维图中左键轨道旋转、右键平移。

---

## 检查与性能实测

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo test --workspace --all-features --locked
cargo check -p egui-customplot --no-default-features --locked
cargo doc -p egui-customplot --no-deps --no-default-features --locked
cargo doc --workspace --no-deps --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo build -p customplot-demo --release --locked
cargo run -p egui-customplot --example export_snapshot --features export --release --locked -- target/exports/example
```

已归档 Windows 实测摘要（历史记录独立保留不覆盖；无头 CPU + tessellation 基准均**不代表** GPU 或 swapchain 帧率，原生窗口结果单列）：

- **无头千万点 CPU + tessellation**（`1920x1080`，[2026-11-16-dynamic-10m-cpu.csv](docs/benchmarks/2026-11-16-dynamic-10m-cpu.csv)）：
  - `uniform_waveform_10m_live_32k`：`total_p95 = 2.1877 ms`（`p50 = 1.9166 ms`）
  - `indexed_stream_10m_live_32k`：`total_p95 = 6.6787 ms`（`p50 = 6.0130 ms`）
  - `plotdata_auto_10m_pan_hover`：`total_p95 = 1.3365 ms`（`pick_p95 = 0.1149 ms`）
  - `scatter_auto_1m_pan`：`total_p95 = 4.8535 ms`
- **深度缩小 1,000 万点 CPU**（`20x`～`10^12x` 动态深缩 + 悬停拾取，[2026-11-16-deep-zoom-out-10m-cpu.csv](docs/benchmarks/2026-11-16-deep-zoom-out-10m-cpu.csv)）：`total_p50 = 0.8311 ms`、`total_p95 = 1.4256 ms`、`pick_p95 = 0.0115 ms`。
- **Windows 原生 `eframe + wgpu` 窗口**（`AMD Radeon RX 7800 XT (Vulkan)`，`AutoNoVsync`，[2026-11-16-dynamic-10m-window-wgpu.csv](docs/benchmarks/2026-11-16-dynamic-10m-window-wgpu.csv)）：
  - `wgpu_10m_waveform_live_32k`：实测 **699.99 FPS**（2,450 帧，`frame_p50 = 1.3124 ms`、`frame_p95 = 1.7540 ms`，每帧仅上传 131,072 B）
  - `cpu_10m_stream_ring_live_32k`（纯 CPU 增量树 + LOD）：实测 **174.13 FPS**（610 帧，`frame_p50 = 5.5789 ms`、`frame_p95 = 6.3164 ms`）
- **Stage 16 三维与现代图 CPU**（`1280x800`，[2026-11-17-stage16-volshow-opt-cpu.csv](docs/benchmarks/2026-11-17-stage16-volshow-opt-cpu.csv) 与 [2026-11-17-stage16-windows-cpu.csv](docs/benchmarks/2026-11-17-stage16-windows-cpu.csv)）：
  - `volshow_64x64x64_direct_alpha`（静态缓存）：`total_p95 = 2.889 ms`（逐帧旋转未命中缓存：`1.893 ms`）；`pcshow_200k_octree_lod_zbuffer`：`0.890 ms`；`geoglobe_wgs84_coastlines_track`：`0.053 ms`；`modern_suite_5charts_figure`：`0.104 ms`。

复跑基准示例（快速试跑可设 `CUSTOMPLOT_BENCH_SECONDS=2`，正式归档保持默认 30 秒）：

```sh
cargo bench -p egui-customplot --bench dynamic_10m_performance --locked
cargo bench -p egui-customplot --bench stage16_performance --locked
```

---

## 项目文档

- [源码组织与域边界](docs/ARCHITECTURE.md)
- [全量实施进度与验证证据](docs/PROGRESS.md)
- [项目规划与分阶段验收](docs/PLAN.md)
- [Stage 12–16 全量扩展规划（`EXT-01..26`）](docs/EXPANSION_PLAN.md)
- [MATLAB 绘图类型交付对照表（133 项 + 工具箱分账）](docs/MATLAB_PLOT_CHECKLIST.md)
- [egui 原生集成与性能约束](docs/EGUI_INTEGRATION_AND_PERFORMANCE.md)
- 各阶段详细记录：[1](docs/STAGE_1.md) · [2](docs/STAGE_2.md) · [3](docs/STAGE_3.md) · [4](docs/STAGE_4.md) · [5](docs/STAGE_5.md) · [6](docs/STAGE_6.md) · [7](docs/STAGE_7.md) · [8](docs/STAGE_8.md) · [9](docs/STAGE_9.md) · [10](docs/STAGE_10.md) · [11](docs/STAGE_11.md) · [12](docs/STAGE_12.md) · [13](docs/STAGE_13.md) · [14](docs/STAGE_14.md) · [15](docs/STAGE_15.md) · [16](docs/STAGE_16.md)
- 社区规范：[贡献指南](CONTRIBUTING.md) · [更新日志](CHANGELOG.md) · [安全策略](SECURITY.md)

---

## 参与贡献与许可

欢迎提交 Issue 与 Pull Request，参与前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

本项目采用与 [`egui`](https://github.com/emilk/egui) 相同的双重许可（`MIT OR Apache-2.0`）：

- Apache License, Version 2.0（见 [LICENSE-APACHE](LICENSE-APACHE)）
- MIT License（见 [LICENSE-MIT](LICENSE-MIT)）

按 Apache-2.0 第 5 条约定，向本仓库提交贡献均默认按上述双重许可提供，不附加额外条款。
