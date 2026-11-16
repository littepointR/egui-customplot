//! Native egui plotting, with persistent data and explicitly owned view state.
//!
//! Three-tier performance design:
//! - L1 smart defaults: `PlotData::new` auto-indexes large monotonic-X series, `Line` /
//!   `Scatter` default to `Auto` LOD (`LineSampling::Auto`, `MarkerSampling::Auto`), `PlotState`
//!   includes a bounded 2 MiB automatic geometry cache, and hover picking uses Y branch-and-bound pruning.
//! - L2 full control: explicit `LineSampling`, `MarkerSampling`, `RenderBackend`, `PlotCache`,
//!   `PlotData::new_unindexed`, log/reversed/dual-Y axes, 5-layer styling, and `PlotSnapshot` export.
//! - L3 power-user sources & GPU: zero-copy `IndexedStream` (`StreamLine`), compact $O(1)$-sliced
//!   `UniformWaveform` (`WaveformLine`), and optional `feature = "wgpu"` compute/instancing via host `RenderState`.
//!
//! ```
//! use egui_customplot::{Line, Plot, PlotData, PlotState};
//!
//! let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
//! let mut state = PlotState::default(); // retain this alongside your app model
//! # egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
//! Plot::new("signal").show(ui, &mut state, &[Line::new(&data)]);
//! # }).drop_without_applying_deltas();
//! ```
//!
//! Preprocess outside the per-pass UI closure; opt into approximation explicitly:
//! ```
//! use egui_customplot::{Line, LineSampling, PlotCache, PlotData, PlotStream};
//! let mut stream = PlotStream::new(4096, 4096 * 16)?;
//! stream.append_once(1, &[[1.0, 2.0], [2.0, 3.0]])?;
//! let mut data: PlotData = stream.snapshot(); // explicit O(N) publication
//! data.build_line_index(1024 * 1024)?;
//! let line = Line::new(&data).sampling(LineSampling::MinMax);
//! let mut cache = PlotCache::new(8 * 1024 * 1024); // retain in the host
//! // Plot::new("signal").show_cached(ui, state, &[line], &mut cache);
//! # let _ = (line, &mut cache);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod axis;
mod data;
mod diagnostics;
mod interaction;
mod layout;
mod model;
mod render;
mod widget;

