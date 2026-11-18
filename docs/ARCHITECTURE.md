# 源码组织与依赖边界

本文件约束**已经落地**的目录，不是将来再整理的愿望清单。当前一个库 crate 加一个 demo，公开类型统一从 `egui_customplot` 根导出；模块迁移不要求用户更改导入路径。

## 目录与职责

```text
crates/egui-customplot/
  src/
    lib.rs                    # 库说明、私有模块声明、公开导出
    diagnostics.rs            # 跨阶段诊断契约，不执行绘制或采样
    data/
      points.rs               # Arc 样本、自动单调 X 树构建（>=8,192 样本，64 MiB 上限）与显式禁用入口
      index/                  # 单调 X 索引（内联极值、gallop 划分与 2D 屏幕 AABB 最优优先原始拾取剪枝）
      stream.rs               # 固定容量环形样本、批次幂等与显式快照发布
      stream_indexed/         # 固定容量环形缓冲 + 128 槽增量物理线段树，连续有效的像素列只查询 Y 极值，混合有效域按段处理
      waveform/               # 均匀步长 f32 波形源（x=x0+i*dx）+ 64 样本增量树；extrema.rs 按像素列裁剪边界扫描，pick.rs 做二维拾取，写入记录脏区间供上传
      grid.rs                 # 共享边缘 / 行优先标量、数值 / log 域预处理
      extents.rs              # 独立图元四坐标域固定范围缓存
      bars.rs                 # 原始柱记录、两方向范围摘要
      error_bars.rs           # 非对称误差偏移、整项范围 / 域摘要
      quiver.rs               # Arc 原点 / 位移记录、真实端点及四域范围摘要
      histogram.rs            # 显式分箱计数、归一化和 BarData 发布
      triangulate.rs          # f64 耳切三角化与 Eberly 孔洞桥接
      area.rs / polygon.rs    # AreaBandData（堆积/上下带）与凹多边形 / 置信带四域三角化
      patch.rs / shapes.rs    # PatchData 通用面片与 DataRect / 椭圆范围摘要
      polyshape.rs            # 带孔多区域多边形与四域预处理
      field2d.rs / contour/   # 二维节点标量场、函数采样预算与 Marching Squares 等值提取
      image.rs                # RGBA8 / RGB8 / 索引 / 灰度图像及像素坐标
      function/               # fplot 自适应采样 / fimplicit 极点守卫 / ezplot 预算
      delaunay/               # Bowyer-Watson TriMesh2D、裁剪 Voronoi 与 AlphaShapeData
      flow/                   # 二维规则向量场、RK4 streamline/streamslice 与 FeatherData
      spy.rs                  # 1-based CSR 稀疏矩阵模式存储与窗口剔除
      animation.rs            # 有界环形 AnimatedLine2D 与可重放 CometPlayback 游标
      bar_groups.rs / hist_legacy.rs # 分组 / 堆叠柱预处理与 Pareto / rose / histfit 数据
      bubble.rs / bivariate.rs # Swarm / Bubble 与 histogram2 / binscatterHex 双变量聚合
      distribution/           # KDE 核密度与 BoxChart / Violin / Raincloud 分位摘要
      pie.rs / clouds.rs      # PieChart / DonutChart 与 WordCloud / BubbleCloud 螺旋防重叠布局
      multivariate/           # Categorical / Parallel / Stacked / ScatterHistogram / PlotMatrix 数据
      composite.rs            # StackedPlot / ScatterHistogram / PlotMatrix 复合切片与作用域聚合
      polar/                  # PolarCurve / PolarHistogram / Rose / PolarBubble / Compass / 自适应 fpolarplot 数据
      graph/                  # GraphData / Circle / Force / Sugiyama Layered / TreeData / Liu EliminationTree 数据
      plot3d/                 # Stage 9A～9C、13、16: 线/散点/柱/茎/矢量/多边形、曲面网格、ScalarVolume3D / VolumeSliceData / Isosurface3D / Isocaps3D / TetraMesh3D / Patch3DData / AlphaShape3DData / Graph3DData / PointCloud3DData / VolumeTransferFunction / Stream*3D / ConePlot3D
      geo/                    # Stage 10: GeoPathData / GeoScatterData / GeoDensityData / GeoBubbleData / OFFLINE_LAND_RINGS / GeoAttribution / GeoTileCache (零网络 I/O)
      geoglobe.rs             # Stage 16: WGS84 GeoGlobeData / GlobeTrack3D / GlobeMarker3D
      confusion.rs / sky.rs / curvilinear.rs # Stage 13: ConfusionMatrixData / SkyPlotData / CurvilinearField2D
      probplot.rs / ecdf.rs / clustering.rs / clustergram.rs / biplot.rs / spc.rs / financial.rs # Stage 14: 概率分位、KM ECDF、层次聚类/Clustergram、PCA Biplot、SPC 控制图与 OHLC/Waterfall
      smith.rs / zplane.rs / control/ / dsp/ # Stage 15: Smith 圆图、Aberth-Ehrlich 零极点、传递函数频响/根轨迹与 Radix-2+Bluestein FFT/STFT/Welch/Morlet CWT
      modern/                 # Stage 16: RadarData / SankeyData / TreemapData / SunburstData / GanttData（分拆于 radial_flow.rs / hier_gantt.rs / gantt.rs）
    model/
      line.rs                 # 折线的借用描述（默认 LineSampling::Auto / RenderBackend::Auto）与 Line::primitive
      line_pattern.rs         # 线型语义与参数合法性
      interpolation.rs        # 线性 / 前阶 / 后阶连接语义，原始数据不变
      sampling.rs             # LineSampling::{Auto, Exact, Visible, MinMax} / MarkerSampling / RenderBackend
      fast_series.rs          # StreamLine / WaveformLine 零拷贝借用描述符
      scatter.rs              # 散点描述与逐点样式校验（默认 MarkerSampling::Auto / RenderBackend::Auto）
      marker.rs               # 标记形状与尺寸
      series.rs               # 混合序列、稳定 ID、默认色
      style.rs                # 可选值覆盖、解析后的绘制 / 拾取共用样式
      defaults.rs             # Context 级 PlotDefaults 全局视图与外观默认覆盖（含 set_plot_defaults / plot_defaults）
      theme.rs                # 科学浅色 / 监控深色，可共享 palette
      annotation.rs           # 不参与自动范围的基础注解与 Data/Normalized EX-027 图元描述
      layer/                  # 兼容既有 Series 的有序异构 PlotLayer 与统一样式解析（含 stage8c / stage8d）
      summary/                # 合并预处理范围、身份与跨图元 payload 指针去重（含 stage8c / stage8d）
      heatmap.rs / color_scale.rs # 数值单元描述与显式共享色域
      bars.rs / error_bars.rs  # 柱 / 误差棒样式描述，不保存视图
      stems.rs                # 借用点样本的杆、方向 / 基线与 O(1) 范围
      quiver.rs               # 无隐式缩放的矢量场、逻辑点箭头及样式
      area.rs / polygon.rs / polyshape.rs # 面积、多边形与带孔区域图层模型
      patch.rs / shapes.rs / contour.rs   # 通用面片、矩形椭圆与等高线 / 填充等高线模型
      pcolor.rs / image.rs    # Pcolor 伪彩色网格（Flat / Interp）与图像图层模型
      function.rs / triangulation.rs # fplot / fimplicit 与 TriPlot / Voronoi / AlphaShape 图层模型
      flow.rs / spy.rs / animation.rs # Feather / Streamlines、Spy、AnimatedLine / Comet 图层模型
      bubble_swarm.rs / distribution.rs # Bubble / Swarm / Bivariate 与 Box / Violin / Raincloud 图层模型
      pie_clouds.rs / multivariate.rs   # Pie / Donut、WordCloud / BubbleCloud 与 ParallelCoords 图层模型
      polar.rs / polar_region.rs / graph.rs # Polar* 图层、PolarRegionAnnotation 与 GraphPlot / GraphHighlight
      plot3d/                 # Stage 9A～9C、13、16: scene_opts / Layer3D / surfaces / mesh_tri / volume_flow / stage13 / stage16 (VolumeRender3D, PointCloud3D) / summarize_plot3d
      geo.rs                  # Stage 10: GeoLine / GeoScatter / GeoDensity / GeoBubbles / GeoLayer / summarize_geo
      color_scale.rs / confusion.rs / curvilinear.rs / stage14_stats.rs / stage14_spc_fin.rs / stage15.rs / modern.rs # Stage 12～16 扩展色谱（11 种）、混淆矩阵、曲线场、概率/ECDF/聚类/双标图/SPC/金融、DSP/控制/RF 与现代图表图层模型
    axis/
      geo/                    # Stage 10: GeoPoint / GeoBounds(S^1最小弧/跨±180°) / GeoProjection(WebMercator/Equirectangular) / GeoTransform / antimeridian 分割 / 经纬刻度
      geoglobe.rs             # Stage 16: GeoGlobeTransform WGS84 球面正射投影、地平线背面剔除与轨角拖拽
      sky.rs / smith.rs       # Stage 13/15: SkyTransform 天球反向仰角投影与 SmithTransform 共形莫比乌斯变换
      scale.rs / probability.rs / time.rs # Linear / Log10 / Probit / Weibull（Acklam 逆正态）与 datetime/duration 时间刻度/历法
      bounds.rs               # f64 范围验证、适配与基本运算
      transform.rs            # 正反坐标映射、变换空间导航运算
      polar.rs                # PolarTransform 极坐标零点 / 方向与三种负半径映射
      aspect.rs               # 等比例约束，只扩展变换空间范围
      ticks.rs                # 坐标轴配置、主次刻度、数字格式
      grid.rs / rect.rs       # 单元可见查询、定位、f64 矩形 / 轴向线段裁剪
      segment.rs              # 共用归一化线段裁剪、向量投影及图网络曲线/自环几何
      polygon.rs              # [0,1]^2 Sutherland-Hodgman 三角裁剪与点环判定
      plot3d/                 # Stage 9A/9B/16: 三维坐标裁剪/投影、ClipVertex3D Gouraud 裁剪、Kay-Kajiya 射线包围盒与平面立方体求交
      y_axis.rs / linked.rs   # 左右归属、共享范围与变换组装
    layout/
      plot.rs / polar.rs / plot3d.rs # 宿主空间分配、极坐标正圆 / 三维 Plot3D 排布、标题 / 轴标签留白
      legend.rs               # egui 图例排布和控件组装
      figure.rs / color_bar.rs / bubble_legend.rs # 有界 cell 网格、色条与气泡尺寸图例空间分配
    interaction/
      state.rs                # 显式视图、隐藏状态、帧快照、一次性切换、2 MiB 内置 auto_cache 与刷选边界
      navigation.rs           # 输入仲裁、锁轴、框选缩放/数据刷选、同帧多遍导航重算
      picking.rs / point_aux.rs / point_fast.rs / point_stage8b.rs / point_stage8c.rs / point_stage14.rs / point_stage15.rs / graph.rs # 原始点（含流与均匀波形树剪枝拾取）、Stage 8A～16 及图节点/边拾取
      polar.rs / polar_hover.rs # 极坐标多遍旋转/径向缩放、图例开关与点/扇区拾取
      sky.rs / smith.rs / geoglobe.rs # Stage 13/15/16: 天球、Smith 圆图与 WGS84 地球仪多遍交互及要素拾取
      tooltip.rs              # HoverInfo、纯文本格式化与 egui 提示
      link.rs / secondary.rs  # 显式跨图同步、右轴同一导航动作
      cell.rs / bar.rs        # 原始单元 / 柱拾取
      quiver.rs               # 原始向量（Quiver / Feather）两锚点拾取、记录提示，复用点层距离仲裁
      region/                 # 面积 / 多边形 / 面片 / 矩形 / 等高线 / 伪彩 / 图像 / Alpha / Stage 8C、13、14、16（Sankey/Treemap/Sunburst/Gantt）与 GraphEdge 区域命中
      hover.rs                # 填充层遮挡（含 Area / GraphEdge 共层优先级）及原始点最近距离仲裁
      plot3d/                 # Stage 9A～9C、13、16: Plot3DState 导航、Legend、三维点/柱/填充面/扇区/曲面、体切片/四面体/流粒子实时拾取 (volume_hover) 及体渲染/八叉树点云拾取 (stage13_hover)
      geo/                    # Stage 10: GeoPlotState ±180° 平移与锚点缩放、图层/气泡类别 Legend、原始点/气泡/核密度拾取
    render/
      line.rs                 # 选点后的阶梯展开、f64 线段裁剪与路径准备
      line_query.rs           # 采样查询、缓存和参考几何准备的适配
      cache.rs                # 宿主持有的有界 pre-style 折线几何 LRU
      fast_series.rs          # StreamLine / WaveformLine CPU 几何准备与可选 wgpu 调度
      markers.rs              # 标记几何与物理像素羽化；密集小实心圆可交给 egui 圆片图集，快照使用独立网格
      markers_lod.rs          # MarkerSampling::Auto/PixelGrid 像素网格去重与 X 索引裁剪，按绘制顺序保留每格最后一个样本
      markers_grid_tree.rs    # 单调 X 散点/标记的线段树层级单格坍缩与 64 位占用掩码满格剪枝
      wgpu/                   # 可选 feature=wgpu：复用宿主 RenderState 的 WGSL 计算规约与实例化渲染（含散点深缩过绘制节流、15 种 SDF 标记与极端缩放防溢出，无硬件/导出时回退 Painter）
      pattern.rs              # 屏幕空间虚线 phase 与工作量安全阀
      mesh.rs                 # 16,384 三角分批 MeshBatch 与边框描边提交
      axes.rs                 # 网格、刻度文字、标题、边框绘制
      series.rs               # 按序提交数据图元、统计几何成本
      overlay.rs              # 十字线、拾取高亮（含多边形/区域/刷选框与点高亮）
      annotation/             # 轴内文字/参考线/区域及 Figure Normalized EX-027 七类注解提交
      heatmap.rs / color_bar.rs # 精确 cell 分块 mesh、共享色条
      bars.rs / error_bars.rs # 保序柱及独立误差 stem / cap
      stems.rs                # 独立杆 / 基线和原始端点 markers
      quiver.rs               # 独立向量杆、真实端点实线 V 形箭头
      area.rs / polygon.rs / patch.rs / shapes.rs # 面积、多边形/孔洞、面片与椭圆绘制
      contour.rs / pcolor.rs / image.rs # 等值线/填充等高线、伪彩色节点与像素网格绘制
      triangulation.rs / flow.rs / spy.rs / animation.rs # 三角网/Voronoi/AlphaShape、流线/羽状、CSR Spy 与动画绘制
      bubble_bivariate.rs / distribution.rs / pie_clouds.rs # 气泡群、双变量分箱、统计分布（含横纵向）与饼图/词云绘制
      multivariate.rs / confusion.rs / curvilinear.rs / stage14_stats.rs / stage14_spc_fin.rs / stage15.rs / modern/ / stage8c.rs / polar/ / graph.rs # ParallelCoords、ConfusionChart、Curvilinear、Stage 14～16 图层、极坐标与图网络绘制
      sky.rs / smith.rs / geoglobe.rs # Stage 13/15/16: 天球极坐标网格、Smith 共形网、WGS84 球面地平线裁剪绘制
      right_axis.rs           # 右 Y 刻度与标签
      plot3d/                 # Stage 9A～9C、13、16: 三背墙坐标框、SortedPainter 与 360px Gouraud ZBuffer（zbuffer_cache.rs 帧间 span 缓存）、曲面/网格/体切片/等值面/端盖/四面体/流带流管圆锥及 volshow 光线投射（volshow_raycast.rs + 分辨率自适应合成）+ pcshow 八叉树点云绘制
      geo/                    # Stage 10: 周期经度拷贝离线海岸线三角化、HostTiles 顶点染色 Mesh 与离线回退、KDE 与经纬网格绘制
      export/                 # feature=export 的显示几何快照与 PNG / 混合 SVG
        capture.rs            # Painter 范围、宿主 tessellation / atlas 快照
        raster.rs / png.rs    # 有界 CPU 预乘合成与 RGBA8 编码
        vector.rs / svg.rs    # 原生矢量与内嵌字形 / AA PNG patch、裁剪层序
        options.rs / writer.rs # 分辨率 / DPI / 局部预算和有界编码输出
    widget/
      mod.rs                  # Plot 公开配置 builder（含 imshow_axes / spy_axes / graph_axes / confusion_axes / probplot_axes / waterfall_axes / zplane_axes / gantt_axes）
      polar.rs / plot3d.rs / sky.rs / clustergram.rs / smith.rs / bode.rs / geoglobe.rs # 独立坐标系与专项控件及 into_plot_response
      show.rs                 # 每遍生命周期组装，不包含几何 / 输入算法
      response.rs             # 宿主可读取的本遍返回值
      figure/                 # Figure（含图窗级 annotations / polar_plot / plot3d / sky_plot / clustergram / smith_plot / bode_plot / geo_globe）与 cell 原生 Ui 接入，样本仍归宿主
      composite/              # StackedPlot、Pareto / PlotYY 与 ScatterHistogram / PlotMatrix 复合组件
      color_bar.rs / hover.rs # 色条与悬停的阶段组装
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
    styles.rs                 # 样式优先级、主题隔离、渲染 / 拾取一致性
    tooltip.rs                # 原始数据提示、锁外回调与多遍语义
    annotations.rs            # 注解范围、裁剪、无效值和共享生命周期
    line_sampling.rs          # 索引 / 精确路径对照、LOD 工作量和 fallback
    indexed_picking.rs        # 索引 / 原始点拾取对照及舍入边界
    line_cache.rs             # 数据 / 变换失效、预算、释放与样式复用
    stream.rs                 # 环形顺序、覆盖、快照与批次幂等
    stairs.rs / stairs_lifecycle.rs # 原始点 / 派生拐点、精确对照与缓存 / 导航
    export_stairs.rs           # 真实 SVG 路径与两后端像素
    stems*.rs / export_stems.rs # 精确杆、原始拾取 / 资源 / 多遍及导出像素
    quiver*.rs / export_quiver.rs # 向量 / 端点、域 / 裁剪、拾取 / 多遍 / DPI / 导出
    area.rs / polygons_patches.rs # 堆积面积、凹多边形/孔洞、面片与椭圆回归
    contours.rs / pcolor_image.rs # 标量场、等高线/填充、伪彩与图像回归
    export_polygons_contours.rs   # 孔洞穿透像素、Gouraud 栅格回退与纯路径导出
    functions.rs / triangulation_voronoi.rs # 自适应函数/奇点保护、三角剖分/Voronoi/AlphaShape 回归
    flow_spy.rs / animation_annotations.rs  # RK4 流线/Feather、CSR Spy、动画与 EX-027 注解回归
    export_stage8b.rs             # Stage 8B 孔洞像素与多图元/图窗注解 SVG/PNG 导出
    bars_pareto_hist.rs / bubbles_swarm_bivariate.rs # 分组堆叠柱 / Pareto / Swarm / Bubble / 双变量回归
    distributions_pie_clouds.rs / multivariate_composite.rs # Box / Violin / Raincloud / Pie / 云图与复合子图回归
    export_stage8c.rs             # Stage 8C 多变量 / 云图 / 分布混合 SVG/PNG 导出
    polar_plots.rs / polar_hist_regions.rs # 极坐标曲线/散点/气泡/罗盘/fpolarplot/histogram/rose/区域与交互回归
    graphs_networks.rs / trees_elimination.rs # graph / digraph / gplot / treeplot / etreeplot（含万节点链）回归
    export_stage8d.rs             # Stage 8D 极坐标 + 消去树混合 Figure SVG/PNG 导出
    plot3d_core_lines.rs          # Stage 9A plot3 / fplot3 / ezplot3 / comet3 / AnimatedLine3D / 透视 / 多遍导航回归
    plot3d_scatter_discrete.rs    # Stage 9A scatter3 / bubblechart3 / swarmchart3 / bar3 / bar3h / stem3 / pie3 回归
    plot3d_vectors_planes_depth.rs # Stage 9A quiver3 / fill3 / constantplane 与立方体平面裁剪回归
    export_stage9a.rs             # Stage 9A SortedPainter/ZBuffer 混合 Figure SVG/PNG 与互相交叉三维面片逐像素遮挡测试
    plot3d_surfaces.rs            # Stage 9B surf / surfc / surfl / surfnorm / ribbon / ezsurf* / ColorBar / surface primitive 回归
    plot3d_meshes_contours.rs     # Stage 9B contour3 / mesh / meshc / meshz / waterfall / fmesh / ezmesh* / histogram2 3D 回归
    plot3d_analytic_implicit.rs   # Stage 9B fsurf / sphere / cylinder / ellipsoid / fimplicit3（Kuhn + 极点守卫）/ trisurf / 3D trimesh 回归
    export_stage9b.rs             # Stage 9B 混合 Figure 宿主释放导出与 ZBuffer Gouraud Interp 逐像素渐进验证
    plot3d_volumes_slices.rs      # Stage 9C ScalarVolume3D 三线性/梯度、正交+倾斜 slice 与 contourslice 回归
    plot3d_isosurface_tetramesh.rs # Stage 9C isosurface + 次级着色、isocaps 外边界端盖与 tetramesh 共享面剔除回归
    plot3d_flow_volumes.rs        # Stage 9C 3D streamline / streamslice / 幂等 streamparticles / 涡度 streamribbon / 散度 streamtube / coneplot 回归
    export_stage9c.rs             # Stage 9C SortedPainter slice+contourslice 与 ZBuffer iso+caps+streamtube+coneplot 混合导出
    geo_projection_plot_scatter.rs # Stage 10 WebMercator/Equirectangular 逆变换、±180° 最小弧、geoplot 跨日期变更线切分/连续与 geoscatter 多遍导航
    geo_density_bubble_basemap.rs  # Stage 10 geodensityplot 核权重与半径语义、geobubble 面积/分类图例与拾取、署名校验/有界瓦片缓存与离线回退
    export_stage10.rs             # Stage 10 Pacific/HostTiles 双子图混合 SVG(roxmltree/resvg)/PNG 宿主释放导出
    stage11_readiness_audit.rs    # Stage 11 133 项/CAP/EGUI/PERF 全量一致性审计与 ScrollArea 四坐标系同图嵌入/导出回归
    stage12_foundation.rs / export_stage12.rs # Stage 12 datetime/duration、11 色谱、空心标记、框选/刷选与横向分布导出回归
    stage13_base_completion.rs / export_stage13.rs # Stage 13 confusionchart / skyplot / curvilinear / Patch3D / AlphaShape3D / Graph3D / 自适应细分回归
    stage14_stats_bio_finance.rs / export_stage14.rs # Stage 14 probplot / ecdf / dendrogram / clustergram / biplot / SPC / OHLC / Waterfall 回归
    stage15_rf_control_dsp.rs / export_stage15.rs # Stage 15 Smith / ZPlane / Bode-Nyquist-Nichols-Rlocus / FFT-STFT-Welch-CWT 回归
    stage16_expansion_audit.rs / export_stage16.rs # Stage 16 volshow / pcshow / geoglobe / 现代图型与 EXT-01..26 全审计回归
    ux_regressions.rs         # UX/交互回归：复合联动图水平平移无回弹、pan_limits 边界平滑限位、Plot3D 轨道后透视切换、等高线标签明暗对比、ZBuffer 缓存与 PlotDefaults 传递
  benches/
    frame_baseline.rs         # 只测 CPU 构建和 tessellation
    data_performance.rs       # 三档数据、索引 / LOD / 缓存 / 拾取及流式 CPU
    stream_stress.rs          # 有界宿主队列、过载丢弃及长期资源压力
    heatmap_performance.rs    # 精确全区 / 窄区网格 CPU 与容量分账
    stems_performance.rs      # 精确杆 + markers 全区 / 窄区 CPU 与局部容量
    quiver_performance.rs     # 显式向量全区 / 窄区 / 原始拾取 CPU 与局部容量
    polygon_contour_performance.rs # 面积/面片/等高线/填充等高线/pcolor/图像 CPU
    stage8b_performance.rs    # 函数曲线/三角剖分/Voronoi/AlphaShape/流切片/100k Spy CPU
    stage8c_performance.rs    # 双变量 200k / 分布 50k / 气泡群 5k / 平行坐标 / 云图 / 4x4 散点矩阵 CPU
    stage8d_performance.rs    # 极坐标 50k/100k 分箱/rosette/罗盘、Force/Layered 图网络与万节点消去树 CPU
    stage9a_performance.rs    # 三维 50k Helix / 10k 散点气泡 / 5k 蜂群 / 32x32 柱 / 2k 茎矢 / Z-buffer 面片与 fplot3+pie3 CPU
    stage9b_performance.rs    # Stage 9B Interp 128x128 / surfc+surfl / meshz / waterfall+ribbon / contour3 / fimplicit3 48^3 E2E / ZBuffer trisurf CPU
    stage9c_performance.rs    # Stage 9C 64^3 切片与轮廓切片 / 48^3 ZBuffer 等值面端盖 / 2k 四面体 / 流粒子/流带流管/千锥 CPU
    stage10_performance.rs    # Stage 10 50k 跨日期变更线航线 / 20k 散点 / 10k 64x48 密度 / 2k 气泡 / HostTiles 缓存与太平洋四层组合 CPU
    dynamic_10m_performance.rs # Stage 11+ 动态千万点 IndexedStream / UniformWaveform / PlotData::new / 百万 Scatter Auto LOD CPU 基准
    stage16_performance.rs    # Stage 16 64^3 DirectAlpha volshow / 200k octree pcshow / geoglobe / 5 类现代图 CPU 基准
    ../tests/high_perf_sources.rs # L1 自动索引/Auto LOD/分支定界拾取、L3 流/波形与无头/导出回退集成回归
  examples/export_snapshot.rs # 独立 headless 宿主的 PNG / SVG 文件案例
examples/demo/src/
  main.rs                     # eframe 启动（默认 wgpu 渲染器 + register_wgpu，语言检测 --lang/--zh/--en/--ja 与 --glow / --bench-window-10m）
  fonts.rs                    # 按 Context 幂等加载系统 CJK 回退字体（不额外引入打包字体资产或外部依赖）
  i18n.rs                     # 多语言（简体中文 / English / 日本語）上下文同步、矢量外观切换栏与统一文案辅助
  gallery.rs                  # ActiveGallery 27 项分类画廊互斥切换器（支持 6 类下拉分组与前后矢量箭头，含 Custom* 客制化部件专题）
  global_controls.rs          # 独立可折叠右侧绘图控制面板（GlobalPlotControls + ExportPanel，驱动 Context 级 PlotDefaults）
  app.rs                      # 应用生命周期、语言/主题切换、右侧全局控制栏与 27 项 ActiveGallery 画廊路由
  dynamic10m.rs               # --dynamic-10m 千万点实时 120Hz+ 交互/GPU/CPU 展示与真窗口 swapchain 基准驱动
  charts.rs                   # 折线描述、Figure / 联动 / 双轴组装
  scalar.rs                   # 数值网格与显式共享色条
  statistical.rs              # 显式直方预处理、正负柱与误差棒
  stems.rs                    # 独立正负杆 / 基线、原始端点与右 log Y gallery
  quiver.rs                   # 显式旋转矢量场及右 log Y gallery
  polygons.rs                 # 堆积面积、凹多边形 / 带孔 polyshape / Gouraud 面片 / 圆角椭圆
  contours.rs                 # 等高线 / contourf / pcolor / imshow_axes 图像 gallery
  functions.rs                # fplot / fimplicit / ezplot、animatedline / comet 与 EX-027 注解 gallery
  geometry.rs                 # Delaunay / Voronoi / alphaShape、streamslice / feather 与 CSR spy gallery
  distributions.rs            # 分组堆叠柱、Pareto、Violin / Raincloud、Swarm / Bubble 与双变量 gallery
  multivariate.rs             # 词云 / 气泡云、平行坐标与 Stacked / ScatterHistogram / PlotMatrix gallery
  polar.rs                    # 极坐标曲线/fpolar/ezpolar、散点/气泡、polarhistogram/rose、罗盘与极坐标区域 gallery
  networks.rs                 # 无向 Force 图、有向 Layered DAG、gplot 网格、treeplot 与 etreeplot gallery
  plot3d.rs                   # Stage 9A 三维线/参数曲线/comet3/散点气泡/蜂群茎/分组水平柱/矢量交叉面/参考面/3D饼图 gallery
  surfaces3d.rs               # Stage 9B surf/surfc/surfl/surfnorm/mesh*/waterfall/contour3/ribbon/fsurf/fimplicit3/sphere/cylinder/ellipsoid/trisurf gallery
  volumes3d.rs                # Stage 9C slice/contourslice/isosurface/isocaps/tetramesh/streamline/streamslice/streamparticles/streamribbon/streamtube/coneplot gallery
  geo.rs                      # Stage 10 geoplot/geoscatter/geodensityplot/geobubble 与投影/离线海岸线/合成 HostTiles gallery
  stage13.rs / stage14.rs / stage15.rs / stage16.rs # Stage 13～16 扩展专题展厅（--stage13 / --stage14 / --stage15 / --stage16）
  custom_axes.rs / custom_annotations.rs / custom_animation.rs / custom_styling.rs # 客制化部件专题展厅（--custom-axes / --custom-annotations / --custom-animation / --custom-styling：自定义刻度/时间/双轴与 pan_limits、点击固定 DataTip 游标/区间标注/刷选统计、动画时间轴倍速追踪、五层样式继承与色条布局）
  export.rs                   # 导出控件、跨遍候选替换、下一帧防覆盖写盘
  controls.rs                 # 可调参数和控制面板
  data.rs                     # 固定、可复现的示例数据
  diagnostics.rs              # 诊断展示，不实现测量算法
  appearance.rs               # 保留主题 / 注解、属性面板与配置应用
  performance.rs              # 显式预处理 / 预算选项、宿主缓存及状态
  stream.rs                   # Live 信号、按节奏批量发布及多遍去重
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
7. 样式在进入绘制前按 Theme → 父值层 → Plot → Series 解析；绘制、图例和拾取共享解析规则。显式透明 / 零 / false 不得误当缺省。主题不写宿主 Style；格式化回调在锁外逐遍运行，不发布副作用事件。
8. `PlotCache` 与 `PlotStream` 归宿主显式所有，不存入 `PlotState` 或 egui Memory。缓存只保留屏幕坐标，不持有样本；数据版本而非可重用裸指针区分快照。流式批次 ID 由宿主提供，重复布局不能重复追加。
9. `PlotLayer` 不改变旧 Series / data 契约。数值网格、柱、误差棒在 `data` 构造时缓存范围；`summary` 只合并摘要，不能每遍为自动范围偷偷扫描原始记录。绘制和拾取各自的工作量继续计数。
10. `Figure` 不保存 cell 样本 / 状态；`AxisLink` 在全部成员每遍显示后同步，`secondary` 重放同一导航动作。填充层与点层的拾取仲裁属于 interaction，不回流到 widget。
11. 导出是显式 displayed-geometry 快照，不自动恢复原始数据或另建 Context；编码后端属于 render/export。库返回 bytes，宿主在最终布局遍之外处理文件副作用；文字的 atlas 捕获不消费上传增量。
12. 阶梯插值在选点后、裁剪前展开，先验证完整原始点对；不改写样本 / 范围 / 拾取。插值进入 pre-style 缓存键；MinMax 阶梯不能宣称精确保持跳变时刻。
13. Stems 复用 PlotData 范围摘要 / 样本资源去重与可选原始端点拾取索引；动态基线的域验证属于 model，几何属于 render。没有隐式转成折线、重排原始点、杆 LOD 或杆缓存。
14. QuiverData 在构造时验证原点 / 位移 / 真实端点并缓存联合域范围；每遍汇总只合并摘要。轴域提供向量投影及显式精度拒绝，render 在真实端点构造逻辑点箭头；裁剪交点、箭头翼和杆内部不是原始记录锚点。HoverInfo 保持原契约，PlotResponse 额外返回原始向量记录；不增加隐藏 autoscale、LOD、索引或缓存。

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
