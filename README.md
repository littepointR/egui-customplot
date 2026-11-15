# egui-customplot

以 MATLAB 的科学与工程绘图体验为参考，基于 Rust / egui 构建高性能、可定制的绘图库。

**当前状态：阶段 4～7 基础切片与阶段 8A 阶梯线 / 茎状图 / 二维矢量场 / 面积多边形面片等高线伪彩与图像基础切片已实现并持续验证，尚非完整绘图库。**

已实现折线 / 标记 / 散点混合叠加、线性 / 对数 / 反向轴、等比例、标题标签、主次刻度、格式器、可点击图例、原始点提示、两套主题和显式样式继承、基础注解与诊断。库默认只依赖 egui；可选 `export` feature 增加 PNG 编码器，eframe 仅用于 demo；不追求 MATLAB 语法兼容。

另有宿主持有的索引 / 可选 MinMax / 折线缓存 / 环形数据、Figure 网格、显式轴联动、双 Y 轴、有序数值网格 / 柱 / 误差棒和共享色条，以及显式直方图分箱预处理。各能力边界见阶段文档。

## 运行

需要 Rust **1.95+**；本轮实测 Rust 1.100.0 / Windows x64 / egui 0.37.0。

```sh
cargo run -p customplot-demo --release --locked
cargo run -p customplot-demo --release --locked -- --scalar
cargo run -p customplot-demo --release --locked -- --statistical
cargo run -p customplot-demo --release --locked -- --stairs
cargo run -p customplot-demo --release --locked -- --stems
cargo run -p customplot-demo --release --locked -- --quiver
cargo run -p customplot-demo --release --locked -- --polygons
cargo run -p customplot-demo --release --locked -- --contours
```

- 拖动：平移；悬停 + 滚轮：缩放；双击或聚焦后按 Home：自动适配范围。
- Demo 提供信号 / 正值幂律场景，可切换 1 千～1000 万点、NaN 断段、散点、7 种标记、实线 / 虚线、对数 / 反向 / 等比例轴、Figure 双视图、可选 X 联动和右 Y 轴。
- `--scalar` 展示数值网格 / 共享色条 / 折线叠加，`--statistical` 展示显式分箱与正负柱 / 非对称 XY 误差棒；两者也可在控制面板打开。
- `--stairs` 展示固定非等距阶梯信号；`Line::interpolation(LineInterpolation::StepAfter)` 保持后跳变，`StepBefore` 先跳变后保持，默认 Linear。原始点用于 markers / hover；MinMax 可能改变阶跃时刻和保持时长。
- `--stems` 展示独立正负杆 / 基线 / 原始端点与右 log Y 面板；`Stems` 可复用 `PlotData`，通过 `show_layers` 提交。方向不交换原始 XY，log 值轴需正基线；没有杆 LOD / 缓存，密集 markers / 全区杆成本很高。
- `--quiver` 展示旋转矢量场与右 log Y 面板；`QuiverData` 保存原点 + 显式位移，无隐式自动缩放。箭头只放在真实端点；原点 / 端点拾取返回原始记录，极端数值可能明示拒绝；无向量 LOD / 索引 / 缓存。
- `--polygons` 展示堆积 `AreaBandData`（`Cumulative` / `PosNegSplit`）、`Line::primitive`、凹多边形 `Fill`、带孔 `PolyShape`、Uniform/Face/Gouraud `Patch` 与圆角/椭圆 `Rectangles`；Gouraud 面片导出 SVG 时使用内嵌 PNG 栅格回退。
- `--contours` 展示 `ScalarField2D` 抽取的 `Contours` / `FilledContours`（含 `from_function` / `ezcontour*` 显式求值预算）、`Pcolor`（`Flat` / `Interp`）与 `Plot::imshow_axes()` + `PlotImage`；高密度等值带为纯三角裁剪提交，无后台 GPU 加速。
- Export 面板保存 PNG / 混合 SVG，默认 `target/exports/plot`、拒绝覆盖；只保留当前显示采样。文字内嵌为字形像素，不是完整矢量字体 / 自动 Exact 重绘，见 [阶段 7](docs/STAGE_7.md)。
- 属性面板可切科学浅色 / 监控深色 / 宿主主题，调整父默认和局部覆盖、注解及带单位的提示；主题不污染宿主样式。
- 性能面板可显式构建单调 X 索引、切换 Exact / Visible / 近似 MinMax、启用宿主折线缓存和有界 Live 环形数据；库默认仍为 Exact。
- **大数据选项是压力场景，不是帧率承诺。** 索引 / 缓存 / 环形缓冲仅限制各自明示的资源；标记仍全量扫描，没有专用 GPU 后端。Demo 数据重建 / 索引构建是显式同步操作。
- 不要求宿主使用 eframe，也不创建第二个窗口系统、事件循环或 GPU device。

## 嵌入 egui

跨帧保留数据与每张图独立的 `PlotState`，不要每帧重新构造数据：

