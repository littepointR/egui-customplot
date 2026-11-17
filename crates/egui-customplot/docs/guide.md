<div class="cp-lang-en">

# Complete User Guide & Plot Catalog

> Back to [`Crate Overview`](crate). Switch **EN | 简体中文** at the top-right or sidebar to view this guide in Simplified Chinese.

`egui-customplot` is an embeddable scientific and engineering visualization library built directly on [`egui`]. It covers MATLAB-class 2D, 3D, statistical, control-system, RF, geographic, and modern diagram families while keeping host data ownership, explicit view state, and multi-pass layout predictability first-class.

---

## 1. Architecture & Ownership Contract

1. **Host owns raw samples**: Retain [`PlotData`], [`GridData`], [`ContourData`], [`Surface3DData`], [`ScalarVolume3D`], or [`PointCloud3DData`] in your application model and pass cheap borrowed layers per frame.
2. **Caller owns interaction state**: Keep one [`PlotState`] (or [`Plot3DState`], [`PolarState`], [`GeoPlotState`], [`GeoGlobeState`], [`SkyPlotState`], [`SmithPlotState`], [`BodePlotState`], [`ClustergramState`]) per widget instance.
3. **egui multi-pass idempotency**: During discarded `request_discard` layout passes, interactive navigation, [`AxisLink`] synchronization, and [`PlotStream`] appends deduplicate per frame so pan/zoom/animation never double-step.
4. **No side-effect hijacking**: No hidden global `egui::Style` mutation, background thread spawning inside the core library, or network requests (including offline [`GeoPlot`] / [`GeoGlobePlot`]).

### Minimal Interactive 2D Plot Example

```rust
use egui_customplot::{Axis, Line, Plot, PlotData, PlotState};

let samples = PlotData::new(vec![[0.0, 0.0], [1.0, 2.0], [2.0, 1.0]]);
let mut state = PlotState::default();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
let resp = Plot::new("my_plot")
    .height(260.0)
    .allow_scroll_zoom(true)
    .allow_box_zoom(true)
    .x_axis(Axis::new().label("Time (s)"))
    .y_axis(Axis::new().label("Voltage (V)"))
    .show(ui, &mut state, &[Line::new(&samples).name("Ch1")]);
assert!(resp.transform.rect().width() >= 0.0);
# }).drop_without_applying_deltas();
```

---

## 2. 2D Cartesian & Scientific Plot Families

### Line, Scatter, Stems, Bars & Error Bars
- **XY series**: [`Line`] (linear / [`LineInterpolation::StepAfter`] `stairs` / [`LineInterpolation::StepBefore`], dashed/dotted [`LinePattern`]), [`Scatter`] ([`MarkerShape`]), [`Series`], [`LineSampling`] (`Auto`, `Exact`, `Visible`, `MinMax`), [`MarkerSampling`] (`Auto`, `Exact`, `PixelGrid`).
- **Bars, Stems, Error Bars & Waterfall**: [`Bars`] with [`BarData`] (grouped, stacked, negative-stacked, horizontal/vertical), [`Stems`], [`ErrorBars`] with [`ErrorBarData`], [`Waterfall2D`] with [`Plot::waterfall_axes`].

### Areas, Polygons, Patches & Geometric Tessellation
- **Areas & Polygons**: [`Area`] / [`AreaBandData`] (`NaN` splits bands cleanly), [`Fill`] / [`PolygonData`], [`PolyShape`] / [`PolyShapeData`] (multi-region polygons with holes), [`Patch`] / [`PatchData`] (flat or Gouraud vertex coloring via [`PatchColoring`]), [`Rectangles`] / [`DataRect`].
- **2D Computational Geometry**: [`TriPlot`] / [`TriMesh2D`], [`Voronoi`] / [`VoronoiData`], [`AlphaShape`] / [`AlphaShapeData`], [`Spy`] / [`SpyData`] + [`Plot::spy_axes`].

