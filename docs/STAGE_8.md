# 阶段 8：二维类型补全切片

日期：2026-11-15。**阶梯线基础切片验证通过，阶段 8 与对应 MATLAB 类型的完整验收未完成。** 133 项对照全部保留；本页不把可绘制等同于统计 / 几何计算或完整交付。

## 阶梯线（ML-003）

```rust
use egui_customplot::{Line, LineInterpolation, PlotData};
let data = PlotData::new(vec![[0.0, 1.0], [0.7, 3.0], [2.0, -1.0]]);
let steps = Line::new(&data).interpolation(LineInterpolation::StepAfter);
// Plot::new("steps").show(ui, &mut state, &[steps]);
```

- 默认仍为 `Linear`；`StepAfter` 保持 y[i] 到 x[i+1] 后跳变（MATLAB-style stairs）；`StepBefore` 在 x[i] 先跳变再保持。两者从首样本开始、在末样本终止，不添加首尾平台，不要求均匀 X，也不自动排序。
- 在选择原始样本后、视区裁剪前展开拐点；同一原始点对两端必须都有限且满足坐标域。NaN / Inf / 非正 log 点使整对断开，不生成悬空平台。对数 / 反向 / 右 Y 轴使用同一坐标语义。
- `Exact` 是参考几何；`Visible` 需要显式单调 X 索引，保留边界邻居与重复 X；没有索引时精确回退并诊断。`MinMax` 先近似选点再展开，因此**可能改变跳变时刻和保持时长**，不是精确阶梯简化算法。
- markers、自动范围、提示、`Series::data()` 始终使用原始点；合成拐点不可拾取。`PlotCache` 键包含插值模式，但与颜色、宽度和虚线样式独立；缓存仍不持有样本。
- `visited_segments` 统计展开前选中的原始点对，命中缓存时为零；`output_points` 包含派生拐点。虚线在展开后的屏幕路径处理，既有工作量安全阀继续适用。
- PNG / 混合 SVG 导出已有显示几何，不恢复 MinMax 丢弃的阶跃。文字、DPI、局部资源限制继续遵循 [阶段 7](STAGE_7.md)。

## 演示与正确性证据

`cargo run -p customplot-demo --release --locked -- --stairs` 展示固定非等距输入：信号 8 点（含 1 个 NaN）、参考 4 点；控制面板可切换三种插值。Live 数据和其他 gallery 同样可使用插值选项。

- `tests/stairs.rs` 6 项：非等距 / 重复 / 乱序 X、端点、空 / 单点、断段、域验证、log / 反向、裁剪、Visible 与 Exact 对照、MinMax 工作量与原始范围。
- `tests/stairs_lifecycle.rs` 4 项：双轴原始拾取与 markers、近似丢弃点仍可拾取、缓存 / 快照 / 样式 / 显隐、虚线、导航多遍幂等。
- `tests/export_stairs.rs` 1 项：解析真实 SVG 路径，PNG 解码和无外部字体 resvg 像素对照，检查平台有墨迹、对角线无墨迹。
- demo 增加固定预设、多遍不重建样本、流式不重复追加，四个 gallery 均覆盖真实 PNG / SVG 编码。路径断言只捕获图表，不把 egui 控件三角形误算为数据路径。
- 本轮统一 all-features **173 项通过**（库单元 32、集成 129、demo 10、rustdoc 2）；fmt、无默认 feature 库检查、严格 all-targets / all-features Clippy、Release demo 构建通过。
- [Windows 四预设生命周期记录](benchmarks/2026-11-15-stage8-stairs-windows-gallery-smoke.txt)：SHA256 `CF1ADE6832148F7F58FDE7CFD10D2744EF3DF0C76105650518A5359201D39819`，均响应、正常关闭且退出码 0。不是人工视觉、GPU、FPS 或长期资源验收。

## 性能与未完成项

`data_performance` 新增 `CUSTOMPLOT_BENCH_INTERPOLATION=linear|after|before`（默认 linear，其他值显式失败）。三档数据与既有六场景保持相同测量口径，stream 场景仍只测发布与索引，不绘制。

StepAfter 正式 30 秒 / 场景 CPU 记录正在运行，**结果尚未验收，不作为性能结论**；完成后独立登记环境、原始 CSV、stderr 与退出码，不覆盖旧记录。

未完成：固定硬件门槛、人工及跨平台显示、GPU / 整应用资源、精确阶梯 LOD、完整 MATLAB stairs 属性与布局兼容。阶段 8 其余统计、多变量、函数、极坐标、几何、网络类型继续推进；阶段 4 / 6 / 7 遗留验收也不因本切片消失。
