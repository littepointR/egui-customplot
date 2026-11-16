# 阶段 9：三维与体数据（Stage 9A + 9B 进行中）

日期：2026-11-16。**阶段 9A（三维核心坐标系、双深度引擎及点 / 线 / 柱 / 茎 / 矢量 / 饼 / 参考面等 14 项三维图型）与阶段 9B（三维曲面、网格、解析几何、隐式等值面与三维等高线等 22 项及 2 项三维分支）已实现并通过自动化测试、Gouraud `Interp` Z-buffer 逐像素插值验证、16 预设原生窗口烟雾与 30 秒 CPU 基准验证；阶段 9C（三维体切片与流可视，9 项）、阶段 10（地理图，4 项）与全量交付门槛仍未完成。**

## Stage 9A：三维核心坐标系、双深度模式与 14 项三维图型

- **Checklist 覆盖（14 项转为 `开发中`，并扩展 3 项既有能力的三维分支）：**
  - 新增 `10` 项 `ML`：`ML-002`（`plot3`）、`ML-011`（`fplot3`）、`ML-014`（`scatter3`）、`ML-016`（`bubblechart3`）、`ML-018`（`swarmchart3`）、`ML-036`（`bar3`）、`ML-037`（`bar3h`）、`ML-040`（`stem3`）、`ML-057`（`quiver3`）、`ML-080`（`comet3`）。
  - 新增 `2` 项 `EX`：`EX-003`（`fill3`）、`EX-032`（`constantplane`）；新增 `2` 项 `LG`：`LG-002`（`pie3`）、`LG-009`（`ezplot3`）。
  - 扩展 3 项已有条目至三维：`ML-078`（`AnimatedLine3D`）、`EX-001`（`Line3D::primitive`）、`EX-026`（`Text3D`）。
  - 133 项对照状态由 `84 开发中` 推进至 **98 开发中（60 ML + 26 EX + 12 LG）/ 35 未实现**，尚无条目通过完整交付门槛。

- **核心模块与渲染 / 导出边界：**
  - `axis/plot3d/`（`Bounds3D`、`DataAspect3D`、`Axis3D`、`Camera3D` 默认 MATLAB `view(-37.5, 30)` 与 `view_2d_xy` `(0, 90)`、`Projection3D::{Orthographic, Perspective}`、`Transform3D`、`[0,1]^3` Liang-Barsky 线段裁剪 / Sutherland-Hodgman 平面三角裁剪 / `intersect_plane_unit_cube`）、`layout/plot3d.rs`。
  - `data/plot3d/`：宿主不可变 `Arc` payload 与 `(Linear | Log10)^3` 八域缓存 `CachedExtents3D`（含 `domain_invalid` 计数）—— `Point3DData`、`ParametricCurve3DData`、`AnimatedLine3D` / `Comet3DPlayback`、`Bubble3DData`、`Swarm3DData`、`Bar3DData`（`Detached` / `Grouped` / `Stacked`、`Vertical` / `Horizontal`）、`Stem3DData`、`Pie3DData`（挤压圆柱扇区与分离）、`Quiver3DData`（含显式 `.autoscale()`）、`Polygon3DData`（Newell 法线投影 + 耳切剖分）。
  - `model/plot3d/` & `interaction/plot3d/` & `widget/plot3d.rs`：`Layer3D`、`DepthMode::{SortedPainter, ZBuffer}`、`BoxStyle3D`、`ConstantPlane3D`、`Text3D`、`Plot3DState`（多遍幂等左键轨道旋转、右键平移、滚轮缩放、Home/双击复位与 Legend 单次开关）、深度优先拾取 `Hovered3DPoint` / `Hovered3DBar` / `Hovered3DRegion`（`Pie3D` / `Fill3D` 命中、`Line3D::primitive` 跳过图例与拾取），以及 `FigureCell::{plot3d, show_plot3d}`；`diagnostics.rs` 新增 `Plot3DDiagnostics`。
  - `render/plot3d/`：`axes.rs` 绘制随视角翻转的三背墙网格与外轮廓刻度标签；`scene.rs` 执行 Lambertian 面片着色与精确排序 `SortedPainter`；`zbuffer.rs` 提供最长边限制为 `360px` 的有界软件深度缓冲，采用水平 RLE 批处理生成 `TextureId::default()` + `WHITE_UV` 的 `Mesh::add_colored_rect` 矩形批次，与 `PlotSnapshot::capture` 完全兼容。
  - **混合 SVG 导出声明：** 在 `DepthMode::ZBuffer` 及 Lambertian 分色三维三角网格模式下，`PlotSnapshot::to_svg` 复用 `render/export/svg.rs` 将变色 / Z-buffer 网格嵌入为 base64 PNG `<image>` 栅格图层（无 `<text>` 外部字体依赖），**不宣称三维渲染为纯矢量 `<path>`**。