### Scalar Fields, Heatmaps, Contours & Vector Flows
- **Grids & Images**: [`Heatmap`] / [`GridData`], [`Pcolor`] ([`PcolorShading`]), [`CurvilinearPcolor`] / [`CurvilinearField2D`], [`PlotImage`] / [`ImageData`] + [`Plot::imshow_axes`], paired with standalone [`ColorBar`] sharing the same [`ColorScale`] and [`ColorMap`].
- **Contours**: [`Contours`] / [`ContourData`] (with high-contrast labeled contour pills) and [`FilledContours`] / [`FilledContourData`].
- **Vector Flows & Function Curves**: [`Quiver`], [`Feather`], [`Streamlines`], [`FunctionPlot`] / [`FunctionCurveData`] (`fplot`), [`ImplicitCurve`] / [`ImplicitCurveData`] (`fimplicit`), [`Comet`] & [`AnimatedLine`].

---

## 3. Statistical, Multivariate, SPC & Financial Charts

- **Distributions**: [`HistogramData`], [`Histogram2`] / [`Histogram2Data`], [`BinScatter`] / [`BinScatterData`], [`BoxChart`], [`ViolinPlot`] / [`ViolinData`], [`Raincloud`] / [`RaincloudData`], [`SwarmChart`] / [`SwarmData`], [`EcdfPlot`] / [`EcdfData`], [`ProbabilityPlot`] / [`ProbabilityPlotData`] (`Probit`, `Weibull`, QQ).
- **Categoricals & Matrix Analytics**: [`PieChart`] (`pie`, `donut`), [`WordCloud`], [`BubbleChart`], [`BubbleCloud`], [`BubbleLegend`], [`CategoricalHeatmap`], [`ConfusionChart`] + [`Plot::confusion_axes`], [`ParallelPlot`], [`Biplot`] / [`BiplotData`], [`GraphPlot`] / [`GraphData`] / [`TreeData`] / [`EliminationTree`] (`etreeplot`), [`DendrogramPlot`], [`Clustergram`].
- **SPC & Financial**: [`ControlChart`] / [`ControlChartData`], [`Candlestick`] & [`OhlcBars`] backed by [`OhlcData`].
- **Multi-Panel Composite Layouts**: [`Figure`] (`256`-cell grid via [`FigureCell`]), [`StackedPlot`], [`ScatterHistogram`], [`PlotMatrix`], [`PlotYy`], [`ParetoChart`].

---

## 4. 3D Cartesian, Volume Raycasting & Point Clouds

[`Plot3D`] combines [`Camera3D`] (azimuth/elevation orbit, pan, `0.1..=20.0` zoom, [`Projection3D::Orthographic`] vs [`Projection3D::Perspective`]) with either fast [`DepthMode::SortedPainter`] or exact pixel [`DepthMode::ZBuffer`] (including static-frame `ZBuffer` RLE span caching in [`Plot3DState`]).

- **3D Series & Surfaces**: [`Line3D`], [`Scatter3D`], [`Bubble3D`], [`Swarm3D`], [`Comet3D`], [`Bar3D`], [`Stem3D`], [`Pie3D`], [`Quiver3D`], [`Fill3D`], [`Surface3D`], [`MeshSurface3D`], [`Contour3D`], [`Ribbon3D`], [`TriSurf3D`], [`Patch3D`], [`AlphaShape3D`], [`Graph3D`].
- **Volumetric, Implicit & 3D Flow**: [`VolumeSlice3D`], [`Isosurface3D`], [`Isocaps3D`], [`TetraMesh3D`], [`Streamline3D`], [`StreamParticles3D`], [`StreamRibbon3D`], [`StreamTube3D`], [`ConePlot3D`], [`ConstantPlane3D`], [`Text3D`].
- **CPU Volume Raycasting (`EXT-23`)**: [`VolumeRender3D`] + [`ScalarVolume3D`] + [`VolumeTransferFunction`] ([`VolumeRenderMode::DirectAlpha`], `MaximumIntensity`, `IsosurfaceRay`), occluding cleanly against opaque 3D primitives in [`DepthMode::ZBuffer`].
- **Octree Point Clouds (`EXT-24`)**: [`PointCloud3D`] + [`PointCloud3DData`] with frustum + screen-size octree LOD and branch-and-bound picking.

