# 阶段 8：二维类型补全切片

日期：2026-11-15。**阶梯线与茎状图基础切片验证通过，阶段 8 与对应 MATLAB 类型的完整验收未完成。** 133 项对照全部保留；本页不把可绘制等同于统计 / 几何计算或完整交付。

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
- [完整本地检查记录](benchmarks/2026-11-15-stage8-stairs-windows-checks.txt) 保留各项测试与构建输出。实现提交 `e4623b4`；此前阶段 4～7 提交 `b01561e` 的 [Windows / Ubuntu CI](https://github.com/littepointR/egui-customplot/actions/runs/43697081327) 已全部成功；该旧 CI 不能代替后加图元的验证。
- [Windows 四预设生命周期记录](benchmarks/2026-11-15-stage8-stairs-windows-gallery-smoke.txt)：SHA256 `CF1ADE6832148F7F58FDE7CFD10D2744EF3DF0C76105650518A5359201D39819`，均响应、正常关闭且退出码 0。不是人工视觉、GPU、FPS 或长期资源验收。

## 阶梯线 CPU 记录

`data_performance` 新增 `CUSTOMPLOT_BENCH_INTERPOLATION=linear|after|before`（默认 linear，其他值显式失败）。三档数据与既有六场景保持相同测量口径，stream 场景仍只测发布与索引，不绘制。

StepAfter 正式记录已完成：三档数据 × 六绘制场景各 30 秒，另加 30 秒 stream 发布，退出码 0、stderr 为空，18 行绘制结果和 stream 行齐全，记录的 index / cache payload 均在其声明的预算内。新 [环境 / binary SHA256](benchmarks/2026-11-15-stage8-stairs-windows-cpu-environment.txt)、[原始 CSV](benchmarks/2026-11-15-stage8-stairs-windows-cpu.csv)、[stderr](benchmarks/2026-11-15-stage8-stairs-windows-cpu.stderr.txt) 独立保存，不覆盖历史。

以下为 CPU UI 构建 + tessellation 的合计 p95（ms）；各自分项 p95 不能直接相加。Exact / Visible 在 0.1% X 视区；MinMax 全区和缓存场景为显式近似，不能与精确全数据几何等同。

| 原始点数 | Exact 窄区 | Visible 同窄区 | MinMax 全区 | MinMax 平移 | 缓存 MinMax 全区 | 索引 hover + MinMax |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 100,000 | 1.0631 | 0.0194 | 0.8286 | 0.9420 | 0.2617 | 0.9994 |
| 1,000,000 | 27.3732 | 0.1020 | 1.3182 | 1.1383 | 0.3798 | 1.9835 |
| 10,000,000 | 93.4107 | 0.9856 | 3.1157 | 3.0118 | 0.4496 | 5.4708 |

条件：i9-14900KF、Windows 10.0.26300、Rust 1.100.0、Release bench、1920×1080 逻辑绘图区、DPI=1、无 markers，机器同时进行开发 / 构建，非隔离发布门槛。在千万点全区 MinMax 最后样本中选择 4,864 点、访问 12,040 树节点和扫描 121,021 点；索引 hover 仍访问 135,417 个原始候选点，并非常数时间。stream 行的追加 + 完整 snapshot + 索引 p95 为 1.5832 ms，不绘制，因此不是阶梯流式帧耗时。**不据此宣称 GPU 时间、FPS、整应用峰值、零泄漏或相对历史条件的性能提升。**

## 茎状图（ML-039）

```rust
use egui_customplot::{PlotData, StemOrientation, Stems};
let data = PlotData::new(vec![[1.0, 2.0], [2.0, -1.0], [4.0, 3.0]]);
let stems = Stems::new(&data).baseline(0.0).orientation(StemOrientation::Vertical);
// Plot::new("discrete").show_layers(ui, &mut state, &[stems.into()]);
```

- `Stems` 借用已有 `PlotData`；`PlotLayer::Stems` 是非穷尽异构入口的新增分支，不改变 `Series`、`Series::data()` 或旧折线入口。NaN / Inf 只省略其记录，各杆独立，不连接相邻样本。
- 默认垂直杆、基线 0、显示原始端点 marker 和基线。`Horizontal` 改变从基线延伸的坐标分量，不交换原始 XY。正负杆支持非等距 / 重复 / 乱序位置，零长杆没有线段，但仍可显示端点 marker。
- 基线是数据坐标：垂直为 Y、水平为 X。非有限基线或 log 值轴上的非正基线使整层（含 markers）省略，并递增 `invalid_stem_baselines`。端点必须满足联合 XY 域；原始无效端点仍计入通用点诊断。
- 自动范围 O(1) 合并 `PlotData` 的四域缓存，并纳入有效基线；隐藏基线不从杆范围删除基线。基线只跨有效原始端点的固定坐标范围，不无限延伸至视口，不虚构两端相同固定坐标的跨度。
- 杆在 f64 数据空间裁剪后投影；视区交点不产生新 marker / 基线。markers 保持原始端点；极远 marker 投影溢出会明示 numeric rejection，不移到视区边缘。
- 杆样式继承与折线一致；基线可独立 `baseline_stroke`，始终实线。每杆的虚线相位独立，负杆也从基线方向开始；裁剪在虚线之前，离屏基线的可见杆从裁剪边界重新起相。无效虚线不阻止有效 marker / 独立基线。
- 只拾取原始端点，不拾取杆内、基线或裁剪交点；marker 关闭时非零可见杆端点仍可拾取，零长杆不可。复用显式 X 索引筛选候选，不对杆做近似 LOD；左右轴 / 反向 / 提示 / 图例及多遍导航走同一宿主生命周期。
- `visited_stems` 计原始杆扫描，`output_stem_segments` 计虚线处理后的杆路径，`output_stem_baselines` 单计基线；markers 沿用通用计数。样本 / 索引字节跨 Line / Stems 去重。`PlotCache` 仍只缓存折线，不缓存杆或保留样本。

### 茎状图验证

新增 `tests/stems.rs` 5 项、`stems_lifecycle.rs` 5 项、`stems_navigation.rs` 2 项、`export_stems.rs` 1 项。覆盖精确几何 / 非有限 / 域 / 巨大离屏值、方向 / 重复位置 / 零长 / 正负虚线、独立基线 / 样式与拾取一致性、双轴 / 索引 / 显隐 / 父裁剪 / 32 次销毁、导航和图例多遍幂等，以及真实 SVG 路径与 PNG / resvg 像素。payload 引用计数释放不是整应用零泄漏证明。

`--stems` 提供固定正负 / NaN / 重复位置案例与右 log Y 面板，可调方向、markers、基线与虚线；demo 测试确认八种组合在双遍中不重建数据版本。全部五个 gallery 均执行真实 PNG / SVG 编码。

[Windows 五预设生命周期记录](benchmarks/2026-11-15-stage8-stems-windows-gallery-smoke.txt)：SHA256 `EFAD6D3B72C6850AC55AD9381D22F97B32AA6D9153B54464C46055E693DA7DE8`，均响应、正常关闭且退出码 0。不是人工视觉或 GPU 验收。

加入清单状态统计护栏后，茎状图最终统一 all-features **188 项通过**（库单元 32、集成 143、demo 11、rustdoc 2）；默认 workspace 测试也通过。fmt、无默认 feature 库检查、严格 all-targets / all-features Clippy、Release demo 构建同批通过，binary hash 与上述 native smoke 一致。[all-features 与构建原始记录](benchmarks/2026-11-15-stage8-stems-windows-checks.txt)、[默认 workspace 测试原始记录](benchmarks/2026-11-15-stage8-stems-windows-default-tests.txt) 独立保存；此前 173 项和两个旧 native binary 证据保留。阶梯实现提交 `e4623b4` 的 [CI](https://github.com/littepointR/egui-customplot/actions/runs/43697850975) 当前 Ubuntu 成功、Windows 尚在运行，不标记本批跨平台验收完成。

### 茎状图 CPU / 局部容量记录

`stems_performance` 的 1 千 / 1 万 / 10 万点、全区 / 2% 窄区各测 30 秒（预热 5 帧），Release、1920×1080 逻辑绘图区、DPI=1，markers / baseline 开启，hover / 索引关闭。六行齐全且时长均 ≥30 秒、退出码 0、stderr 为空；新 [环境 / binary hash](benchmarks/2026-11-15-stage8-stems-windows-cpu-environment.txt)、[CSV](benchmarks/2026-11-15-stage8-stems-windows-cpu.csv)、[stderr](benchmarks/2026-11-15-stage8-stems-windows-cpu.stderr.txt) 保留。与阶梯一样使用共享 i9-14900KF / Windows / Rust 1.100.0，后台同时进行开发和构建，不能当作隔离硬件门槛。

| 点数 | 全区 CPU 合计 p95 ms | 窄区 CPU 合计 p95 ms | 全区 geometry capacity B | 窄区 geometry capacity B |
| --- | ---: | ---: | ---: | ---: |
| 1,000 | 1.0780 | 0.0253 | 2,375,280 | 37,184 |
| 10,000 | 12.7655 | 0.1803 | 19,034,352 | 298,112 |
| 100,000 | 631.3393 | 3.3797 | 152,594,928 | 4,750,592 |

这是独立精确杆 + 原始 markers，不应套用折线 MinMax 的低成本结论。所有场景仍扫描全部原始杆和 markers；10 万点窄区输出 2,000 杆、2,010 markers（marker 中心允许半径范围内离屏），不是抽样或自动聚合。`geometry capacity` 仅为已计入的坐标 Vec 和 marker mesh 保留容量，不含全部临时分配、Shape 元数据、tessellation 或 GPU / 进程峰值。全区 10 万点有显著 CPU / 容量风险，本次存在高尾延迟；不掩盖、不推导 FPS，也不以此单独决定 GPU 后端。

## 剩余验收

固定硬件门槛、人工及跨平台显示、GPU / 整应用资源、精确阶梯 LOD、杆批处理 / 渲染索引和完整 MATLAB 属性 / 布局兼容仍缺。阶段 8 其余统计、多变量、函数、极坐标、几何、网络类型继续推进；阶段 4 / 6 / 7 遗留验收也不因本切片消失。