### Stage 9A 正确性与生命周期证据

- 新增库单元 2 项（`camera` 正交 / 透视与 `clip` 对角平面六边形交线）、集成 13 项：
  - `tests/plot3d_core_lines.rs`（4 项：`plot3` 校验 / NaN 断段 / Log10 轴域计数与不跨越断段 / 反向轴 / `Line3D::primitive` 跳过图例与拾取 / `Text3D`；`fplot3` & `ezplot3` 自适应细分与奇点预算；`comet3` & `AnimatedLine3D` 多遍幂等；透视放大、轨道/平移/缩放、图例显隐与 Home 复位多遍一致）。
  - `tests/plot3d_scatter_discrete.rs`（4 项：`scatter3` 同视线前景点深度优先拾取；`bubblechart3` + `BubbleLegend` 与 `swarmchart3` 保留 `raw_index`；`bar3` / `bar3h` 三模式两方向与 `Hovered3DBar`；`stem3`；`pie3` 挤压圆柱扇区与 `Hovered3DRegion` 扇区拾取）。
  - `tests/plot3d_vectors_planes_depth.rs`（3 项：`quiver3` 原生与显式 autoscale + 四面体棱锥头与端点拾取；`fill3` 退化拒绝与凹多边形耳切 + 区域拾取；`constantplane` 轴向与对角 `x+y+z=0` 切分立方体为六边形）。
  - `tests/export_stage9a.rs`（2 项：`SortedPainter` + `ZBuffer` 双子图 `Figure` 释放宿主数据后 SVG 无 `<text` / PNG 导出验证；以及互相交叉斜平面 `z = x`（红）与 `z = -x`（蓝）在 `Camera3D::view_2d_xy()` 下通过 `DepthMode::ZBuffer` 输出 PNG，实测左侧 `x = -0.45` 蓝通道主导、右侧 `x = +0.45` 红通道主导的逐像素深度遮挡证据）。
  - 以及 demo 15 预设（`--plot3d` 6 单元 `Figure` 及 `plot3d_tests`）。
- 统一 `cargo test --workspace --all-features --locked` 合计 **287 项通过**（70 测试套件：库单元 41、集成 223、demo 21、rustdoc 2，0 失败 / 0 忽略），全部 7 项构建 / 测试 / 严格 Clippy 见 [Stage 9A 最终检查记录](benchmarks/2026-11-16-stage9a-windows-final-checks.txt)。
- Release `customplot-demo.exe`（SHA256 `22F809A789E23FD0C71D9179C53732C8EA52DE44D3E561E8CC05AA5D34A7E5A8`）经 [十五预设 Windows 原生生命周期记录](benchmarks/2026-11-16-stage9a-windows-gallery-smoke.txt) 验证，含 `--plot3d` 在内的全部 15 个预设均达到 `input_idle=True`、`responsive=true`、`close_requested=True` 且退出码 `0`。不代表跨平台显示实机、硬件 GPU 光栅化或长时运行内存回归。

### Stage 9A CPU 基准测量

`stage9a_performance`（binary SHA256 `52CBE3AB3F7257E5061963EF087F4E6E937DAF0E48365D9D043D0916EC37C63F`，基于 `2174e53` + Stage 9A 实现）在共享 Windows 10.0.26300 / `i9-14900KF` / Rust 1.100.0 / `egui 0.37.0` / canvas `2024×1124`（单图 `height=1080` / 双图 `2×1080`、DPI=1、`show_legend=false`、`show_hover=false`）下对 7 项预处理测 p50（5 次）、7 场景各预热 5 帧并采集 30 秒，退出码 0、stderr 为空（[原始 CSV](benchmarks/2026-11-16-stage9a-windows-cpu.csv)、[环境记录](benchmarks/2026-11-16-stage9a-windows-cpu-environment.txt)、[空 stderr](benchmarks/2026-11-16-stage9a-windows-cpu.stderr.txt)）。

