# 阶段 12：通用工程底座（`EXT-01..05`）

日期：2026-11-17。依据 [扩展规划](EXPANSION_PLAN.md)，Stage 12 在不改变 [MATLAB 133 项冻结基线](MATLAB_PLOT_CHECKLIST.md) 的前提下，补齐高频科学工程底座能力 `EXT-01..05`：原生日期时间/时长坐标轴、11 种科学色谱、15 种实/空心标记、多遍幂等框选缩放与数据刷选、以及横向分布图（`BoxChart` / `ViolinPlot` / `Raincloud`）。

## 1. 实现与架构落位

| 扩展编号 | 能力范围 | 新增/扩展核心类型与方法 | 源码域落位 |
| --- | --- | --- | --- |
| `EXT-01` | 日期时间与时长轴 | `TimeAxisMode`、`CivilDateTime`、`civil_to_unix_secs`、`unix_secs_to_civil`、`format_duration_hms`、`Axis::{datetime, datetime_with_offset, duration, time_mode}` | [axis/time.rs](../crates/egui-customplot/src/axis/time.rs)、[axis/ticks.rs](../crates/egui-customplot/src/axis/ticks.rs) |
| `EXT-02` | 11 种 MATLAB / 科学色谱 | `ColorMap::{Viridis, Parula, Turbo, Inferno, Plasma, Magma, Cividis, Hot, Coolwarm, Gray, BlueRed}` 及 `ColorMap::ALL`（严格保留原 3 种 5 档 sRGB 采样像素） | [model/color_scale.rs](../crates/egui-customplot/src/model/color_scale.rs) |
| `EXT-03` | 空心与多角标记 | `MarkerShape::{Asterisk, Pentagram, Hexagram, CircleHollow, SquareHollow, DiamondHollow, TriangleUpHollow, TriangleDownHollow}`、`MarkerStyle::{new, hollow}`、真中空 CPU 四环网格与 `scatter.wgsl` SDF | [model/marker.rs](../crates/egui-customplot/src/model/marker.rs)、[render/markers.rs](../crates/egui-customplot/src/render/markers.rs)、[render/markers_lod.rs](../crates/egui-customplot/src/render/markers_lod.rs)、[scatter.wgsl](../crates/egui-customplot/src/render/wgpu/scatter.wgsl) |
| `EXT-04` | 框选缩放与数据刷选 | `Plot::{allow_box_zoom, allow_brush}`、`PlotState::{brushed_bounds, set_brushed_bounds, clear_brush}`、`PlotResponse::{brushed_bounds, brush_finished}`、`PlotData::brushed_indices`、`SelectionCapture` 跨 `request_discard` 帧锁存及右 Y 同步 | [interaction/selection.rs](../crates/egui-customplot/src/interaction/selection.rs)、[interaction/navigation.rs](../crates/egui-customplot/src/interaction/navigation.rs)、[interaction/secondary.rs](../crates/egui-customplot/src/interaction/secondary.rs)、[render/overlay.rs](../crates/egui-customplot/src/render/overlay.rs)、[data/points.rs](../crates/egui-customplot/src/data/points.rs) |
| `EXT-05` | 横向分布图 | `BoxChart` / `ViolinPlot` / `Raincloud` 支持 `.orientation(BarOrientation::Horizontal)`、`bounds_oriented` 边界缓存、双模式绘制、点/区域拾取及 `region/stage8c.rs` 环命中坐标修复 | [data/distribution/mod.rs](../crates/egui-customplot/src/data/distribution/mod.rs)、[model/distribution.rs](../crates/egui-customplot/src/model/distribution.rs)、[render/distribution/mod.rs](../crates/egui-customplot/src/render/distribution/mod.rs)、[render/distribution/box_chart.rs](../crates/egui-customplot/src/render/distribution/box_chart.rs) |

## 2. 演示程序与多语言更新

- [controls.rs](../examples/demo/src/controls.rs) 与 [charts.rs](../examples/demo/src/charts.rs)：主视图默认启用 `.allow_box_zoom(true).allow_brush(true)`，新增 X 轴数值/日期时间/时长格式切换、全 15 种标记中/英/日选择器、刷选命中点数显示及清除按钮。
- [scalar.rs](../examples/demo/src/scalar.rs) 与 [contours.rs](../examples/demo/src/contours.rs)：色谱下拉框接入全部 11 种 `ColorMap::ALL`。
- [distributions.rs](../examples/demo/src/distributions.rs)：新增横向分布图开关，切换 `BoxChart` / `ViolinPlot` / `Raincloud` 为 `BarOrientation::Horizontal` 并同步类目至 Y 轴。

## 3. 验证结果与诚实边界

- **专项测试**：[stage12_foundation.rs](../crates/egui-customplot/tests/stage12_foundation.rs) 5 项全通过（覆盖闰年/毫秒进位/UTC+8/多尺度刻度、11 色谱采样及原 3 色谱逐字节一致性、15 标记 Exact/PixelGrid LOD、单/双遍 `request_discard` 框选与 Alt+刷选/Esc 清除及右 Y 同步、横向分布图 `Log10` 边界及非单位坐标点/区域拾取）；[export_stage12.rs](../crates/egui-customplot/tests/export_stage12.rs) 通过（含 `CircleHollow` 导出 PNG 中心镂空像素实测与 `roxmltree` 无 `<text` SVG 校验）；[architecture.rs](../crates/egui-customplot/tests/architecture.rs) 与 [checklist.rs](../crates/egui-customplot/tests/checklist.rs) 通过。
- **保留边界**：时区采用显式固定秒偏移（无外部 TZ 数据库时区夏令时规则）；`BarOrientation::Horizontal` 覆盖分布图（`BoxChart`/`ViolinPlot`/`Raincloud`），`SwarmChart` 暂保持纵向。