```rust
use egui_customplot::{Line, Plot, PlotData, PlotState};

// 初始化时：
let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
let mut state = PlotState::default();

// 在宿主的 UI 闭包中（ui: &mut egui::Ui）：
// Plot::new("signal")
//     .height(320.0)
//     .allow_scroll_zoom(true)
//     .show(ui, &mut state, &[Line::new(&data)]);
```

库的滚轮缩放**默认关闭**，避免抢走父 ScrollArea 的滚动；demo 显式开启。`show` 返回 egui `Response`、坐标变换和本遍诊断；UI 多遍调用不是一次性事件通知。`PlotState` 保存本地手动视图，实际联动 / 双轴显示范围读取返回的 `transform` / `right_transform`。

异构图元使用 `PlotLayer` 与 `show_layers`，不改变旧的 `Line` / `Series::data()` 契约。样本预处理放在 UI 更新节奏之外：

```rust
use egui_customplot::{Bar, BarData, Bars, ErrorBar, ErrorBarData, ErrorBars};
let bars = BarData::new(vec![Bar::new(1.0, 2.0), Bar::new(2.0, -1.0)]);
let errors = ErrorBarData::new(vec![ErrorBar::new([1.0, 2.0]).y_error([0.2, 0.4])]);
// Plot::new("measurements").show_layers(ui, &mut state, &[
//     Bars::new(&bars).into(), ErrorBars::new(&errors).into(),
// ]);
```

## 检查与基准

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo bench -p egui-customplot --bench frame_baseline --locked
cargo bench -p egui-customplot --bench data_performance --locked
cargo bench -p egui-customplot --bench heatmap_performance --locked
cargo bench -p egui-customplot --bench stems_performance --locked
cargo bench -p egui-customplot --bench quiver_performance --locked
cargo bench -p egui-customplot --bench polygon_contour_performance --locked
cargo run -p egui-customplot --example export_snapshot --features export --release --locked -- target/exports/example
```

基准默认每场景采样 30 秒。`frame_baseline` 保留空白 egui、空图、10 万点静态 / 平移；`data_performance` 增加 10 万 / 100 万 / 1000 万点索引 / LOD / 缓存 / 拾取及流式记录。**只测 CPU UI 构建和 tessellation，不测窗口呈现或 GPU**。快速试跑可设置环境变量 `CUSTOMPLOT_BENCH_SECONDS=2`，不得与正式 30 秒记录混用。

`data_performance` 可用 `CUSTOMPLOT_BENCH_INTERPOLATION=linear|after|before` 选择连接方式；采样和测量边界不变，stream 行仍不绘图。

## 交付记录与长期对照

- [源码职责、目录和扩展规则](docs/ARCHITECTURE.md)：按 data / model / axis / layout / interaction / render / widget 分域，带架构回归检查。
- [全量实施进度](docs/PROGRESS.md)
- [阶段 8 阶梯线 / 茎状图 / 二维矢量场及二维类型补全](docs/STAGE_8.md)
- [阶段 7 显示快照、PNG / 混合 SVG 与字体 / 分辨率边界](docs/STAGE_7.md)
- [阶段 6 多面板、双轴、数值网格、柱 / 误差棒与直方图](docs/STAGE_6.md)
- [阶段 5 GPU 暂缓决策、证据与重新评估条件](docs/STAGE_5.md)
- [阶段 4 索引、LOD、缓存、流式与资源边界](docs/STAGE_4.md)
- [阶段 3 样式、主题、提示与注解](docs/STAGE_3.md)
- [阶段 2 功能、验证与限制](docs/STAGE_2.md)
- [首轮实现、测试证据与已知限制](docs/STAGE_1.md)

- [项目规划与分阶段验收](docs/PLAN.md)
- [MATLAB 绘图类型完整交付对照](docs/MATLAB_PLOT_CHECKLIST.md)：R2026b 官方总览 82 项，另含本体补充与传统接口；工具箱扩展单列。
- [egui 集成、性能与资源占用约束](docs/EGUI_INTEGRATION_AND_PERFORMANCE.md)：原生嵌入、多遍布局、按需重绘、CPU / GPU 预算与长期验收要求。
- 推荐推进顺序：最小闭环 → 绘图能力 → 定制能力 → 数据性能 → 按需 GPU 加速 → 导出与发布。

尚无独立印刷布局 / 全矢量文字导出、专用 GPU 后端、全局资源硬预算、完整类别 / 堆积 / 三维 / 地理能力；Figure 暂不支持跨行列布局。索引拾取仅按 X 筛选，密集同 X / 多断段仍可能很贵；标记、柱 / 误差棒 / 茎状图 / 矢量场仍扫描原始记录。流式快照发布 / 索引不是增量更新；全区百万 cell 热力图和密集杆 / markers 的 CPU / 几何成本很高，没有静默 LOD 或帧率承诺。MATLAB 清单已有 18 项“开发中”，没有图类型通过完整交付门槛；不能声称已交付 133 项能力。历史 Windows / Ubuntu 自动检查不替代本轮 CI 或跨平台显示实机验证。

许可证尚未决定，workspace 包均设为 `publish = false`；本项目没有默认授予开源许可。