- **预处理中位耗时（p50 ms）：** `plot3_helix_50k=0.6149`、`scatter3_bubble3_10k=0.3703`、`swarm3_5k=0.2625`、`bar3_grouped_32x32=0.0242`、`stem3_quiver3_2k=0.0494`、`fill3_planes_zbuffer=0.0293`、`fplot3_pie3_combo=0.0144`（`fcurve_pts=513`）。

| 场景 | items | frames | UI p95 (ms) | Tess p95 (ms) | 合计 p50 / p95 / p99 (ms) | 输出图元 / meshes | 样本字节 | 访问项 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `plot3_helix_50k` | 50,000 | 2,577 | 5.7293 | 6.4822 | 11.2849 / 12.0752 / 12.6211 | 49,999 / 2 | 1,200,000 | 50,000 |
| `scatter3_bubble3_10k` | 10,000 | 4,391 | 0.9682 | 6.1710 | 6.4479 / 7.0088 / 7.2916 | 10,000 / 2 | 400,000 | 10,000 |
| `swarm3_5k` | 5,000 | 70,543 | 0.4106 | 0.1832 | 0.3934 / 0.6008 / 0.6642 | 5,000 / 2 | 280,000 | 5,000 |
| `bar3_grouped_32x32` | 1,024 | 4,119 | 5.8654 | 2.0515 | 7.1303 / 7.8101 / 8.0981 | 24,576 / 2 | 73,728 | 1,024 |
| `stem3_quiver3_2k` | 2,000 | 12,332 | 2.0489 | 0.7266 | 2.2730 / 2.7063 / 2.8951 | 8,222 / 2 | 72,000 | 2,000 |
| `fill3_planes_zbuffer` | 67 | 81,173 | 0.3970 | 0.0175 | 0.3618 / 0.4165 / 0.5438 | 406 / 2 | 9,216 | 64 |
| `fplot3_pie3_combo` | 519 | 210,495 | 0.1146 | 0.0379 | 0.1400 / 0.1536 / 0.2041 | 743 / 4 | 12,776 | 519 |

- **严格口径与已知瓶颈：** 以上仅测 headless CPU UI 构建与 `egui` tessellation，不包含 wgpu/glow GPU 上传、GPU 硬件光栅化、swapchain 呈现、实际帧率（FPS）或整应用进程峰值内存；`ZBuffer` 分辨率受限于 `360px` 有界软件栅格化边长与 `WHITE_UV` RLE 矩形批处理，放大分辨率后软光栅成本按像素面积上升，不能等效于专用 GPU 深度缓冲。

## Stage 9B：三维曲面、网格、解析几何、隐式等值面与三维等高线（22 项 + 2 项三维分支）

- **Checklist 覆盖（22 项转为 `开发中`，并扩展 2 项既有能力的三维分支）：**
  - 新增 `12` 项 `ML`：`ML-053`（`contour3`）、`ML-059`（`surf`）、`ML-060`（`surfc`）、`ML-061`（`surfl`）、`ML-062`（`ribbon`）、`ML-064`（`fsurf`）、`ML-065`（`fimplicit3`）、`ML-066`（`mesh`）、`ML-067`（`meshc`）、`ML-068`（`meshz`）、`ML-069`（`waterfall`）、`ML-070`（`fmesh`）。
  - 新增 `6` 项 `EX`：`EX-006`（`surface`）、`EX-007`（`surfnorm`）、`EX-008`（`sphere`）、`EX-009`（`cylinder`）、`EX-010`（`ellipsoid`）、`EX-013`（`trisurf`）；新增 `4` 项 `LG`：`LG-013`（`ezmesh`）、`LG-014`（`ezmeshc`）、`LG-015`（`ezsurf`）、`LG-016`（`ezsurfc`）。
  - 扩展 2 项已有条目至三维：`ML-022`（`Bar3DData::from_histogram2` 支持三维分箱直方棱柱）、`EX-012`（`TriSurf3D::wireframe` 支持非规则 3D 三角网格线框及可选隐藏线遮挡）。
  - 133 项对照状态由 `98 开发中` 推进至 **120 开发中（72 ML + 32 EX + 16 LG）/ 13 未实现**，尚无条目通过完整交付门槛。