```rust
use egui_customplot::{
    Bounds3D, ColorMap, DepthMode, Layer3D, Plot3D, Plot3DState, PointCloud3D,
    PointCloud3DData, Projection3D, ScalarVolume3D, VolumeRender3D, VolumeTransferFunction,
};

let bounds = Bounds3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])?;
let vol = ScalarVolume3D::from_function(bounds, [12, 12, 12], 5_000, |x, y, z| {
    (-(x * x + y * y + z * z)).exp()
})?;
let cloud = PointCloud3DData::from_xyz(vec![[0.0, 0.0, 0.0], [0.5, 0.4, -0.3]])?;
let tf = VolumeTransferFunction::ramp(ColorMap::Viridis, 0.05, 0.85);
let mut state = Plot3DState::default();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
Plot3D::new("vol_and_pc")
    .projection(Projection3D::perspective_default())
    .depth_mode(DepthMode::ZBuffer)
    .show(
        ui,
        &mut state,
        &[
            Layer3D::from(PointCloud3D::new(&cloud).max_points(4096)),
            Layer3D::from(VolumeRender3D::new(&vol).transfer(tf).sample_steps(24)),
        ],
    );
# }).drop_without_applying_deltas();
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

## 5. Polar, Sky, Smith, Geographic, RF/Control & Modern Diagrams

- **Polar & Aerospace**: [`PolarPlot`] (`polarplot`, `polarscatter`, `polarbubblechart`, `polarhistogram`, `compassplot`, [`PolarRegionAnnotation`]), [`SkyPlot`] (`skyplot` azimuth-elevation sky tracks & elevation mask).
- **RF, Control & DSP**: [`SmithPlot`] (impedance/admittance/immittance grids), [`ZPlanePlot`], [`BodePlot`] (gain/phase margin annotations), [`NyquistPlot`], [`NicholsPlot`], [`RootLocus`], [`CwtPlot`], [`SpectrogramData`], [`PspectrumData`], [`fft_radix2`] / [`fft_arbitrary`].
- **Offline 2D Map & 3D WGS84 Globe**: [`GeoPlot`] (`geoplot`, `geoscatter`, `geobubble`, `geodensityplot` with [`GeoOfflinePolicy`] offline tile contract) and [`GeoGlobePlot`] / [`GeoGlobeData`] (WGS84 globe with coastline rings, graticule, and altitude-extruded tracks).
- **Modern Diagrams (`EXT-26`)**: [`RadarChart`], [`SankeyChart`], [`TreemapChart`], [`SunburstChart`], [`GanttChart`] + [`Plot::gantt_axes`].

---

## 6. Customization, 10M Streaming & Offline Export

### Global Defaults, Pan Limits, Time Axes & Annotations

```rust
use egui_customplot::{
    Annotation, AnnotationCoord, Axis, AxisScale, Line, Plot, PlotData, PlotDefaults, PlotState,
    PlotTheme, set_plot_defaults,
};
use std::sync::Arc;

let data = PlotData::new(vec![[1.0, 1.0], [10.0, 4.0], [100.0, 9.0]]);
let mut state = PlotState::default();
let annots: Arc<[Annotation]> = vec![
    Annotation::text([10.0, 4.0], "knee"),
    Annotation::text_arrow(AnnotationCoord::Normalized, [0.6, 0.85], [0.5, 0.5], "trend"),
]
.into();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
set_plot_defaults(
    ui.ctx(),
    Some(PlotDefaults {
        allow_scroll_zoom: Some(true),
        theme: Some(Arc::new(PlotTheme::scientific_light())),
        ..Default::default()
    }),
);
Plot::new("custom_demo")
    .x_axis(Axis::new().label("f (Hz)").scale(AxisScale::Log10))
    .y_axis(Axis::new().label("Gain"))
    .pan_limits(Some([0.5, 200.0]), Some([0.0, 12.0]))
    .annotations(annots.clone())
    .show(ui, &mut state, &[Line::new(&data).name("response")]);
