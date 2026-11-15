# 阶段 2：基础二维切片与源码分层

日期：2026-11-15。**本轮基础功能切片已通过下述自动检查和 Windows 启动检查；不是全量 MATLAB 图类型交付，也不替代视觉 / 跨平台完整验收。** 仓库仍为私有。

## 已实现与入口

| 领域 | 实现 |
| --- | --- |
| 源码组织 | 七个职责域，widget 只组装；demo 的入口、状态、数据、控制面板、诊断分离；见 [ARCHITECTURE](ARCHITECTURE.md) |
| 坐标轴 | Linear / Log10 任意组合、反向轴、同一变换共享绘制 / 导航 / 拾取；无效 log 坐标断段，缓存按有效 XY 对联合统计 |
| 刻度与文字 | nice 主刻度、对数次网格、自定义主刻度格式器、标题、X / Y 标签和旋转 Y 标签 |
| 等比例 | `data_aspect(1.0)` 令 X/Y 单位屏幕长度相同；log 轴单位为 decade；只扩展视区、不裁掉数据，手动范围也参与约束 |
| 混合序列 | 保留 `show(..., &[Line])`；`show_series(..., &[Series])` 支持折线 / 散点混合叠加，不复制样本 |
| 标记 | 折线标记和 Scatter；7 种实心形状；逐点颜色 / 半径，长度及非有限半径校验；每序列一份批量 mesh，边缘羽化一个物理像素 |
| 基础线型 | Solid / Dashed；屏幕空间长度与间隔，保持一条可见路径中跨顶点的 dash phase；路径断开 / 裁剪入口重置 phase |
| 图例 | 命名序列、稳定显式 ID、单帧切换去重、显隐独立于导航、移除 ID 时清理隐藏状态；重复 ID 有诊断 |
| 数据探索 | 原始点最近邻拾取、返回原始索引和值、十字光标、悬浮提示；隐藏 / 透明 / 零半径 / 无效半径散点不可拾取 |
| 导航 | 对数空间缩放 / 平移、反向轴、独立轴向锁；保留帧初范围，按最终矩形重算，避免多遍重复累加 |
| Demo | 信号和正值幂律场景；实心标记、逐点颜色 / 尺寸、对数 / 反向轴、虚线、等比例、图例、提示和独立双视图 |

### 交互与错误约定

- `PlotState::reset` 只复位导航，不重置图例；图例在数据之后提交，切换后的数据在下一布局遍 / 重绘使用新状态。
- 显式 `.id_salt(...)` 推荐用于重排、重名或重命名序列；未指定时按非空名称或索引推导。重名并不自动获得独立身份。
- `data_aspect` 会扩展手动视区；等比例约束下只缩放一个轴会破坏锁定轴，因此任一 zoom 轴锁定时滚轮不缩放，平移仍可单独锁轴。无法表示的扩展回退自然视区并报告 `aspect_constraint_failed`。
- 非有限点和非正 log 点断线；`domain_invalid_points` 只统计有限但超域的 XY 对，避免与 `non_finite_points` 重复混淆。原始数据不改写。
- 虚线长度 / 间隔必须有限且至少 0.5 egui point；不合法时只跳过 stroke，不跳过标记，并计入 `numeric_rejections`。每序列至多 100,000 条可见 dash，超限报告 `pattern_truncated`，不伪装完整绘制。这只是虚线工作量安全阀，**不是全库资源硬预算**。
- 拾取距离按点中心度量，不是精确的 marker 轮廓命中测试；扫描原始样本，不拾取 LOD 代表点。当前 hover 时 O(N)，关闭 `show_hover` 可避免扫描。

## 验证

验证命令（依赖已缓存，可离线）：

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo build -p customplot-demo --release --locked --offline
```

本轮共 **59 项测试通过**（24 个库单元测试、31 个集成测试、3 个 demo 测试、1 个 rustdoc）。包含数学 / 范围 / 刻度、数据共享与 log 联合域、标记网格合法性与物理像素羽化、虚线 phase 与安全阀，以及下列集成测试：

- [原生 egui 历史回归](../crates/egui-customplot/tests/egui_integration.rs)：12 项，全部保留。
- [混合序列与拾取](../crates/egui-customplot/tests/series.rs)：6 项。
- [图例](../crates/egui-customplot/tests/legend.rs)：3 项，含真实 pointer 按下 / 释放与单遍 / 双遍切换。
- [对数轴](../crates/egui-customplot/tests/log_axes.rs)：3 项，含最终布局改变后的锚点、锁轴与无效手动范围。
- [等比例](../crates/egui-customplot/tests/aspect.rs)：3 项，含线性 / 对数单位、无漂移、锚点和锁轴冲突约定。
- [线型](../crates/egui-customplot/tests/line_styles.rs)：2 项。
- [架构护栏](../crates/egui-customplot/tests/architecture.rs)：职责目录和源码文件规模。
- [全量清单护栏](../crates/egui-customplot/tests/checklist.rs)：82 / 35 / 16 个稳定 ID 和官方总览函数集合。
- demo 数据和无窗口宿主测试 3 项；库级 rustdoc 示例继续运行。

Windows Release 冒烟：窗口标题 `egui-customplot | Stage 2`，响应正常，正常关闭退出码 0。自动无窗口用例运行信号 / 幂律、虚线、标记、等比例及双视图；**不等于手工确认全部视觉效果或真实鼠标手感**。视觉快照、跨 DPI / 窗口 / 多 viewport 组合、Linux / macOS / Web 实机仍需验收。

## 性能证据及限制

- 新增独立 [30 秒 / 场景 CPU 记录](benchmarks/2026-11-15-stage2-windows-cpu.csv)，不覆盖阶段 1 原始文件。
- 同一 CPU 脚本、Windows x64 / i9-14900KF / Rust 1.100.0 / egui 0.37.0，画布 2024×1124、绘图区 1920×1080、1 point = 1 pixel；每场景预热 30 帧。
- 场景仍为空白 egui、空图、10 万点折线静态 / 平移。该记录不包含散点、虚线、拾取性能，也不是 GPU / 窗口呈现或 FPS 测量；背景负载、电源和 GPU / 驱动条件仍不足以构成发布门槛。
- 折线每遍仍扫描全部段；标记每遍重建 mesh；拾取 hover 时扫描全部原始点。没有 LOD、几何缓存、空间索引或流式缓冲。不要把按共享样本去重或一份 mesh 等同于整个流程已达到高性能目标。
- 诊断区分准备 / 拾取、原始样本 payload / 提交几何 capacity，不计进程 RSS、缓存峰值、GPU / 驱动或 present；虚线准备的中间路径也不包含在提交 capacity 中。

## 尚未核销

1. 通用 Theme → Figure → Axes → Series 继承、注解、可定制提示和高级属性面板留在阶段 3；日期 / 类别轴与丰富二维留在后续阶段。
2. 目前为 7 种实心标记，没有空心边框 / 填充独立样式；只有实线与虚线，没有完整 MATLAB LineSpec / 点线 / 点划线兼容层。
3. 刻度密度自适应，但布局仍使用固定上限留白；特别长的格式器输出、小控件和密集 legend 需要更完善的文字测量及视觉回归。
4. 对数点超域会断线；宽动态范围仍受有限 f64 数据跨度和变换可表示性限制。不宣称覆盖任意数值极端。
5. 没有 PNG / SVG，因此 `plot`、`scatter`、`loglog`、`semilogx`、`semilogy` 只能标为开发中，不能按清单门槛标为已交付。
6. 阶段 3～11、整应用资源与全部平台验收未完成，继续按 [PROGRESS](PROGRESS.md) 推进。
