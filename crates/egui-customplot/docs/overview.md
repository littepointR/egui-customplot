<div class="cp-lang-en">

# egui-customplot

> **Guide**: Read the **[`guide`]** (Complete User Guide & Plot Catalog). Use the **EN | 简体中文** toggle in the top header or sidebar to switch documentation language across all pages.

Native, embeddable scientific and technical plotting for [`egui`], designed with MATLAB-grade chart breadth, host-owned immutable data, explicit caller-owned view state, and deterministic multi-pass UI behavior.

## Core Design Principles

1. **Host-Owned Data (`Arc`)**: Raw samples live in host structs such as [`PlotData`], [`GridData`], [`Surface3DData`], [`ScalarVolume3D`], and [`PointCloud3DData`]. [`PlotState`] and [`Plot3DState`] never retain sample buffers or GPU textures.
2. **Explicit State & Multi-Pass Idempotency**: Pass `&mut PlotState` / `&mut Plot3DState` explicitly; navigation, axis links ([`AxisLink`]), and streaming snapshots deduplicate across discarded `egui::Context::request_discard` layout retries.
3. **Zero Window / Device Hijacking**: Every widget paints directly into the caller's `&mut egui::Ui` without creating windows, event loops, or separate `wgpu::Device` instances.
4. **Three-Tier Performance (`L1` / `L2` / `L3`)**:
   - **L1 Smart Defaults**: [`PlotData::new`] auto-indexes large monotonic-X series (`>= AUTO_INDEX_MIN_POINTS`); [`Line`] / [`Scatter`] default to `Auto` LOD ([`LineSampling::Auto`], [`MarkerSampling::Auto`]); [`PlotState`] embeds a bounded 2 MiB auto cache.
   - **L2 Full Control**: Explicit [`LineSampling`], [`MarkerSampling`], [`RenderBackend`], [`PlotCache`], [`Plot::pan_limits`], log/reversed/time/dual-Y axes, context-wide [`PlotDefaults`], and self-contained SVG/PNG capture (`PlotSnapshot`).
   - **L3 Streaming & GPU**: Zero-copy [`IndexedStream`] ([`StreamLine`]), `O(1)`-sliced [`UniformWaveform`] ([`WaveformLine`]), and optional `wgpu` compute/instancing (`register_wgpu`).

## Cargo Feature Flags

| Feature | Default | Description |
| :--- | :---: | :--- |
| `default` | Yes | Core CPU + tessellation plotting ([`Plot`], [`Plot3D`], [`PolarPlot`], [`GeoPlot`], etc.) |
| `export` | No | Enables `PlotSnapshot`, `ExportOptions`, and `CaptureLimits` for self-contained SVG & PNG export |
| `wgpu` | No | Enables `register_wgpu` / `WgpuPlotConfig` compute + instanced scatter/waveform pipelines |

## Quick Examples

### Minimal 2D Plot

```rust
use egui_customplot::{Line, Plot, PlotData, PlotState};

let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
let mut state = PlotState::default(); // retain in your app model
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
Plot::new("signal")
    .allow_scroll_zoom(true)
    .show(ui, &mut state, &[Line::new(&data)]);
# }).drop_without_applying_deltas();
```

### Streaming Snapshot, Index & Host Cache

```rust
use egui_customplot::{Line, LineSampling, PlotCache, PlotData, PlotStream};

let mut stream = PlotStream::new(4096, 4096 * 16)?;
stream.append_once(1, &[[1.0, 2.0], [2.0, 3.0]])?;
let mut data: PlotData = stream.snapshot(); // explicit O(N) publication
data.build_line_index(1024 * 1024)?;
let line = Line::new(&data).sampling(LineSampling::MinMax);
let mut cache = PlotCache::new(8 * 1024 * 1024); // retain in the host
// Plot::new("signal").show_cached(ui, state, &[line], &mut cache);
# let _ = (line, &mut cache);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 3D Surface & Z-Buffer Occlusion

```rust
use egui_customplot::{DepthMode, Layer3D, Plot3D, Plot3DState, Surface3D, Surface3DData};

let grid = [-1.0, 0.0, 1.0];
let z = [0.2, 0.6, 0.2, 0.6, 1.0, 0.6, 0.2, 0.6, 0.2];
let surf = Surface3DData::from_grid(&grid, &grid, &z)?;
let mut state = Plot3DState::default();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
Plot3D::new("surface")
    .depth_mode(DepthMode::ZBuffer)
    .allow_scroll_zoom(true)
    .show(ui, &mut state, &[Layer3D::from(Surface3D::new(&surf))]);