# }).drop_without_applying_deltas();
```

- **Time & Duration Ticks**: [`Axis::datetime`], [`Axis::datetime_with_offset`], [`Axis::duration`].
- **Linking Multiple Plots**: [`AxisLink`] synchronizes X/Y ranges across arbitrary [`PlotState`] instances once per pass without snap-back jitter.
- **10M High-Frequency Streaming & `wgpu`**: Use [`IndexedStream`] ([`StreamLine`]) or [`UniformWaveform`] ([`WaveformLine`]); with `feature = "wgpu"`, call `register_wgpu` once at app startup to accelerate large scatter/waveform pipelines on the host GPU context.
- **Self-Contained Export (`feature = "export"`)**: Wrap widgets inside `PlotSnapshot::capture` and write deterministic vector SVG or raster PNG via `ExportOptions` (bounded by `CaptureLimits`).

</div>

<div class="cp-lang-zh">

# 简体中文完整使用指南与全图种手册

> 返回 [`Crate 总览`](crate)。点击页面右上角或左侧栏顶部的 **EN | 简体中文** 按钮可切换英文版文档。

`egui-customplot` 是面向 [`egui`] 的原生嵌入式科学计算与工程可视化库：完整覆盖对标 MATLAB 的二维、三维、统计、控制、射频、地理与现代信息图谱，坚持**宿主持有数据、调用方显式持有视图状态、`egui` 多遍布局严格幂等**的设计原则。

---

## 1. 核心架构与所有权约定

1. **原始样本完全由宿主持有**：将 [`PlotData`]、[`GridData`]、[`ContourData`]、[`Surface3DData`]、[`ScalarVolume3D`]、[`PointCloud3DData`] 等不可变/按版本共享的数据对象留在业务模型中，每帧只传入轻量的借用图层描述符。
2. **交互视图状态由调用方显式持有**：为每个控件维护一个对应的状态实例（[`PlotState`]、[`Plot3DState`]、[`PolarState`]、[`GeoPlotState`]、[`GeoGlobeState`]、[`SkyPlotState`]、[`SmithPlotState`]、[`BodePlotState`]、[`ClustergramState`]）。状态对象不保留原始样本切片或 GPU 纹理。
3. **`egui` 多遍布局幂等性**：当容器触发 `request_discard` 多次布局时，鼠标/滚轮导航、多图 [`AxisLink`] 同步以及 [`PlotStream`] 数据追加都会基于帧号/遍序号去重，只提交一次副作用。
4. **绝不越权劫持宿主环境**：不创建独立窗口或事件循环、不修改宿主全局 `egui::Style`、不在核心库内启动隐式后台线程或发起网络请求（含完全离线运行的 [`GeoPlot`] 与 [`GeoGlobePlot`]）。

### 最小二维交互绘图示例

```rust
use egui_customplot::{Axis, Line, Plot, PlotData, PlotState};

let samples = PlotData::new(vec![[0.0, 0.0], [1.0, 2.0], [2.0, 1.0]]);
let mut state = PlotState::default();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
let resp = Plot::new("my_plot")
    .height(260.0)
    .allow_scroll_zoom(true)
    .allow_box_zoom(true)
    .x_axis(Axis::new().label("时间 (s)"))
    .y_axis(Axis::new().label("电压 (V)"))
    .show(ui, &mut state, &[Line::new(&samples).name("通道 1")]);
