# 阶段 16：直接体渲染、八叉树点云、三维地球仪、现代图型与全扩展收敛（`EXT-23..26`）

日期：2026-11-17。依据 [扩展规划](EXPANSION_PLAN.md)，Stage 16 在保持 [MATLAB 133 项冻结基线](MATLAB_PLOT_CHECKLIST.md) 与 [MATLAB_PLOT_BASELINE.json](MATLAB_PLOT_BASELINE.json) 不变的前提下，完成最后 4 项扩展 `EXT-23..26` 与 Stage 12–16 总审计收敛：

## 1. 实现与架构落位

| 扩展编号 | 能力范围 | 新增/扩展核心类型与方法 | 源码域落位 |
| --- | --- | --- | --- |
| `EXT-23` | `volshow` 三维光线投射体绘制 | `VolumeTransferFunction`（32 节点分段线性 RGBA8 LUT、`ramp`/`gaussian_band`/`from_stops`）、`VolumeRenderMode::{DirectAlpha, MaximumIntensity, IsosurfaceRay}`、`VolumeRender3D`、Kay-Kajiya 单位立方体求交 `Camera3D::screen_ray_unit` / `unit_to_data`、Front-to-Back Alpha 合成与软件 `ZBuffer` 遮挡融合及射线峰值拾取 `Hovered3DRegion` | [axis/plot3d/camera.rs](../crates/egui-customplot/src/axis/plot3d/camera.rs)、[data/plot3d/volshow.rs](../crates/egui-customplot/src/data/plot3d/volshow.rs)、[model/plot3d/stage16.rs](../crates/egui-customplot/src/model/plot3d/stage16.rs)、[render/plot3d/volshow.rs](../crates/egui-customplot/src/render/plot3d/volshow.rs)、[interaction/plot3d/stage13_hover.rs](../crates/egui-customplot/src/interaction/plot3d/stage13_hover.rs) |
| `EXT-24` | `pcshow` 海量三维点云 | `PointCloud3DData`（XYZ + 可选 RGB / Intensity + 8 叉树空间索引、`query_lod` 视锥与屏幕尺寸自适应 LOD、`pick_nearest` 屏幕分支定界拾取）、`PointCloud3D`（ZBuffer / Painter 兼容） | [data/plot3d/pointcloud.rs](../crates/egui-customplot/src/data/plot3d/pointcloud.rs)、[model/plot3d/stage16.rs](../crates/egui-customplot/src/model/plot3d/stage16.rs)、[render/plot3d/volshow.rs](../crates/egui-customplot/src/render/plot3d/volshow.rs)、[interaction/plot3d/stage13_hover.rs](../crates/egui-customplot/src/interaction/plot3d/stage13_hover.rs) |
| `EXT-25` | `geoglobe` / `geoplot3` 三维 WGS84 地球仪 | `GeoGlobeTransform`（正射半球投影、球面地平线背面消隐与逆变换）、`GeoGlobeData` / `GlobeTrack3D` / `GlobeMarker3D`（轨迹高程挤出与 `OFFLINE_LAND_RINGS` 零网络海岸线）、`GeoGlobeState`（多遍幂等轨角拖拽与滚轮缩放）、`GeoGlobePlot` / `FigureCell::show_geoglobe` / `HoveredGlobeItem` | [axis/geoglobe.rs](../crates/egui-customplot/src/axis/geoglobe.rs)、[data/geoglobe.rs](../crates/egui-customplot/src/data/geoglobe.rs)、[interaction/geoglobe.rs](../crates/egui-customplot/src/interaction/geoglobe.rs)、[render/geoglobe.rs](../crates/egui-customplot/src/render/geoglobe.rs)、[widget/geoglobe.rs](../crates/egui-customplot/src/widget/geoglobe.rs) |
| `EXT-26` | 现代可视化图型与全扩展总收敛 | `RadarData`/`RadarChart`（多轴极蛛网 + 顶点拾取）、`SankeyData`/`SankeyChart`（最长路分列与流守恒 Bézier 丝带）、`TreemapData`/`TreemapChart`（Bruls–Huizing–van Wijk Squarified `[0, 1]^2` 严格面积守恒）、`SunburstData`/`SunburstChart`（父子角域守恒层环）与 `GanttData`/`GanttChart`/`Plot::gantt_axes`（进度条、里程碑、依赖箭头与区域拾取） | [data/modern/radial_flow.rs](../crates/egui-customplot/src/data/modern/radial_flow.rs)、[data/modern/hier_gantt.rs](../crates/egui-customplot/src/data/modern/hier_gantt.rs)、[data/modern/gantt.rs](../crates/egui-customplot/src/data/modern/gantt.rs)、[model/modern.rs](../crates/egui-customplot/src/model/modern.rs)、[render/modern/](../crates/egui-customplot/src/render/modern/mod.rs)、[interaction/region/stage16.rs](../crates/egui-customplot/src/interaction/region/stage16.rs) |

