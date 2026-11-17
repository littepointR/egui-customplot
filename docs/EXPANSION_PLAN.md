# 全量 MATLAB 及高频工程图种补齐规划（Stage 12–16）

本文件记录在 Stage 11（[MATLAB 绘图类型交付对照表](MATLAB_PLOT_CHECKLIST.md) 133 项本体条目全量落入“开发中”）之后，按用户要求制定的**完整补齐与验收收敛规划**，覆盖：
1. MATLAB 本体总览边缘漏项与维度/变体缺口；
2. 通用工程底座（时间轴、科学色谱、空心/多边形标记、框选缩放、数据刷选、横向分布图）；
3. 原与本体分账的高频工具箱工程图型（`TB-01..21` 重点图型）；
4. 现代可视化图表与硬件点云/体绘制/地球仪扩展；
5. 全阶段基准、导出、多语言 demo 与就绪审计收敛。

## 1. 范围边界与命名规则

- **不改变既有 133 项冻结基线**：[MATLAB 绘图类型交付对照表](MATLAB_PLOT_CHECKLIST.md)（82 `ML` + 35 `EX` + 16 `LG` = 133 项）与 [MATLAB_PLOT_BASELINE.json](MATLAB_PLOT_BASELINE.json) 保持冻结与可审计，新增扩展以独立编号 **`EXT-01`..`EXT-26`** 追踪，绝不虚增或混淆 133 分母。
- **零重型外部依赖扩张**：默认 feature 保持仅依赖 `egui`（无网络请求、无重型字体打包、无 Python/系统脚本依赖）；数学分布逆函数（正态/Weibull）、FFT/STFT、层次聚类、Smith 圆图共形映射、K 线、Sankey/Treemap/Sunburst 等核心算法均在库内纯 Rust 自研落地。
- **架构与质量底线**：遵循 [AGENTS.md](../AGENTS.md) 与 [ARCHITECTURE.md](ARCHITECTURE.md) 七域分层、单文件 `<= 400` 行上限、多遍布局幂等（`request_discard` 严禁重复消费或写盘）、宿主持有数据、PNG / 混合矢量 SVG 导出、`zh-CN` / `en` / `ja` demo 多语言完整覆盖，以及每阶段“全量测试 + fmt + strict clippy + 导出校验”完成后方能进入下一阶段。

## 2. 分阶段路线图（Stage 12–16，共 26 项扩展 `EXT-01`..`EXT-26`）