# }).drop_without_applying_deltas();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Architecture & Module Map

Source modules are organized strictly across seven domain boundaries (`axis`, `data`, `interaction`, `layout`, `model`, `render`, `widget`) plus `diagnostics`:

| Coordinate / Role | Primary Widget(s) | State & Sync | Key Layers |
| :--- | :--- | :--- | :--- |
| **2D Cartesian & Composite** | [`Plot`], [`Figure`], [`StackedPlot`], [`ScatterHistogram`], [`PlotMatrix`], [`PlotYy`], [`ParetoChart`] | [`PlotState`], [`AxisLink`] | [`PlotLayer`], [`Line`], [`Heatmap`], [`Contours`], [`BoxChart`], [`ViolinPlot`] |
| **3D Cartesian & Volume** | [`Plot3D`], [`ColorBar`] | [`Plot3DState`] | [`Layer3D`], [`Surface3D`], [`VolumeRender3D`], [`PointCloud3D`], [`Isosurface3D`] |
| **Polar & RF / Sky** | [`PolarPlot`], [`SkyPlot`], [`SmithPlot`] | [`PolarState`], [`SkyPlotState`], [`SmithPlotState`] | [`PolarLayer`], [`PolarLine`], [`PolarHistogram`] |
| **Control & Bioinformatics** | [`BodePlot`], [`Clustergram`] | [`BodePlotState`], [`ClustergramState`] | [`NyquistPlot`], [`NicholsPlot`], [`RootLocus`], [`ZPlanePlot`], [`DendrogramPlot`] |
| **Geographic & Globe** | [`GeoPlot`], [`GeoGlobePlot`] | [`GeoPlotState`], [`GeoGlobeState`] | [`GeoLayer`], [`GeoScatter`], [`GeoBubbles`], [`GeoDensity`] |
| **Global Styling & Export** | [`set_plot_defaults`], `PlotSnapshot`, `register_wgpu` | [`PlotDefaults`], [`PlotCache`] | See [`guide`] |

</div>

<div class="cp-lang-zh">

# egui-customplot 概览

> **使用手册**：请阅读 **[`guide`]（完整使用指南与全图种手册）**。点击页面右上角或左侧栏顶部的 **EN | 简体中文** 按钮可在全站无缝切换文档语言。

面向 [`egui`] 的原生嵌入式科学计算与工程绘图库：提供对标 MATLAB 的全系二维、三维、统计、控制、射频、地理与现代信息图表，坚持宿主持有不可变样本、调用方显式持有视图状态、兼容 `egui` 多遍布局且绝不劫持窗口与渲染上下文的架构设计。

## 核心设计原则

1. **宿主持有样本（`Arc` 按版本共享）**：原始数据完全由宿主（[`PlotData`]、[`GridData`]、[`Surface3DData`]、[`ScalarVolume3D`]、[`PointCloud3DData`]）持有；[`PlotState`] 与 [`Plot3DState`] 仅保存视图范围与交互状态，不长期滞留样本内存或 GPU 纹理。
2. **显式状态与多遍布局幂等**：每帧显式传入 `&mut PlotState` / `&mut Plot3DState`；缩放拖拽、多图联动（[`AxisLink`]）以及流式快照追加在 `egui::Context::request_discard` 触发重复布局时自动去重。
3. **不自建窗口与图形设备**：所有控件均直接绘制到调用方的 `&mut egui::Ui`，绝不另建独立窗口、事件循环或私占 `wgpu::Device`。
4. **三级性能体系（`L1` / `L2` / `L3`）**：
   - **L1 智能默认**：[`PlotData::new`] 对单调非降 X 大序列（`>= AUTO_INDEX_MIN_POINTS`）自动建索引；[`Line`] / [`Scatter`] 默认启用自适应 LOD（[`LineSampling::Auto`]、[`MarkerSampling::Auto`]）；[`PlotState`] 内置 2 MiB 有界几何自动缓存。
   - **L2 完全可控**：支持显式指定 [`LineSampling`]、[`MarkerSampling`]、[`RenderBackend`]、[`PlotCache`]、[`Plot::pan_limits`]、对数/反转/时间/双 Y 坐标轴、上下文全局配置 [`PlotDefaults`] 与自包含矢量/位图导出（`PlotSnapshot`）。
   - **L3 千万点流与 GPU**：零拷贝 [`IndexedStream`]（[`StreamLine`]）、`O(1)` 切片 [`UniformWaveform`]（[`WaveformLine`]），以及基于宿主设备复用的可选 `wgpu` Compute + Instancing 管线（`register_wgpu`）。