- **核心模块与渲染 / 混合导出：**
  - `data/plot3d/`：`Surface3DData`（规则网格、标量场、参数曲面、有限差分法线、共享等高线与 `normals_quiver`）、`Contour3DData`（真实 `z = level` 三维等值线抽取与拼接）、`Ribbon3DData`（多序列带状条带）、`analytic_surf.rs`（`from_explicit_fn` / `from_parametric_fn` / `ez_*` 极点悬崖保护与求值预算、`sphere` / `cylinder` / `ellipsoid`）、`TriMesh3DData`（Indexed 顶点/边去重/Delaunay XY/光滑法线）、`ImplicitSurface3DData`（6 四面体 Kuhn Marching Tetrahedra + 中心差分梯度法线 + 有理极点断链保护）、`Bar3DData::from_histogram2`。
  - `model/plot3d/`、`axis/plot3d/`、`render/plot3d/`、`interaction/plot3d/`、`layout/plot3d.rs`、`widget/plot3d.rs`：`SurfaceShading::{Faceted, Flat, Interp}`（`ClipVertex3D` `[u, rgba]` Gouraud 裁剪 + `ZBuffer::raster_tri` 双线性顶点插值）、`LightSource3D` / `colormap_lighting`（`surfl`）、`MeshSurface3D`（`Grid` / `CurtainZ` / `Waterfall` + 背景色 `depth_bias = +1.5e-4` 隐藏线遮挡）、`Contour3D`、`Ribbon3D`、`TriSurf3D`（面色/线框/`from_implicit`）、`Plot3D::color_bar` 与 `surface_hover.rs`（点 / 三角面片拾取，`Surface3D::primitive` 与 `Line3D::primitive` 同样跳过图例与拾取）。
  - 混合 SVG 导出仍适用与 Stage 9A 一致的边界：`SortedPainter` / `ZBuffer` 彩色表面与栅格网格以 base64 PNG `<image>` 嵌入 SVG（无 `<text>`），不宣称为纯矢量 `<path>`。

### Stage 9B 正确性与生命周期证据

- 新增 10 项 Stage 9B 集成测试与 1 项 demo 测试（均在 `--all-features` 与默认特性下验证）：
  - `tests/plot3d_surfaces.rs`（3 项：`surf` 三种着色模式、NaN 空洞、`Plot3D::color_bar` 留白与点/面拾取、`Surface3D::primitive` 跳过图例与拾取；`surfc` / `ezsurf` / `ezsurfc` 预算拦截与底面等高线；`surfl` 随光源方位改变颜色；`ribbon` 条带拾取与 `surfnorm` 单位球外法线方向检验）。
  - `tests/plot3d_meshes_contours.rs`（2 项：`contour3` 从 `ScalarField2D` 与 `Surface3DData` 在精确 `z = level` 抽取并拾取；`mesh` 隐藏线开关、`meshc` 底面等高线、`meshz` 垂帘、`waterfall` 拓扑分段对比、`fmesh`/`ezmesh*` 与 `histogram2` 3D 棱柱直方渲染）。
  - `tests/plot3d_analytic_implicit.rs`（3 项：`fsurf` 参数环面与 `sphere`/`cylinder`/`ellipsoid` 几何及外法线精度；`fimplicit3` 单位球零等值面误差 `< 0.08` 且梯度法线外向点积 `> 0.90`、空水平集安全跳过与有理极点 `discontinuities() > 0` 保护；`trisurf` 区域拾取与 3D `trimesh` 隐藏线控制）。
  - `tests/export_stage9b.rs`（2 项：`SortedPainter` `surfc`+`color_bar`+`contour3` 与 `ZBuffer` `fimplicit3`+`sphere`+`cylinder` 双子图 `Figure` 释放宿主数据后 SVG 无 `<text` / PNG 导出；以及扁平二维 Gouraud `SurfaceShading::Interp` 网格在 `DepthMode::ZBuffer` 下左→中→右红蓝通道平滑插值逐像素证据）。
  - `examples/demo/src/surfaces3d.rs`（`--surfaces3d` 6 单元 `Figure` 及 `surfaces3d_tests`）。
- 统一 `cargo test --workspace --all-features --locked` 合计 **298 项通过**（74 测试套件：库单元 41、集成 233、demo 22、rustdoc 2，0 失败 / 0 忽略），全部 7 项工作区最终检查见 [Stage 9B 最终检查记录](benchmarks/2026-11-16-stage9b-windows-final-checks.txt)。
- Release `customplot-demo.exe`（SHA256 `D40227A84EE190DE99ED23518E13D4FD42AD6F65C307E68566CF3224A6F7B072`）经 [十六预设 Windows 原生生命周期记录](benchmarks/2026-11-16-stage9b-windows-gallery-smoke.txt) 验证，含 `--surfaces3d` 在内的全部 16 个预设均达到 `input_idle=True`、`responsive=true`、`close_requested=True` 且退出码 `0`。不代表跨平台显示实机、硬件 GPU 光栅化或长时运行内存回归。

