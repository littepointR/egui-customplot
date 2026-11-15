# 阶段 4：索引、LOD、缓存与有界流式切片

日期：2026-11-15。**本阶段仍未完整验收。** 当前实现单调 X 数据的显式索引、可见查询、可选 min/max LOD、原始点索引拾取、宿主持有的有界折线缓存及批量环形缓冲。已完成限定范围的 30 分钟 headless 压力；CPU 结果不能替代 GPU / 整应用资源和跨平台视觉验收。

## 数据与查询契约

- `PlotData::build_line_index(budget_bytes)` 是显式同步预处理；库不会在 `Plot::show` 中偷偷建索引。样本仍归宿主，不进入 `PlotState`。
- 每个有限 XY run 内 X 必须非递减，允许重复 X、常量 X，以及 NaN / Inf 断段后重新开始。仅有 log 域断段不允许在有限 XY run 内重启 X。乱序返回 `LineIndexError::UnsortedX`，原始数据仍可用 Exact 绘制。
- 索引包括四种 linear / log 域组合的有效 run 表和 64 点叶块的 Y min/max 树。查询逐 run 二分，范围保留两端相邻点，跨视区且端点均在外的线段不会消失。
- 建立前计算所需字节，再使用 `try_reserve_exact`，检查实际 Vec capacity；预算不足返回错误，不替换现有索引。预算仅约束单份索引 payload 与结构体元数据，不包括 Arc / 分配器头、样本、其他快照和临时绘制。
- clone 共享已建立的不可变索引；`clear_line_index` 只释放当前持有者的引用。诊断对样本与索引分别按身份去重。
- `PlotData::version()` 是进程内的不可变快照版本。clone 共享版本，新构造产生新版本；不使用可能被重用的裸地址充当跨帧缓存版本。

## 折线路径与拾取

| 策略 | 语义与成本 |
| --- | --- |
| `LineSampling::Exact`（默认） | 参考路径逐段访问，不要求有序 X 或索引 |
| `Visible` | 索引筛出有效 run / 可见 X 邻域，再走相同的精确裁剪；仍可能输出所有可见点 |
| `MinMax` | 显式近似，每个物理屏幕列保留首 / 尾 / Y 极值，保持原始索引顺序；每 run 列数最多 16,384，不连接域断段 |

`Visible` / `MinMax` 无索引时回退 Exact，并增加 `lod_fallback_series`，不静默缺图。MinMax 不是每一条原始线段的精确替代；不保证子像素路径相同或保留列内全部振荡。虚线应用于所选路径，不能将它的相位解释成未精简原始路径的弧长。

树查询每列最多扫描两个部分叶块，其余使用树节点；当数据量小于列数 × 叶块上限时，扫描量仍可能接近原始长度。**总成本还包含 run 数。** 多断段 / 大量相同 X 的数据可能仍输出 / 扫描很多点，不能声称始终 O(屏幕宽度)。

拾取始终返回原始点 / 原始索引，与渲染 LOD 独立。有索引时按光标 X 邻域筛候选，最后仍执行相同的坐标域、样式、屏幕范围和精确距离过滤，保持最近点及平距时原始提交顺序。查询含屏幕舍入保护区；反变换溢出会保守扩大候选而非漏选。它**不是二维空间索引**：重复 X / 稠密列和 break-heavy 数据仍可能访问大量点。没有索引时继续全量参考扫描。

折线标记与 Scatter mesh 仍逐遍扫描原始点；MinMax 不会擅自减少标记或改变逐点颜色 / 半径。

## 有界缓存

`PlotCache::new(budget_bytes)` 由宿主持有，通过新增的 `Plot::show_cached` / `show_series_cached` 传入；原 `show` / `show_series` 保持无缓存默认行为。

- 缓存的是**裁剪后、线型 / 颜色 / 宽度应用前**的折线坐标，不是整个 egui 图层。键包含快照版本、最终视区 / 矩形、DPI、轴类型 / 方向、采样模式和是否有索引；同一数据的不同视图不会误用。
- 颜色 / 宽度 / 虚线 / 标记属性在缓存后应用；改样式可命中但必须立即产生新的绘制样式，不需要扫描原始数据。新版本、视区、布局 / DPI、轴或采样改变会重新准备。
- LRU 淘汰；最多 64 个条目；超预算结果正常绘制但拒绝保留，不为一次超大结果先淘汰现有缓存。`set_budget` 立即收缩，`clear` / drop 释放几何。
- 计费包括占用条目元数据与保留 Vec capacity，不包括 VecDeque 空闲槽、分配器头、提交 egui 的临时副本、标记、tessellation 或 GPU 资源；不是进程总内存预算。
- 命中仍需为 egui 复制坐标、应用虚线、提交和 tessellation，不等于零分配、零 GPU 上传或零整帧成本。缓存不持有样本，旧样本可独立释放；旧几何由预算 / LRU / 显式 clear 管理。

