# 阶段 6：多面板与数值二维图元切片

日期：2026-11-15。**基础实现与专项测试通过，完整验收未完成。**

## 容器、联动和双轴

- `Figure` 是即时网格布局，1～64 列、最多 256 cell；宿主分别持有每个 `PlotState` 和样本。父裁剪、方向 / justified 布局及显式主题 / 样式继承均经测试；尚无跨行列 cell、公共图例 / 标题的完整布局系统。
- `AxisLink::synchronize(ctx, &mut [&mut PlotState])` 在每一遍显示全部成员后显式调用。自动并集、导航及整组 Home 不重复应用；选中轴 scale 不同、等比例约束、不同 Context / viewport 或过期视图均返回错误，不部分应用。
- 联动不擅改宿主 pass limit；剩余布局遍不足时请求下一帧收敛。`PlotState::bounds()` 是本地手动范围；实际共享范围读取 `PlotResponse::transform`。
- `Plot::right_y_axis(Axis)` 配合 `.y_axis(YAxis::Right)`：共享 X，独立 Y 范围 / scale / 方向；同一屏幕导航应用两侧。未配置 Right 时回退 Left 并诊断。AxisLink 的 Y 只联动主轴；右轴等比例 / 右轴跨图联动不在当前契约内。

## 兼容的异构图层入口

`show` / `show_series` 及缓存版本仍可用，`Series::data()` 契约不变。新增非穷尽 `PlotLayer`，通过 `show_layers` / `show_layers_cached` 和 FigureCell 同名入口提交有序图层。缓存仍只保留折线坐标，不保留网格、柱图、误差棒或样本引用。

## 数值网格和色条

- `GridData` 共享不可变 X / Y 边缘和行优先值；边缘有限且严格递增，row 0 为最小 Y。`uniform` 在数据空间生成边缘，精度塌缩报错。NaN / Inf 单元省略，保留原始行列。
- `Heatmap` 是分片常量数值 cell：不插值、不自动平滑 / LOD。可见边缘二分定位，mesh 最多 16,384 cell；log 坐标跨非正域的整个 cell 省略，不把它发明成部分有效单元。
- `ColorScale` 显式共享给 Heatmap / 独立 `ColorBar`。linear / log 色域与坐标域独立；有限越界值 clamp，非有限及 log 非正值不绘制 / 拾取。Viridis / Gray / BlueRed 是不透明 sRGB 分段插值，不宣称感知均匀。
- `HoveredCell` 返回原始行、列、值及数据范围。父裁剪、改色图、log / 反向 / 右轴、显隐和释放均已覆盖。
- 这对应 `imagesc` 数值切片，**不是 MATLAB 类别 / 表格聚合的 `heatmap`**，也不是顶点着色语义的 `pcolor`。

### CPU 基准

[原始记录](benchmarks/2026-11-15-stage6-heatmap-windows-cpu.csv)：i9-14900KF / Windows 10.0.26300 / Rust 1.100.0 / egui 0.37.0，Release bench，1920×1080 / DPI 1。每场景预热 5 帧后 30 秒；同机并发流式压力和开发，非隔离发布门槛。

| cell / 视区 | 测量帧数 | UI p95 ms | tess p95 ms | 合计 p95 ms | 访问 / 输出 cell | mesh | geometry capacity B |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 65,536 全区 | 4,645 | 4.8923 | 4.6540 | 9.1704 | 65,536 | 4 | 6,815,744 |
| 1,048,576 窄区 | 1,202,558 | 0.0340 | 0.0076 | 0.0419 | 289 | 1 | 53,248 |
| 1,048,576 全区 | 283 | 60.7130 | 77.1864 | 134.8719 | 1,048,576 | 64 | 109,051,904 |

65,536 / 1,048,576 cell 样本 payload 为 528,400 / 8,405,008 B；构造含输入生成分别约 0.7339 / 13.3568 ms（完整重复记录见 CSV）。合计分位数来自逐帧合计，不能把分项 p95 相加代替。全区百万 cell 成本高，不能声称达到实时帧率。以上均无 GPU / FPS 结论。

## 柱图、误差棒与直方图

- `BarData` / `Bar` 共享原始记录；`Bars` 提供垂直 / 水平、数据空间宽度、独立基线、填充和可选边框。正负值均支持；零面积柱不绘制 / 拾取。无效宽度、非有限 / 溢出记录保留原始索引但省略并计数。
- 构造时缓存两方向 × 四坐标域范围；显示汇总不扫描原始样本。整柱跨 log 非正域时整项省略，包括默认零基线；log 轴需显式正基线。
- 裁剪在 f64 中先完成；只绘制真实边框，不在视区交点创建假边。逐条提交 fill → border → 下一柱，保持透明混合层序。
- `ErrorBarData` / `ErrorBar` 的 `x_error([lower, upper])` / `y_error` 是非负偏移，不是端点或置信水平。`ErrorBars` 不连接中心线；端帽宽度使用逻辑点。裁剪不在 stem 交点创建假端帽。无效 cap 只禁用 cap；无效记录 / 坐标域拒绝整项。原始中心通过 `HoveredPoint` 返回。
- `HoveredBar` 返回原始柱记录 / 原始索引 / 方向与轴。最后提交的命中填充层（Bar 或 Heatmap）遮挡前面的点层，之后点层仍按最近距离 / 平距先提交规则仲裁。同层重叠柱最后一条优先。透明柱内部不捕获；仅边框时只拾取真实边。旧 HoverInfo 格式器可读柱顶原始点。
- `HistogramData` 是显式、同步、宿主调用的预处理，绝不每遍自动分箱。边界 `[left,right)`、最后一箱含右边；非有限 / 越界单独计数。支持 Count、Probability、Density、CumulativeCount / Probability；归一化分母为范围内样本，空集返回零。`to_bars` 发布独立 BarData，不能表示的派生数值显式报错。
- 暂无分组 / 堆积 / 类别柱、自动分箱估计器、类别直方图、误差量统计推断、柱 / 误差棒索引或 LOD。Bars / ErrorBars 绘制和拾取仍扫描原始记录；没有该类图的正式性能 / 导出验收。

## 可运行案例与回归证据

```sh
cargo run -p customplot-demo --release --locked -- --scalar
cargo run -p customplot-demo --release --locked -- --statistical
```

默认案例支持 Figure / X 联动 / 双 Y；scalar 展示色图、log 色域 / 坐标、反向 Y 和折线覆盖；statistical 展示固定 4000 观测的显式分箱与正负柱 / 非对称误差棒。

- `tests/figure.rs` 4 项，`linked_axes.rs` 4 项，`dual_y_axes.rs` 6 项，`dual_y_navigation.rs` 2 项。
- `grid_data.rs` 3 项、`heatmap.rs` 5 项、`heatmap_lifecycle.rs` 3 项、`color_bar.rs` 2 项。
- `bars.rs` 6 项、`error_bars.rs` 5 项、`histogram.rs` 3 项、`primitive_lifecycle.rs` 3 项；包含原始索引 / 样式 / 无效输入、巨大视区交叉、层序、共享去重、重复释放和多遍。
- demo 的 scalar / statistical 与双轴联动也有无窗口多遍测试；这不能替代所有组合的人工视觉验收。

完整 workspace、Clippy、Release 和窗口启动的最新批次见 [进度](PROGRESS.md)。导出、固定硬件资源验收、跨平台视觉和清单中剩余图类型仍未完成。