## 2. 演示程序、多语言与 Windows Release 生命周期

- [stage16.rs](../examples/demo/src/stage16.rs) 与 [stage16_tests.rs](../examples/demo/src/stage16_tests.rs)：新增 `--stage16` 独立专题画廊（2×4 网格 8 个面板：3D 曲面遮挡下体渲染 `Alpha`/`MIP`/`IsoRay`、40k 螺旋 RGB 点云、WGS84 地球仪与 LEO 轨线、雷达图、桑基图、矩形树图、旭日图与甘特图），完整接入中/英/日 UI（23 组预设）。
- Release Windows 窗口生命周期验证（[2026-11-17 Smoke 记录](benchmarks/2026-11-17-stage16-windows-gallery-smoke.txt)）：构建 `target/release/customplot-demo.exe`（SHA256 `0C21E6038783961C642AF30A9597B836A64E60FF718CDD0EFCF0E2555547C3D4`），实测 `default`、`--stage13`、`--stage14`、`--stage15`、`--stage16` 五组窗口均响应正常并退出码 `0`（注：仅代表窗口生命周期烟雾，不代表硬件 FPS 或跨平台视觉验收）。

## 3. CPU 构建与 Tessellation 基准（独立归档，仅测 CPU）

测试文件：[stage16_performance.rs](../crates/egui-customplot/benches/stage16_performance.rs)，原始 CSV：[2026-11-17 Stage 16 CPU 基准](benchmarks/2026-11-17-stage16-windows-cpu.csv)（1280×800 无头 `egui::Context` CPU + tessellation）：

| 场景 | 样本帧数 | CPU Build p50 (ms) | CPU Total p50 (ms) | CPU Total p95 (ms) | CPU Total Max (ms) | Meshes / Vertices |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `volshow_64x64x64_direct_alpha` | 24 | 63.640 | 63.833 | 64.745 | 64.894 | 2 / 21,528 |
| `pcshow_200k_octree_lod_zbuffer` | 2,054 | 0.692 | 0.698 | 0.890 | 1.572 | 2 / 360 |
| `geoglobe_wgs84_coastlines_track` | 32,273 | 0.030 | 0.044 | 0.053 | 0.412 | 1 / 3,526 |
| `modern_suite_5charts_figure` | 21,995 | 0.045 | 0.062 | 0.104 | 0.487 | 10 / 3,004 |

## 4. 专项测试与保留边界

- **专项正确性与全扩展审计**：[stage16_expansion_audit.rs](../crates/egui-customplot/tests/stage16_expansion_audit.rs)（验证 `ext23` 三种体绘模式与射线拾取、`ext24` 20 万点云八叉树有界 LOD 与最近拾取、`ext25` 地球仪背面消隐与两遍拖拽幂等、`ext26` 五类现代图不变量/区域拾取及 `EXT-01..26` 全收敛审计）与 [export_stage16.rs](../crates/egui-customplot/tests/export_stage16.rs)（8 图联合自包含 SVG + 多色 PNG 导出）通过。
- **架构护栏**：[architecture.rs](../crates/egui-customplot/tests/architecture.rs) 确认全部库源码均按七域划分且无文件超过 400 行上限（`gantt.rs` / `hier_gantt.rs` 已按职责分拆）。
- **诚实开放边界**：64³ CPU 直接体光线投射当前约为 ~64 ms（尚无 Compute Shader GPU 体渲染后端）；跨平台硬件实机 GPU 渲染、印刷级矢量排版与 `publish = false` 暂不发布边界继续保留。