## 有界流式数据

`PlotStream::new(capacity_points, budget_bytes)` 只在构造时分配固定样本存储。`append_once(batch_id, points)` 批量追加：

1. 宿主提供单调递增批次 ID；重复最后一个 ID 不做操作，旧 ID 返回 `StaleBatch`，不修改数据。重复 ID 的内容不重新比较。
2. 超容量采用明确的**覆盖最旧**策略；超大批次只保留尾部。`AppendReport` 分开报告移除的旧点和丢弃的输入前缀，不伪装成无损背压。
3. `as_slices` 无分配地提供顺序切片；`snapshot` 显式 O(N) 复制为新的不可变 `PlotData`，没有预建索引。发布和建索引应合并到宿主数据更新节奏，而不是每个 show / 布局遍重做。
4. 旧快照由其持有者控制；环形缓冲预算**不包含快照**。本切片没有追加型分块数据源、增量索引 / GPU 上传、后台生产队列或全局快照预算，不能宣称小批追加只更新新增数据。
5. `clear` 保留批次水位，避免已消费批次在布局重试后重新应用；空批次消耗批次 ID，但不增加样本修订号。库不主动请求 repaint。

Demo Live 模式每 tick 追加 256 点、保留 65,536 点，按约 30 Hz 合并快照和可选重建索引；同帧多遍保持相同数据版本。定时 repaint 只属于显式开启 Live 模式的宿主示例。

## 诊断与验证入口

新增分账：索引字节、query run / 比较 / 节点 / 叶块扫描 / 选点、候选与选点 Vec capacity、索引拾取工作、缓存 hit / miss / 拒绝 / 保留字节，以及独立 query 时间。`prepare_duration` 不再重复包含 query 时间。capacity 不是 allocator 瞬时峰值；各序列求和也不等于同时驻留峰值。

