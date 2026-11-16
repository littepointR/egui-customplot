# 阶段 10：地理绘图与离线 / 缓存底图契约

日期：2026-11-16。**Stage 10 全部 4 项地理图型（`ML-041` `geoplot`、`ML-042` `geoscatter`、`ML-043` `geodensityplot`、`ML-044` `geobubble`）、地理坐标投影与跨日期变更线处理、显式底图署名 / 字节预算缓存 / 离线回退契约均已实现并完成 CPU 测试与原生 18 预设窗口生命周期烟雾验证（[交付对照清单](MATLAB_PLOT_CHECKLIST.md) 已达 133/133“开发中”、0 未实现）。** 所有地理条目将在 Stage 11 与全量 133 项及 `CAP` / `EGUI` / `PERF` 矩阵一并完成终验。

## 地理坐标、投影与跨日期变更线

- `axis/geo/` 提供 `GeoPoint { lat, lon }`、`wrap_lon_180`、`shortest_dlon`、`haversine_km`、`GeoProjection::{WebMercator, Equirectangular}`、`GeoBounds` 与 `GeoTransform`：
  - **非有限 / 越域处理：** `NaN` / `Inf` 计入 `GeoDiagnostics::non_finite_points` 并断开航线；有限但 `|lat| > 90°` 的样本计入 `domain_invalid_points` 并剔除，绝不静默夹逼进有效数据。`WebMercator` 仅在投影计算阶段将纬度映射至 `±85.05112878°`，原始经纬度与悬停读数保持不变。
  - **原生跨 `±180°` 视口：** `GeoBounds` 以 `(center_lon, lon_span)` 与 `[lat_min, lat_max]` 表达，`GeoBounds::from_lat_lon` 支持 `lon_lim = [120.0, -110.0]` 式跨日期变更线区间；`GeoBounds::from_points` 在 $S^1$ 圆周上寻找最大经度间隔并取补弧，避免 `[176°, -177°]` 自动撑成 `353°` 全球窗口。
  - **接缝切分与连续投影：** `project_geo_polyline` 在与 `center_lon` 相对经度达 `±180°` 的接缝处精确插值纬度、累加 `dateline_splits` 并拆分为左右两段独立折线（世界视角下东京→火奴鲁鲁航线无横贯图面的水平错连；切至太平洋视角 `lon_lim([120.0, -110.0])` 则作为单一连续折线输出）。

## 4 项地理图型与数据边界

- **`ML-041` `geoplot`（`GeoPathData` / `GeoLine`）：** 支持多航线叠加、虚线样式（每段独立起相、NaN/域外断开）、可选原始点 marker 与跨日界线切分。
- **`ML-042` `geoscatter`（`GeoScatterData` / `GeoScatter`）：** 支持经纬度散点、逐点尺寸、颜色标量 + `ColorScale` + `ColorMap`、逐点 RGBA 与 `ColorBar`、`GeoPlotState` 原生平移跨 `±180°` 循环与按需 `.allow_scroll_zoom(true)` 锚点缩放。
- **`ML-043` `geodensityplot`（`GeoDensityData` / `GeoDensity`）：** 球面四次双权重核函数 $(1 - (d/R)^2)^2$（`haversine_km`），支持固定物理半径 `GeoRadiusMode::Kilometers` 与随视口像素自适应的 `ZoomAdaptivePixels`；支持样本权重（负权计入 `domain_invalid_points`）与半透明顶点色批处理网格（不依赖外部纹理，`PlotSnapshot` 可无损捕获导出）。
- **`ML-044` `geobubble`（`GeoBubbleData` / `GeoBubbles`）：** 面积比例半径映射、确定性类别着色与类别显隐开关、可交互图例及 `BubbleLegend` 联动，悬停可拾取经纬度、标签、类别及数值。

## 底图来源、许可署名、LRU 缓存与离线契约

**严格声明：本库不引入任何网络依赖（零 `std::net`、零后台线程、零文件 I/O），亦不提供、访问或声称有权使用 MathWorks 官方底图服务。**

- `GeoAttribution` 强制要求非空署名文本，可附带许可信息（内置离线矢量底图默认展示 `GeoAttribution::builtin_offline()`）；构造 `GeoTileCache` 时传入空白署名返回 `GeoError::AttributionRequired`。
- `GeoTileCache` 为宿主持有的按字节预算受限 LRU 缓存：单瓦片超预算返回 `GeoError::CacheBudgetExceeded`；超总预算按 LRU 自动淘汰并维持 `retained_bytes() <= budget_bytes()`。
- `GeoBasemapStyle::{None, OfflineVectorLight, OfflineVectorDark, HostTiles}`：
  - 内置 `OFFLINE_LAND_RINGS` 静态简化矢量陆地轮廓在经度偏移 `[-360°, 0°, +360°]` 上动态剔除与耳切三角化渲染，跨 `±180°` 平移无多边形撕裂。
  - `HostTiles` 通过 `visible_tiles_for_bounds`（跨日期变更线 `x mod 2^z` 回绕）查询宿主缓存并以未加纹理的 `WHITE_UV` 顶点染色 `Mesh` 提交；缓存未命中按 `GeoOfflinePolicy::{FallbackToVector, BlankWithNotice}` 处理并显式计入 `tile_cache_hits` / `tile_cache_misses` / `offline_fallback_tiles` 及可视化署名提示。

## 测试、演示与 CPU 基准证据

