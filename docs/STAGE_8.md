# 阶段 8：二维类型补全切片

日期：2026-11-16。**阶梯线、茎状图、二维矢量场与 Stage 8A（面积 / 多边形 / 面片 / 等高线 / 伪彩 / 图像）基础切片验证通过，阶段 8 与对应 MATLAB 类型的完整验收未完成。** 133 项对照全部保留；本页不把可绘制等同于统计 / 几何计算或完整交付。

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

加入清单状态统计护栏后，茎状图最终统一 all-features **188 项通过**（库单元 32、集成 143、demo 11、rustdoc 2）；默认 workspace 测试也通过。fmt、无默认 feature 库检查、严格 all-targets / all-features Clippy、Release demo 构建同批通过，binary hash 与上述 native smoke 一致。[all-features 与构建原始记录](benchmarks/2026-11-15-stage8-stems-windows-checks.txt)、[默认 workspace 测试原始记录](benchmarks/2026-11-15-stage8-stems-windows-default-tests.txt) 独立保存；此前 173 项和两个旧 native binary 证据保留。随后确认阶梯提交 `e4623b4` 的 [CI](https://github.com/littepointR/egui-customplot/actions/runs/43697850975) 与茎状图提交 `fc77b72` 的 [CI](https://github.com/littepointR/egui-customplot/actions/runs/43698813888) 均为 Windows / Ubuntu 成功；这不是后加 Quiver 或跨平台显示实机的验收。

### 茎状图 CPU / 局部容量记录

`stems_performance` 的 1 千 / 1 万 / 10 万点、全区 / 2% 窄区各测 30 秒（预热 5 帧），Release、1920×1080 逻辑绘图区、DPI=1，markers / baseline 开启，hover / 索引关闭。六行齐全且时长均 ≥30 秒、退出码 0、stderr 为空；新 [环境 / binary hash](benchmarks/2026-11-15-stage8-stems-windows-cpu-environment.txt)、[CSV](benchmarks/2026-11-15-stage8-stems-windows-cpu.csv)、[stderr](benchmarks/2026-11-15-stage8-stems-windows-cpu.stderr.txt) 保留。与阶梯一样使用共享 i9-14900KF / Windows / Rust 1.100.0，后台同时进行开发和构建，不能当作隔离硬件门槛。

| 点数 | 全区 CPU 合计 p95 ms | 窄区 CPU 合计 p95 ms | 全区 geometry capacity B | 窄区 geometry capacity B |
| --- | ---: | ---: | ---: | ---: |
| 1,000 | 1.0780 | 0.0253 | 2,375,280 | 37,184 |
| 10,000 | 12.7655 | 0.1803 | 19,034,352 | 298,112 |
| 100,000 | 631.3393 | 3.3797 | 152,594,928 | 4,750,592 |

这是独立精确杆 + 原始 markers，不应套用折线 MinMax 的低成本结论。所有场景仍扫描全部原始杆和 markers；10 万点窄区输出 2,000 杆、2,010 markers（marker 中心允许半径范围内离屏），不是抽样或自动聚合。`geometry capacity` 仅为已计入的坐标 Vec 和 marker mesh 保留容量，不含全部临时分配、Shape 元数据、tessellation 或 GPU / 进程峰值。全区 10 万点有显著 CPU / 容量风险，本次存在高尾延迟；不掩盖、不推导 FPS，也不以此单独决定 GPU 后端。

## 二维矢量场（ML-056）

```rust
use egui_customplot::{Quiver, QuiverArrow, QuiverData};
let field = QuiverData::new(vec![
    QuiverArrow::new([1.0, 2.0], [0.5, -1.0]), // end = [1.5, 1.0]
]);
let arrows = Quiver::new(&field).head_size(10.0, 8.0);
// Plot::new("field").show_layers(ui, &mut state, &[arrows.into()]);
```

- `QuiverArrow` 为原始 `origin + vector`，`endpoint()` 返回相加后的真实端点；不接受非有限原点 / 分量或端点加法溢出。`QuiverData` 是宿主持有的不可变 Arc 记录与四联合域范围摘要，构造 O(N)、每遍自动范围 O(1)，包含原点和端点；克隆 / 重复图层的原始 payload 去重。
- **无隐式自动缩放 / 归一化**，需要缩放时由宿主显式发布新位移数据。`Quiver` 通过 `PlotLayer` 进入宿主，不改变 `Series::data()`、旧 show / show_series 或 HoverInfo。零向量和相加后落在同一可表示原点的微小位移保留原始记录 / 合法范围，但没有杆 / 箭头 / 拾取。
- 整条箭头的原点和真实端点必须满足 XY 联合坐标域，不能只显示跨越非法 log 域的部分。左右 Y / 反向 / 等比例使用共用变换。包括 log 轴在内，杆是**两端变换后的直线弦**，不是对连续数据空间参数曲线的自适应细分。
- 在 f64 归一化空间裁剪后转 f32。对可能穿过视区的极远向量，若归一化不有限，或 `max(abs(a), abs(b)) × f64::EPSILON × 32 × screen_span > 0.01` 逻辑点，则省略整条箭头并计 `numeric_rejections`。这是保守的数值取消风险护栏，不是任意极值精确裁剪证明；普通折线仍沿用原参考路径的既有数值边界。
- 默认 10×8 egui 逻辑点的开放 V 形头部，只放在**真实端点位于数据视区内且杆具有可表示可见长度**的箭头上，视区交点不生成假头。头部按完整未裁剪投影长度的 40% 上限同比缩小，不按可见残段缩小；Painter 正常裁剪头部翼，不移动真实端点。高 DPI 不改变逻辑点尺寸。
- 颜色 / 线宽 / 杆虚线按 Theme → 父 → Plot → 图层继承；头部始终实线。每条杆在裁剪后独立从可见起点起虚线相位；继承的 point marker 字段不适用于向量。隐藏头部不隐藏杆，头部尺寸为零时省略头部，负值 / 非有限尺寸另计一次诊断；无效线宽 / 虚线省略整层。既有杆虚线安全阀继续有效，头部独立提交。
- 只拾取原始原点和真实端点，不拾取杆内部、翼或裁剪交点；先按屏幕距离，再按提交顺序 / 原始记录顺序，完全相同距离的同条记录以原点优先。填充层遮挡沿用已有点层仲裁。
- `PlotResponse::hovered_point` / 原 HoverInfo 格式器得到获胜的真实锚点和原始记录下标；额外 `hovered_vector` 保存原始 `QuiverArrow`、真实端点、`VectorAnchor`、轴归属和距离。默认提示列出原点、原始位移和端点；不从箭头几何反推分量。
- 计数独立为 `input / invalid / domain_invalid / visited / visited_pick_quiver_arrows`、`output_quiver_shafts / heads` 与 `unique_quiver_bytes`。没有向量渲染索引、拾取索引、LOD 或向量缓存；绘制与拾取各自扫描全部记录。geometry capacity 仅计提交路径的点 Vec 容量，不是临时总峰值或 GPU / 进程占用。

### 矢量场正确性与生命周期

新增库专项 16 项：`quiver.rs` 6、`quiver_picking.rs` 4、`quiver_lifecycle.rs` 3、`quiver_dpi.rs` 2、`export_quiver.rs` 1。覆盖正负 / 对角 / 反向 / log / 精确端点、全记录域拒绝 / 加法溢出、空 / 零 / 微小 / 短向量、裁剪不生假头、极远值明示拒绝、头部尺寸和虚线、两真实锚点及旧格式器、填充遮挡、左右轴、32 次引用释放、多遍导航 / 图例与裁剪、样式继承、1 / 1.5 / 2 host DPI 和两输出倍率。

导出专项解析真实 SVG 杆 / V 形路径，PNG 与不加载外部字体的 resvg 同时检查杆 / 真头有墨迹、裁剪交点的假头位置无墨迹；捕获后原始数据和宿主 Context 可销毁。局部 Arc 引用释放不代表整应用零泄漏。

`--quiver` 提供显式旋转场（77 条含一条零向量，另加一条 NaN）与右 log Y 面板，可调头部、杆虚线及反向 X；demo 专项确认八种组合在双遍中不重建 payload。全部六个 gallery 均执行真实 PNG / SVG 编码。

最终统一 all-features **205 项通过**（库单元 32、集成 159、demo 12、rustdoc 2）；默认 workspace、fmt、无默认 feature 库检查、严格 all-targets / all-features Clippy、Release demo 构建同批通过。[all-features 与构建原始记录](benchmarks/2026-11-15-stage8-quiver-windows-checks.txt)、[默认 workspace 测试原始记录](benchmarks/2026-11-15-stage8-quiver-windows-default-tests.txt) 已独立归档，未覆盖历史证据。证据 / 文档收尾后再执行上述七项检查（含默认与 all-features 两组严格 Clippy）全部通过，见 [带命令 / 退出码的最终检查](benchmarks/2026-11-15-stage8-quiver-windows-final-checks.txt)。[六预设 Windows 生命周期](benchmarks/2026-11-15-stage8-quiver-windows-gallery-smoke.txt) 全部启动 / 响应 / 正常关闭通过，native 记录 SHA256 与本批最终 Release binary 一致：`42153A36A8363EE8963AFB7737756D2DA200D686D2F510E833D9C515E81EDAC8`。生命周期脚本不验证完整视觉、GPU / FPS 或整应用资源。

### 矢量场 CPU 测量

`quiver_performance` 对 1 千 / 1 万 / 10 万条显式二维记录，测全区 / 约 2% X 窄区 / 全区原始锚点拾取，每场景预热 5 帧并采集 30 秒，记录 UI、tessellation、合计 p50 / p95 / p99 及拾取 p95，固定 1920×1080 逻辑绘图区、DPI=1、实线杆、10×8 点头部，无自动缩放或 LOD。首轮因 T3 server 重启中断，CSV 为空、无退出码，已确认无残余进程；[旧环境记录](benchmarks/2026-11-15-stage8-quiver-windows-cpu-environment.txt) 保留且明确无有效性能结论。

同一 binary 的 `retry1` 独立重测已验收：九场景齐全、每场景 ≥30 秒、退出码 0、stderr 为空，三档绘制 / 原始拾取访问计数与完整扫描契约一致。[正式 CSV](benchmarks/2026-11-15-stage8-quiver-windows-cpu-retry1.csv)、[环境 / 完成状态](benchmarks/2026-11-15-stage8-quiver-windows-cpu-retry1-environment.txt)、[stderr](benchmarks/2026-11-15-stage8-quiver-windows-cpu-retry1.stderr.txt)。CPU 为 i9-14900KF，Windows 10.0.26300，Rust 1.100.0，normal priority；共享机器上并行开发 / 检查 / 构建，不是隔离的固定硬件发布门槛。benchmark binary SHA256 已复核为 `5254C02E0A5E6D30A096A7010763F4C44E370F0AD44516136FD6867936B8584A`，基于 `fc77b72` 加本切片未提交实现。

| 记录数 | 全区 CPU 合计 p95 ms | 窄区 CPU 合计 p95 ms | 全区含原始拾取合计 p95 ms | 原始拾取自身 p95 ms | 全区路径点 Vec capacity B |
| --- | --- | --- | --- | --- | --- |
| 1,000 | 0.5889 | 0.0419 | 0.7867 | 0.0702 | 40,000 |
| 10,000 | 8.4369 | 0.1648 | 6.9711 | 0.3814 | 400,000 |
| 100,000 | 74.7509 | 2.2253 | 82.0513 | 3.7368 | 4,000,000 |

合计逐帧计算 UI 构建 + egui tessellation 后取分位数，不把两个独立 p95 相加；不含呈现等待 / GPU、mesh 销毁、原始数据构造或统计样本 Vec 增长。各场景独立计时且共享机器负载变化，1 万记录含拾取合计低于无拾取场景不能解释为拾取加速。三档原始 payload 分别为 32,000 / 320,000 / 3,200,000 B；窄区仍逐条访问全部记录，只把输出杆 / 头部降为 20 / 200 / 2,000。10 万全区的 CPU 成本明显超出候选 16.7 ms 交互预算；没有向量索引、批处理或 LOD，不承诺密集全区实时性。路径点 Vec capacity 不含顶层 Vec、egui shapes / meshes、所有临时分配、进程峰值或 GPU，不能据此声称总占用或零泄漏。

## 面积、多边形、面片、等高线、伪彩与图像（Stage 8A：ML-005 / 051 / 052 / 055 / 063 / 081，EX-001 / 002 / 004 / 005 / 016 / 025，LG-011 / 012）

```rust
use egui_customplot::{AreaBandData, AreaStackMode, ContourData, ContourLevels, Contours, ScalarField2D};
// 面积堆叠 / 多边形 / 面片 / 等高线 / Pcolor / 图像均由宿主持有不可变 Arc payload，通过 show_layers 提交
```

- **可扩展分域诊断与资源去重：** `PlotDiagnostics` 新增 `patches: PatchDiagnostics`、`contours: ContourDiagnostics`、`images: ImageDiagnostics`，并在 `model::summary` 中跨混合图层按真实 Arc 数据指针去重 `unique_bytes`，避免多图元叠加时重复累计共享样本字节。
- **面积与底层几何（ML-005 / EX-001 / EX-002 / EX-004 / EX-005 / EX-016）：**
  - `Area` 同时支持单曲线（复用 `PlotData` 与显式 `LineIndex` 区间筛选）与 `AreaBandData::{from_band, stack}`（`Cumulative` / `PosNegSplit`；对非单调 X 轴显式回退全范围扫描而非二分错位）。NaN 断段与 log 值轴非法基线明示递增 `invalid_area_baselines`。
  - `Line::primitive` 提供无隐式重置范围的数据空间折线图元；`PolygonData` / `Fill` 与 `PolyRegion` / `PolyShapeData` / `PolyShape` 基于 f64 耳切三角化与 Eberly 孔洞桥接预计算四坐标域三角索引；`PatchData` / `Patch` 支持 `Uniform` / `FaceColors` / `FaceScalars` 与 Gouraud `VertexColors` / `VertexScalars`；`DataRect` / `Rectangles` 支持圆角曲率与椭圆细分及精确圆角命中判定。
  - 所有几何在 `[0, 1]^2` 归一化空间按 Sutherland-Hodgman 算法裁剪三角，再通过 `MeshBatch`（16,384 三角分批）提交给 `Painter`。
- **等高线、伪彩与图像（ML-051 / ML-052 / ML-055 / ML-063 / ML-081 / EX-025 / LG-011 / LG-012）：**
  - `ScalarField2D` 持有矩形网格节点值，提供 `sample_function` / `ez_sample`（强制检查 `max_evaluations` 预算）。
  - `ContourData`（四三角中心分裂 Marching Squares，避免 2×2 鞍点二义性）与 `FilledContourData`（半平面三角裁剪）在宿主侧预处理一次；`Contours` 支持 `ColorScale`、虚线与最长可读片段居中标签，`FilledContours` 支持可选 `border_lines`。
  - `Pcolor` 复用 `ScalarField2D`，提供 `PcolorShading::{Flat, Interp}`（Gouraud 双三角），沿矩形可见节点二分裁剪；`ImageData::{rgba8, rgb8, from_pixel_centers, indexed, grayscale}` 与 `PlotImage` / `Plot::imshow_axes()`（top-down Y 反转 + 1:1 `data_aspect`）支持透明像素跳过。
- **拾取与混合导出边界：** `PlotResponse::hovered_region`（`HoveredRegion`）遵循顶层遮挡优先级，正确跳过 `PolyShape` 孔洞、`PlotImage` 透明像素及包含无效顶点的 `Patch` 面；`Area` 上边界与 `Contours` 等高线继续走原始点/顶点精确拾取。纯色平铺三角导出为原生 SVG `<path>`，Gouraud 渐变 `Patch` / `PcolorShading::Interp` 在 SVG 中自动触发内嵌 PNG `<image>` 栅格回退，无外部 `<text>` 依赖。

### Stage 8A 正确性与生命周期证据

- 新增库单元 3 项（三角化与 Eberly 孔洞桥接）、集成 16 项：`tests/area.rs`（4 项）、`tests/polygons_patches.rs`（5 项）、`tests/contours.rs`（3 项）、`tests/pcolor_image.rs`（2 项）、`tests/export_polygons_contours.rs`（2 项，含 resvg 无字体像素校验孔洞与 SVG path/raster 分流），以及 demo `--polygons` / `--contours` 测试 2 项。
- 统一 all-features **226 项通过**（库单元 35、集成 175、demo 14、rustdoc 2），`fmt`、默认 workspace 测试、无默认 feature 检查、默认与 `--all-features` 严格 Clippy、Release demo 构建全部通过，见 [最终 7 项检查记录](benchmarks/2026-11-16-stage8a-windows-final-checks.txt)。
- [scripts/smoke-demo.ps1](../../scripts/smoke-demo.ps1) 已修复 Windows IME 浮窗 (`UAC_InputIndicatorOverlayWnd`) 干扰 `.NET MainWindowHandle` 的问题（首轮未修前中止留档 [gallery-smoke](benchmarks/2026-11-16-stage8a-windows-gallery-smoke.txt)）；修正后八预设原生窗口（`default`、`--scalar`、`--statistical`、`--stairs`、`--stems`、`--quiver`、`--polygons`、`--contours`）均响应且正常退出，见 [retry1 窗口记录](benchmarks/2026-11-16-stage8a-windows-gallery-smoke-retry1.txt)，Release `customplot-demo.exe` SHA256 核对一致为 `8601BD85105E86D6556719B384B295164040A8FA722C9AFC7EC13ACD43EBF885`。

### Stage 8A CPU 基准测量

`polygon_contour_performance`（binary SHA256 `1A638594F10B3087CBB804518210F0BA7E5D49F6B43A0BFAA2C42D3CE9823A8E`）对 7 个场景各预热 5 帧并采集 30 秒，固定 1920×1080 逻辑绘图区、DPI=1、`show_grid=false`、`show_hover=false`，共享 i9-14900KF / Windows 10.0.26300 / Rust 1.100.0 环境；退出码 0、stderr 仅 cargo 编译输出。[环境记录](benchmarks/2026-11-16-stage8a-windows-cpu-environment.txt)、[原始 CSV](benchmarks/2026-11-16-stage8a-windows-cpu.csv)、[stderr](benchmarks/2026-11-16-stage8a-windows-cpu.stderr.txt) 独立保存。

- **预处理中位耗时（256×256 场、12 等级）：** `ContourData::from_field` p50 = **9.5039 ms**（106 条等高线）；`FilledContourData::from_field` p50 = **34.2380 ms**（13 个带、333,078 个预切分三角）。

| 场景 | 输入规模 | CPU UI p95 ms | Tess p95 ms | 合计 p95 ms | 几何 Capacity B | 样本 Payload B |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `area_100k_full` | 100,000 点（199,998 三角） | 19.1648 | 16.6086 | **37.1281** | 20,316,160 | 1,600,000 |
| `area_100k_narrow_2pct` | 100,000 点索引窄视区（4,004 三角） | 1.2321 | 0.0677 | **1.3064** | 409,600 | 1,600,000 |
| `patch_10k_faces_full` | 10,000 四边面片（20,000 Gouraud 三角） | 2.6928 | 0.4265 | **3.0793** | 1,966,080 | 563,224 |
| `contour_256x256_12levels` | 256×256 场 106 等高线（212 路径） | 0.6372 | 2.1595 | **2.7624** | 422,912 | 585,616 |
| `contourf_256x256_12levels` | 256×256 场 13 等值带（333,078 三角） | 37.8326 | 20.4669 | **57.3684** | 31,850,496 | 18,817,136 |
| `pcolor_256x256_flat` | 256×256 `Pcolor::Flat`（65,025 cell） | 2.7496 | 3.2586 | **5.9008** | 6,815,744 | 528,384 |
| `image_256x256_rgba` | 256×256 `ImageData::rgba8`（65,536 像素） | 2.6010 | 3.9874 | **6.3668** | 6,815,744 | 266,256 |

- **严格口径与已知瓶颈：** 以上仅测 headless CPU UI 构建 + egui tessellation，不包含 wgpu/glow 上传、GPU 光栅化、呈现等待、FPS 或进程峰值内存。`area_100k_full`（37.1 ms）与 `contourf_256x256_12levels`（57.4 ms）因提交十万至三十余万级精确三角，已超出 16.7 ms 交互预算；在后续阶段引入多边形简化 / 等值带抽取合并或可选 GPU 路径前，不宣称大规模全视区 `area` / `contourf` 的实时帧率。

## 剩余验收

固定硬件门槛、人工及跨平台显示、GPU / 整应用资源、精确阶梯 LOD、杆 / 向量批处理与索引、Quiver 自动缩放 / 全属性和完整 MATLAB 布局兼容仍缺。阶段 8 其余统计、多变量、函数、极坐标、几何、网络类型继续推进；阶段 4 / 6 / 7 遗留验收也不因本切片消失。