assert!(resp.transform.rect().width() >= 0.0);
# }).drop_without_applying_deltas();
```

---

## 2. 二维笛卡尔与科学工程图谱

### 折线、散点、火柴杆、柱状与误差棒
- **XY 序列与采样**：[`Line`]（支持线性、[`LineInterpolation::StepAfter`] `stairs` 阶梯图、[`LineInterpolation::StepBefore`] 及 [`LinePattern`] 虚点线）、[`Scatter`]（多种 [`MarkerShape`]）、[`Series`]；可通过 [`LineSampling`]（`Auto` / `Exact` / `Visible` / `MinMax`）与 [`MarkerSampling`]（`Auto` / `Exact` / `PixelGrid`）显式控制降采样策略。
- **柱状、火柴杆、误差棒与瀑布图**：[`Bars`] / [`BarData`]（分组、堆叠、正负堆叠、水平/垂直）、[`Stems`]、[`ErrorBars`] / [`ErrorBarData`]、[`Waterfall2D`]（配 [`Plot::waterfall_axes`]）。

### 面积带、多边形、面片与计算几何
- **区域填充**：[`Area`] / [`AreaBandData`]（支持堆叠与 `NaN` 自动断口）、[`Fill`] / [`PolygonData`]、[`PolyShape`] / [`PolyShapeData`]（支持外环与多内孔洞）、[`Patch`] / [`PatchData`]（支持按面平涂与 [`PatchColoring`] Gouraud 顶点插值）、[`Rectangles`] / [`DataRect`]（圆角矩形与椭圆）。
- **二维几何与稀疏矩阵**：[`TriPlot`] / [`TriMesh2D`]（Delaunay 三角剖分）、[`Voronoi`] / [`VoronoiData`]（泰森多边形）、[`AlphaShape`] / [`AlphaShapeData`]（α-形状边界）、[`Spy`] / [`SpyData`] + [`Plot::spy_axes`]。

### 标量场、热力图、等高线与流场动画
- **网格与图像**：[`Heatmap`] / [`GridData`]（`NaN` 显示为透明空洞且保留悬停网格拾取）、[`Pcolor`]（[`PcolorShading`]）、[`CurvilinearPcolor`] / [`CurvilinearField2D`]（曲率结构化网格）、[`PlotImage`] / [`ImageData`] + [`Plot::imshow_axes`]，可搭配独立 [`ColorBar`] 共用同一 [`ColorScale`] 与 [`ColorMap`]。
- **等高线**：[`Contours`] / [`ContourData`]（带按胶囊底色自适应亮暗对比度的数值标签）与 [`FilledContours`] / [`FilledContourData`]。
- **流场与自适应函数曲线**：[`Quiver`]、[`Feather`]、[`Streamlines`]、[`FunctionPlot`] / [`FunctionCurveData`]（对应 `fplot` 振荡自动加密与极点断裂）、[`ImplicitCurve`] / [`ImplicitCurveData`]（对应 `fimplicit`）、[`Comet`] 与 [`AnimatedLine`]。

---

## 3. 统计分布、多变量分析、SPC 与金融图表

- **分布与离散统计**：[`HistogramData`]、[`Histogram2`] / [`Histogram2Data`]、[`BinScatter`] / [`BinScatterData`]、[`BoxChart`]（箱线图）、[`ViolinPlot`] / [`ViolinData`]（小提琴图）、[`Raincloud`] / [`RaincloudData`]（云雨图）、[`SwarmChart`] / [`SwarmData`]（蜂群散点）、[`EcdfPlot`]（经验分布）、[`ProbabilityPlot`]（`Probit` / `Weibull` / 正态分位图）。
- **分类、词云与关系网络**：[`PieChart`]（饼图/圆环图）、[`WordCloud`]、[`BubbleChart`]、[`BubbleCloud`] 与 [`BubbleLegend`]、[`CategoricalHeatmap`]、[`ConfusionChart`] + [`Plot::confusion_axes`]、[`ParallelPlot`]、[`Biplot`]（PCA 双标图）、[`GraphPlot`] / [`GraphData`] / [`TreeData`] / [`EliminationTree`]（消元树）、[`DendrogramPlot`] 与 [`Clustergram`]（层次聚类热图）。
- **工业 SPC 与金融 K 线**：[`ControlChart`] / [`ControlChartData`]、[`Candlestick`] 与 [`OhlcBars`]（基于 [`OhlcData`]）。
- **复合多面板布局**：[`Figure`]（最多 256 格网格，支持子图级 `EX-027` 注解）、[`StackedPlot`]、[`ScatterHistogram`]、[`PlotMatrix`]、[`PlotYy`] 与 [`ParetoChart`]。

---

## 4. 三维笛卡尔绘图、光线投射体渲染与八叉树点云

[`Plot3D`] 统一管理 [`Camera3D`]（方位角/仰角旋转、拖拽平移、`0.1..=20.0` 倍缩放、[`Projection3D::Orthographic`] 正交与 [`Projection3D::Perspective`] 透视投影）并支持快速画家排序 [`DepthMode::SortedPainter`] 与逐像素精确遮挡 [`DepthMode::ZBuffer`]（静态/悬停帧自动走 [`Plot3DState`] RLE span 缓存加速）。

- **三维序列与曲面几何**：[`Line3D`]、[`Scatter3D`]、[`Bubble3D`]、[`Swarm3D`]、[`Comet3D`]、[`Bar3D`]、[`Stem3D`]、[`Pie3D`]、[`Quiver3D`]、[`Fill3D`]、[`Surface3D`]、[`MeshSurface3D`]、[`Contour3D`]、[`Ribbon3D`]、[`TriSurf3D`]、[`Patch3D`]、[`AlphaShape3D`]、[`Graph3D`]。
- **体切片、等值面与三维流场**：[`VolumeSlice3D`]、[`Isosurface3D`]、[`Isocaps3D`]、[`TetraMesh3D`]、[`Streamline3D`]、[`StreamParticles3D`]、[`StreamRibbon3D`]、[`StreamTube3D`]、[`ConePlot3D`]、[`ConstantPlane3D`]、[`Text3D`]。
- **CPU 体渲染光线投射（`EXT-23`）**：[`VolumeRender3D`] + [`ScalarVolume3D`] + [`VolumeTransferFunction`]（支持 [`VolumeRenderMode::DirectAlpha`] / `MaximumIntensity` / `IsosurfaceRay`），在 [`DepthMode::ZBuffer`] 下与不透明网格实现精确前后遮挡混合。
- **高密度三维点云（`EXT-24`）**：[`PointCloud3D`] + [`PointCloud3DData`] 内置八叉树视锥剔除、屏幕自适应 LOD 与分支定界最近点拾取。

```rust
use egui_customplot::{
    Bounds3D, ColorMap, DepthMode, Layer3D, Plot3D, Plot3DState, PointCloud3D,
    PointCloud3DData, Projection3D, ScalarVolume3D, VolumeRender3D, VolumeTransferFunction,
};

