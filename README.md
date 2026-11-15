# egui-customplot

以 MATLAB 的科学与工程绘图体验为参考，基于 Rust / egui 构建高性能、可定制的绘图库。

**当前状态：阶段 2 基础二维切片可运行，尚非完整绘图库。**

已实现折线 / 标记 / 散点混合叠加、线性 / 对数 / 反向轴、等比例、标题标签、主次刻度、格式器、可点击图例、原始点提示，以及基础样式和诊断。库只依赖 egui，eframe 仅用于 demo；不追求 MATLAB 语法兼容。

## 运行

需要 Rust **1.95+**；本轮实测 Rust 1.100.0 / Windows x64 / egui 0.37.0。

```sh
cargo run -p customplot-demo --release --locked
```

- 拖动：平移；悬停 + 滚轮：缩放；双击或聚焦后按 Home：自动适配范围。
- Demo 提供信号 / 正值幂律场景，可切换 1 千～100 万点、NaN 断段、散点、7 种标记、实线 / 虚线、对数 / 反向 / 等比例轴、图例和独立双视图。
- **百万点选项是压力场景，不是帧率承诺。** 当前逐段扫描，没有 LOD、几何缓存或专用 GPU 后端。
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

库的滚轮缩放**默认关闭**，避免抢走父 ScrollArea 的滚动；demo 显式开启。`show` 返回 egui `Response`、坐标变换和本遍诊断；UI 多遍调用不是一次性事件通知，最终视图从 `PlotState` 读取。

## 检查与基准

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo bench -p egui-customplot --bench frame_baseline --locked
```

基准默认每场景采样 30 秒，包含空白 egui、空图、10 万点静态 / 平移场景；**只测 CPU UI 构建和 tessellation，不测窗口呈现或 GPU**。快速试跑可设置环境变量 `CUSTOMPLOT_BENCH_SECONDS=2`，不得与正式 30 秒记录混用。

## 交付记录与长期对照

- [源码职责、目录和扩展规则](docs/ARCHITECTURE.md)：按 data / model / axis / layout / interaction / render / widget 分域，带架构回归检查。
- [全量实施进度](docs/PROGRESS.md)
- [阶段 2 功能、验证与限制](docs/STAGE_2.md)
- [首轮实现、测试证据与已知限制](docs/STAGE_1.md)

- [项目规划与分阶段验收](docs/PLAN.md)
- [MATLAB 绘图类型完整交付对照](docs/MATLAB_PLOT_CHECKLIST.md)：R2026b 官方总览 82 项，另含本体补充与传统接口；工具箱扩展单列。
- [egui 集成、性能与资源占用约束](docs/EGUI_INTEGRATION_AND_PERFORMANCE.md)：原生嵌入、多遍布局、按需重绘、CPU / GPU 预算与长期验收要求。
- 推荐推进顺序：最小闭环 → 绘图能力 → 定制能力 → 数据性能 → 按需 GPU 加速 → 导出与发布。

尚无导出、LOD / 拾取索引、流式数据、资源硬预算或完整主题继承；拾取 hover 时 O(N)，大量标记仍可能占用较多内存。MATLAB 清单中已有 5 项进入“开发中”，没有图类型通过完整交付门槛；不能声称已交付 133 项能力。Linux CI 已配置，跨平台显示尚未实机验证。

许可证尚未决定，workspace 包均设为 `publish = false`；本项目没有默认授予开源许可。