- 新增 7 项库集成测试与 1 项 demo `--geo` 测试（均通过默认特性与 `--all-features --locked` 验证）：
  - `tests/geo_projection_plot_scatter.rs`（3 项：`WebMercator`/`Equirectangular` 正反投影往返、`S^1` 最小弧与太平洋日期变更线范围；`geoplot` 世界视角 `dateline_splits == 1` vs 太平洋视角 `dateline_splits == 0`、`NaN`/`|lat|>90°` 隔离；`geoscatter` 逐点样式、2 帧一致 `id_salt` 拾取、多遍布局缩放/平移幂等）。
  - `tests/geo_density_bubble_basemap.rs`（3 项：`geodensityplot` 权重 `2x` 精确倍率、紧凑支集、公里/像素半径对比与负权拒收；`geobubble` 面积半径比例、分类显隐、`BubbleLegend` 与拾取；空白署名拒收、LRU 预算淘汰与 `HostTiles` 命中/离线矢量回退计数）。
  - `tests/export_stage10.rs`（1 项：含太平洋 `geoplot+geobubble+BubbleLegend` 与 `HostTiles+geodensityplot+geoscatter+ColorBar` 的双子图 `Figure`，宿主数据/缓存/状态释放后 SVG（`roxmltree` + 无字体 `resvg` 且无 `<text`）与 PNG 解码非白像素验证）。
  - `examples/demo/src/geo.rs`（`--geo` 2×2 `Figure` 及 `geo_tests`），全 demo 24 项测试通过（含 18 预设自包含 SVG / PNG 导出）。
- 全部 7 项工作区最终检查（含 `--all-features` 下 81 个测试套件共 315 项测试全部通过、`fmt`、`--no-default-features check`、默认/全特性严格 `clippy` 与 Release demo SHA256 一致性校验）见 [Stage 10 最终检查记录](benchmarks/2026-11-16-stage10-windows-final-checks.txt)。
- Release `customplot-demo.exe`（SHA256 `AB78AB8E39B9F6DEA6F8EB06F57C91FC68DB5968B2ECF2B3F6D6C87380DC3DC6`）经 [十八预设 Windows 原生生命周期记录](benchmarks/2026-11-16-stage10-windows-gallery-smoke.txt) 验证，含 `--geo` 在内的全部 18 个预设均达到 `input_idle=True`、`responsive=true`、`close_requested=True` 且退出码 `0`。不代表跨平台显示实机、硬件 GPU 光栅化或长时运行内存回归。

### Stage 10 CPU 基准测量

`stage10_performance`（binary SHA256 `FEE197BA746F213061E7C2351F89AE3E179FFBC60D6A1A37DEEF6C2452697E6F`，基于 `a1407df` + Stage 10 实现）在共享 Windows 10.0.26300 / `Intel(R) Core(TM) i9-14900KF` / Rust 1.100.0 / `egui 0.37.0` / canvas `2024×1124`（`GeoPlot` `height=1080`、DPI=1、`show_legend=false`、`show_hover=false`）下对 6 项预处理测 p50（5 次）、6 场景各预热 5 帧并采集 30 秒，退出码 0、stderr 为空（[原始 CSV](benchmarks/2026-11-16-stage10-windows-cpu.csv)、[环境记录](benchmarks/2026-11-16-stage10-windows-environment.txt)、[空 stderr](benchmarks/2026-11-16-stage10-windows-cpu.stderr.txt)）。

- **预处理中位耗时（p50 ms）：** `geoplot_routes_50k_dateline=1.7880`、`geoscatter_20k_styled=1.6607`、`geodensity_10k_weighted_64x48=0.2563`、`geobubble_2k_categorical_legend=0.0825`、`geobasemap_host_tiles_cached=0.0014`、`geo_combo_pacific_4layers=0.0001`。

| 场景 | items | frames | UI p95 (ms) | Tess p95 (ms) | 合计 p50 / p95 / p99 (ms) | 输出图元 / meshes | 样本字节 | 访问项 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `geoplot_routes_50k_dateline` | 50,000 | 4,798 | 3.6480 | 3.1924 | 5.9495 / 6.7022 / 8.5784 | 209 / 24 | 800,000 | 50,000 |
| `geoscatter_20k_styled` | 20,000 | 1,260 | 15.2462 | 14.7675 | 22.3603 / 29.6859 / 36.2201 | 20,011 / 24 | 960,000 | 20,000 |
| `geodensity_10k_weighted_64x48` | 10,000 | 2,410 | 12.7974 | 0.0439 | 12.2674 / 12.8354 / 18.1353 | 128 / 25 | 240,000 | 10,000 |
| `geobubble_2k_categorical_legend` | 2,000 | 4,108 | 1.6189 | 6.5057 | 6.7912 / 7.9458 / 9.6510 | 2,011 / 25 | 128,000 | 2,000 |
| `geobasemap_host_tiles_cached` | 10 | 193,667 | 0.1502 | 0.0697 | 0.1366 / 0.2172 / 0.3774 | 22 / 27 | 40,960 | 5 |
| `geo_combo_pacific_4layers` | 32,005 | 1,656 | 14.2810 | 4.4913 | 17.7427 / 18.5566 / 19.2502 | 7,561 / 25 | 1,328,080 | 26,858 |

- **严格口径与已知瓶颈：** 以上仅测 headless CPU UI 构建与 `egui` tessellation，不含 GPU 上传 / 硬件光栅化 / swapchain 呈现 / 真实 FPS；2 万点无空间索引 `geoscatter` 每帧线性投影+抗锯齿扇形构建+tessellation 合计 p95 为 `29.6859 ms`，大规模散点尚需屏幕空间分桶 / GPU 实例化优化；内置离线海岸线仅为简化全球轮廓，高倍缩放细节需宿主接入本地 / 离线合法瓦片集。

## 后续工作

- **Stage 11：** 逐项核销 133 项条目及 `CAP-01..20`、`EGUI-01..10`、`PERF-01..12`，客观报告覆盖情况、已测证据与保留边界。