let bounds = Bounds3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0])?;
let vol = ScalarVolume3D::from_function(bounds, [12, 12, 12], 5_000, |x, y, z| {
    (-(x * x + y * y + z * z)).exp()
})?;
let cloud = PointCloud3DData::from_xyz(vec![[0.0, 0.0, 0.0], [0.5, 0.4, -0.3]])?;
let tf = VolumeTransferFunction::ramp(ColorMap::Viridis, 0.05, 0.85);
let mut state = Plot3DState::default();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
Plot3D::new("vol_and_pc")
    .projection(Projection3D::perspective_default())
    .depth_mode(DepthMode::ZBuffer)
    .show(
        ui,
        &mut state,
        &[
            Layer3D::from(PointCloud3D::new(&cloud).max_points(4096)),
            Layer3D::from(VolumeRender3D::new(&vol).transfer(tf).sample_steps(24)),
        ],
    );
# }).drop_without_applying_deltas();
# Ok::<(), Box<dyn std::error::Error>>(())
```

---

## 5. 极坐标、星空图、射频史密斯圆图、控制系统、地球仪与现代图表

- **极坐标与空天观测**：[`PolarPlot`]（`polarplot` / `polarscatter` / `polarbubblechart` / `polarhistogram` / `compassplot` 及 [`PolarRegionAnnotation`] 扇区高亮）、[`SkyPlot`]（方位角-仰角星空卫星轨迹与仰角掩膜）。
- **射频、自控与信号谱**：[`SmithPlot`]（阻抗/导纳/混合史密斯圆图）、[`ZPlanePlot`]（Z 平面零极点）、[`BodePlot`]（幅相频响应 + 增益/相位裕度标注）、[`NyquistPlot`]、[`NicholsPlot`]、[`RootLocus`]、[`CwtPlot`]（连续小波尺度谱）、[`SpectrogramData`]、[`PspectrumData`] 与 [`fft_radix2`] / [`fft_arbitrary`]。
- **离线二维地图与三维 WGS84 地球仪**：[`GeoPlot`]（完全遵循 [`GeoOfflinePolicy`] 离线瓦片/合成底图契约，不发起网络请求）及 [`GeoGlobePlot`] / [`GeoGlobeData`]（WGS84 正射三维地球仪、海岸线、经纬网与高程挤出轨迹）。
- **现代交互图表（`EXT-26`）**：[`RadarChart`]（雷达图）、[`SankeyChart`]（桑基图）、[`TreemapChart`]（矩形树图）、[`SunburstChart`]（旭日图）、[`GanttChart`] + [`Plot::gantt_axes`]（甘特图）。

---

## 6. 客制化、千万点流式/`wgpu` 加速与离线导出

### 全局默认值、拖拽限位、对数轴与图面注解

```rust
use egui_customplot::{
    Annotation, AnnotationCoord, Axis, AxisScale, Line, Plot, PlotData, PlotDefaults, PlotState,
    PlotTheme, set_plot_defaults,
};
use std::sync::Arc;

