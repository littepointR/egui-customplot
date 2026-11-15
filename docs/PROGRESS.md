# 全量实施进度

用户目标：完整完成项目计划并验证。此表用于跨轮次持续推进，不能替代逐项测试证据。

| 阶段 | 当前状态 | 证据 / 缺口 |
| --- | --- | --- |
| 0 规划 | 已建立 | PLAN、133 项 MATLAB 对照、EGUI / PERF 约束 |
| 1 最小闭环 | 原型验证通过 | STAGE_1：21 项测试、Windows 启动、30 秒 CPU 基线；整应用资源测量仍缺 |
| 2 基础二维能力 | 基础切片验证通过，完整验收未完成 | STAGE_2：线性 / 对数 / 反向 / 等比例、散点 / 标记 / 实线 / 虚线、刻度文字 / 图例 / 拾取；自动检查与 Windows 启动通过，视觉 / 跨平台完整验收仍缺 |
| 3 可定制化 | 切片验证通过，完整验收未完成 | STAGE_3 历史 73 项检查保留；已补 Figure 样式继承；任意 UI 扩展及完整视觉回归仍缺 |
| 4 数据性能 | 基础切片验证通过，完整验收未完成 | [STAGE_4](STAGE_4.md)：显式索引 / Visible / MinMax / 原始拾取 / 有界宿主缓存 / 环形数据、三档数据各场景 30 秒 CPU 原始记录；限定旧版 headless 1800 秒过载压力与局部预算断言通过；GPU / 总资源 / 增量发布仍缺 |
| 5 GPU 决策 | 已记录暂缓决策，完整验收未完成 | [STAGE_5](STAGE_5.md)：保留 Painter，明确热力图全区 CPU 风险和重新评估条件；尚无同条件 wgpu / GPU / 呈现证据 |
| 6 丰富二维能力 | 基础切片验证通过，完整验收未完成 | [STAGE_6](STAGE_6.md)：Figure / AxisLink / 双 Y、数值 Heatmap / ColorBar、Bars / ErrorBars、显式分箱 HistogramData、专项与 demo 多遍测试；显示快照导出见阶段 7，分组 / 堆积 / 类别和完整视觉 / 资源验收仍缺 |
| 7 导出交付 | 显示快照切片验证通过，完整验收未完成 | [STAGE_7](STAGE_7.md)：可选 export feature、PNG / 混合 SVG、宿主字体 atlas、裁剪 / 透明度 / 分辨率 / DPI、局部限制与 demo 防重复写盘；真实像素 / XML / 无外部字体 resvg 及 Windows 窗口生命周期回归通过，独立印刷布局 / 全矢量字体及平台完整验收仍缺 |
| 8 二维类型补全 | 未完成 | 统计、多变量、函数、极坐标、几何、网络 |
| 9 三维与体数据 | 未完成 | 三维轴、相机、深度、曲面、体、流 |
| 10 地理绘图 | 未完成 | 投影、地图资源策略及四类本体地理图 |
| 11 全量核销 | 未完成 | ML / EX / LG 与适用 CAP / EGUI / PERF 逐项证据 |

开放范围与外部依赖：工具箱扩展需冻结产品清单；许可证未确定，因此不发布 crates.io 或宣称开源授权；平台实机与固定硬件性能结论不能由普通 CI 代替。先用现有默认值完成可实现部分，不以这些开放项阻止代码开发。

## 源码结构约束（已落实）

- 七个职责目录已经落地，`widget/show.rs` 只负责组装；公开类型仍经 crate 根导出。目录、数据流和扩展位置见 [ARCHITECTURE](ARCHITECTURE.md)。
- `AGENTS.md` 固化后续开发约定；`tests/architecture.rs` 检查目录及 400 行源码上限，避免回退到散乱大文件。
- demo 拆分入口、应用、控制面板、数据、诊断；MATLAB 清单保留 133 条稳定 ID，并新增自动完整性检查。

最近基础批次为 workspace 148 项、fmt / Clippy / Release demo 全通过；Windows 三个 gallery 启动 / 响应 / 正常关闭均通过，见 [阶段 6 窗口记录](benchmarks/2026-11-15-stage6-windows-gallery-smoke.txt)。新增导出后，完整 all-features 159 项已通过，另加 2 项采样 / 非整数尺寸测试与严格 all-features Clippy 也通过；最终统一批次正在复验。1800 秒旧版 headless 压力已汇总且边界保留，详见 STAGE_4。所有阶段仍按清单保留完整验收缺口。

上述 148 / 159 项为历史批次。阶段 7 最终统一批次 **161 项通过**（库单元 32、集成 118、demo 9、rustdoc 2）；fmt、无默认 feature 库检查、严格 all-targets / all-features Clippy、Release demo 构建同批通过。最新阶段 7 binary 的三个 Windows gallery 启动 / 响应 / 正常关闭均通过，见 [新窗口记录](benchmarks/2026-11-15-stage7-windows-gallery-smoke.txt)；不代表跨平台视觉、GPU 或性能验收。

下一阶段继续补阶段 8 阶梯线等二维类型，同时保留阶段 4 增量 / 总预算及阶段 6 / 7 完整交付缺口。133 项对照持续保留；工具箱、跨平台实机和许可证等开放边界不以本地 CPU 检查冒充已解决。
