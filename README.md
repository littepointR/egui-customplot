English | [简体中文](README.zh-CN.md)

# egui-customplot

A native, embeddable scientific and engineering plotting library for [`egui`](https://github.com/emilk/egui), inspired by MATLAB's breadth of plot types while embracing Rust ownership, host-owned immutable data, explicit view state, and deterministic multi-pass UI behavior.

> **Project Status (`0.1.0-dev`, pre-crates.io)**
>
> - **Implemented & tested**: Stages 1–16 — 133/133 items in the [MATLAB Plot Checklist](docs/MATLAB_PLOT_CHECKLIST.md) (`82 ML + 35 EX + 16 LG`) plus 26 extended engineering/modern capabilities (`EXT-01..26`) in [EXPANSION_PLAN](docs/EXPANSION_PLAN.md), the three-tier 10M-point `L1/L2/L3` performance engine, optional `feature = "wgpu"` hardware compute/instanced acceleration, optional `feature = "export"` PNG/hybrid-SVG snapshots, bilingual (`EN | 简体中文`) `rustdoc`, and a 27-gallery tri-lingual (`EN / 简体中文 / 日本語`) interactive demo.
> - **Open boundaries (honest release status)**: Physical Linux/macOS/WebAssembly display verification, hardware GPU acceleration for 3D surfaces/volume raycasting/large distributions, standalone print-layout vector typography, and full long-duration app-wide resource regression remain open boundaries. As a result, all 133 checklist items remain marked **"开发中" (in progress, 0 delivered)** and workspace crates remain `publish = false` until crates.io release criteria are finalized.
> - **Safety & isolation**: `#![forbid(unsafe_code)]` workspace-wide (`[workspace.lints.rust] unsafe_code = "forbid"`), zero window/event-loop/device hijacking, zero implicit background threads or network requests in the core crate (including offline maps/globe).

---

## Key Capabilities

- **Core Architecture**
  - **Host-owned data (`Arc`)**: Immutable/versioned containers (`PlotData`, `GridData`, `ContourData`, `Surface3DData`, `ScalarVolume3D`, `PointCloud3DData`, etc.) stay with the host application; `PlotState` / `Plot3DState` never retain sample buffers or GPU textures.
  - **egui multi-pass idempotency**: Safe under `egui::Context::request_discard` retries — navigation, box-zoom/brushing, `AxisLink` synchronization, `PlotStream` / `FrameClock` updates, and deferred demo exports apply once per frame.
  - **Seven-domain modular layout**: Strictly organized across `axis`, `data`, `interaction`, `layout`, `model`, `render`, `widget` (+ `diagnostics`), enforced by `tests/architecture.rs` (`<= 400` lines per library source file).
- **Three-Tier Performance (`L1` / `L2` / `L3`)**
  - **L1 Zero-config smart defaults**: `PlotData::new` auto-builds a monotonic-X index (`>= 8,192` samples, 64 MiB cap); `LineSampling::Auto` & `MarkerSampling::Auto` preserve peaks and switch to exact rendering when zoomed in; `PlotState` embeds a bounded 2 MiB auto cache; raw-point picking uses 2D screen-AABB branch-and-bound pruning.
  - **L2 Explicit customization**: `LineSampling::{Auto, Exact, Visible, MinMax}`, `MarkerSampling::{Auto, Exact, PixelGrid}`, `RenderBackend::{Auto, Painter, Wgpu}`, 5-layer style inheritance (`defaults -> Theme -> inherited/Figure -> Plot -> Series`), `PlotDefaults`, `PlotCache`, `Plot::pan_limits`, dual Y, linear/log10/probit/weibull/datetime/duration/categorical axes, and self-contained PNG/SVG snapshots (`feature = "export"`).
  - **L3 10M-point live streaming & GPU**: Zero-copy `IndexedStream` (`StreamLine`), `O(1)`-sliced `UniformWaveform` (`WaveformLine`), and optional `feature = "wgpu"` (`register_wgpu`) WGSL compute + instanced rendering reusing the host `egui_wgpu::RenderState`, with automatic CPU `Painter` fallback when unavailable or exporting.
- **Coordinate Systems & Chart Breadth**
  - **2D Cartesian & Geometry**: Line / stairs / stems / scatter / error bars / bars (grouped, stacked, horizontal) / area / fill / patch (Flat & Gouraud) / polyshape (holes) / rectangles / heatmap / pcolor / curvilinear grid / contours & filled contours / `imshow` / adaptive `fplot` & `fimplicit` / quiver / feather / streamlines / Delaunay `triplot` / Voronoi / `alphaShape` / CSR `spy` / `animatedline` & `comet` / `EX-027` annotations.
  - **Statistics, Bio, SPC, Finance & Composites**: `histogram` / `histogram2` / `binscatter` / `boxchart` / `violinplot` / `raincloud` / `swarmchart` / `bubblechart` / `pie` & `donut` / `wordcloud` & `bubblecloud` / `categoricalHeatmap` / `confusionchart` / `parallelplot` / `biplot` / `ecdf` (Kaplan-Meier + Greenwood) / `probplot` (`Probit`, `Weibull`, `qqplot`) / `dendrogram` / `clustergram` / `controlchart` (SPC + Nelson rules) / `candlestick` & `ohlc` / `waterfall` / `Figure` (up to 256 cells), `StackedPlot`, `ScatterHistogram`, `PlotMatrix`, `PlotYy`, `ParetoChart`.
  - **Polar, Sky, RF, Control & DSP**: `PolarPlot` (`polarplot`, `polarscatter`, `polarbubblechart`, `polarhistogram`, `rose`, `compass`, `thetaregion`/`radiusregion`/`polarregion`), `SkyPlot`, `SmithPlot` (conformal Z/Y/YZ grids), `ZPlanePlot`, `BodePlot`, `NyquistPlot`, `NicholsPlot`, `RootLocus`, and pure-Rust Radix-2 + Bluestein FFT / Welch / STFT / Morlet CWT (`CwtPlot`).
  - **3D & Volumetric**: `Plot3D` with `Orthographic` / `Perspective`, `SortedPainter` / bounded `ZBuffer` (with idle-frame RLE span caching), `plot3`, `fplot3`, `comet3`, `scatter3`, `bubblechart3`, `swarmchart3`, `bar3`/`bar3h`, `stem3`, `quiver3`, `fill3`, `constantplane`, `pie3`, `surf`/`surfc`/`surfl`/`surfnorm`, `mesh`/`meshc`/`meshz`/`waterfall`, `contour3`, `ribbon`, `fsurf`/`fmesh`/`fimplicit3`, `sphere`/`cylinder`/`ellipsoid`/`trisurf`, `Patch3D`, `AlphaShape3D`, `Graph3D`, `slice`, `contourslice`, `isosurface`, `isocaps`, `tetramesh`, 3D stream plots/cones, CPU `VolumeRender3D` (`volshow`) raycasting, and octree-LOD `PointCloud3D` (`pcshow`).
  - **Geographic & Modern Diagrams**: Offline 2D `GeoPlot` (`geoplot`, `geoscatter`, `geodensityplot`, `geobubble`, dateline splitting, `GeoTileCache` + `GeoOfflinePolicy`), 3D WGS84 `GeoGlobePlot`, plus `RadarChart`, `SankeyChart`, `TreemapChart`, `SunburstChart`, and `GanttChart`.

---

## Requirements & Cargo Features

- **Rust**: `1.95+` (Edition `2024`; tested on Rust `1.100.0` / Windows x64 / `egui 0.37.0`)
- **Cargo features (`crates/egui-customplot/Cargo.toml`)**:
  - `default = []`: Core CPU + `egui::Painter` plotting (depends only on `egui`).
  - `export`: Enables `PlotSnapshot`, `ExportOptions`, and `CaptureLimits` for PNG & hybrid-vector SVG output (`png` dependency).
  - `wgpu`: Enables `register_wgpu`, `register_wgpu_with_config`, `set_wgpu_budget`, and `WgpuPlotConfig` (`egui-wgpu`, `wgpu`, `bytemuck`).

---

## Quick Start

Because `publish = false` is currently set while release readiness boundaries remain open, depend on the workspace crate via Git or local path:

```toml
[dependencies]
egui = "0.37.0"
egui-customplot = { git = "https://github.com/littepointR/egui-customplot" }
# Optional features:
# egui-customplot = { git = "https://github.com/littepointR/egui-customplot", features = ["export", "wgpu"] }
```

### Minimal 2D Plot

Keep sample data and `PlotState` in your application model across frames (do not rebuild `PlotData` every frame):

```rust
use egui_customplot::{Line, Plot, PlotData, PlotState};

// App initialization:
let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
let mut state = PlotState::default();

// Inside your egui UI closure (`ui: &mut egui::Ui`):
Plot::new("signal")
    .height(320.0)
    .allow_scroll_zoom(true) // off by default so parent ScrollArea is not hijacked
    .allow_box_zoom(true)
    .show(ui, &mut state, &[Line::new(&data).name("Signal")]);
```

### Heterogeneous Layers (`PlotLayer`)

```rust
use egui_customplot::{Bar, BarData, Bars, ErrorBar, ErrorBarData, ErrorBars, Plot, PlotState};

let bars = BarData::new(vec![Bar::new(1.0, 2.0), Bar::new(2.0, -1.0)]);
let errors = ErrorBarData::new(vec![ErrorBar::new([1.0, 2.0]).y_error([0.2, 0.4])]);
let mut state = PlotState::default();

Plot::new("measurements").show_layers(
    ui,
    &mut state,
    &[Bars::new(&bars).into(), ErrorBars::new(&errors).into()],
);
```

See the [API & User Guide](crates/egui-customplot/docs/guide.md) or build the interactive rustdoc (`EN | 简体中文` toggle in the header/sidebar):

```sh
cargo doc --workspace --no-deps --all-features --open
```

---

## Interactive Demo (`customplot-demo`)

The demo showcases **27 mutually exclusive galleries**, global `PlotDefaults` controls, OS light/dark theme sync, PNG/SVG export, and runtime switching across **English / 简体中文 / 日本語** (interactive launch defaults to Simplified Chinese or `$CUSTOMPLOT_LANG` / `$LANG`):

```sh
cargo run -p customplot-demo --release --locked
cargo run -p customplot-demo --release --locked -- --en --dynamic-10m
cargo run -p customplot-demo --release --locked -- --glow --plot3d
```

Available CLI options:

- **Language & renderer**: `--en`, `--zh` / `--zh-cn`, `--ja`, `--lang=<code>`, `--glow` (use `eframe::Renderer::Glow` instead of default `Wgpu`), `--uncapped`, `--bench-window-10m=<path.csv>`
- **2D core & fields**: (default 2D signal/power-law), `--scalar`, `--statistical`, `--stairs`, `--stems`, `--quiver`, `--polygons`, `--contours`, `--functions`, `--geometry`
- **Stats, polar & networks**: `--distributions`, `--multivariate`, `--polar`, `--networks`
- **3D, volumes & geo**: `--plot3d`, `--surfaces3d`, `--volumes3d`, `--geo`
- **Extensions & 10M**: `--stage13`, `--stage14`, `--stage15`, `--stage16`, `--dynamic-10m`
- **Customization showcases**: `--custom-axes`, `--custom-annotations`, `--custom-animation`, `--custom-styling`

Controls in the plot area: drag to pan, scroll wheel to zoom, right-drag or `Shift`+drag for box zoom, `Alt`+drag to brush points (`Esc` clears brush), double-click or focused `Home` to reset view; in 3D plots, left-drag orbits and right-drag pans.

---

## Verification & Benchmarks

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo test --workspace --all-features --locked
cargo check -p egui-customplot --no-default-features --locked
cargo doc -p egui-customplot --no-deps --no-default-features --locked
cargo doc --workspace --no-deps --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo build -p customplot-demo --release --locked
cargo run -p egui-customplot --example export_snapshot --features export --release --locked -- target/exports/example
```

Recorded Windows measurements (historical CSVs are never overwritten; CPU + tessellation benches do **not** represent GPU or swapchain FPS unless explicitly noted):

- **Headless 10M CPU + tessellation** (`1920x1080`, [2026-11-16-dynamic-10m-cpu.csv](docs/benchmarks/2026-11-16-dynamic-10m-cpu.csv)):
  - `uniform_waveform_10m_live_32k`: `total_p95 = 2.1877 ms` (`p50 = 1.9166 ms`)
  - `indexed_stream_10m_live_32k`: `total_p95 = 6.6787 ms` (`p50 = 6.0130 ms`)
  - `plotdata_auto_10m_pan_hover`: `total_p95 = 1.3365 ms` (`pick_p95 = 0.1149 ms`)
  - `scatter_auto_1m_pan`: `total_p95 = 4.8535 ms`
- **Deep zoom-out 10M CPU** (`20x` to `10^12x` + hover, [2026-11-16-deep-zoom-out-10m-cpu.csv](docs/benchmarks/2026-11-16-deep-zoom-out-10m-cpu.csv)): `total_p50 = 0.8311 ms`, `total_p95 = 1.4256 ms`, `pick_p95 = 0.0115 ms`.
- **Native Windows `eframe + wgpu` swapchain** (`AMD Radeon RX 7800 XT (Vulkan)`, `AutoNoVsync`, [2026-11-16-dynamic-10m-window-wgpu.csv](docs/benchmarks/2026-11-16-dynamic-10m-window-wgpu.csv)):
  - `wgpu_10m_waveform_live_32k`: **699.99 FPS** (2,450 frames, `frame_p50 = 1.3124 ms`, `frame_p95 = 1.7540 ms`, 131,072 B/frame upload)
  - `cpu_10m_stream_ring_live_32k` (CPU incremental tree + LOD): **174.13 FPS** (610 frames, `frame_p50 = 5.5789 ms`, `frame_p95 = 6.3164 ms`)
- **Stage 16 3D/Modern CPU** (`1280x800`, [2026-11-17-stage16-volshow-opt-cpu.csv](docs/benchmarks/2026-11-17-stage16-volshow-opt-cpu.csv) & [2026-11-17-stage16-windows-cpu.csv](docs/benchmarks/2026-11-17-stage16-windows-cpu.csv)):
  - `volshow_64x64x64_direct_alpha` (cached idle): `total_p95 = 2.889 ms` (`orbiting_uncached`: `1.893 ms`), `pcshow_200k_octree_lod_zbuffer`: `0.890 ms`, `geoglobe_wgs84_coastlines_track`: `0.053 ms`, `modern_suite_5charts_figure`: `0.104 ms`.

To run benchmarks manually (`CUSTOMPLOT_BENCH_SECONDS=2` for quick smoke runs; keep `30` s for official runs):

```sh
cargo bench -p egui-customplot --bench dynamic_10m_performance --locked
cargo bench -p egui-customplot --bench stage16_performance --locked
```

---

## Documentation

- [Architecture & Domain Boundaries](docs/ARCHITECTURE.md)
- [Implementation Progress & Evidence Log](docs/PROGRESS.md)
- [Project Plan & Milestones](docs/PLAN.md)
- [Stages 12–16 Expansion Plan (`EXT-01..26`)](docs/EXPANSION_PLAN.md)
- [MATLAB Plot Delivery Checklist (133 items + Toolboxes)](docs/MATLAB_PLOT_CHECKLIST.md)
- [egui Integration & Performance Contracts](docs/EGUI_INTEGRATION_AND_PERFORMANCE.md)
- Stage Reports: [1](docs/STAGE_1.md) · [2](docs/STAGE_2.md) · [3](docs/STAGE_3.md) · [4](docs/STAGE_4.md) · [5](docs/STAGE_5.md) · [6](docs/STAGE_6.md) · [7](docs/STAGE_7.md) · [8](docs/STAGE_8.md) · [9](docs/STAGE_9.md) · [10](docs/STAGE_10.md) · [11](docs/STAGE_11.md) · [12](docs/STAGE_12.md) · [13](docs/STAGE_13.md) · [14](docs/STAGE_14.md) · [15](docs/STAGE_15.md) · [16](docs/STAGE_16.md)
- Community & Policies: [Contributing](CONTRIBUTING.md) · [Changelog](CHANGELOG.md) · [Security](SECURITY.md)

---

## Contributing & License

Contributions are welcome — please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option, matching [`egui`](https://github.com/emilk/egui). Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository shall be dual-licensed as above, without any additional terms or conditions (Apache-2.0 Section 5).
