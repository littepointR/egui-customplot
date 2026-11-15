# 阶段 3：可定制化切片

日期：2026-11-15。**主题、样式继承、格式化提示、基础注解及 demo 属性面板已经实现并通过下列检查。** 这是阶段 3 的可运行切片，不是完整 Figure 对象系统、通用 UI 扩展或全量视觉验收。

## 实现与约定

- `PlotTheme::scientific_light()` / `monitoring_dark()`：两套可共享主题，宿主保留 `Arc<PlotTheme>`。默认不指定主题时仍读取当前 Ui 的局部 Visuals / 字体，不修改全局样式。
- 覆盖顺序：egui / 库默认 → Theme → `inherited_style` → Plot `style` / builder → Line / Scatter → 散点逐点覆盖。`Option` 区分未指定与透明色、零宽、零半径、false；多次设置按字段合并，后一次显式值优先。
- `PlotStyle` 覆盖背景、文字、主次网格、边框、十字线及默认序列样式；`SeriesStyle` 覆盖颜色、宽度、线型、标记形状 / 半径和折线标记开关。`.width(...)` 不再意外固定调色板颜色。
- 空主题 palette 回退库调色板；无效 plot stroke 宽度被省略并计入 `numeric_rejections`，不会将 NaN 宽度传入 Painter。
- `HoverInfo` / `hover_formatter`：返回原始点 / 索引 / 序列名与光标坐标；`PlotResponse.hover_text` 可供宿主使用。回调在 Context 锁外、**每个悬停 show 遍**运行，包括被丢弃遍，不是每帧一次的事件；必须保持纯函数、低开销。尚不支持任意 egui UI 提示回调。
- `Annotation::{Text, XLine, YLine, XRegion, YRegion}`：保留并共享 `Arc<[Annotation]>`，按提交顺序在数据后、图例前绘制；不影响自动范围、显隐或原始点拾取。
- 注解继承同一坐标变换及父裁剪；文字锚点必须在视区内才提交，文字自身继续裁剪。参考线的标签跟随线色，默认取主题边框；区域先与数据视区求交，允许无限端点和 log 域外端点。NaN / 逆序区间、无效点域 / 线宽分别拒绝；空 / 透明 / 零面积 / 屏外项目跳过，不作为错误。
- `output_annotations` 按项目计数（带标签的参考线算一项），`annotation_rejections` 单列；它们不是像素覆盖或 GPU 工作量。
- demo 的 `appearance.rs` 单独持有两套主题和固定注解，属性面板可切主题、父默认线宽、局部背景、序列颜色、自定义提示与注解。主图 / 独立副图复用配置，不复制样本或标签。

## 验证证据

共 **73 项测试通过**：24 个库单元测试、45 个集成测试、3 个 demo 测试、1 个 rustdoc。保留阶段 2 全部 59 项，新增：

- [styles.rs](../crates/egui-customplot/tests/styles.rs)：7 项。验证真实 Painter 路径的继承优先级、宽度与调色板独立、空 palette、显式零 / 透明 / false、渲染与拾取一致、逐点覆盖、混合主题不污染宿主、无效 stroke 宽度。
- [tooltip.rs](../crates/egui-customplot/tests/tooltip.rs)：3 项。原始样本及名字、光标无命中、禁用 / 裁剪跳过、自定义回调可读取 Context、多遍回调次数和默认文本。
- [annotations.rs](../crates/egui-customplot/tests/annotations.rs)：4 项。自动范围不变、Arc 释放、对数 / 反向 / 无穷区间、无效值、父裁剪、提交次序和屏外跳过。
- demo 无窗口测试已扩展为信号 / 正值幂律 × 宿主 / 科学浅色 / 监控深色，两视图和注解、虚线、标记、等比例共同运行。

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo build -p customplot-demo --release --locked --offline
```

上述检查通过。Windows Release 窗口标题 `egui-customplot | Stage 3`、响应正常、关闭退出码 0。没有据此宣称全部属性组合的手工视觉验收，也未新增 GPU / FPS 性能结论。

阶段 2 提交 `f795b318c2ac9cd9172695f3d1a9b6b4aad7d985` 的 Windows / Ubuntu CI 已成功：[run 43691054459](https://github.com/littepointR/egui-customplot/actions/runs/43691054459)。该结果不是本阶段提交的 CI 证据。

## 未完成范围

1. 实际 Figure / 多子图对象与布局在阶段 6；`inherited_style` 目前是显式值层，不冒称已实现 Figure 容器。复杂长期对象句柄、任意图元 trait、任意 egui UI 提示 / 菜单、标注编辑 / 选区尚无实现。
2. 固定留白、图例固定位置、长文字布局、字体 / 中文实机、跨 DPI 和跨平台视觉回归仍待完成。注解不参与自动留白或导出，区域目前不带独立标签。
3. 相同数据仍逐遍准备全部线段和标记，hover 仍 O(N) 原始点扫描；样式继承不等于缓存失效机制。下一阶段继续数据查询、LOD、缓存、索引、流式和预算。
4. 五类注解仅使 EX-026 / 028～031 进入开发中；没有任何条目越过含性能 / 导出的完整交付门槛。完整 133 条清单继续保留。
