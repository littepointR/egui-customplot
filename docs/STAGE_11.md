# 阶段 11：全量对照核销与就绪审计（133 项 + CAP / EGUI / PERF）

日期：2026-11-16。**Stage 11 对 [MATLAB 绘图类型交付对照表](MATLAB_PLOT_CHECKLIST.md)（82 项 ML + 35 项 EX + 16 项 LG = 133 项）、[EGUI-01..10 与 PERF-01..12](EGUI_INTEGRATION_AND_PERFORMANCE.md)、CAP-01..20 通用能力矩阵以及 TB-01..21 工具箱分账完成交叉核销与自动化一致性验证；全量 133 项已 100% 落入“开发中”（0 未实现、0 静默删除），同时坚持“已交付 = 0”的诚实口径，明确保留未完成的多平台实机、硬件 GPU / FPS、独立印刷级排版、长时全应用资源回归与暂不发布边界（许可已明确为与 egui 一致的 `MIT OR Apache-2.0`）。**

## 1. 133 项对照汇总（按分类与阶段）

| 分类 | ID 范围 | 条目数 | 当前状态 | 主要实现阶段与模块 | 专项测试与演示预设 | 保留“开发中”未升“已交付”的边界 |
| --- | --- | ---: | --- | --- | --- | --- |
| A1 线图 / 函数曲线 | ML-001..012 | 12 | 12 开发中 | 阶段 1/2/8/9：`model/line.rs`, `function.rs`, `plot3d/`, `StackedPlot` | `series`, `log_axes`, `stairs`, `area`, `functions`, `plot3d_core_lines`; `--stairs`, `--polygons`, `--functions`, `--plot3d` | 缺复数适配细节、跨平台显示与印刷排版验收 |
| A2 散点 / 气泡 / 蜂群 | ML-013..020 | 8 | 8 开发中 | 阶段 2/8/9：`Scatter`, `BubbleChart`, `SwarmChart`, `BinScatter`, `Spy`, `Scatter3D`, `Bubble3D`, `Swarm3D` | `series`, `indexed_picking`, `bubbles_swarm_bivariate`, `flow_spy`, `plot3d_scatter_discrete`; `--distributions`, `--geometry`, `--plot3d` | 缺空心标记/球体光照/大规模散点 GPU 实例化与跨平台验收 |
| A3 分布与多变量 | ML-021..033 | 13 | 13 开发中 | 阶段 6/8/9：`HistogramData`, `Histogram2`, `ScatterHistogram`, `BoxChart`, `ViolinPlot`, `Raincloud`, `Pie/Donut`, `Word/BubbleCloud`, `CategoricalHeatmap`, `ParallelPlot`, `PlotMatrix` | `histogram`, `bars_pareto_hist`, `bubbles_swarm_bivariate`, `distributions_pie_clouds`, `multivariate_composite`; `--statistical`, `--distributions`, `--multivariate` | 缺大型分布 CPU 开销优化（如 25k+ raincloud/violin）、表适配器与视觉验收 |
| A4 离散数据 | ML-034..040 | 7 | 7 开发中 | 阶段 6/8/9：`Bars`, `Bar3D` (`Detached/Grouped/Stacked/Horizontal`), `ParetoChart`, `Stems`, `Stem3D` | `bars`, `bars_pareto_hist`, `stems*`, `plot3d_scatter_discrete`; `--statistical`, `--stems`, `--plot3d` | 缺密集 100k 全区杆降采样与跨平台完整验收 |
| A5 地理图 | ML-041..044 | 4 | 4 开发中 | 阶段 10：`axis/geo/`, `data/geo/`, `model/geo.rs`, `render/geo/`, `GeoPlot` | `geo_projection_plot_scatter`, `geo_density_bubble_basemap`, `export_stage10`; `--geo` | 缺高倍本地瓦片细节、大规模散点索引与严禁使用 MathWorks 在线底图 |
| A6 极坐标图 | ML-045..050 | 6 | 6 开发中 | 阶段 8：`PolarTransform`, `PolarPlot`, `PolarLine/Scatter/Bubble/Histogram`, `CompassPlot` | `polar_plots`, `polar_hist_regions`, `export_stage8d`; `--polar` | 缺极坐标交互编辑器与跨平台视觉验收 |
| A7 等高线图 | ML-051..055 | 5 | 5 开发中 | 阶段 8/9：`Contours`, `FilledContours`, `Contour3D` (`contour3`/`contourslice`) | `contours`, `plot3d_meshes_contours`, `plot3d_volumes_slices`, `export_polygons_contours`; `--contours`, `--surfaces3d`, `--volumes3d` | 缺自动平滑避让标签、自适应函数等高线细化与完整验收 |
| A8 向量场 | ML-056..058 | 3 | 3 开发中 | 阶段 8/9：`Quiver`, `Quiver3D`, `Feather` | `quiver*`, `flow_spy`, `plot3d_vectors_planes_depth`, `export_quiver`; `--quiver`, `--geometry`, `--plot3d` | 缺 100k 全区 `quiver` 视窗裁剪 LOD 与完整验收 |
| A9 曲面与网格 | ML-059..070 | 12 | 12 开发中 | 阶段 8/9：`Pcolor`, `Surface3D`, `MeshSurface3D`, `Ribbon3D`, `ImplicitSurface3DData` | `pcolor_image`, `plot3d_surfaces`, `plot3d_meshes_contours`, `plot3d_analytic_implicit`, `export_stage9b`; `--contours`, `--surfaces3d` | 缺硬件 GPU 曲面光栅化、曲面参数自适应细分与视觉回归 |
| A10 体数据与三维流 | ML-071..077 | 7 | 7 开发中 | 阶段 8/9：`Streamlines`, `ScalarVolume3D`, `VolumeSlice3D`, `Streamline3D`, `StreamParticles3D`, `StreamRibbon3D`, `StreamTube3D`, `ConePlot3D` | `flow_spy`, `plot3d_volumes_slices`, `plot3d_flow_volumes`, `export_stage9c`; `--geometry`, `--volumes3d` | 缺 GPU 体绘制与大规模流场加速及完整验收 |
| A11 动画 | ML-078..080 | 3 | 3 开发中 | 阶段 8/9：`AnimatedLine2D`, `AnimatedLine3D`, `CometPlayback`, `Comet3DPlayback` | `animation_annotations`, `plot3d_core_lines`; `--functions`, `--plot3d` | 缺视频 / GIF 编码导出与实时帧调度实机验收 |
| A12 图像 | ML-081..082 | 2 | 2 开发中 | 阶段 6/8：`Heatmap`, `PlotImage` (`ImageData::{rgba8, rgb8, indexed, grayscale}`) | `heatmap`, `heatmap_lifecycle`, `pcolor_image`; `--scalar`, `--contours` | 缺硬件纹理采样过滤 / 高分辨率 GPU 直传与全区大热力图优化 |
| B1..B4 补充能力 | EX-001..035 | 35 | 35 开发中 | 阶段 3/8/9：`Line/Surface3D::primitive`, `Fill/Fill3D`, `Patch`, `Rectangles`, `sphere/cylinder/ellipsoid`, `Tri*/Voronoi/Alpha/PolyShape/TetraMesh`, `Isosurface/Isocaps3D`, `Graph/Tree/EliminationTree`, `imshow`, `Annotation/Text3D/ConstantPlane3D/PolarRegionAnnotation` | `polygons_patches`, `triangulation_voronoi`, `graphs_networks`, `trees_elimination`, `annotations`, `animation_annotations`, `plot3d_*`, `polar_hist_regions`; `--polygons`, `--geometry`, `--networks`, `--polar`, `--surfaces3d`, `--volumes3d` | 缺 3D alphaShape/三维图网络、对象句柄动态树与跨平台验收 |
| C 传统历史接口 | LG-001..016 | 16 | 16 开发中 | 阶段 8/9：`pie/pie3`, `hist`, `polar/compass/rose`, `plotyy`, `ezplot/ezplot3/ezpolar/ezcontour*/ezmesh*/ezsurf*` | `distributions_pie_clouds`, `bars_pareto_hist`, `polar_*`, `dual_y_axes`, `functions`, `plot3d_*`; 多个预设 | 使用类型化闭包与显式构建器替代旧字符串表达式解释器；缺完整属性兼容 |
| **合计（核心本体）** | **ML+EX+LG** | **133** | **133 开发中 / 0 未实现** | **覆盖 2D / Polar / 3D / Geo 四坐标系** | **18 预设 gallery 与 14 项 CPU 基准套件** | **0 已交付：保持未解决边界透明可见** |

