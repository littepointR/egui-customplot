# MATLAB 绘图类型交付对照表

## 1. 范围、版本与计数口径

- 参考版本：**MATLAB R2026b**；核对日期：**2026-11-15**。
- A 表：官方 [MATLAB 绘图类型](https://ww2.mathworks.cn/help/matlab/creating_plots/types-of-matlab-plots.html) 总览中的 **82 个去重绘图函数**，全部列入，不因首版只做二维而省略三维、地理或体数据类型。
- B 表：从官方分类页和函数页补充的 **35 项图元、几何、图网络及装饰绘制能力**。
- C 表：**16 项传统 / 历史接口**，保留视觉效果和迁移对照；其中只有明确注明的接口属于官方不推荐项。
- A + B + C 共 **133 个函数 / 方法 / 能力对照条目**，不是 133 种互不重叠的基本图形。例如 `semilogx` 是折线与轴变换的组合，`surf` 和 `surface` 可复用内核。
- D 表单列工具箱扩展，不混入 MATLAB 本体完成率；E 表单列通用能力，不把布局、配色等误计为新图类型。
- **完整性边界：** A 表对已冻结的官方总览完整覆盖；官方自己将该总览称为“常见的图形函数”，所以另补 B、C 表。本文不把它冒称为“所有 MathWorks 工具箱、所有对象重载、所有第三方自定义图表”的穷举。工具箱的完整盘点必须另行冻结产品及版本清单；D 表目前是已核对的扩展索引，不是全工具箱承诺。
- 官方总览的函数名快照保存在 [MATLAB_PLOT_BASELINE.json](MATLAB_PLOT_BASELINE.json)，用于后续检查漏项。官方 `/help/` 页面会滚动更新，更新参考版本时应同步修改快照和本文。

**当前状态：ML-001 / 007 / 008 / 009 / 013 共 5 项“开发中”，其余 128 项未实现；没有图类型通过完整交付门槛。** 基础二维实现和限制见 [阶段 2](STAGE_2.md)，[首轮记录](STAGE_1.md) 保留历史证据。收录是长期交付对照，不意味着全部进入 0.1，也不意味着复制 MATLAB 调用语法、表达式解析器或专有算法。

## 2. 怎样判定“交付”

每项状态：`未实现 → 开发中 → 待验收 → 已交付`。不能仅有一张截图就标记“已交付”。每个条目需关联以下证据：

1. **案例：** 可运行示例、固定输入、预期结果，以及与对应 MATLAB 图类型的差异说明。
2. **正确性：** 数据语义、坐标、边界值、缺失值、统计或几何结果有测试；函数采样明确容差和失败行为。
3. **定制与交互：** 适用于该图的颜色、线 / 面 / 标记样式、图例 / 色条、导航、拾取和动态更新均有验证；不适用项说明原因。
4. **性能：** 记录数据规模、硬件、后端、预处理时间、p95 / p99 帧耗时及内存；不能把折线的百万点指标套到所有图类型。
5. **导出与文档：** 提供 API 说明和导出案例；三维 / 体数据若采用栅格嵌入 SVG，必须标为混合导出，不能宣称纯矢量。

下表“专项验收”是上述通用门槛之外的最低要求，不是完整 MATLAB 属性兼容规范。视觉等价、数据语义等价、API 语法兼容应分别说明；本库不默认承诺第三项。

egui 原生集成及性能 / 资源占用还需通过 [专项约束](EGUI_INTEGRATION_AND_PERFORMANCE.md) 中的 EGUI / PERF 验收项；支持图类型数量不能替代可嵌入性、低空闲占用和有界资源验证。

## A. 官方总览全量清单：82 项

同一函数在官方多个分类出现时只记一项：`stairs` 归入线图，`swarmchart` / `swarmchart3` 归入散点图。图例、轴类型和图元组合仍需分别测试。

### A1. 线图：12 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-001 | 开发中 | [`plot`](https://ww2.mathworks.cn/help/matlab/ref/plot.html) | 二维折线 / 标记图，2D | 已有折线 / 标记 / 混合叠加 / 断段 / 基础线型；复数适配、导出与完整验收仍缺，见阶段 2 |
| ML-002 | 未实现 | [`plot3`](https://ww2.mathworks.cn/help/matlab/ref/plot3.html) | 三维折线，3D | XYZ 轨迹、深度遮挡、相机与拾取 |
| ML-003 | 未实现 | [`stairs`](https://ww2.mathworks.cn/help/matlab/ref/stairs.html) | 阶梯图，2D | 阶跃边界、端点、非等距 X |
| ML-004 | 未实现 | [`errorbar`](https://ww2.mathworks.cn/help/matlab/ref/errorbar.html) | 误差棒图，2D | 水平 / 垂直、对称 / 非对称误差、端帽 |
| ML-005 | 未实现 | [`area`](https://ww2.mathworks.cn/help/matlab/ref/area.html) | 面积图 / 堆积面积图，2D | 基线、堆积、正负值与透明叠加 |
| ML-006 | 未实现 | [`stackedplot`](https://ww2.mathworks.cn/help/matlab/ref/stackedplot.html) | 共用 X 轴的纵向堆叠图，复合 2D | 各行独立 Y 轴、共享 X、时间数据与联动 |
| ML-007 | 开发中 | [`loglog`](https://ww2.mathworks.cn/help/matlab/ref/loglog.html) | 双对数折线，2D | 已有双 log 轴、非正值断段、导航 / 拾取与测试；导出和完整验收仍缺 |
| ML-008 | 开发中 | [`semilogx`](https://ww2.mathworks.cn/help/matlab/ref/semilogx.html) | X 对数折线，2D | 已有 X log / Y linear 及正反变换；导出和完整验收仍缺 |
| ML-009 | 开发中 | [`semilogy`](https://ww2.mathworks.cn/help/matlab/ref/semilogy.html) | Y 对数折线，2D | 已有 X linear / Y log 及正反变换；导出和完整验收仍缺 |
| ML-010 | 未实现 | [`fplot`](https://ww2.mathworks.cn/help/matlab/ref/fplot.html) | 显式 / 参数函数曲线，2D | 自适应采样、参数曲线、不连续点与预算限制 |
| ML-011 | 未实现 | [`fplot3`](https://ww2.mathworks.cn/help/matlab/ref/fplot3.html) | 参数函数曲线，3D | 三分量回调、参数区间、采样误差 |
| ML-012 | 未实现 | [`fimplicit`](https://ww2.mathworks.cn/help/matlab/ref/fimplicit.html) | 隐式曲线，2D | F(x,y)=0、多连通分支与奇异区域 |

### A2. 散点图和气泡图：8 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-013 | 开发中 | [`scatter`](https://ww2.mathworks.cn/help/matlab/ref/scatter.html) | 散点图，2D | 已有逐点颜色 / 半径、7 种实心标记与拾取；空心样式、大小语义适配、导出及完整验收仍缺 |
| ML-014 | 未实现 | [`scatter3`](https://ww2.mathworks.cn/help/matlab/ref/scatter3.html) | 散点图，3D | 深度排序 / 测试、三维拾取 |
| ML-015 | 未实现 | [`bubblechart`](https://ww2.mathworks.cn/help/matlab/ref/bubblechart.html) | 气泡图，2D | 气泡大小映射、大小范围与气泡图例 |
| ML-016 | 未实现 | [`bubblechart3`](https://ww2.mathworks.cn/help/matlab/ref/bubblechart3.html) | 气泡图，3D | 大小映射、深度与透明遮挡 |
| ML-017 | 未实现 | [`swarmchart`](https://ww2.mathworks.cn/help/matlab/ref/swarmchart.html) | 蜂群图，2D | 分类分组、避让布局、稳定可复现 |
| ML-018 | 未实现 | [`swarmchart3`](https://ww2.mathworks.cn/help/matlab/ref/swarmchart3.html) | 蜂群图，3D | 三维分组 / 抖动策略与原数据对应关系 |
| ML-019 | 未实现 | [`binscatter`](https://ww2.mathworks.cn/help/matlab/ref/binscatter.html) | 分箱散点密度图，2D | 二维分箱、计数映射、箱边界与色条 |
| ML-020 | 未实现 | [`spy`](https://ww2.mathworks.cn/help/matlab/ref/spy.html) | 稀疏矩阵非零模式图，2D | 行列方向、非零定位、大型稀疏输入 |

### A3. 数据分布与多变量图：13 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-021 | 未实现 | [`histogram`](https://ww2.mathworks.cn/help/matlab/ref/matlab.graphics.chart.primitive.histogram.html) | 一维直方图，2D | 箱边界、计数 / 概率 / 密度 / 累积归一化、分类数据 |
| ML-022 | 未实现 | [`histogram2`](https://ww2.mathworks.cn/help/matlab/ref/matlab.graphics.chart.primitive.histogram2.html) | 二维分箱直方图，2D / 3D | 二元分箱、柱体 / 平铺显示、归一化 |
| ML-023 | 未实现 | [`scatterhistogram`](https://ww2.mathworks.cn/help/matlab/ref/scatterhistogram.html) | 带边缘分布的散点图，复合 2D | 中央散点与边缘直方 / 密度的一致分组 |
| ML-024 | 未实现 | [`boxchart`](https://ww2.mathworks.cn/help/matlab/ref/boxchart.html) | 箱线图，2D | 四分位数、须、异常值、分组与方向 |
| ML-025 | 未实现 | [`violinplot`](https://ww2.mathworks.cn/help/matlab/ref/violinplot.html) | 小提琴图，2D | 核密度、带宽、归一化与分组 |
| ML-026 | 未实现 | [`raincloudplot`](https://ww2.mathworks.cn/help/matlab/ref/raincloudplot.html) | 雨云图，复合 2D | 半小提琴、箱线和原始点的一致统计与布局 |
| ML-027 | 未实现 | [`piechart`](https://ww2.mathworks.cn/help/matlab/ref/piechart.html) | 饼图，2D | 扇区比例、标签、图例与零 / 非法值规则 |
| ML-028 | 未实现 | [`donutchart`](https://ww2.mathworks.cn/help/matlab/ref/donutchart.html) | 环形图，2D | 内孔、扇区比例与中心标注布局 |
| ML-029 | 未实现 | [`wordcloud`](https://ww2.mathworks.cn/help/matlab/ref/wordcloud.html) | 词云，2D | 权重字号、文本避碰、字体和确定性布局 |
| ML-030 | 未实现 | [`bubblecloud`](https://ww2.mathworks.cn/help/matlab/ref/bubblecloud.html) | 气泡云，2D | 权重大小、圆形打包、标签与拾取 |
| ML-031 | 未实现 | [`heatmap`](https://ww2.mathworks.cn/help/matlab/ref/heatmap.html) | 热力图，2D | 类别轴、表格聚合、缺失值、排序与色条；区别于 imagesc |
| ML-032 | 未实现 | [`parallelplot`](https://ww2.mathworks.cn/help/matlab/ref/parallelplot.html) | 平行坐标图，2D | 多轴缩放 / 标准化、分类分组、轴间连线 |
| ML-033 | 未实现 | [`plotmatrix`](https://ww2.mathworks.cn/help/matlab/ref/plotmatrix.html) | 散点矩阵，复合 2D | 两两变量组合、对角分布与子图标签 |

### A4. 离散数据图：7 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-034 | 未实现 | [`bar`](https://ww2.mathworks.cn/help/matlab/ref/bar.html) | 竖向柱状图，2D | 分组 / 堆积、正负值、柱宽与基线 |
| ML-035 | 未实现 | [`barh`](https://ww2.mathworks.cn/help/matlab/ref/barh.html) | 横向条形图，2D | 横向分组 / 堆积、分类标签 |
| ML-036 | 未实现 | [`bar3`](https://ww2.mathworks.cn/help/matlab/ref/bar3.html) | 柱状图，3D | 柱体布局、分组 / 堆积模式、遮挡与着色 |
| ML-037 | 未实现 | [`bar3h`](https://ww2.mathworks.cn/help/matlab/ref/bar3h.html) | 横向条形图，3D | 横向柱体布局与深度关系 |
| ML-038 | 未实现 | [`pareto`](https://ww2.mathworks.cn/help/matlab/ref/pareto.html) | 帕累托图，复合 2D | 降序柱、累计百分比、双刻度与截取规则 |
| ML-039 | 未实现 | [`stem`](https://ww2.mathworks.cn/help/matlab/ref/stem.html) | 针状图，2D | 针杆、端点标记、基线与方向 |
| ML-040 | 未实现 | [`stem3`](https://ww2.mathworks.cn/help/matlab/ref/stem3.html) | 针状图，3D | 三维位置、针杆与基面 |

`stairs` 已记录为 ML-003，不重复计数。

### A5. 地理图：4 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-041 | 未实现 | [`geoplot`](https://ww2.mathworks.cn/help/matlab/ref/geoplot.html) | 经纬度线图，地理 2D | 地理投影、跨日期变更线、底图与导航 |
| ML-042 | 未实现 | [`geoscatter`](https://ww2.mathworks.cn/help/matlab/ref/geoscatter.html) | 地理散点图，地理 2D | 经纬度定位、逐点样式、底图缩放 |
| ML-043 | 未实现 | [`geodensityplot`](https://ww2.mathworks.cn/help/matlab/ref/geodensityplot.html) | 地理密度图，地理 2D | 空间密度、半径 / 缩放语义、权重与透明度 |
| ML-044 | 未实现 | [`geobubble`](https://ww2.mathworks.cn/help/matlab/ref/geobubble.html) | 地理气泡图，地理 2D | 大小 / 类别映射、气泡图例、地理范围 |

底图数据源、许可、署名、网络缓存和离线策略必须单独落实；本库支持地理绘图不意味着可直接使用 MathWorks 的底图服务。

### A6. 极坐标图：6 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-045 | 未实现 | [`polarplot`](https://ww2.mathworks.cn/help/matlab/ref/polarplot.html) | 极坐标折线，极坐标 2D | 角度方向 / 零点、径向范围、跨周连线 |
| ML-046 | 未实现 | [`polarhistogram`](https://ww2.mathworks.cn/help/matlab/ref/polarhistogram.html) | 极坐标直方图 / 玫瑰图，极坐标 2D | 环绕分箱、归一化、扇区宽度 |
| ML-047 | 未实现 | [`polarscatter`](https://ww2.mathworks.cn/help/matlab/ref/polarscatter.html) | 极坐标散点，极坐标 2D | 角度 / 半径变换、逐点颜色 / 大小 |
| ML-048 | 未实现 | [`polarbubblechart`](https://ww2.mathworks.cn/help/matlab/ref/polarbubblechart.html) | 极坐标气泡图，极坐标 2D | 极坐标定位、大小映射和图例 |
| ML-049 | 未实现 | [`compassplot`](https://ww2.mathworks.cn/help/matlab/ref/compassplot.html) | 罗盘箭头图，极坐标 2D | 方向 / 模长、箭头样式与角刻度 |
| ML-050 | 未实现 | [`fpolarplot`](https://ww2.mathworks.cn/help/matlab/ref/fpolarplot.html) | 函数极坐标曲线，极坐标 2D | r=f(θ) 回调、定义域、自适应采样 |

### A7. 等高线图：5 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-051 | 未实现 | [`contour`](https://ww2.mathworks.cn/help/matlab/ref/contour.html) | 等高线，2D | 层级选择、连通性、鞍点与标签 |
| ML-052 | 未实现 | [`contourf`](https://ww2.mathworks.cn/help/matlab/ref/contourf.html) | 填充等高线，2D | 等值区域、孔洞、边界和色带 |
| ML-053 | 未实现 | [`contour3`](https://ww2.mathworks.cn/help/matlab/ref/contour3.html) | 等高线，3D | 等值线处于各自 Z 高度及三维遮挡 |
| ML-054 | 未实现 | [`contourslice`](https://ww2.mathworks.cn/help/matlab/ref/contourslice.html) | 体数据切片等高线，3D | 切片插值、等值提取、平面位置 |
| ML-055 | 未实现 | [`fcontour`](https://ww2.mathworks.cn/help/matlab/ref/fcontour.html) | 函数等高线，2D | 双变量函数采样、层级与误差预算 |

### A8. 向量场：3 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-056 | 未实现 | [`quiver`](https://ww2.mathworks.cn/help/matlab/ref/quiver.html) | 箭头向量场，2D | 分量方向、自动 / 手动缩放、箭头头部 |
| ML-057 | 未实现 | [`quiver3`](https://ww2.mathworks.cn/help/matlab/ref/quiver3.html) | 箭头向量场，3D | 三维方向、尺度、遮挡 |
| ML-058 | 未实现 | [`feather`](https://ww2.mathworks.cn/help/matlab/ref/feather.html) | 羽状图，2D | 沿基线排列向量、分量与复数输入适配 |

流线与流切片在 A10 记录，不重复计数。

### A9. 曲面图和网格图：12 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-059 | 未实现 | [`surf`](https://ww2.mathworks.cn/help/matlab/ref/surf.html) | 实体曲面，3D | 规则 / 参数网格、顶点颜色、法线、光照与透明 |
| ML-060 | 未实现 | [`surfc`](https://ww2.mathworks.cn/help/matlab/ref/surfc.html) | 曲面 + 底部等高线，复合 3D | 共享数据 / 色域与等高线投影 |
| ML-061 | 未实现 | [`surfl`](https://ww2.mathworks.cn/help/matlab/ref/surfl.html) | 基于颜色图光照的曲面，3D | 光照计算与颜色映射；区别于普通 surf 加灯光 |
| ML-062 | 未实现 | [`ribbon`](https://ww2.mathworks.cn/help/matlab/ref/ribbon.html) | 带状图，3D | 多序列带宽、位置与颜色 |
| ML-063 | 未实现 | [`pcolor`](https://ww2.mathworks.cn/help/matlab/ref/pcolor.html) | 伪彩色网格图，2D | 顶点 / 面着色及边缘行为；区别于 image 的像素语义 |
| ML-064 | 未实现 | [`fsurf`](https://ww2.mathworks.cn/help/matlab/ref/fsurf.html) | 显式 / 参数函数曲面，3D | 定义域、自适应细分、参数曲面 |
| ML-065 | 未实现 | [`fimplicit3`](https://ww2.mathworks.cn/help/matlab/ref/fimplicit3.html) | 隐式曲面，3D | F(x,y,z)=0、拓扑、法线与提取误差 |
| ML-066 | 未实现 | [`mesh`](https://ww2.mathworks.cn/help/matlab/ref/mesh.html) | 线框曲面，3D | 网格边、面可见性与隐藏线 |
| ML-067 | 未实现 | [`meshc`](https://ww2.mathworks.cn/help/matlab/ref/meshc.html) | 网格 + 底部等高线，复合 3D | 网格与等高线共享范围 / 颜色 |
| ML-068 | 未实现 | [`meshz`](https://ww2.mathworks.cn/help/matlab/ref/meshz.html) | 带垂帘的网格，3D | 边界垂帘、基面与遮挡 |
| ML-069 | 未实现 | [`waterfall`](https://ww2.mathworks.cn/help/matlab/ref/waterfall.html) | 瀑布图，3D | 分行线 / 面、基面与深度遮挡 |
| ML-070 | 未实现 | [`fmesh`](https://ww2.mathworks.cn/help/matlab/ref/fmesh.html) | 函数线框曲面，3D | 函数 / 参数曲面采样与网格密度 |

### A10. 体数据与流可视化：7 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-071 | 未实现 | [`streamline`](https://ww2.mathworks.cn/help/matlab/ref/streamline.html) | 流线，2D / 3D | 起始点、积分方向 / 步长、停止条件与回流 |
| ML-072 | 未实现 | [`streamslice`](https://ww2.mathworks.cn/help/matlab/ref/streamslice.html) | 流切片，2D / 3D | 平面内向量投影、密度、方向指示 |
| ML-073 | 未实现 | [`streamparticles`](https://ww2.mathworks.cn/help/matlab/ref/streamparticles.html) | 流粒子动画，时变 3D | 粒子沿流线运动、速度与时间控制 |
| ML-074 | 未实现 | [`streamribbon`](https://ww2.mathworks.cn/help/matlab/ref/streamribbon.html) | 流带，3D | 带宽、扭转 / 旋度、法线 |
| ML-075 | 未实现 | [`streamtube`](https://ww2.mathworks.cn/help/matlab/ref/streamtube.html) | 流管，3D | 管径 / 散度映射、网格质量 |
| ML-076 | 未实现 | [`coneplot`](https://ww2.mathworks.cn/help/matlab/ref/coneplot.html) | 圆锥向量场，3D | 圆锥方向、长度 / 尺度与采样位置 |
| ML-077 | 未实现 | [`slice`](https://ww2.mathworks.cn/help/matlab/ref/slice.html) | 体数据切片，3D | 轴对齐 / 指定切面、体插值、色彩与透明度 |

等值面和端盖未出现在官方总览，已补为 EX-023 / EX-024。`volshow` 属于工具箱扩展，不冒充 MATLAB 本体函数。

### A11. 动画：3 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-078 | 未实现 | [`animatedline`](https://ww2.mathworks.cn/help/matlab/ref/animatedline.html) | 增量动画曲线，2D / 3D | 批量追加、有界历史、暂停 / 清空与资源复用 |
| ML-079 | 未实现 | [`comet`](https://ww2.mathworks.cn/help/matlab/ref/comet.html) | 彗星轨迹，时变 2D | 头部、尾迹长度、播放速度与时间一致性 |
| ML-080 | 未实现 | [`comet3`](https://ww2.mathworks.cn/help/matlab/ref/comet3.html) | 彗星轨迹，时变 3D | 三维轨迹、尾迹和相机交互 |

### A12. 图像：2 项

| ID | 状态 | MATLAB 函数 | 图类型 / 维度 | 专项验收 |
| --- | --- | --- | --- | --- |
| ML-081 | 未实现 | [`image`](https://ww2.mathworks.cn/help/matlab/ref/image.html) | 图像 / 索引色图，2D | RGB / 索引色、像素边界、坐标方向、透明度 |
| ML-082 | 未实现 | [`imagesc`](https://ww2.mathworks.cn/help/matlab/ref/imagesc.html) | 缩放色值图像，2D | 标量到色图映射、色域、缺失值与色条 |

## B. 本体补充清单：35 项

这些条目不在 A 表总览中，但对“完整绘图能力”不可忽略。它们包括基础图元、组合能力及对象方法，不全部是独立统计图类型。

### B1. 图元、几何与网格：17 项

| ID | 状态 | MATLAB 函数 / 方法 | 绘制能力 | 专项验收 |
| --- | --- | --- | --- | --- |
| EX-001 | 未实现 | [`line`](https://ww2.mathworks.cn/help/matlab/ref/line.html) | 2D / 3D 底层线图元 | 独立创建 / 更新、不隐式清除其他对象 |
| EX-002 | 未实现 | [`fill`](https://ww2.mathworks.cn/help/matlab/ref/fill.html) | 二维填充多边形 / 置信带 | 凹多边形、多个区域、颜色与透明度 |
| EX-003 | 未实现 | [`fill3`](https://ww2.mathworks.cn/help/matlab/ref/fill3.html) | 三维填充多边形 | 面朝向、三角化与深度 |
| EX-004 | 未实现 | [`patch`](https://ww2.mathworks.cn/help/matlab/ref/patch.html) | 通用 2D / 3D 面片 | Faces / Vertices、逐面 / 逐顶点颜色与透明 |
| EX-005 | 未实现 | [`rectangle`](https://ww2.mathworks.cn/help/matlab/ref/rectangle.html) | 矩形 / 圆角矩形 / 椭圆 | 数据坐标、曲率、填充与边框 |
| EX-006 | 未实现 | [`surface`](https://ww2.mathworks.cn/help/matlab/ref/surface.html) | 底层三维曲面图元 | 网格、面 / 边样式、纹理及更新 |
| EX-007 | 未实现 | [`surfnorm`](https://ww2.mathworks.cn/help/matlab/ref/surfnorm.html) | 曲面及法线可视化 | 法线计算、方向和长度 |
| EX-008 | 未实现 | [`sphere`](https://ww2.mathworks.cn/help/matlab/ref/sphere.html) | 球面 | 无输出绘制用法、网格分辨率与极点 |
| EX-009 | 未实现 | [`cylinder`](https://ww2.mathworks.cn/help/matlab/ref/cylinder.html) | 圆柱 / 旋转曲面 | 无输出绘制用法、半径轮廓及接缝 |
| EX-010 | 未实现 | [`ellipsoid`](https://ww2.mathworks.cn/help/matlab/ref/ellipsoid.html) | 椭球面 | 中心、三轴半径、网格分辨率 |
| EX-011 | 未实现 | [`triplot`](https://ww2.mathworks.cn/help/matlab/ref/triplot.html) | 二维三角剖分线图 | 顶点 / 索引输入、边去重与裁剪 |
| EX-012 | 未实现 | [`trimesh`](https://ww2.mathworks.cn/help/matlab/ref/trimesh.html) | 三角网格线框，2D / 3D | 非规则网格、着色与隐藏线 |
| EX-013 | 未实现 | [`trisurf`](https://ww2.mathworks.cn/help/matlab/ref/trisurf.html) | 三角曲面，3D | 连接关系、法线、面色与拾取 |
| EX-014 | 未实现 | [`tetramesh`](https://ww2.mathworks.cn/help/matlab/ref/tetramesh.html) | 四面体网格，3D | 单元面、内部面 / 透明策略与索引 |
| EX-015 | 未实现 | [`voronoi`](https://ww2.mathworks.cn/help/matlab/ref/voronoi.html) | 二维 Voronoi 图 | 有界边、视区裁剪、重复 / 退化点策略 |
| EX-016 | 未实现 | [`plot(polyshape)`](https://ww2.mathworks.cn/help/matlab/ref/polyshape.plot.html) | 多区域 / 带孔多边形 | 孔洞、边界、多个不连通区域 |
| EX-017 | 未实现 | [`plot(alphaShape)`](https://ww2.mathworks.cn/help/matlab/ref/alphashape.plot.html) | 二维 / 三维 alpha 形状 | 边界与区域渲染；几何生成算法单独说明 |

### B2. 图网络与树：5 项

| ID | 状态 | MATLAB 函数 / 方法 | 绘制能力 | 专项验收 |
| --- | --- | --- | --- | --- |
| EX-018 | 未实现 | [`plot(graph)`](https://ww2.mathworks.cn/help/matlab/ref/graph.plot.html) | 无向网络图，2D / 3D | 节点 / 边属性、布局、自环、多重边与标签 |
| EX-019 | 未实现 | [`plot(digraph)`](https://ww2.mathworks.cn/help/matlab/ref/graph.plot.html) | 有向网络图，2D / 3D | 箭头、方向、层级布局和高亮 |
| EX-020 | 未实现 | [`gplot`](https://ww2.mathworks.cn/help/matlab/ref/gplot.html) | 邻接矩阵图 | 邻接关系与指定节点坐标 |
| EX-021 | 未实现 | [`treeplot`](https://ww2.mathworks.cn/help/matlab/ref/treeplot.html) | 树 / 森林图 | 父索引、多根节点、层级布局 |
| EX-022 | 未实现 | [`etreeplot`](https://ww2.mathworks.cn/help/matlab/ref/etreeplot.html) | 稀疏矩阵消去树图 | 消去树计算与层级可视化分别验证 |

### B3. 体几何与图像显示：3 项

| ID | 状态 | MATLAB 函数 | 绘制能力 | 专项验收 |
| --- | --- | --- | --- | --- |
| EX-023 | 未实现 | [`isosurface`](https://ww2.mathworks.cn/help/matlab/ref/isosurface.html) | 体数据等值面，3D | 等值面提取、拓扑、法线与多等值层 |
| EX-024 | 未实现 | [`isocaps`](https://ww2.mathworks.cn/help/matlab/ref/isocaps.html) | 等值面端盖，3D | 切面封口、上下阈值、颜色与边缘对齐 |
| EX-025 | 未实现 | [`imshow`](https://ww2.mathworks.cn/help/matlab/ref/imshow.html) | 图像显示视图，2D | 灰度 / RGB / 索引色、显示范围、等像素比例；参考版本中属于本体 |

### B4. 文本、参考线、参考面与区域：10 项

| ID | 状态 | MATLAB 函数 | 绘制能力 | 专项验收 |
| --- | --- | --- | --- | --- |
| EX-026 | 未实现 | [`text`](https://ww2.mathworks.cn/help/matlab/ref/text.html) | 2D / 3D 数据文本 | 坐标锚定、旋转、对齐、字体与裁剪 |
| EX-027 | 未实现 | [`annotation`](https://ww2.mathworks.cn/help/matlab/ref/annotation.html) | 图窗注解图元 | line、arrow、doublearrow、textarrow、textbox、rectangle、ellipse 七种；图窗 / 数据坐标分离 |
| EX-028 | 未实现 | [`xline`](https://ww2.mathworks.cn/help/matlab/ref/xline.html) | 竖直常量参考线 | 多条线、标签、缩放后的跨区显示 |
| EX-029 | 未实现 | [`yline`](https://ww2.mathworks.cn/help/matlab/ref/yline.html) | 水平常量参考线 | 多条线、标签、图例和裁剪 |
| EX-030 | 未实现 | [`xregion`](https://ww2.mathworks.cn/help/matlab/ref/xregion.html) | X 区间填充带 | 区间端点、无限延展方向、透明叠加 |
| EX-031 | 未实现 | [`yregion`](https://ww2.mathworks.cn/help/matlab/ref/yregion.html) | Y 区间填充带 | 区间端点、范围更新和标签 |
| EX-032 | 未实现 | [`constantplane`](https://ww2.mathworks.cn/help/matlab/ref/constantplane.html) | 三维无限参考平面 | 平面方程、可见范围裁剪与透明深度 |
| EX-033 | 未实现 | [`thetaregion`](https://ww2.mathworks.cn/help/matlab/ref/thetaregion.html) | 极坐标角度扇区 | 跨零角度、方向和径向裁剪 |
| EX-034 | 未实现 | [`radiusregion`](https://ww2.mathworks.cn/help/matlab/ref/radiusregion.html) | 极坐标圆盘 / 圆环 | 内外半径与角度范围裁剪 |
| EX-035 | 未实现 | [`polarregion`](https://ww2.mathworks.cn/help/matlab/ref/polarregion.html) | 极坐标环形扇区 | 角度区间与半径区间组合 |

## C. 传统及历史接口对照：16 项

不因官方推荐新接口就从对照表删除。交付重点是对应的图形能力与语义说明，不为复现旧调用方式引入 MATLAB 字符串表达式解释器。

| ID | 状态 | MATLAB 接口 | 图类型 / 定位 | 对照与迁移要求 |
| --- | --- | --- | --- | --- |
| LG-001 | 未实现 | [`pie`](https://ww2.mathworks.cn/help/matlab/ref/pie.html) | 传统二维饼图 | 与 piechart 关联；验证分离扇区、标签及小于整圆时的行为 |
| LG-002 | 未实现 | [`pie3`](https://ww2.mathworks.cn/help/matlab/ref/pie3.html) | 三维饼图 | 仍有独立视觉效果；扇区厚度、分离、遮挡与相机，不用二维饼图冒充 |
| LG-003 | 未实现 | [`hist`](https://ww2.mathworks.cn/help/matlab/ref/hist.html) | 旧直方图，官方不推荐 | 映射 histogram；箱中心与箱边界语义差异须测试 |
| LG-004 | 未实现 | [`polar`](https://ww2.mathworks.cn/help/matlab/ref/polar.html) | 旧极坐标折线，官方不推荐 | 映射 polarplot，说明旧笛卡尔模拟轴与真实极坐标轴差异 |
| LG-005 | 未实现 | [`compass`](https://ww2.mathworks.cn/help/matlab/ref/compass.html) | 旧罗盘图，官方不推荐 | 对照 compassplot，验证复数 / 分量输入适配 |
| LG-006 | 未实现 | [`rose`](https://ww2.mathworks.cn/help/matlab/ref/rose.html) | 旧角直方图，官方不推荐 | 对照 polarhistogram，不默认视为分箱与归一化完全等价 |
| LG-007 | 未实现 | [`plotyy`](https://ww2.mathworks.cn/help/matlab/ref/plotyy.html) | 旧双 Y 轴组合，官方不推荐 | 对照显式双 Y 轴模型；左右刻度、图元归属与联动 |
| LG-008 | 未实现 | [`ezplot`](https://ww2.mathworks.cn/help/matlab/ref/ezplot.html) | 旧二维函数图，官方不推荐 | 对照 fplot / fimplicit；显式、参数与隐式曲线 |
| LG-009 | 未实现 | [`ezplot3`](https://ww2.mathworks.cn/help/matlab/ref/ezplot3.html) | 旧三维参数曲线，官方不推荐 | 对照 fplot3 |
| LG-010 | 未实现 | [`ezpolar`](https://ww2.mathworks.cn/help/matlab/ref/ezpolar.html) | 旧极坐标函数图，官方不推荐 | 对照 fpolarplot |
| LG-011 | 未实现 | [`ezcontour`](https://ww2.mathworks.cn/help/matlab/ref/ezcontour.html) | 旧函数等高线，官方不推荐 | 对照 fcontour |
| LG-012 | 未实现 | [`ezcontourf`](https://ww2.mathworks.cn/help/matlab/ref/ezcontourf.html) | 旧函数填充等高线，官方不推荐 | 函数采样 + contourf，不能用仅线条的 fcontour 替代填充 |
| LG-013 | 未实现 | [`ezmesh`](https://ww2.mathworks.cn/help/matlab/ref/ezmesh.html) | 旧函数网格，官方不推荐 | 对照 fmesh |
| LG-014 | 未实现 | [`ezmeshc`](https://ww2.mathworks.cn/help/matlab/ref/ezmeshc.html) | 旧函数网格 + 等高线，官方不推荐 | 函数采样 + meshc |
| LG-015 | 未实现 | [`ezsurf`](https://ww2.mathworks.cn/help/matlab/ref/ezsurf.html) | 旧函数曲面，官方不推荐 | 对照 fsurf |
| LG-016 | 未实现 | [`ezsurfc`](https://ww2.mathworks.cn/help/matlab/ref/ezsurfc.html) | 旧函数曲面 + 等高线，官方不推荐 | 函数采样 + surfc |

## D. 工具箱扩展索引：与本体分账

以下函数 / 视图已核对官方页面的产品归属，但**本节不声称覆盖这些工具箱的全部可视化接口**。同一种视觉图形可能来自多个产品；同名接口也不保证语义相同。扩展项需在产品范围冻结后拆成逐函数交付条目，不能用本节的一行概括整个工具箱完成。

| 扩展编号 | 所属产品 | 图类型与已核对接口 | 交付边界 |
| --- | --- | --- | --- |
| TB-01 | Statistics and Machine Learning Toolbox | 箱线图 [`boxplot`](https://ww2.mathworks.cn/help/stats/boxplot.html)；二元直方图 [`hist3`](https://ww2.mathworks.cn/help/stats/hist3.html) | 不与本体 boxchart / histogram2 混淆；hist3 官方不推荐 |
| TB-02 | Statistics and Machine Learning Toolbox | 分组散点 [`gscatter`](https://ww2.mathworks.cn/help/stats/gscatter.html)、分组散点矩阵 [`gplotmatrix`](https://ww2.mathworks.cn/help/stats/gplotmatrix.html)、边缘直方散点 [`scatterhist`](https://ww2.mathworks.cn/help/stats/scatterhist.html) | 与本体 scatterhistogram 分别对照 |
| TB-03 | Statistics and Machine Learning Toolbox | Q-Q 图 [`qqplot`](https://ww2.mathworks.cn/help/stats/qqplot.html)、概率图 [`probplot`](https://ww2.mathworks.cn/help/stats/probplot.html)、正态概率图 [`normplot`](https://ww2.mathworks.cn/help/stats/normplot.html)、Weibull 概率图 [`wblplot`](https://ww2.mathworks.cn/help/stats/wblplot.html) | 分位数、参考分布与概率刻度需统计计算支持 |
| TB-04 | Statistics and Machine Learning Toolbox | 经验 CDF [`cdfplot`](https://ww2.mathworks.cn/help/stats/cdfplot.html) / [`ecdf`](https://ww2.mathworks.cn/help/stats/ecdf.html)、核密度 [`ksdensity`](https://ww2.mathworks.cn/help/stats/ksdensity.html) | 区分绘制预计算数据与提供统计估计器 |
| TB-05 | Statistics and Machine Learning Toolbox | 双标图 [`biplot`](https://ww2.mathworks.cn/help/stats/biplot.html)、Andrews 曲线 [`andrewsplot`](https://ww2.mathworks.cn/help/stats/andrewsplot.html)、平行坐标 [`parallelcoords`](https://ww2.mathworks.cn/help/stats/parallelcoords.html)、字形图 [`glyphplot`](https://ww2.mathworks.cn/help/stats/glyphplot.html) | 与本体 parallelplot 分开；字形布局和数据变换需说明 |
| TB-06 | Statistics and Machine Learning Toolbox | 聚类树 [`dendrogram`](https://ww2.mathworks.cn/help/stats/dendrogram.html)、交互轮廓图 [`surfht`](https://ww2.mathworks.cn/help/stats/surfht.html)、多变量图 [`multivarichart`](https://ww2.mathworks.cn/help/stats/multivarichart.html) | 聚类 / 模型计算不因可渲染就视为实现 |
| TB-07 | Statistics and Machine Learning Toolbox | 控制图 [`controlchart`](https://ww2.mathworks.cn/help/stats/controlchart.html)、交互作用图 [`interactionplot`](https://ww2.mathworks.cn/help/stats/interactionplot.html)、主效应图 [`maineffectsplot`](https://ww2.mathworks.cn/help/stats/maineffectsplot.html) | 明确统计规则、控制限与分组算法 |
| TB-08 | Statistics and Machine Learning Toolbox | 混淆矩阵图 [`confusionchart`](https://ww2.mathworks.cn/help/stats/confusionchart.html) | 类别顺序、归一化、行列摘要，不仅是普通热力图 |
| TB-09 | Control System Toolbox | Bode 图 [`bode`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.bode.html) / [`bodemag`](https://ww2.mathworks.cn/help/control/ref/bodemag.html)、Nyquist 图 [`nyquist`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.nyquist.html)、Nichols 图 [`nichols`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.nichols.html) | 频率响应计算与绘图分层；单位、相位展开、稳定裕度另验收 |
| TB-10 | Control System Toolbox | 根轨迹 [`rlocus`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.rlocus.html)、奇异值响应 [`sigma`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.sigma.html) | 需要系统模型及分析计算，不能仅以折线支持替代 |
| TB-11 | Control System Toolbox | 阶跃响应 [`step`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.step.html)、冲激响应 [`impulse`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.impulse.html)、任意输入响应 [`lsim`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.lsim.html)、初始状态响应 [`initial`](https://ww2.mathworks.cn/help/control/ref/dynamicsystem.initial.html) | SISO / MIMO、时域仿真和多面板语义 |
| TB-12 | Signal Processing Toolbox | 频谱 / PSD [`pspectrum`](https://ww2.mathworks.cn/help/signal/ref/pspectrum.html)、[`periodogram`](https://ww2.mathworks.cn/help/signal/ref/periodogram.html)、[`pwelch`](https://ww2.mathworks.cn/help/signal/ref/pwelch.html) | 频谱估计、窗函数、单位与单 / 双边谱 |
| TB-13 | Signal Processing Toolbox | 时频谱 [`spectrogram`](https://ww2.mathworks.cn/help/signal/ref/spectrogram.html) / [`stft`](https://ww2.mathworks.cn/help/signal/ref/stft.html) | 时频计算与热力图渲染分别验收 |
| TB-14 | Signal Processing Toolbox | 滤波器频响 [`freqz`](https://ww2.mathworks.cn/help/signal/ref/freqz.html)、零极点图 [`zplane`](https://ww2.mathworks.cn/help/signal/ref/zplane.html) | 复平面、单位圆、幅相坐标及滤波器计算 |
| TB-15 | Wavelet Toolbox | 小波尺度图 [`cwt`](https://ww2.mathworks.cn/help/wavelet/ref/cwt.html)、小波相干图 [`wcoherence`](https://ww2.mathworks.cn/help/wavelet/ref/wcoherence.html) | 小波变换、对数尺度及影响锥等分析语义 |
| TB-16 | Image Processing Toolbox | 体渲染 [`volshow`](https://ww2.mathworks.cn/help/images/ref/volshow.html)、切片浏览 [`sliceViewer`](https://ww2.mathworks.cn/help/images/ref/sliceviewer.html)、图像对比 [`imshowpair`](https://ww2.mathworks.cn/help/images/ref/imshowpair.html)、拼图 [`montage`](https://ww2.mathworks.cn/help/images/ref/montage.html) | 体绘制传递函数、切片导航、配准坐标与合成方式 |
| TB-17 | Computer Vision Toolbox | 点云显示 [`pcshow`](https://ww2.mathworks.cn/help/vision/ref/pcshow.html) | 点云尺寸、RGB / 强度映射、相机与拾取 |
| TB-18 | Mapping Toolbox | 三维地球轨迹 [`geoplot3`](https://ww2.mathworks.cn/help/map/ref/geoplot3.html)、地理图标 [`geoiconchart`](https://ww2.mathworks.cn/help/map/ref/geoiconchart.html) | 地球 / 高程 / 地形与地图资源许可；不是本体 geoplot 的别名 |
| TB-19 | RF Toolbox | Smith 圆图 [`smithplot`](https://ww2.mathworks.cn/help/rf/ref/smithplot.html) | 复阻抗 / 反射系数变换及专用网格 |
| TB-20 | Financial Toolbox | K 线图 [`candle`](https://ww2.mathworks.cn/help/finance/candle.html)、OHLC 竹线图 [`highlow`](https://ww2.mathworks.cn/help/finance/highlow.html) | 开高低收语义、交易日期与涨跌样式 |
| TB-21 | Bioinformatics Toolbox | 聚类热图 [`clustergram`](https://ww2.mathworks.cn/help/bioinfo/ref/clustergram.html) | 行列聚类树、重排、缩放与数据映射 |

其他专业产品的绘图仍是开放范围，例如天线 / 雷达方向图、通信星座与眼图、机器人 / PDE / 深度学习模型可视化等。未逐产品清点之前，不能把它们标为“无遗漏”或默认计入本库已承诺范围。File Exchange / 第三方的 Sankey、雷达蜘蛛图、treemap 等，也不能只因能在 MATLAB 运行就列为 MATLAB 本体内置函数。

## E. 跨图类型交付项：不计入 133 项

以下项目为各图类型共用的能力矩阵，尚未整项验收；线性 / 对数 / 反向 / 等比例、刻度、混合序列、图例与拾取已有局部实现，见 [阶段 2](STAGE_2.md)。每种图需要标注适用 / 不适用，并引用相应测试。

| 编号 | 能力 | MATLAB 对照 / 验收范围 |
| --- | --- | --- |
| CAP-01 | 多图布局 | `tiledlayout`、`nexttile`、`subplot`；跨行列、间距、公共标题 / 图例 / 色条 |
| CAP-02 | 多图元组合 | `hold` 对应的显式叠加；折线 + 散点、柱 + 误差棒、等高线 + 向量场等 |
| CAP-03 | 多坐标轴 | `yyaxis`、多 Axes 叠加；数据所属轴、独立尺度与联动 |
| CAP-04 | 坐标类型 | 线性 / 对数、反向轴、日期时间 / 时长、类别轴、等比例与手动范围 |
| CAP-05 | 坐标系 | 笛卡尔、极坐标、地理、三维相机；不能仅替换标签假装支持 |
| CAP-06 | 刻度网格 | 主次刻度、格式器、指数、日期 / 角度格式、主次网格 |
| CAP-07 | 文本标识 | 标题、副标题、XYZ 标签、总标题、图例、气泡图例、色条与等高线标签 |
| CAP-08 | 字体排版 | 中文与用户字体、旋转、换行、上下标 / 希腊字母；完整 LaTeX 仍是独立范围 |
| CAP-09 | 样式主题 | 颜色序列、线型、标记、填充、透明度、主题继承和局部覆盖 |
| CAP-10 | 颜色映射 | 连续 / 离散色图、色域、逐点 / 面 / 顶点着色、索引色与 truecolor |
| CAP-11 | 基本交互 | 平移、锚点缩放、框选缩放、复位、三维旋转、键鼠映射 |
| CAP-12 | 数据探索 | 数据提示、十字线、原始数据拾取、数据刷选、序列显隐 |
| CAP-13 | 多图联动 | `linkaxes` / `linkprop` 对应能力；事件传播与防循环 |
| CAP-14 | 动态更新 | 追加 / 替换 / 删除、固定历史窗口、重绘节流、缓存失效 |
| CAP-15 | 三维呈现 | 正交 / 透视、深度、法线、光照、材质、纹理、透明度与裁剪 |
| CAP-16 | 图像组合 | `imtile` 类图像平铺作为组合能力；图像数据处理不另计为一种图表 |
| CAP-17 | 图形变换 | 图元分组、平移 / 旋转 / 缩放、场景层次与动画变换 |
| CAP-18 | 输出 | PNG / SVG、尺寸 / DPI、背景、字体、混合栅格声明；动画输出单独立项 |
| CAP-19 | 大数据 | 分图类型 LOD / 索引策略、内存预算、取消过时任务、冷 / 热路径性能 |
| CAP-20 | 输入与计算 | 数组 / 分组 / 时间 / 类别输入适配；统计、几何、流积分与函数采样的计算责任明确 |

不作为独立图类型重复计数的例子：`contourc` 计算等值线数据；`histcounts` / `histcounts2` / `histc` 计数；`stream2` / `stream3` 计算流线；`delaunay` / `convhull` / `voronoin` 计算几何；`colormap` / `clim` / `shading` 控制呈现；`figure` / `axes` 管理容器。是否实现这些计算接口，与是否具备对应可视化需分别记录。

## 3. 与项目阶段的关系

1. **阶段 1～2：** 先完成基础二维可用切片，不使用“MATLAB 全类型支持”的宣传。
2. **阶段 3～7：** 完善定制化、数据性能、渲染后端、常用二维扩展与导出。
3. **阶段 8：** 补齐统计 / 多变量 / 函数 / 极坐标 / 几何 / 图网络二维能力。
4. **阶段 9：** 独立建设三维与体数据模块，补齐三维图、曲面网格、流与等值面。
5. **阶段 10：** 补齐地理绘图及地图资源策略。
6. **阶段 11：** 核销 A / B / C 全部条目和适用的 CAP 项，复核历史接口对应效果、版本差异和导出限制；任何未实现项都必须保留可见，不能从分母静默删除。
7. **工具箱扩展：** 待单独冻结产品范围后分包实施；不拿“能画线 / 热力图”代替“已实现对应领域分析图”。

## 4. 官方来源与复核方法

### 4.1 总览及分类来源

- [S01：MATLAB 绘图类型，R2026b](https://ww2.mathworks.cn/help/matlab/creating_plots/types-of-matlab-plots.html)
- [S02：线图](https://ww2.mathworks.cn/help/matlab/line-plots.html)
- [S03：数据分布图](https://ww2.mathworks.cn/help/matlab/pie-charts-bar-plots-and-histograms.html)
- [S04：离散数据图](https://ww2.mathworks.cn/help/matlab/stem-and-stair-plots.html)
- [S05：地理图](https://ww2.mathworks.cn/help/matlab/geographic-plots.html)
- [S06：极坐标图](https://ww2.mathworks.cn/help/matlab/polar-plots.html)
- [S07：等高线图](https://ww2.mathworks.cn/help/matlab/contour-plots-1.html)
- [S08：向量场](https://ww2.mathworks.cn/help/matlab/vector-fields.html)
- [S09：曲面图和网格图](https://ww2.mathworks.cn/help/matlab/surface-and-mesh-plots-1.html)
- [S10：体数据可视化](https://ww2.mathworks.cn/help/matlab/volume-visualization.html)
- [S11：动画](https://ww2.mathworks.cn/help/matlab/animation-1.html)
- [S12：图像](https://ww2.mathworks.cn/help/matlab/graphics-images.html)
- [S13：多边形图元](https://ww2.mathworks.cn/help/matlab/geometry.html)
- [S14：三角剖分](https://ww2.mathworks.cn/help/matlab/triangulation-representation.html)
- [S15：边界区域 / alpha 形状](https://ww2.mathworks.cn/help/matlab/2-d-and-3-d-bounding-regions.html)
- [S16：多边形形状](https://ww2.mathworks.cn/help/matlab/elementary-polygons.html)
- [S17：图和网络算法](https://ww2.mathworks.cn/help/matlab/graph-and-network-algorithms.html)
- [S18：稀疏矩阵及树图](https://ww2.mathworks.cn/help/matlab/sparse-matrices.html)
- [S19：标签和注解](https://ww2.mathworks.cn/help/matlab/titles-and-labels.html)

逐项链接给出函数级来源；传统接口以对应函数页的版本历史和“不推荐”提示为准。工具箱归属按文档产品路径核对，不能用搜索到同名函数就判断属于 MATLAB 本体。

### 4.2 更新与完整性检查

- 将官方总览表格中的函数链接提取并按函数名去重，和快照中的 82 个名字做集合差异比较。
- A 表的函数集合必须与快照相等；ID 必须唯一，不能通过重复计数提高覆盖率。
- B / C 表维持独立编号与状态；新增条目要记录来源及图类型 / 图元 / 方法 / 兼容组合的定位。
- 每次发布同时报告 A、B、C 的覆盖率、适用 CAP 项和 D 的已冻结范围，不能只报一个模糊的“MATLAB 兼容率”。
