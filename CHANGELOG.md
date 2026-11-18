# Changelog

All notable milestones for `egui-customplot` are documented in this file.
Detailed per-stage verification logs and benchmark records live under `docs/STAGE_*.md` and `docs/PROGRESS.md`.

## [Unreleased] (`0.1.0-dev`)

### Added

- **Core 2D Architecture & Interaction (Stages 1–7)**:
  - Host-owned `Arc` `PlotData`, linear/log10/reversed coordinate axes, `data_aspect` equal-unit locking, nice ticks, custom formatters, interactive legends, raw-point hover picking, `PlotTheme` (`scientific_light`, `monitoring_dark`) with 5-layer style inheritance, `Annotation` primitives, `Figure` multi-cell layouts, `AxisLink` synchronization, dual Y axes, `GridData`/`Heatmap`/`ColorBar`, `Bars`/`ErrorBars`/`HistogramData`, and optional `feature = "export"` `PlotSnapshot` PNG/hybrid-SVG export.
- **Complete 2D MATLAB Parity (Stage 8A–8D)**:
  - Step plots (`LineInterpolation::{StepAfter, StepBefore}`), `Stems`, `Quiver`, stacked/split `Area`, concave/holed `Fill` & `PolyShape`, Flat/Gouraud `Patch`, `Rectangles`, `Contours`/`FilledContours`, `Pcolor`, `PlotImage` (`Plot::imshow_axes`).
  - Adaptive `FunctionPlot` (`fplot`/`ezplot`), `ImplicitCurve` (`fimplicit`), Delaunay `TriPlot`, `Voronoi`, `AlphaShape`, `Streamlines` (RK4), `Feather`, 1-based CSR `Spy`, `AnimatedLine2D`/`CometPlayback`, and `EX-027` normalized figure annotations.
  - Grouped/stacked bars, `ParetoChart`, `SwarmChart`, `BubbleChart` + `BubbleLegend`, `Histogram2`, `BinScatter` (rect/hex), `BoxChart`, `ViolinPlot`, `Raincloud`, `PieChart`/`donut`, `WordCloud`/`BubbleCloud`, `CategoricalHeatmap`, `ParallelPlot`, `StackedPlot`, `ScatterHistogram`, and `PlotMatrix`.
  - `PolarPlot` (`PolarLine`, `PolarScatter`, `PolarBubble`, `PolarHistogram`, `RoseData`, `CompassPlot`, `PolarRegionAnnotation`), `GraphPlot` (Force/Sugiyama Layered, multigraph, self-loops), `TreeData`, and Liu `O(nnz)` `EliminationTree`.
- **3D & Volumetric Visualization (Stage 9A–9C)**:
  - `Plot3D` with `Camera3D`, `Orthographic` / `Perspective` projections, `DepthMode::{SortedPainter, ZBuffer}`, `plot3`, `fplot3`, `comet3`, `scatter3`, `bubblechart3`, `swarmchart3`, `bar3`/`bar3h`, `stem3`, `quiver3`, `fill3`, `constantplane`, `pie3`.
  - `Surface3D`, `MeshSurface3D` (hidden-line occlusion, curtain, waterfall), `Contour3D`, `Ribbon3D`, `TriSurf3D`, `ImplicitSurface3DData` (Marching Tetrahedra + pole guards), `LightSource3D`, and `sphere`/`cylinder`/`ellipsoid`.
  - `ScalarVolume3D`, orthogonal/oblique `VolumeSlice3D`, `Isosurface3D`, `Isocaps3D`, `TetraMesh3D`, and 3D `Streamline3D` / `StreamParticles3D` / `StreamRibbon3D` / `StreamTube3D` / `ConePlot3D`.
- **Geographic Visualization & Readiness Audit (Stages 10–11)**:
  - Offline `GeoPlot` (`WebMercator` / `Equirectangular`, `S^1` minimum-arc bounds, antimeridian splitting, `GeoLine`, `GeoScatter`, `GeoDensity`, `GeoBubbles`, `OFFLINE_LAND_RINGS`, `GeoAttribution`, bounded LRU `GeoTileCache`, and `GeoOfflinePolicy`).
  - Complete 133-item checklist traceability audit and cross-coordinate `ScrollArea` embedding tests.
- **10M-Point 120Hz+ Engine & Optional `wgpu` Backend**:
  - `L1` auto monotonic-X indexing (`>= 8,192` points, 64 MiB cap), `LineSampling::Auto`, `MarkerSampling::Auto` with hierarchical `markers_grid_tree` deep-zoom pruning, embedded 2 MiB `PlotState` auto cache, and 2D screen-AABB branch-and-bound raw-point picking.
  - `L3` zero-copy `IndexedStream` (`StreamLine`), `O(1)`-sliced `UniformWaveform` (`WaveformLine`), and optional `feature = "wgpu"` (`register_wgpu`) compute + instanced scatter/waveform pipelines with seamless CPU `Painter` fallback.
- **Extended Engineering, Domain Toolboxes & Modern Charts (Stages 12–16, `EXT-01..26`)**:
  - `EXT-01..05`: `Axis::datetime`/`duration` (`CivilDateTime`), 11 `ColorMap` palettes, 15 solid/hollow `MarkerShape` variants (CPU + WGSL SDF), multi-pass idempotent box zoom & data brushing (`PlotState::brushed_bounds`, `PlotData::brushed_indices`), and horizontal `BoxChart`/`ViolinPlot`/`Raincloud`.
  - `EXT-06..12`: `ConfusionChart`, `SkyPlot`, `CurvilinearPcolor`/`CurvilinearField2D`, `Patch3D`, `AlphaShape3D`, `Graph3D`, and watertight adaptive `fcontour`/`fsurf` subdivision.
  - `EXT-13..18`: `AxisScale::{Probit, Weibull}` / `ProbabilityPlot`, Kaplan-Meier `EcdfPlot`, `DendrogramPlot` & `Clustergram`, PCA/Varimax `Biplot`, `ControlChart` (SPC + Nelson rules), `Candlestick`, `OhlcBars`, and `Waterfall2D`.
  - `EXT-19..22`: `SmithPlot`, `ZPlanePlot` (Aberth-Ehrlich root finding), `BodePlot`/`NyquistPlot`/`NicholsPlot`/`RootLocus`, and pure-Rust Radix-2 + Bluestein FFT / Welch / STFT / Morlet CWT (`CwtPlot`).
  - `EXT-23..26`: CPU `VolumeRender3D` raycaster (`volshow`) with `ZBuffer` occlusion & span caching, 200k+ octree-LOD `PointCloud3D` (`pcshow`), WGS84 `GeoGlobePlot`, plus `RadarChart`, `SankeyChart`, `TreemapChart`, `SunburstChart`, and `GanttChart`.
  - Context-wide `PlotDefaults` (`set_plot_defaults`) and `Plot::pan_limits`.
- **Interactive Demo, Localization & Documentation**:
  - `customplot-demo` expanded to 27 mutually exclusive galleries (including `--dynamic-10m` and 4 `--custom-*` showcases), global controls panel, wall-clock decoupled animations, vector theme selector, Windows OS theme sync, and `简体中文 / English / 日本語` runtime UI switching.
  - Bilingual (`EN | 简体中文`) `rustdoc` crate overview, user guide, and API documentation with interactive header/sidebar language toggle (`crates/egui-customplot/docs/rustdoc-i18n-header.html`).
  - Open-source community files (`README.md`, `README.zh-CN.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `SECURITY.md`, GitHub issue/PR templates, CI `cargo doc` checks), `MIT OR Apache-2.0` dual licensing (`publish = false` retained pending crates.io release readiness).