## 2. 工具箱扩展分账核销（TB-01..21，不计入 133 分母）

以下 [对照清单 D 节](MATLAB_PLOT_CHECKLIST.md#d-%E5%B7%A5%E5%85%B7%E7%AE%B1%E6%89%A9%E5%B1%95%E7%B4%A2%E5%BC%95%E4%B8%8E%E6%9C%AC%E4%BD%93%E5%88%86%E8%B4%A6) 全部 21 项工具箱条目经逐条核对保持独立分账，不混入 133 项分母，亦不因可画相似线 / 散点 / 热力图而宣称对应专业领域工具箱已实现：

| 扩展编号 | 所属产品 | 已核对接口范围 | 本库分账核销状态 |
| --- | --- | --- | --- |
| `TB-01` | Statistics and Machine Learning | `boxplot` / `hist3` | 不混入本体 `BoxChart` / `Histogram2` |
| `TB-02` | Statistics and Machine Learning | `gscatter` / `gplotmatrix` / `scatterhist` | 不混入本体 `ScatterHistogram` / `PlotMatrix` |
| `TB-03` | Statistics and Machine Learning | `qqplot` / `probplot` / `normplot` / `wblplot` | 缺分位数与概率刻度统计计算，未实现 |
| `TB-04` | Statistics and Machine Learning | `cdfplot` / `ecdf` / `ksdensity` | 缺完整估计器接口（与本体绘制层分离） |
| `TB-05` | Statistics and Machine Learning | `biplot` / `andrewsplot` / `parallelcoords` / `glyphplot` | 不混入本体 `ParallelPlot`，字形 / 载荷另账 |
| `TB-06` | Statistics and Machine Learning | `dendrogram` / `surfht` / `multivarichart` | 缺聚类树计算，不因可画树线而宣称支持 |
| `TB-07` | Statistics and Machine Learning | `controlchart` / `interactionplot` / `maineffectsplot` | 缺统计控制限与效应方差规则，未实现 |
| `TB-08` | Statistics and Machine Learning | `confusionchart` | 缺类别归一化 / 行列摘要语义，不冒充热力图 |
| `TB-09` | Control System | `bode` / `bodemag` / `nyquist` / `nichols` | 缺线性系统模型与频响相位展开计算 |
| `TB-10` | Control System | `rlocus` / `sigma` | 缺根轨迹 / 奇异值求解分析，未实现 |
| `TB-11` | Control System | `step` / `impulse` / `lsim` / `initial` | 缺时域系统仿真引擎，未实现 |
| `TB-12` | Signal Processing | `pspectrum` / `periodogram` / `pwelch` | 缺加窗与 PSD 频谱估计计算，未实现 |
| `TB-13` | Signal Processing | `spectrogram` / `stft` | 缺 STFT 时频求解与本体热力图渲染分账 |
| `TB-14` | Signal Processing | `freqz` / `zplane` | 缺复平面零极点与滤波器频响计算，未实现 |
| `TB-15` | Wavelet | `cwt` / `wcoherence` | 缺小波变换与影响锥语义，未实现 |
| `TB-16` | Image Processing | `volshow` / `sliceViewer` / `imshowpair` / `montage` | 缺体绘制传递函数与影像配准复合 |
| `TB-17` | Computer Vision | `pcshow` | 缺海量点云 GPU 实例化管线，未实现 |
| `TB-18` | Mapping | `geoplot3` / `geoiconchart` | 与 Stage 10 2D `GeoPlot` 与 MathWorks 服务明确分离 |
| `TB-19` | RF | `smithplot` | 缺 Smith 复阻抗变换与专用圆图网格，未实现 |
| `TB-20` | Financial | `candle` / `highlow` | 缺交易日历与金融 OHLC 专属规则，未实现 |
| `TB-21` | Bioinformatics | `clustergram` | 缺层次聚类重排与双重树图联动，未实现 |

## 3. 跨图型通用能力矩阵核销（CAP-01..20）

| 编号 | 能力 | 核销结论 | 本库对应实现与接口 | 自动化验证与证据 | 保留边界 |
| --- | --- | --- | --- | --- | --- |
| `CAP-01` | 多图布局 | 部分实现（开发中） | `Figure`、`FigureLayout`、`FigureCell::{show_plot, show_series, show_layers, show_polar, show_plot3d, show_geo}`、`StackedPlot`、`PlotMatrix` | `tests/figure.rs`, `tests/multivariate_composite.rs`, `tests/stage11_readiness_audit.rs` | 暂不支持跨多行/列合并 tile 与公共共享 ColorBar/图例自动合并 |
| `CAP-02` | 多图元组合 | 已实现基础叠加 | `Plot::show_layers`, `PolarPlot`, `Plot3D`, `GeoPlot` 及各类非穷尽 `Layer` 枚举 | `tests/series.rs`, `tests/polygons_patches.rs`, `tests/plot3d_surfaces.rs`, `tests/geo_projection_plot_scatter.rs` | `GeoDensity`/`HostTiles` 顺序叠加受固定层级约定 |
| `CAP-03` | 多坐标轴 | 已实现双 Y 轴与复合共享轴 | `YAxis::{Left, Right}`, `Plot::right_y_axis`, `PlotYy`, `PlotMatrix`, `ScatterHistogram` | `tests/dual_y_axes.rs`, `tests/dual_y_navigation.rs`, `tests/multivariate_composite.rs` | 暂限单 X + 左/右双 Y；不支持任意多重叠独立 Axes 对象树 |
| `CAP-04` | 坐标类型 | 部分实现（线性/对数/反向/等比例/分类） | `AxisScale::{Linear, Log10}`, `Axis::{reversed, categories, explicit_ticks}`, `Plot::data_aspect`, `DataAspect3D` | `tests/log_axes.rs`, `tests/aspect.rs`, `tests/bars_pareto_hist.rs`, `tests/plot3d_core_lines.rs` | 缺失原生日期时间 `datetime` / 时长 `duration` 专属格式与日历刻度器 |
| `CAP-05` | 坐标系 | 四套原生坐标系全覆盖 | 笛卡尔 `PlotTransform`、极坐标 `PolarTransform`、三维相机 `Transform3D`、地理 `GeoTransform` | `tests/stage11_readiness_audit.rs`（同图四坐标系嵌入 + 混合导出）、`tests/polar_plots.rs`、`tests/plot3d_core_lines.rs`、`tests/geo_projection_plot_scatter.rs` | 三维投影与地理投影仍各自受限于对应文档边界 |
| `CAP-06` | 刻度网格 | 已实现主次刻度/网格/格式器/角度/经纬格式 | `AxisTick`, 自定义格式回调、`Axis::minor_grid`, 极坐标径角网格、`geo_graticule_ticks` (`N/S/E/W`) | `src/axis/ticks.rs`, `tests/tooltip.rs`, `tests/polar_plots.rs`, `tests/geo_projection_plot_scatter.rs` | 无自动日期时间智能分段刻度 |
| `CAP-07` | 文本标识 | 已实现标题/标签/图例/气泡图例/色条/等高线标签 | `Plot::title`, `Axis::label`, `LegendEntry`, `BubbleLegend`, `ColorBar`, `Contours` 标签、`Text3D` | `tests/legend.rs`, `tests/color_bar.rs`, `tests/bubbles_swarm_bivariate.rs`, `tests/contours.rs`, `tests/export_stage9a.rs` | 仅实现单级标题/副标题需合并文本或注解；导出字形不转可搜索矢量文字 |
| `CAP-08` | 字体排版 | 复用宿主 egui 字体与点单位 | 继承当前 `Ui` 字体与点单位，支持自定义字体与旋转/对齐 `Annotation` | `tests/export_fonts.rs`, `tests/styles.rs` | 缺完整 LaTeX 排版与上下标/希腊字母富文本引擎 |
| `CAP-09` | 样式主题 | 已实现 5 层覆盖与科学浅色/监控深色主题 | 库默认 → `PlotTheme::{scientific, monitoring}` → `Figure` → `PlotStyle` → `SeriesStyle` | `tests/styles.rs`, `tests/figure.rs`, `--scalar` 等多预设切换 | 不支持运行时动态属性表全反射 |
| `CAP-10` | 颜色映射 | 已实现连续色图/Linear & Log 色域/逐点面顶点真彩 | `ColorMap::{Viridis, Gray, BlueRed}`, `ColorScale::{new, with_scale}`, `PatchColoring::{Uniform, PerFace, PerVertexInterp}`, `ImageData`, `Surface3D` Gouraud | `tests/heatmap.rs`, `tests/color_bar.rs`, `tests/polygons_patches.rs`, `tests/plot3d_surfaces.rs` | 颜色图目前内置 3 种常用色谱与自定义逐点 RGBA |
| `CAP-11` | 基本交互 | 已实现平移/锚点缩放/双击&Home复位/三维轨道旋转/地理回绕平移 | `PlotState`, `PolarState`, `Plot3DState`, `GeoPlotState`, `NavigationOptions` | `tests/egui_integration.rs`, `tests/aspect.rs`, `tests/polar_plots.rs`, `tests/plot3d_core_lines.rs`, `tests/geo_projection_plot_scatter.rs` | 尚缺显式鼠标拉框放大（box zoom）模式切换 |
| `CAP-12` | 数据探索 | 已实现原始点/格网/柱/区域/矢/3D/地理拾取与图例显隐 | `HoverInfo`, `HoveredPoint`/`Cell`/`Bar`/`Region`/`Vector`/`Polar*`/`3D*`/`GeoPoint`, `Legend` 开关 | `tests/indexed_picking.rs`, `tests/tooltip.rs`, `tests/quiver_picking.rs`, `tests/plot3d_surfaces.rs`, `tests/geo_density_bubble_basemap.rs` | 暂缺交互式矩形数据刷选（data brushing）回调状态返回 |
| `CAP-13` | 多图联动 | 已实现 `AxisLink` X/Y/XY 对等与防循环联动 | `AxisLink::{link_x, link_y, link_xy}`, `LinkError`, `LinkReport` | `tests/linked_axes.rs`, `tests/multivariate_composite.rs` | 仅支持同一 X/Y 逻辑轴数值范围联动，暂无通用任意属性 `linkprop` |
| `CAP-14` | 动态更新 | 已实现有界环形流与帧幂等动画追加 | `PlotStream`, `AnimatedLine2D`, `AnimatedLine3D`, `CometPlayback`, `Comet3DPlayback`, `StreamParticlesPlayback` | `tests/stream.rs`, `tests/animation_annotations.rs`, `tests/plot3d_core_lines.rs`, `tests/plot3d_flow_volumes.rs` | 增量实时重索引与流式绘图端到端增量性能仍受 Stage 4 记录边界约束 |
| `CAP-15` | 三维呈现 | 已实现正交/透视、双深度路径、Blinn-Phong、Gouraud 与裁剪 | `Projection3D`, `DepthMode::{SortedPainter, ZBuffer}`, `LightSource3D`, `[0, 1]^3` 裁剪 | `tests/plot3d_core_lines.rs`, `tests/plot3d_vectors_planes_depth.rs`, `tests/plot3d_surfaces.rs`, `tests/export_stage9b.rs` | 软件 `360px` `ZBuffer` 仅作用于 CPU 端，缺硬件 wgpu 3D 渲染后端与真半透明 OIT |
| `CAP-16` | 图像组合 | 通过 `Figure` + `PlotImage` / `Plot::imshow_axes` 组合 | `Figure` 行列网格、`ImageData::{rgba8, rgb8, indexed, grayscale}`, `PlotImage`, `Plot::imshow_axes` | `tests/pcolor_image.rs`, `tests/figure.rs`, `tests/export_polygons_contours.rs` | 无单独 `imtile` 拼接缩略图数据压缩器 |
| `CAP-17` | 图形变换 | 部分支持容器/相机/数据变换；无通用场景图节点变换 | `Transform3D`, `PolarTransform`, `GeoTransform`；`PlotSnapshot` 显式拒绝非恒等 `TSTransform` 变换 | `tests/export_contract.rs`, `tests/plot3d_core_lines.rs`, `tests/geo_projection_plot_scatter.rs` | 未实现 MATLAB `hgtransform` 可嵌套场景层次节点树 |
| `CAP-18` | 输出 | 已实现显示快照 PNG + 混合矢量 SVG | `#[cfg(feature = "export")] PlotSnapshot::{capture, to_png, to_svg}`, `ExportOptions`, `CaptureLimits` | `tests/export_*.rs`（14 项导出专项含 `roxmltree` 与无外部字体 `resvg` 像素校验） | 显示快照不按印刷尺寸重排版；字体嵌为 PNG patch；动画视频/GIF 输出未实现 |
| `CAP-19` | 大数据 | 已实现单调 X 树索引/LOD/有界缓存与预算拒绝 | `PlotData::build_line_index`, `LineSampling::{Exact, Visible, MinMax}`, `PlotCache`, 预算限额错误 | `tests/line_sampling.rs`, `tests/line_cache.rs`, `tests/indexed_picking.rs`, 14 项 `benches/*` | 任意散点/极坐标/三维面片/高密地理散点缺空间索引与后台 worker 取消队列 |
| `CAP-20` | 输入与计算 | 显式宿主持有切片/网格/场/图结构；明确计算与绘制分层 | `PlotData`, `GridData`, `HistogramData`, `ScalarField2D`, `VectorField2D/3D`, `ScalarVolume3D`, `TriMesh2D/3D`, `GraphData`, `EliminationTree` | 对应各专项单元及集成测试（`contour3`, `Voronoi`, `EliminationTree` `O(nnz)` 等） | 不提供 MATLAB 表格字符串脚本或动态求值解释器 |

## 4. egui 原生集成契约核销（EGUI-01..10）

| ID | 结合点 | 核销状态 | 本库落实机制 | 核对测试 | 开放边界 |
| --- | --- | --- | --- | --- | --- |
| `EGUI-01` | 原生嵌入 | 已验证 | 所有 widget（`Plot`, `PolarPlot`, `Plot3D`, `GeoPlot`, `Figure`, `ColorBar`, `BubbleLegend`）均经 `&mut egui::Ui`、空间分配与 `Sense` 渲染并返回含原 `Response` 的结构体；不绑定 `eframe` | `tests/egui_integration.rs`, `tests/figure.rs`, `tests/stage11_readiness_audit.rs` | 跨 macOS / Linux / WebAssembly 实机窗口未测 |
| `EGUI-02` | 布局与裁剪 | 已验证 | 服从父 `Ui::clip_rect`、`available_rect` 与 `ScrollArea` / `Panel` / `Grid`；内部独立分配标题、轴刻度、数据区与图例/色条 | `tests/egui_integration.rs`, `tests/layout` 相关断言, `tests/stage11_readiness_audit.rs` | 极窄（`< 60px`）容器下部分图例主动隐藏以防越界 |
| `EGUI-03` | ID 与状态 | 已验证 | `IdSalt` 构建持久化控件 ID，样本驻留宿主 `Arc` / 借用，视图状态显式驻留 `PlotState` / `PolarState` / `Plot3DState` / `GeoPlotState` | `tests/egui_integration.rs`, `tests/series.rs`, `tests/geo_projection_plot_scatter.rs` | 多 OS viewport 共享同数据独立交互仅验证框架 ID，缺多原生 OS 窗口实机 |
| `EGUI-04` | 输入仲裁 | 已验证 | 滚轮缩放默认 `allow_scroll_zoom = false` 将滚轮留给父 `ScrollArea`；图例区与 Ctrl/Cmd 修饰键隔离；`ui.is_enabled()` 保护 | `tests/egui_integration.rs`, `tests/tooltip.rs`, `tests/stage11_readiness_audit.rs` | 右键菜单未内建复杂树状属性弹窗 |
| `EGUI-05` | 多遍布局 | 已验证 | `*State::begin_frame` 绑定 `cumulative_frame_nr` 与基准快照；`PlotStream::append_once` / `*Playback::advance_once` 严格防重放；`export` 延迟到下一帧写盘 | `tests/egui_integration.rs`, `tests/stream.rs`, `tests/animation_annotations.rs`, `examples/demo/src/*_tests.rs` | 要求宿主连续帧稳态运行，不介入 egui 外部自定义 pass |
| `EGUI-06` | 主题与字体 | 已验证 | `resolve_plot_style` 读取宿主 `Ui::visuals` / 字体而不修改 `Context::style`；支持科学浅色/监控深色主题与显式 `pixels_per_point` | `tests/styles.rs`, `tests/quiver_dpi.rs`, `tests/export_fonts.rs` | 中文等 CJK 字体依赖宿主事先加载，示例无外部字体包 |
| `EGUI-07` | 原生 UI 扩展 | 已验证 | 悬停格式回调在持锁外执行；`LegendEntry` 用原生 `selectable_label`；`PlotResponse` 向宿主暴露拾取/范围事件 | `tests/tooltip.rs`, `tests/legend.rs`, `tests/interaction` 测试 | 暂无官方稳定跨版本的任意自定义图元扩展 trait（留待需求冻结） |
| `EGUI-08` | 重绘调度 | 已验证 | 静止图不调用 `request_repaint`（`repaint_delay > Duration::ZERO`）；动画仅在活跃修订增量时按需触发 `sync_repaint`；绝不替换 `set_request_repaint_callback` | `tests/egui_integration.rs`, `tests/animation_annotations.rs`, `tests/stage11_readiness_audit.rs` | 流式背压策略需由宿主在推进数据前配置 |
| `EGUI-09` | 渲染与资源 | 已验证 | 零第二窗口、零第二 `Context`、零自建线程池或 GPU device；全部走宿主 `Ui::painter_at` 与 `Shape::mesh` / `Shape::line` | `tests/egui_integration.rs`, `tests/architecture.rs`, `scripts/smoke-demo.ps1` | 尚未提供可选 `egui-wgpu` `CallbackTrait` 硬件加速后端 |
| `EGUI-10` | 可访问性与生命周期 | 部分实现 | 大数据批量 `Mesh` / `Path` 提交而非逐点 widget；`Weak` 引用计数测试验证 100 次创建销毁不泄漏样本/索引/缓存 | `tests/egui_integration.rs`, `data::points::tests`, `tests/line_cache.rs`, `tests/geo_density_bubble_basemap.rs` | 仅完成基础可聚焦与 `Home` 复位键；完整无障碍屏幕阅读器语义树仍缺 |

## 5. 性能与资源约束核销（PERF-01..12）

| ID | 验收维度 | 核销状态 | 已归档测量 / 代码防线 | 保留边界 |
| --- | --- | --- | --- | --- |
| `PERF-01` | 分项帧耗时 | 已落实现场测量口径 | 14 项 `benches/*` 独立上报 `ui_p50/p95/p99`、`tess_p50/p95/p99` 与 `total_p50/p95/p99` + 预处理 `p50` | 全为 Windows headless CPU，不含硬件 GPU 提交/光栅/swapchain 耗时与 FPS |
| `PERF-02` | 分档数据规模 | 已测量 1k～10M 规模 | `data_performance`（100k / 1M / 10M 点）、各阶段 bench 覆盖 2k～200k 典型工程规模 | `stems` / `quiver` / `geoscatter` 等未建全量 LOD 的图型在大规模全区下耗时高 |
| `PERF-03` | 隐藏工作量可观测 | 已接诊断计数器 | `PlotDiagnostics` 及 9 个子域结构统计 `input`/`visited`/`output`/`capacity_bytes`/`unique_bytes`/`cache_hits`/`tile_cache_hits` | 不等同于操作系统私有字节峰值 |
| `PERF-04` | 多图共存 | 已测多子图 + 屏外跳过 | `tests/figure.rs`, `tests/multivariate_composite.rs`, `is_rect_visible` 跳过离屏绘制 | 16 图同屏 30 秒真实窗口压测与滚动区域大量离屏图对比 bench 待补 |
| `PERF-05` | 空闲 CPU / 唤醒 | 已测逻辑静态零唤醒 | `egui_integration` 与 `stage11_readiness_audit` 确认静止帧无主动立即重绘请求；`frame_baseline` 保留空白对照 | 缺 30 分钟真实桌面闲置进程采样对比 |
| `PERF-06` | 流式负载与背压 | 部分测量 | `stream_stress` 完成 1800 秒旧版过载压力（[STAGE_4](STAGE_4.md)）；`PlotStream` 有界环形与丢弃计数通过 `tests/stream.rs` | 当时压力循环仅测流发布+索引而未同步全帧实时重绘；需新端到端基准 |
| `PERF-07` | 资源释放与防泄漏 | 已测引用与缓存归零 | 100 次大数据集销毁 `Weak` 归零（`egui_integration.rs`）、`GeoTileCache` 与 `PlotCache` 预算严格限制（`line_cache.rs`, `geo_density_bubble_basemap.rs`） | 未跑连续进程级工作集长期泄漏压测 |
| `PERF-08` | 交互压力 | 已测连续移动/缩放与正确性 | `data_performance` 含平移场景；各图元多遍丢弃导航幂等与锚点缩放通过测试 | 缺交互期跨线程后台取消重算任务压测 |
| `PERF-09` | 样式失效隔离 | 已验证失效隔离 | `PlotCache` 键不包含线宽/虚线/颜色等非几何索引键（`tests/line_cache.rs`） | 极坐标/三维/地理暂无跨帧持久几何缓存（每帧轻量重建） |
| `PERF-10` | 预算超限与拒绝 | 已验证可控错误 | `build_line_index` 预算拒收、`CaptureLimits`/`ExportOptions` 上限拒收、`GeoTileCache::insert` `CacheBudgetExceeded` | `egui` 内部字体图集与系统分配失败不受本库硬限额控制 |
| `PERF-11` | 产物与最小依赖 | 已验证默认轻量特征 | 默认 `cargo check --no-default-features` 仅依赖 `egui`（无网络/线程/PNG）；`export` feature 按需启用 `png`；测试期外无隐式后台工作线程 | 未采集跨优化等级二进制体积矩阵报告 |
| `PERF-12` | 回归门槛与源码护栏 | 已建立自动化回归 | `tests/architecture.rs`（7 域边界 + `<= 400` 行/文件）+ `tests/checklist.rs`（133 条完整性）+ `stage11_readiness_audit.rs`；18 份 CSV 基准不可覆盖 | 固定硬件阈值与跨平台 GPU CI 回归待建立 |

## 6. Stage 11 自动化核销与最终验证证据

- **新增 Stage 11 全量就绪审计测试套件 `crates/egui-customplot/tests/stage11_readiness_audit.rs`（2 项）：**
  1. `checklist_capabilities_egui_perf_and_benchmarks_are_complete_and_traceable`：自动化交叉验证 `docs/MATLAB_PLOT_CHECKLIST.md` 133 行全部处于 `"开发中"` 且无 `"未实现"`、每一行均显式引用对应实施阶段（`Row.contains("阶段")` 为 133/133）、`CAP-01..20` 与 `TB-01..21` 全部对齐 `STAGE_11.md`、`EGUI-01..10` 与 `PERF-01..12` 全部对齐 `EGUI_INTEGRATION_AND_PERFORMANCE.md` 与 `STAGE_11.md`、18 个 `scripts/smoke-demo.ps1` 预设齐全、14 个 `Cargo.toml` 基准目标与 `docs/benchmarks/` CSV 归档可追溯。
  2. `all_four_coordinate_systems_embed_idempotently_in_one_figure_and_export`：在 `egui::ScrollArea` 中嵌入含 2D 笛卡尔 `Plot` + 极坐标 `PolarPlot` + 3D `Plot3D` (`ZBuffer`) + 2D 地理 `GeoPlot` 四坐标系共存的 `Figure`，验证两遍 `request_discard` 幂等、静止帧无立即重绘请求（`EGUI-08`），并在 `feature = "export"` 下于宿主释放后验证自包含 SVG（`roxmltree` 合法且无 `<text`）与解码 PNG 渲染结果。
- 全部 7 项工作区最终检查（含 `--all-features` 下 82 个测试套件共 317 项测试全部通过、`fmt`、`--no-default-features check`、默认/全特性严格 `clippy` 与 Release demo SHA256 一致性校验）已归档于 [Stage 11 最终检查记录](benchmarks/2026-11-16-stage11-windows-final-checks.txt)。
- Release `customplot-demo.exe`（SHA256 `F901268AFA20E29B23035738D9B129A806A3D67B341FC8D371DC41E4E0D9A507`）经 [十八预设 Windows 原生生命周期记录](benchmarks/2026-11-16-stage11-windows-gallery-smoke.txt) 验证，全部 18 个预设均达到 `input_idle=True`、`title=egui-customplot | Stage 11`、`responsive=true`、`close_requested=True` 且退出码 `0`。不代表跨平台显示实机、硬件 GPU 光栅化或长时运行内存回归。

## 7. 明确保持开放与未完成事项

1. **许可与发布**：全仓库已采用与 egui 相同的双重许可 `MIT OR Apache-2.0`（`LICENSE-MIT` / `LICENSE-APACHE`），workspace 包目前保持 `publish = false`，暂不发布 crates.io。
2. **跨平台实机**：Linux / macOS / WebAssembly 显示与原生输入尚未在对应实机环境验证。
3. **可选 `wgpu` 后端与真实渲染性能增量记录（2026-11-16）**：在保留 Stage 5/Stage 11 历史基线的前提下，已补充实现三层高性能架构（`PlotData::new` 自动单调索引 + `LineSampling::Auto` / `MarkerSampling::Auto` + Y 分支定界拾取 + `IndexedStream` + `UniformWaveform` + 可选 `feature = "wgpu"` WGSL 计算/实例化通道及 CPU 回退）。Windows 实机归档证据包括：[CPU + tessellation 千万点基准](benchmarks/2026-11-16-dynamic-10m-cpu.csv)（4 场景 `total_p95` 为 `1.3365 ms`～`6.6787 ms`，均 `< 8.33 ms`）、[原生窗口 `eframe + wgpu` 基准](benchmarks/2026-11-16-dynamic-10m-window-wgpu.csv)（`AMD Radeon RX 7800 XT (Vulkan)`，`wgpu_10m_waveform_live_32k` = **699.99 FPS / p95 1.7540 ms**，`cpu_10m_stream_ring_live_32k` = **174.13 FPS / p95 6.3164 ms**）及 19 预设（含 `--dynamic-10m`）Windows 原生烟雾 [gallery smoke](benchmarks/2026-11-16-dynamic-10m-gallery-smoke.txt)（SHA256 `D8C0BFF83B4F521275EE2DFC820274FF860348763BEFCC79828079AC163E11A0`）。3D 曲面/体渲染与大型分布图的 GPU 管线及非 Windows GPU 实机仍保持开放。
4. **独立印刷排版导出**：`PlotSnapshot` 为混合矢量 + 宿主字形栅格（`Stage 7`）；尚无印刷尺寸重排或纯矢量字体轮廓转换。
5. **专业工具箱扩展**：`TB-01`～`TB-21` 仍保持与本体分账，需另行冻结领域产品范围后立项实施。