### Stage 9B CPU 基准测量

`stage9b_performance`（binary SHA256 `0BDE7DE1E62033B871217A0E356A9C8DABED276D21D0F577EEED89E21DC5BC9F`，基于 `ec2375f` + Stage 9B 实现）在共享 Windows 10.0.26300 / `i9-14900KF` / Rust 1.100.0 / `egui 0.37.0` / canvas `2024×1124`（`Plot3D` `height=1080`、DPI=1、`show_legend=false`、`show_hover=false`）下对 7 项预处理测 p50（5 次）、7 场景各预热 5 帧并采集 30 秒，退出码 0、stderr 为空；其中 `fimplicit3_48x48x48` 在每帧计时循环内直接端到端执行 `48³ = 110,592` 体素求值、等值面提取与 `ZBuffer` 提交（[原始 CSV](benchmarks/2026-11-16-stage9b-windows-cpu.csv)、[环境记录](benchmarks/2026-11-16-stage9b-windows-environment.txt)、[空 stderr](benchmarks/2026-11-16-stage9b-windows-cpu.stderr.txt)）。

- **预处理中位耗时（p50 ms）：** `surf_interp_128x128=0.5011`、`surfc_surfl_64x64=0.7527`、`meshz_hidden_100x100=0.7089`、`waterfall_ribbon_64x32=0.1842`、`contour3_128x128_16levels=1.9957`、`fimplicit3_48x48x48_extract=21.4677`、`trisurf_analytic_zbuffer=5.2044`。

| 场景 | items | frames | UI p95 (ms) | Tess p95 (ms) | 合计 p50 / p95 / p99 (ms) | 输出图元 / meshes | 样本字节 | 访问项 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `surf_interp_128x128` | 16,384 | 2,059 | 14.3518 | 0.7737 | 14.4275 / 15.0735 / 15.4613 | 32,258 / 3 | 917,504 | 16,129 |
| `surfc_surfl_64x64` | 4,096 | 4,976 | 5.8898 | 0.6835 | 5.9053 / 6.5082 / 6.6795 | 14,863 / 2 | 272,616 | 3,969 |
| `meshz_hidden_100x100` | 10,000 | 2,221 | 10.2354 | 3.9640 | 13.2816 / 14.0730 / 14.5055 | 40,986 / 2 | 560,000 | 9,801 |
| `waterfall_ribbon_64x32` | 2,560 | 14,810 | 1.9416 | 0.2528 | 2.0047 / 2.1724 / 2.4493 | 7,120 / 2 | 172,032 | 536 |
| `contour3_128x128_16levels` | 10,220 | 16,641 | 0.8198 | 1.0619 | 1.6808 / 1.8653 / 2.0999 | 10,193 / 2 | 245,408 | 27 |
| `fimplicit3_48x48x48` (E2E) | 110,592 | 677 | 45.1441 | 0.0795 | 44.4138 / 45.2393 / 45.6800 | 43,560 / 2 | 2,304,328 | 43,560 |
| `trisurf_analytic_zbuffer` | 3,222 | 14,233 | 2.2310 | 0.0150 | 2.1095 / 2.2474 / 2.3536 | 4,320 / 2 | 177,032 | 3,096 |

- **严格口径与已知瓶颈：** 以上仅测 headless CPU UI 构建与 `egui` tessellation，不含 GPU 上传 / 硬件光栅化 / swapchain 呈现 / 真实 FPS；高精度隐式网格若每帧重采样 `48³` 体素 + 软光栅达 ~45 ms，动态视角交互依赖宿主跨帧缓存已提取的 `ImplicitSurface3DData`（静态网格渲染则仅耗时级提取一次），`meshz` 与高密 `Surface3D` 无隐式网格 LOD。

## 后续范围

- **Stage 9C（9 项）：** 三维体切片与流可视化（`contourslice`、`streamparticles`/`streamribbon`/`streamtube`/`coneplot`/`slice`、`tetramesh`、`isosurface`/`isocaps`）。
- **Stage 10（4 项）与 Stage 11：** 地理绘图与全量 133 项逐项核销验收。
