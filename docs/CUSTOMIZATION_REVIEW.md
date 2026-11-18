# 可定制性与模块替换审查

2026-11-19。审查对象为 `main` 在 `5ce698c` 上的本地工作树，含未提交的高清导出、图例、游标、阶跃响应及演示变更。按 [架构目标](ARCHITECTURE.md) 跟踪宿主定制从公开入口到布局、交互、显示与导出的路径。盘点覆盖库的七个域、demo、示例、测试、基准和 CI；对照关键调用链核查，并另行通过公开入口复现了两项高优先级问题。

下表记录改动前的审查基线。实施后的接口、验收与剩余边界见 [开发进度](PROGRESS.md#组件替换与三维坐标)。

## 可复现的故障与配置失效路径

| 用户操作 | 当前行为与代码依据 | 调整与回归判据 |
| --- | --- | --- |
| **P1，已复现**。调用 `Plot3D::new("compact").height(32.0).title("T").show(ui, &mut Plot3DState::default(), &[])` | 标题与底边共预留 36 个逻辑点，剩余视口无法构造变换。回退分支继续使用同一矩形，随后 `expect("default 3d transform")` 触发 panic。[视口布局](../crates/egui-customplot/src/layout/plot3d.rs) · [变换与回退](../crates/egui-customplot/src/widget/plot3d.rs) | 由布局统一裁减装饰或报告视口不足，导航与绘制共同处理退化帧。通过带标题的 32、36、37、38 高度和窄宽度验证边界。 |
| **P1，已复现**。给 3D 图的 X 轴设置 `AxisScale::Probit`，传入 X 为 `[0.01, 0.99]`、Y 为 `[1, 2]`、Z 为 `[5, 6]` 的两个点 | X 数据全部有效，但原数据空间的 6% 留白会越过 `(0, 1)`。创建变换失败后，三个轴一起回退到默认线性范围，Y/Z 的读数也丢失。[范围计算](../crates/egui-customplot/src/layout/plot3d.rs) · [整图回退](../crates/egui-customplot/src/widget/plot3d.rs) | 将有效域、留白与退化范围放入轴契约。逐轴在变换空间留白，单轴失败只处理该轴。验证接近两端、单点、夹杂无效样本时的范围、投影与拾取。 |
| 给 3D 线设线宽，给点设方形，再切到深度缓冲 | 排序绘制消费 `Stroke` 和标记形状。软件深度路径的 `raster_seg` 只取颜色，每步写一个缓冲像素，`raster_marker` 用圆盘覆盖。放大导出时缓冲分辨率还会改变线的逻辑粗细。[排序绘制](../crates/egui-customplot/src/render/plot3d/scene.rs) · [光栅化](../crates/egui-customplot/src/render/plot3d/zbuffer/raster.rs) | 两条路径消费同一份已解析线宽和形状，按实际缓冲比例换算逻辑点。用水平粗线和方形标记检查两种深度模式、1×/3× 输出的线宽和角点。 |
| 在全局关闭网格，或为 3D 图使用与宿主相反的文字颜色 | `Plot3D::show` 将全局网格设置解析到临时变量 `dummy_grid`；绘轴只读逐轴网格配置。3D 色标接收色阶和刻度，其文字取色直接读取 `ui.visuals()`。[组装](../crates/egui-customplot/src/widget/plot3d.rs) · [轴](../crates/egui-customplot/src/render/plot3d/axes.rs) · [色标](../crates/egui-customplot/src/render/color_bar.rs) | 将已解析的网格开关及图级、元素级样式传给布局与绘制。保留独立色标控件使用宿主主题的默认行为。验证全局关闭、单轴关闭、图主题与宿主主题相反三例。 |
| 配置 `.label("Z").visible(false)`；反转轴后旋转盒体 | 隐藏轴会清空刻度并关闭该轴网格，渲染器仍按非空标题绘字。`back_face_unit` 按反转取负，相机的 `project_unit` 已经接收定向后的盒坐标。同一相机下反转数据方向会改变所选物理背面。[轴配置](../crates/egui-customplot/src/axis/plot3d/mod.rs) · [投影](../crates/egui-customplot/src/axis/plot3d/camera.rs) · [背面与标题](../crates/egui-customplot/src/render/plot3d/axes.rs) | 明确整轴显隐及单元素覆盖的语义。按投影深度识别物理背面，按轴方向解析固定的数据端点。分别测试标题、刻度、网格以及同一相机下的前后面。 |
| 播放未结束时悬停在 3D 彗星轨迹的后半段 | 绘制按播放进度截取样本，拾取枚举整条轨迹，尚未画出的点也会参与命中。[绘制](../crates/egui-customplot/src/render/plot3d/layers.rs) · [拾取](../crates/egui-customplot/src/interaction/plot3d/hover.rs) | 让图层一次给出当前帧的可见样本范围，由绘制与拾取共用。验证中途播放时头部可拾取，未来样本不可拾取。 |

## 哪些调整能让宿主真正替换实现

| 部分 | 当前可用入口与阻碍 | 建议的职责边界 |
| --- | --- | --- |
| 轴、刻度与坐标变换 | `Axis3D` 可设 formatter 与显式刻度，但 `paint_axes_3d` 同时决定选边、间距、字体和绘制。`AxisScale` 的正反变换为内置枚举，部分数据范围摘要又以两个轴是否为 Log10 编出四个缓存槽。[轴比例](../crates/egui-customplot/src/axis/scale.rs) · [范围摘要](../crates/egui-customplot/src/data/extents.rs) | 先开放每轴选边、文字布局及绘制，向布局传投影几何与文字测量器。后续引入自定义比例尺时，一起约定有效域、正反变换、默认范围与缓存版本。保留内置轴的四槽快路径，自定义域可由数据源提供摘要。自定义 3D 投影同时提供深度和拾取射线。 |
| 位置、装饰与主题 | 二维图例按文字测宽，3D 按画布比例分配图例矩形；导航另行接收矩形排除输入。`PlotDefaults` 的部分布尔值是全局开关，颜色类样式则已有逐层覆盖语义。[二维图例](../crates/egui-customplot/src/layout/legend.rs) · [三维布局](../crates/egui-customplot/src/layout/plot3d.rs) · [默认设置](../crates/egui-customplot/src/model/defaults.rs) | 先让自定义图例提供测量、布局、命中区和绘制，用同一布局产物驱动留白与输入。标题、色标、注记再逐项复用。对每个字段区分「继承值」与「强制禁用」；现有全局控制保留原有优先级。 |
| 数据源与图层 | 工作树的 `PlotLayer` 有 57 个枚举变体，`Layer3D` 有 29 个。内置路径从图层分派到范围、样式、图例、绘制、拾取与缓存；宿主无法把自己的变体送进去参与全流程。[二维图层](../crates/egui-customplot/src/model/layer/mod.rs) · [绘制分派](../crates/egui-customplot/src/render/series.rs) · [三维缓存](../crates/egui-customplot/src/render/plot3d/zbuffer_key.rs) | 保留现有枚举及入口，以借用的宿主图层接入混合列表。契约包括稳定 ID、数据/样式版本、坐标域内的范围、批量准备与绘制，以及可选拾取和图例。出现自定义样式或变换时应有可靠的缓存失效方式。先用一个带测量区间的图层走通全程，再推广到 3D。 |
| 导航、拾取与组合响应 | `SharedCursors::show` 同时读取鼠标修饰键、更新 pin 并绘制。复合或专用图转换成 `PlotResponse` 时 `from_embedded` 会构造 `[0,1]²` 线性变换。例如 Bode 保有真实 `mag`、`phase` 响应，扁平转换后的变换不足以读取频率。[共享游标](../crates/egui-customplot/src/interaction/cursors.rs) · [响应适配](../crates/egui-customplot/src/widget/response.rs) · [Bode 子图](../crates/egui-customplot/src/widget/bode.rs) | 把公共输出限定为矩形、命中与诊断；数据读数使用相应子图的真实变换。保留旧适配器，另提供类型明确的子视图入口。将按键绑定、动作应用及覆盖层绘制分开提供默认实现。对一次输入触发多遍布局的情况，动作最多提交一次，几何可重算。 |
| 后端与导出 | `render_at` 已支持目标密度重绘。快照捕获拒绝任意 GPU callback 与非默认图集纹理。目前内置路径可识别捕获状态，宿主缺少同等的绘制目标接口。[重绘入口](../crates/egui-customplot/src/render/export/target.rs) · [捕获边界](../crates/egui-customplot/src/render/export/capture.rs) | 随绘制批次传入绘制目标、裁剪区、密度及支持的能力。宿主通过一个可选适配器提交导出替代图元；无适配器时报告具体不支持的图元。沿用现有捕获预算与设备所有权。 |
| 高阶图表与 demo | `FigureCell::ui` 已允许宿主放入任意 UI，这是可复用的入口。`StepResponsePlot` 内部创建 `Plot`、轴、标记文案和曲线，使用者只可单独设高度；Bode 则在 `show` 中直接计算子图矩形并绘标题。[FigureCell](../crates/egui-customplot/src/widget/figure/cell.rs) · [阶跃图](../crates/egui-customplot/src/widget/step.rs) · [Bode](../crates/egui-customplot/src/widget/bode.rs) | 保留一行调用的预设，开放具名子图、曲线和标注的配置入口。demo 用公开接口实现一个自定义场景，同时产出截图、测量结果和导出文件。 |

## 迁移顺序与交付条件

1. **校正已有设置。** 优先修复紧凑视口崩溃和 Probit 导致的整图坐标回退。随后给深度缓冲线宽和形状、整轴显隐、网格与色标、动画可见范围补针对性回归并修复。这几项可独立交付。
2. **以 Z 轴打通第一个替换点。** `axis` 持有逐轴选项，`layout` 负责投影后选边、测字、避让及对齐，`render` 按布局画，`interaction` 持有上帧选边。内置自动、固定棱及宿主自定义共用入口。验收覆盖正交/透视、反向、平移、俯视、临界角往返、重试和 1×/3× 导出；单独替换 Z 时 X/Y 保持原策略。
3. **迁移装饰与交互。** 先在二维和三维图上使用同一个自定义图例，证明长文本可测量、窄图可裁剪，点击隐藏只发生一次且不会带动平移。再把图例的测量和输入约定推广到色标及组合图标题，把真实子视图提供给测量工具。
4. **打通宿主图层和导出。** 用 crate 外代码实现一个可拾取、可进图例的图层，和内置图层混排；测试自动范围、重排序、数据版本变化及 PNG/SVG 导出。验证有、无导出适配器两条路径。保留 crate 根导出、现有 `show*` 签名和已公开枚举的使用方式。
5. **推广到数据源、投影和专用图。** 先以一个具有不同有效域的比例尺、一个需要反投影的视图验证接口，再用于地理、极坐标等图族。用现有千万点流基准比较默认路径和一个自定义批次，记录 CPU 构建与缓存指标并另存结果。动态分派限于每图层或批次，宿主继续持有原始样本、相机状态与设备。

## 验证入口

现有 `architecture` 测试覆盖目录和文件长度；`styles`、`legend`、`figure`、`export_layout` 和 `time_measurements` 覆盖部分默认路径。后续每个替换点的验收都应通过 crate 根的公开接口运行。在同一张图上保留一项内置行为并替换另一项，比单独测试策略函数更能发现调用链丢失配置。

本次执行 `cargo test --workspace --all-features --locked --offline`，408 项通过，0 项失败，2 项按项目标记忽略（重写 README 配图、需要原生计算适配器的 wgpu 测试）。上述六个针对性测试目标也分别运行，共 19 项通过，已包含在工作区总数中。另用两个临时集成用例从 `Plot3D::show` 入口复现表首两项故障，均未计入上述 408 项；用例只用于本次审查。其余失效路径为静态核查结果，建议的像素对比、扩展接口和性能对比属于后续验收。