## Cargo 特性开关

| 特性 | 默认启用 | 说明 |
| :--- | :---: | :--- |
| `default` | 是 | 核心 CPU + 矢量网格绘制全图种（[`Plot`]、[`Plot3D`]、[`PolarPlot`]、[`GeoPlot`] 等） |
| `export` | 否 | 启用 `PlotSnapshot`、`ExportOptions`、`CaptureLimits` 自包含矢量 SVG 与 PNG 离线导出 |
| `wgpu` | 否 | 启用 `register_wgpu` / `WgpuPlotConfig` 千万点级 GPU 加速与实例化渲染管线 |

## 快速上手示例

### 最小二维折线图

```rust
use egui_customplot::{Line, Plot, PlotData, PlotState};

let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
let mut state = PlotState::default(); // 由宿主应用持久持有
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
Plot::new("signal")
    .allow_scroll_zoom(true)
    .show(ui, &mut state, &[Line::new(&data)]);
# }).drop_without_applying_deltas();
```

### 实时流快照、索引与宿主缓存

```rust
use egui_customplot::{Line, LineSampling, PlotCache, PlotData, PlotStream};

let mut stream = PlotStream::new(4096, 4096 * 16)?;
stream.append_once(1, &[[1.0, 2.0], [2.0, 3.0]])?;
let mut data: PlotData = stream.snapshot(); // 显式 O(N) 发布快照
data.build_line_index(1024 * 1024)?;
let line = Line::new(&data).sampling(LineSampling::MinMax);
let mut cache = PlotCache::new(8 * 1024 * 1024); // 由宿主持有缓存
// Plot::new("signal").show_cached(ui, state, &[line], &mut cache);
# let _ = (line, &mut cache);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 三维曲面与 Z-Buffer 深度遮挡

```rust
use egui_customplot::{DepthMode, Layer3D, Plot3D, Plot3DState, Surface3D, Surface3DData};

let grid = [-1.0, 0.0, 1.0];
let z = [0.2, 0.6, 0.2, 0.6, 1.0, 0.6, 0.2, 0.6, 0.2];
let surf = Surface3DData::from_grid(&grid, &grid, &z)?;
let mut state = Plot3DState::default();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
Plot3D::new("surface")
    .depth_mode(DepthMode::ZBuffer)
    .allow_scroll_zoom(true)
    .show(ui, &mut state, &[Layer3D::from(Surface3D::new(&surf))]);
# }).drop_without_applying_deltas();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 七域架构与核心入口总览

库内源码严格按 `axis`、`data`、`interaction`、`layout`、`model`、`render`、`widget` 七域及 `diagnostics` 划分职责边界：

| 坐标系 / 职责 | 核心控件 | 状态与联动 | 常用图层 / 专题 |
| :--- | :--- | :--- | :--- |
| **二维笛卡尔与复合布局** | [`Plot`]、[`Figure`]、[`StackedPlot`]、[`ScatterHistogram`]、[`PlotMatrix`]、[`PlotYy`]、[`ParetoChart`] | [`PlotState`]、[`AxisLink`] | [`PlotLayer`]、[`Line`]、[`Heatmap`]、[`Contours`]、[`BoxChart`]、[`ViolinPlot`] |
| **三维笛卡尔与体渲染** | [`Plot3D`]、[`ColorBar`] | [`Plot3DState`] | [`Layer3D`]、[`Surface3D`]、[`VolumeRender3D`]、[`PointCloud3D`]、[`Isosurface3D`] |
| **极坐标 / 星空 / 射频** | [`PolarPlot`]、[`SkyPlot`]、[`SmithPlot`] | [`PolarState`]、[`SkyPlotState`]、[`SmithPlotState`] | [`PolarLayer`]、[`PolarLine`]、[`PolarHistogram`] |
| **控制系统与生物信息** | [`BodePlot`]、[`Clustergram`] | [`BodePlotState`]、[`ClustergramState`] | [`NyquistPlot`]、[`NicholsPlot`]、[`RootLocus`]、[`ZPlanePlot`]、[`DendrogramPlot`] |
| **离线地图与地球仪** | [`GeoPlot`]、[`GeoGlobePlot`] | [`GeoPlotState`]、[`GeoGlobeState`] | [`GeoLayer`]、[`GeoScatter`]、[`GeoBubbles`]、[`GeoDensity`] |
| **全局主题、缓存与导出** | [`set_plot_defaults`]、`PlotSnapshot`、`register_wgpu` | [`PlotDefaults`]、[`PlotCache`] | 详见 [`guide`] 完整使用手册 |

</div>
