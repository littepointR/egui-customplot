# 源码组织与依赖边界

本文件约束**已经落地**的目录，不是将来再整理的愿望清单。当前一个库 crate 加一个 demo，公开类型统一从 `egui_customplot` 根导出；模块迁移不要求用户更改导入路径。

## 目录与职责

```text
crates/egui-customplot/
  src/
    lib.rs                    # 库说明、私有模块声明、公开导出
    diagnostics.rs            # 跨阶段诊断契约，不执行绘制或采样
    data/
      points.rs               # Arc 样本、坐标域联合范围缓存
    model/
      line.rs                 # 折线的借用描述与 builder
      line_pattern.rs         # 线型语义与参数合法性
      scatter.rs              # 散点描述与逐点样式校验
      marker.rs               # 标记形状与尺寸
      series.rs               # 混合序列、稳定 ID、默认色
    axis/
      scale.rs                # Linear / Log10 数值域
      bounds.rs               # f64 范围验证、适配与基本运算
      transform.rs            # 正反坐标映射、变换空间导航运算
      aspect.rs               # 等比例约束，只扩展变换空间范围
      ticks.rs                # 坐标轴配置、主次刻度、数字格式
    layout/
      plot.rs                 # 宿主空间分配、标题 / 轴标签留白
      legend.rs               # egui 图例排布和控件组装
    interaction/
      state.rs                # 显式视图、隐藏状态、帧快照、一次性切换
      navigation.rs           # 输入仲裁、锁轴、同帧多遍导航重算
      picking.rs              # 原始点拾取、提示内容（不构建数据 mesh）
    render/
      line.rs                 # f64 线段裁剪与路径准备
      markers.rs              # 标记批量 mesh 与物理像素羽化
      pattern.rs              # 屏幕空间虚线 phase 与工作量安全阀
      axes.rs                 # 网格、刻度文字、标题、边框绘制
      series.rs               # 按序提交数据图元、统计几何成本
      overlay.rs              # 十字线和拾取高亮
    widget/
      mod.rs                  # Plot 公开配置 builder
      show.rs                 # 每遍生命周期组装，不包含几何 / 输入算法
      response.rs             # 宿主可读取的本遍返回值
  tests/
    support/                  # 无窗口 egui 测试共用驱动
    architecture.rs           # 源码目录与文件规模护栏
    egui_integration.rs       # 首轮原生嵌入 / 导航回归
    series.rs                 # 混合序列、批处理、拾取与显隐
    legend.rs                 # 稳定身份、单次切换和状态回收
    log_axes.rs               # 对数 / 反向轴和多遍布局
    aspect.rs                 # 等比例、幂律单位、锚点与锁轴
    line_styles.rs            # 基础线型与参数拒绝
    checklist.rs              # 133 条 ID 和官方快照完整性
  benches/
    frame_baseline.rs         # 只测 CPU 构建和 tessellation
examples/demo/src/
  main.rs                     # eframe 启动，不放业务功能
  app.rs                      # 应用模型与图表组装
  controls.rs                 # 可调参数和控制面板
  data.rs                     # 固定、可复现的示例数据
  diagnostics.rs              # 诊断展示，不实现测量算法
docs/                         # 规范、阶段证据、原始基准
```

每个源码域以 `mod.rs` 声明与聚合本域实现。只创建实际需要的模块，不建空的 GPU / 导出 / 3D 占位目录，也不按未来功能提前拆大量 crate。

## 数据流与约束

1. `axis` 定义坐标语义；`data` 仅依赖坐标语义计算有限值 / 对数域联合缓存；`model` 借用样本并描述样式，不知道 widget、egui Memory 或渲染资源。
2. `interaction` 管理输入、视图和显隐，不创建数据几何；`layout` 使用宿主 Ui 安排图与图例。图例的点击由 `PlotState::toggle_series` 去重，不能在每遍直接反转布尔值。
3. `render` 借用模型、变换和只读显隐状态，生成 / 提交 Painter 几何；不得反向消费输入或修改导航状态。几何准备与拾取独立计数、计时。
4. `widget` 依次分配 → 汇总缓存范围 → 导航 → 刻度 → 有序绘制 → 图例 → 悬停，连接各域；不是收纳所有逻辑的“大控件文件”。
5. `diagnostics` 是共用返回契约，不是资源所有者。sample payload、geometry capacity、耗时均明确计量范围，不冒充进程 / GPU 峰值。
6. 样本和长期配置由宿主持有；`PlotState` 只保存视图 / 显隐与小型帧快照，不保存数据引用。库不依赖 eframe；demo 才负责原生窗口。

图例在数据之后提交，保持图层次序；切换后请求重绘，数据在下一布局遍 / 重绘使用新显隐状态。多遍返回值是本遍快照，不是可重复向外发布的事件队列。

## 扩展规则

- 数据索引、LOD、流式和数据版本放 `data/`；预算 / 淘汰围绕实际资源所有者设计，不塞入样式模型。
- 新图类型的参数和样式放 `model/`，几何放 `render/`，拾取策略放 `interaction/`，不要把整种图的实现堆进 `widget/show.rs`。
- 新轴语义必须被刻度、绘制、导航和拾取共享。不可在某个后端单独对数据取 log 或反向。
- 主题继承属于模型 / 样式域，子图空间分配属于 `layout/`；导出有第二个真实后端时，再稳定公共场景 / 绘制命令边界。
- 避免 `utils.rs` / `common.rs` 等无领域名称。300 行左右评估职责；`tests/architecture.rs` 拒绝超过 400 行的库源码。不能用少换行规避；职责比行数重要。
- 所有公共类型继续经 `lib.rs` 导出；使用受限可见性，不以 `pub mod` 暴露内部目录作为稳定 API。
- 自动检查规模不代替代码审查和行为测试；新增阶段仍须保留旧测试并补该域的正确性、集成和资源验证。

## 验证入口

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p customplot-demo --release --locked
```

阶段证据和未完成能力见 [PROGRESS](PROGRESS.md)。目录整理不等于阶段 3～11 或 MATLAB 133 项能力已交付。