pub use axis::{
    Axis, Axis3D, AxisScale, AxisTick, Bounds3D, BoundsError, Camera3D, DataAspect3D, GeoBounds,
    GeoError, GeoPoint, GeoProjection, GeoTransform, NegativeRadiusMode, PlotBounds, PlotTransform,
    Point3D, PolarTransform, Projection3D, ThetaDirection, ThetaZeroLocation, Transform3D, YAxis,
    haversine_km, shortest_dlon, wrap_lon_180,
};
pub use data::{
    AUTO_INDEX_MIN_POINTS, AlphaShapeData, AnimatedLine2D, AnimatedLine3D, AppendReport,
    AreaBandData, AreaError, AreaStackMode, Bar, Bar3DBox, Bar3DData, Bar3DMode, Bar3DOrientation,
    BarData, BarOrientation, BarSeriesError, BinCell, BinScatterData, BinShape, BivariateError,
    BivariateNormalization, BoxChartData, BoxSummary, Bubble3DData, Bubble3DPoint, BubbleCloudData,
    BubbleData, BubbleError, BubblePoint, CategoricalHeatmapData, CloudError, Comet3DPlayback,
    CometPlayback, CompassData, CompositeError, Cone3DItem, ConePlot3DData, Contour3DData,
    Contour3DLine, ContourBand, ContourData, ContourError, ContourLevels, ContourLine, DataRect,
    EliminationTree, ErrorBar, ErrorBarData, FeatherData, FilledContourData, FlowError,
    FunctionContourError, FunctionCurveData, FunctionPlotError, FunctionPlotOptions,
    GeoAttribution, GeoBasemapStyle, GeoBubbleData, GeoBubblePoint, GeoDensityData, GeoDensityGrid,
    GeoOfflinePolicy, GeoPathData, GeoRadiusMode, GeoRasterTile, GeoScatterData, GeoScatterPoint,
    GeoTileCache, GeoTileKey, GraphData, GraphEdge, GraphError, GraphLayout, GraphLayoutKind,
    GridData, GridError, Histogram2Data, HistogramData, HistogramError, HistogramNormalization,
    ImageData, ImageError, ImplicitCurveData, ImplicitOptions, ImplicitSurface3DData,
    IndexedStream, IsocapThreshold, Isocaps3DData, Isosurface3DData, LegacyHistData,
    LineIndexError, MarginalKind, MultivariateError, PackedBubble, ParallelData,
    ParallelNormalization, ParametricCurve3DData, ParetoData, PatchData, PatchError, Pie3DData,
    Pie3DSlice, PieData, PieError, PieSlice, PlacedWord, Plot3DError, PlotData, PlotMatrixData,
    PlotPoint, PlotStream, Point3DData, PolarBubbleData, PolarBubblePoint, PolarCurveData,
    PolarError, PolarFunctionData, PolarHistNormalization, PolarHistogramData, PolyRegion,
    PolyShapeData, Polygon3D, Polygon3DData, PolygonData, Quiver3DArrow, Quiver3DData, QuiverArrow,
    QuiverData, RaincloudData, RaincloudEntry, RectData, Ribbon3DData, RoseData, ScalarField2D,
    ScalarVolume3D, ScatterHistogramData, SlicePlane3D, SortOrder, SpyData, SpyError, StatError,
    Stem3DData, Stream3DOptions, StreamError, StreamOptions, StreamParticles3DData,
    StreamParticlesPlayback, StreamRibbon3DData, StreamTube3DData, Streamline3DData,
    Streamline3DLine, StreamlineData, Surface3DData, Swarm3DData, Swarm3DPoint, SwarmData,
    SwarmPoint, TableAggregation, TetraMesh3DData, TreeData, TriMesh2D, TriMesh3DData,
    TriMeshError, UniformWaveform, VectorField2D, VectorField3D, ViolinData, ViolinEntry,
    ViolinHalf, ViolinOptions, ViolinScale, VolumeSliceData, VoronoiData, WaveformError,
    WordCloudData, visible_tiles_for_bounds,
};
pub use diagnostics::{
    ContourDiagnostics, FlowDiagnostics, GeoDiagnostics, GeometryDiagnostics, ImageDiagnostics,
    NetworkDiagnostics, PatchDiagnostics, Plot3DDiagnostics, PlotDiagnostics, PolarDiagnostics,
    StatDiagnostics,
};
pub use interaction::{
    AxisLink, GeoPlotState, HoverInfo, Hovered3DBar, Hovered3DPoint, Hovered3DRegion, HoveredBar,
    HoveredCell, HoveredGeoPoint, HoveredPoint, HoveredPolarBin, HoveredPolarPoint, HoveredRegion,
    HoveredVector, LinkError, LinkReport, Plot3DState, PlotState, PolarState, RegionKind,
    VectorAnchor,
};
pub use layout::{BubbleLegend, BubbleLegendResponse, FigureError, LegendEntry};
pub use model::{
    AlphaShape, AnimatedLine, Annotation, AnnotationCoord, Area, Bar3D, Bars, BinScatter, BoxChart,
    BoxStyle3D, Bubble3D, BubbleChart, BubbleCloud, CategoricalHeatmap, ColorMap, ColorScale,
    ColorScaleError, Comet, Comet3D, CompassPlot, ConePlot3D, ConstantPlane3D, Contour3D, Contours,
    DepthMode, ErrorBars, Feather, Fill, Fill3D, FilledContours, FunctionPlot, GeoBubbles,
    GeoDensity, GeoLayer, GeoLine, GeoScatter, GraphHighlight, GraphPlot, Heatmap, Histogram2,
    ImplicitCurve, Isocaps3D, Isosurface3D, Layer3D, LightSource3D, Line, Line3D,
    LineInterpolation, LinePattern, LineSampling, MarkerSampling, MarkerShape, MarkerStyle,
    MeshSurface3D, MeshSurfaceStyle, ParallelPlot, Patch, PatchColoring, Pcolor, PcolorShading,
    Pie3D, PieChart, PlotImage, PlotLayer, PlotStyle, PlotTheme, PolarBubble, PolarHistogram,
    PolarLayer, PolarLine, PolarRegionAnnotation, PolarRegionKind, PolarScatter, PolyShape, Quiver,
    Quiver3D, Raincloud, Rectangles, RenderBackend, Ribbon3D, Scatter, Scatter3D, Series,
    SeriesError, SeriesStyle, Spy, Stem3D, StemOrientation, Stems, StreamLine, StreamParticles3D,
    StreamRibbon3D, StreamTube3D, Streamline3D, Streamlines, Surface3D, SurfaceShading, Swarm3D,
    SwarmChart, TetraMesh3D, Text3D, TriPlot, TriSurf3D, ViolinPlot, VolumeSlice3D, Voronoi,
    WaveformLine, WordCloud,
};
pub use render::PlotCache;
#[cfg(feature = "export")]
pub use render::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
#[cfg(feature = "wgpu")]
pub use render::{WgpuPlotConfig, register_wgpu, register_wgpu_with_config, set_wgpu_budget};
pub use widget::{
    ColorBar, ColorBarResponse, Figure, FigureCell, FigureResponse, GeoPlot, GeoPlotResponse,
    ParetoChart, Plot, Plot3D, Plot3DResponse, PlotMatrix, PlotResponse, PlotYy, PolarPlot,
    PolarPlotResponse, ScatterHistogram, ScatterHistogramState, StackedPlot, StackedPlotResponse,
};