| 阶段 | ID | 对照目标 / MATLAB 参考 | 核心数据类型 / 控件 | 状态 | 验证与交付门槛 |
| --- | --- | --- | --- | --- | --- |
| 12 | `EXT-01` | `datetime` / `duration` 时间轴 | `Axis::{datetime, datetime_with_offset, duration}`, `TimeAxisMode`, `CivilDateTime` | 已完成切片（[STAGE_12](STAGE_12.md)） | 闰年/跨午夜/毫秒进位往返、多尺度刻度有序性与无崩溃极限测试 |
| 12 | `EXT-02` | MATLAB / 科学色谱补全 | `ColorMap::{Parula, Turbo, Inferno, Plasma, Magma, Cividis, Hot, Coolwarm}`（共 11 种） | 已完成切片（[STAGE_12](STAGE_12.md)） | 既有 3 色谱像素零偏移保留，全部 11 种端点/NaN/混合导出回归 |
| 12 | `EXT-03` | 空心与多角标记 | `MarkerShape::{Asterisk, Pentagram, Hexagram, *Hollow}`（共 15 种 + `wgpu` SDF） | 已完成切片（[STAGE_12](STAGE_12.md)） | CPU 实孔几何与 PNG 中心镂空像素验证、LOD/精确两路径、WGSL SDF 对齐 |
| 12 | `EXT-04` | 框选缩放与数据刷选 | `Plot::{allow_box_zoom, allow_brush}`, `PlotState::brushed_bounds`, `PlotData::brushed_indices` | 已完成切片（[STAGE_12](STAGE_12.md)） | 右键/`Shift` 框选、`Alt` 刷选、`Escape` 清除、两遍幂等、双 Y 联动与高对比叠加 |
| 12 | `EXT-05` | 横向分布图 | `BoxChart`/`ViolinPlot`/`Raincloud::orientation(BarOrientation::Horizontal)` | 已完成切片（[STAGE_12](STAGE_12.md)） | 垂直/水平边界缓存、log 兼容、正确方向渲染及非单位坐标点/区域拾取 |
| 13 | `EXT-06` | `confusionchart`（本体漏项） | `ConfusionMatrixData`, `ConfusionChart`, 行/列精确率与召回率汇总条 | 已完成切片（[STAGE_13](STAGE_13.md)） | 混淆矩阵计算、3 种归一化、对角/非对角色映射、悬停单元格提示与导出 |
| 13 | `EXT-07` | `skyplot`（本体漏项） | `SkyPlotData`, `SkyPlot`, 反向极径（90° 天顶至 0° 地平）、仰角掩膜 `MaskElevation` | 已完成切片（[STAGE_13](STAGE_13.md)） | 方位角顺时针北零点、仰角反向缩放、掩膜扇环与多星轨迹拾取/导出 |
| 13 | `EXT-08` | 曲线参数网格 `pcolor`/`contour(X,Y,Z)` | `CurvilinearField2D`, `CurvilinearPcolor`, `CurvilinearContours` | 已完成切片（[STAGE_13](STAGE_13.md)） | 四边形网格双线性/双三角切片、非自交域校验、等高线抽取与区域/顶点拾取 |
| 13 | `EXT-09` | 通用三维面片 `patch`（3D） | `Patch3DData`, `Patch3D`（任意凸/凹面索引、逐顶点 Gouraud/Blinn-Phong 光照） | 已完成切片（[STAGE_13](STAGE_13.md)） | 与 `DepthMode::ZBuffer` 深度互遮、多面片着色导出及面元拾取 |
| 13 | `EXT-10` | 三维 `alphaShape`（3D 凹包） | `AlphaShape3DData`, `AlphaShape3D`（3D Delaunay/Bowyer-Watson + 外接球半径过滤） | 已完成切片（[STAGE_13](STAGE_13.md)） | 临界 alpha 连通域、边界三角面法线抽取、内腔剔除与区域拾取 |
| 13 | `EXT-11` | 三维网络与有向图 `plot(graph/digraph)` 3D | `GraphData::layout_3d`（3D Force/Spherical）、`Graph3D` + `Plot3D` 图层 | 已完成切片（[STAGE_13](STAGE_13.md)） | 3D 节点/弧边/方向锥箭头、与相机正交/透视投影及三维点拾取集成 |
| 13 | `EXT-12` | 自适应 `fcontour` / `fsurf`（3D 曲面细分） | 梯度驱动网格自适应细化（2D 等高线 / 3D 显式函数曲面 + 预算上界） | 已完成切片（[STAGE_13](STAGE_13.md)） | 陡峭奇点附近高分辨率网格、与均匀基准对比误差收敛及有界预算拒绝 |
| 14 | `EXT-13` | `qqplot` / `probplot` / `normplot` / `wblplot` | `AxisScale::{Probit, Weibull}`、`ProbabilityPlotData`, `ProbabilityPlot` | 已完成切片（[STAGE_14](STAGE_14.md)） | Acklam 逆正态分布高精度分位变换、参考拟合线、非线性刻度网格与回归 |
| 14 | `EXT-14` | `ecdf` / `cdfplot` | `EcdfData`（Kaplan-Meier 右删失与 Greenwood 置信带）、`EcdfPlot` | 已完成切片（[STAGE_14](STAGE_14.md)） | 单调阶梯包络、置信上下带渲染、原始事件拾取 |
| 14 | `EXT-15` | `dendrogram` & `clustergram` | `DendrogramData`（UPGMA/Ward/Single/Complete 层次聚类）、`Clustergram` | 已完成切片（[STAGE_14](STAGE_14.md)） | 行列重排矩阵热图、双侧聚类树分支对齐渲染、分支/叶节点拾取 |
| 14 | `EXT-16` | `biplot`（PCA 主成分双标图） | `BiplotData`（得分点 + 载荷矢量 + `Varimax`/正交投影）、`Biplot` | 已完成切片（[STAGE_14](STAGE_14.md)） | 双向坐标轴载荷箭头与得分散点叠加、`HoveredVector` / `HoveredPoint` 拾取 |
| 14 | `EXT-17` | `controlchart`（SPC 控制图） | `ControlChartData`（$\bar{X}$-R / X-mR / $p$ / $u$ 规则与 UCL/LCL/CL 越界判定） | 已完成切片（[STAGE_14](STAGE_14.md)） | 失控点自动高亮（Red/Marker）、中心线注解与统计诊断计数 |
| 14 | `EXT-18` | `candle` / `highlow` / 2D 增减瀑布图 | `OhlcData`、`Candlestick`、`OhlcBars`、`Waterfall2D`（桥接累计柱） | 已完成切片（[STAGE_14](STAGE_14.md)） | 与 `Axis::datetime` 对接、涨跌配色与虚实芯切换、OHLC 详细 HoverInfo |
| 15 | `EXT-19` | `smithplot`（RF Smith 圆图） | `SmithTransform`（$\Gamma = (z-1)/(z+1)$）、等电阻/等电抗圆网格、`SmithPlot` | 已完成切片（[STAGE_15](STAGE_15.md)） | 共形复阻抗正反变换、多频率轨迹交互缩放平移与复阻抗/VSWR 数据提示 |
| 15 | `EXT-20` | `zplane`（零极点图） | `PoleZeroData`（含重根计数 $\times N$）、`ZPlanePlot` | 已完成切片（[STAGE_15](STAGE_15.md)） | 单位圆、实虚轴、实复数零极点 `o`/`x` 与重数标注、极零点拾取 |
| 15 | `EXT-21` | `bode` / `nyquist` / `nichols` / `rlocus` | `FrequencyResponseData`、`BodePlot`、`NyquistPlot`、`NicholsPlot`、`RootLocus` | 已完成切片（[STAGE_15](STAGE_15.md)） | 对数频率 + 相位展开、稳定裕度注解、奈奎斯特 `-1+j0` 焦点、Nichols 等 M/N 轮廓 |
| 15 | `EXT-22` | `spectrogram` / `pspectrum` / `pwelch` / `cwt` | 内置 Radix-2 FFT + 窗函数（Hann/Hamming/Blackman）+ STFT/Welch/Morlet CWT | 已完成切片（[STAGE_15](STAGE_15.md)） | 纯 Rust DFT/FFT/STFT 计算分离（`data/dsp/`）、对数频率/功率热图展示与测试 |
| 16 | `EXT-23` | `volshow`（3D 光线投射直接体绘制） | `VolumeRender3D`（CPU + 可选 `wgpu` 传递函数/等值采样 Raycaster） | 已完成切片（[STAGE_16](STAGE_16.md)） | RGBA 不透明度传递函数、与相机深度融合、`Plot3D` 集成与导出支持 |
| 16 | `EXT-24` | `pcshow`（海量三维点云） | `PointCloud3DData`（高密度 XYZ/RGB/Intensity + 八叉树/体素 LOD + 可选 `wgpu`） | 已完成切片（[STAGE_16](STAGE_16.md)） | 百万级 3D 点云 LOD 裁剪/采样、逐点色彩/强度映射与相机交互基准 |
| 16 | `EXT-25` | `geoglobe` / `geoplot3`（3D 地球） | `GeoGlobePlot`（WGS84 球面轨迹/海岸线/散点/柱高与背面消隐） | 已完成切片（[STAGE_16](STAGE_16.md)） | 离线矢量海岸线球面投影、轨迹高程挤出、`GeoGlobeState` 轨道交互与导出 |
| 16 | `EXT-26` | 现代图型 + Stage 12–16 总审计收敛 | `RadarChart`, `SankeyChart`, `TreemapChart`, `SunburstChart`, `GanttChart` | 已完成切片（[STAGE_16](STAGE_16.md)） | `tests/stage16_expansion_audit.rs`、CPU 基准归档、19+ 预设 Windows 烟雾与全量检查 |

## 3. 多轮收敛与验收原则

1. **逐阶段落地与零债务推进**：Stage 12–16 按阶段逐次实施；每完成一阶段即通过全工作区测试（含 `--no-default-features`、`--all-features`）、`fmt`、`clippy -D warnings` 与新增专项导出/正确性测试。
2. **文件职责边界与 400 行红线**：所有新扩展按职责置入现有七域子目录，每阶段核查 `tests/architecture.rs`；禁止使用压缩排版或共享杂项文件规避检查。
3. **最终收敛**：Stage 16 完成后新增独立扩展审计测试与基准归档，更新 `docs/STAGE_12.md`～`docs/STAGE_16.md`、`docs/ARCHITECTURE.md` 与 `docs/PROGRESS.md`，并通过 Release demo 全预设窗口生命周期验证。