- `data/index/tests.rs`：四域 run 表、重复 / 重启 / 乱序、预算边界与失败保留、空 / 单点 / 无效、树与原始扫描、保序 / 保峰 / 断段、DPI、视区邻点及大量断段的真实成本。
- `tests/line_sampling.rs`：Visible 与 Exact 的真实 Painter 路径相同，覆盖多域、反向、窄范围 / 边界、多遍；fallback、索引去重、百万点算法计数、标记不被 LOD 隐藏。
- `tests/indexed_picking.rs`：索引 / 全量拾取对照，覆盖 log、反向、重复 X、NaN、Scatter、LOD 和屏幕边界舍入。
- `tests/line_cache.rs`：命中保留几何且跳过查询 / 裁剪，样式立即生效，数据 / 视图 / 布局 / DPI / 轴 / 采样 / 索引失效，预算收缩、超限不保留、重复创建释放。
- `tests/stream.rs`：环形顺序与参考容器对照 10,000 批次、覆盖计数、快照不可变、NaN 断段、容量 / 超限 / 溢出、布局重试幂等。
- demo 无窗口测试覆盖 Live 模式的布局重试与快照版本。

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo build -p customplot-demo --release --locked --offline
cargo bench -p egui-customplot --bench data_performance --locked --offline
```

阶段 3 提交 `8002a41318ada3a4d36592b12d57f4c7e62a8093` 的 Windows / Ubuntu CI 已成功：[run 43691734117](https://github.com/littepointR/egui-customplot/actions/runs/43691734117)。不是阶段 4 提交的 CI 证据。

## 正式 CPU 记录

新增 `benches/data_performance.rs`，独立于历史 `frame_baseline`。固定 1920 × 1080 绘图区、DPI 1、线宽 1.5、无标记，输入为单调正弦 + 尖峰 + 三处 NaN，覆盖 10 万 / 100 万 / 1000 万点。每场景预热 5 帧后采样 30 秒，分别测 Exact 窄视区、Visible 窄视区、MinMax 全区 / 平移、缓存静态和索引拾取；另有 30 秒环形追加 + 快照 + 索引微基准。

输出分列预处理、UI 构建、tessellation、二者合计和查询 / 拾取分位数，以及算法 / 容量计数。Exact / Visible 窄视区使用相同的 [40, 40.1]，不会以“看同一图”模糊全区 / 窄区的差别。原始数据、历史结果保留，不以短时试跑替代正式记录。

[2026-11-15 原始 CSV](benchmarks/2026-11-15-stage4-windows-cpu.csv)：i9-14900KF、Windows 10.0.26300、Rust 1.100.0、egui 0.37.0、bench 优化构建。非隔离机器，不作为固定硬件发布阈值。

| 点数 | 构造 ms | 建索引 ms | 样本 / 索引 B | Exact 窄区合计 p95 ms | Visible 同窄区 p95 ms | MinMax 全区 p95 ms | 缓存静态 p95 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100,000 | 1.1862 | 0.9992 | 1,600,000 / 66,432 | 0.7675 | 0.0138 | 0.7174 | 0.1342 |
| 1,000,000 | 10.9975 | 10.0945 | 16,000,000 / 526,592 | 8.3712 | 0.0444 | 1.1654 | 0.1313 |
| 10,000,000 | 116.8920 | 105.8606 | 160,000,000 / 8,405,248 | 89.9560 | 0.5447 | 3.2124 | 0.2388 |

合计指 headless UI 构建 + egui CPU tessellation，不是 GPU、呈现时间或 FPS。MinMax 与 Exact / Visible 窄区不是同视区 / 同精度对照；缓存静态也不是连续交互结果。1000 万点 indexed_hover 的拾取 p95 为 0.9508 ms、候选 135,417 点，明确保留密集 X 列的成本。

30 秒流式微基准完成 24,526 批次，ring 为 1,048,576 B，追加 + 快照 + 建索引 p95 1.5743 ms；每批丢弃快照，不能代替持续压力和多所有者资源结论。

## 长期压力记录状态

`benches/stream_stress.rs` 与 `scripts/measure-stream-stress.ps1`：宿主生产者约 1 ms / 256 点，队列容量 4，满时明确丢弃新批；消费者每轮最多 4 批，睡眠 33 ms。ring 65,536 点、最多两份快照、cache 1 MiB；每帧两遍布局，每 300 帧清缓存 / 重建视图。脚本每 5 秒采样 private bytes、working set、CPU、handles / threads。

输出前缀 `benchmarks/2026-11-15-stage4-stream-stress-windows`。最终 [环境记录](benchmarks/2026-11-15-stage4-stream-stress-windows-environment.txt) 为 `exit_code=0`、程序持续 1800.0216504 秒；[算法 CSV](benchmarks/2026-11-15-stage4-stream-stress-windows.csv) 有 `completed=true`，stderr 为空。binary SHA256 为 `66EE6733EFFCD7DB4FECF6D0DFD27FE1822190100E2C4BE2D8A8F9C5167D5E1F`；metadata 的 git_head 是阶段 3，实际 binary 含未提交的阶段 4 / Figure，早于双轴 / 热力图 / 柱图，不能证明后者长期稳定。

最终 43,146 次消费更新、172,578 批次接收、172,582 批次成功入队、928,761 批次因过载丢弃，剩余 4 批在退出时丢弃，收发守恒断言通过。ring 65,536 点 / 1,048,576 B、两份快照 / 2,097,152 B、索引合计 90,752 B、cache 528,504 B / 1 条；各轮预算 / 批次幂等 / 两遍布局断言均通过。清空后快照数与 cache bytes 均为 0；ring 在 main 退出时释放，未在退出前测其分配释放。

[进程采样](benchmarks/2026-11-15-stage4-stream-stress-windows-process.csv) 有 358 个样本（约 5～1797 秒）。private bytes 范围 6,418,432～23,076,864 B，handles 62～63，threads 2～5。以下中位数取各窗口排序后的 `floor(N/2)` 样本，不插值：

| 时间窗口 s | 样本数 | private bytes 中位数 B | working set 中位数 B |
| --- | ---: | ---: | ---: |
| [60,360) | 60 | 10,317,824 | 15,921,152 |
| [720,1080) | 72 | 10,260,480 | 16,097,280 |
| [1440,1801) | 72 | 10,137,600 | 15,872,000 |

观察到这些窗口无持续上升的中位数趋势；**这不证明零泄漏**，因为缺少 live allocations / 分配栈、宿主桌面 / 驱动 / GPU 和后加图元的长期数据。33 ms sleep 和更新数是压力节奏，不是呈现帧率；recent_work p95 含本程序的追加、快照、建索引、双遍 UI、tessellation 等 CPU 工作，不是纯追加成本。

## 仍未验收的阶段 4 要求

- 现有 headless 30 分钟压力切片通过；PERF-06 的真实桌面宿主 / GPU / 后加图元持续压力与 live allocations 仍缺。
- 分配失败注入、全局 / 每图 CPU 预算协同、临时构建硬限额和多视图总资源上限；目前各所有者仅对明确列出的自身 payload 设限。
- 进程 live allocations / private bytes / working set、GPU 申请 / 驱动驻留、空闲 CPU / GPU / 唤醒、呈现 / FPS 及端到端应用级固定硬件证据。
- 4 / 16 图性能、任意二维空间拾取、大量重复 X / 大量断段的加速，以及流式分块发布 / 增量索引。
- `egui_plot` 同条件对照、跨平台实机 / 视觉回归、冻结发布门槛。

因此阶段 4 仍是已实现基础能力的切片，不标完整完成；阶段 5 的 GPU 投入决策也不能只由 headless CPU 数字代替。