let data = PlotData::new(vec![[1.0, 1.0], [10.0, 4.0], [100.0, 9.0]]);
let mut state = PlotState::default();
let annots: Arc<[Annotation]> = vec![
    Annotation::text([10.0, 4.0], "拐点"),
    Annotation::text_arrow(AnnotationCoord::Normalized, [0.6, 0.85], [0.5, 0.5], "趋势"),
]
.into();
# egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
set_plot_defaults(
    ui.ctx(),
    Some(PlotDefaults {
        allow_scroll_zoom: Some(true),
        theme: Some(Arc::new(PlotTheme::scientific_light())),
        ..Default::default()
    }),
);
Plot::new("custom_demo")
    .x_axis(Axis::new().label("频率 (Hz)").scale(AxisScale::Log10))
    .y_axis(Axis::new().label("增益"))
    .pan_limits(Some([0.5, 200.0]), Some([0.0, 12.0]))
    .annotations(annots.clone())
    .show(ui, &mut state, &[Line::new(&data).name("幅频响应")]);
# }).drop_without_applying_deltas();
```

- **时间与持续时长坐标轴**：[`Axis::datetime`]、[`Axis::datetime_with_offset`]、[`Axis::duration`]。
- **多图联动**：[`AxisLink`] 在每遍所有成员渲染完毕后统一同步 X/Y 数据范围，绝不篡改轴向与可见性且避免水平拖拽回弹。
- **千万点实时流与 `wgpu` 加速**：实时示波/高频流场景推荐使用 [`IndexedStream`]（[`StreamLine`]）或零开销切片 [`UniformWaveform`]（[`WaveformLine`]）；启用 `feature = "wgpu"` 后，在宿主初始化阶段调用一次 `register_wgpu` 即可复用宿主 GPU 设备执行大规模点线 Compute + Instancing 管线。
- **自包含离线导出（`feature = "export"`）**：通过 `PlotSnapshot::capture` 捕获已绘制图层（严格受 `CaptureLimits` 约束），再用 `ExportOptions` 导出确定性矢量 SVG 或高清 PNG。

</div>
