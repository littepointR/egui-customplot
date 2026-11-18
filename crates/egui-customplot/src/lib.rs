#![doc = include_str!("../docs/overview.md")]
#![warn(rustdoc::broken_intra_doc_links)]

mod axis;
mod data;
mod diagnostics;
mod interaction;
mod layout;
mod model;
mod render;
mod widget;

/// Complete user guide, architecture reference, and plot catalog (`EN` / `简体中文` switchable).
///
/// 完整使用指南、架构约定与全图种速查手册（点击页面右上或侧栏 `EN | 简体中文` 切换语言）。
#[cfg(doc)]
#[doc = include_str!("../docs/guide.md")]
pub mod guide {}

pub use axis::{
    Axis, Axis3D, AxisDecoration3D, AxisEdge3D, AxisLabel3D, AxisLayoutContext3D, AxisScale,
    AxisTick, Bounds3D, BoundsError, Camera3D, CameraLens3D, CivilDateTime, DataAspect3D,
    GeoBounds, GeoError, GeoGlobeTransform, GeoPoint, GeoProjection, GeoTransform,
    NegativeRadiusMode, PlotBounds, PlotTransform, Point3D, PolarTransform, Projection3D,
    ScaleMapping3D, SkyTransform, SmithGridKind, SmithTransform, ThetaDirection, ThetaZeroLocation,
    TimeAxisMode, Transform3D, YAxis, civil_to_unix_secs, format_duration_hms, haversine_km,
    norm_cdf, norm_inv, shortest_dlon, unix_secs_to_civil, wrap_lon_180,
};
pub use data::{
    AUTO_INDEX_MIN_POINTS, AlphaShape3DData, AlphaShapeData, AnimatedLine2D, AnimatedLine3D,
    AppendReport, AreaBandData, AreaError, AreaStackMode, Bar, Bar3DBox, Bar3DData, Bar3DMode,
    Bar3DOrientation, BarData, BarOrientation, BarSeriesError, BinCell, BinScatterData, BinShape,
    BiplotData, BivariateError, BivariateNormalization, BoxChartData, BoxSummary, Bubble3DData,
    Bubble3DPoint, BubbleCloudData, BubbleData, BubbleError, BubblePoint, CategoricalHeatmapData,
    CloudError, ClusterStandardize, ClustergramData, Comet3DPlayback, CometPlayback, CompassData,
    ComplexRoot, CompositeError, Cone3DItem, ConePlot3DData, ConfusionError, ConfusionMatrixData,
    ConfusionNormalization, Contour3DData, Contour3DLine, ContourBand, ContourData, ContourError,
    ContourLevels, ContourLine, ControlChartData, ControlError, CurvilinearField2D, CwtData,
    DataRect, DendrogramBranch, DendrogramData, DspError, EcdfData, EcdfKind, EliminationTree,
    ErrorBar, ErrorBarData, FeatherData, FilledContourData, FinancialError, FlowError,
    FrequencyResponseData, FunctionContourError, FunctionCurveData, FunctionPlotError,
    FunctionPlotOptions, GanttData, GanttTask, GeoAttribution, GeoBasemapStyle, GeoBubbleData,
    GeoBubblePoint, GeoDensityData, GeoDensityGrid, GeoGlobeData, GeoOfflinePolicy, GeoPathData,
    GeoRadiusMode, GeoRasterTile, GeoScatterData, GeoScatterPoint, GeoTileCache, GeoTileKey,
    GlobeMarker3D, GlobeTrack3D, Graph3DData, GraphData, GraphEdge, GraphError, GraphLayout,
    GraphLayout3D, GraphLayoutKind, GridData, GridError, Histogram2Data, HistogramData,
    HistogramError, HistogramNormalization, ImageData, ImageError, ImplicitCurveData,
    ImplicitOptions, ImplicitSurface3DData, IndexedStream, IsocapThreshold, Isocaps3DData,
    Isosurface3DData, LegacyHistData, LineIndexError, LinkageMethod, LinkageStep, MarginalKind,
    ModernError, MultivariateError, OhlcBar, OhlcData, PackedBubble, ParallelData,
    ParallelNormalization, ParametricCurve3DData, ParetoData, Patch3DData, PatchData, PatchError,
    Pie3DData, Pie3DSlice, PieData, PieError, PieSlice, PlacedWord, Plot3DError, PlotData,
    PlotMatrixData, PlotPoint, PlotStream, Point3DData, PointCloud3DData, PolarBubbleData,
    PolarBubblePoint, PolarCurveData, PolarError, PolarFunctionData, PolarHistNormalization,
    PolarHistogramData, PoleZeroData, PolyRegion, PolyShapeData, Polygon3D, Polygon3DData,
    PolygonData, ProbabilityDist, ProbabilityPlotData, PspectrumData, Quiver3DArrow, Quiver3DData,
    QuiverArrow, QuiverData, RadarData, RadarSeries, RaincloudData, RaincloudEntry, RectData,
    Ribbon3DData, RootLocusData, RoseData, SankeyData, SankeyLink, SankeyNode, SankeyRibbon,
    ScalarField2D, ScalarVolume3D, ScatterHistogramData, SkyError, SkyPlotData, SkySatellite,
    SkyTrack, SlicePlane3D, SmithError, SmithPlotData, SmithTrace, SortOrder, SpcChartKind,
    SpcPoint, SpcViolationKind, SpectrogramData, SpyData, SpyError, StabilityMargins, StatError,
    Stem3DData, StepResponseData, StepResponseError, Stream3DOptions, StreamError, StreamOptions,
    StreamParticles3DData, StreamParticlesPlayback, StreamRibbon3DData, StreamTube3DData,
    Streamline3DData, Streamline3DLine, StreamlineData, SunburstData, SunburstInput,
    SunburstSector, Surface3DData, Swarm3DData, Swarm3DPoint, SwarmData, SwarmPoint,
    TableAggregation, TetraMesh3DData, TransferFunction, TransferStop, TreeData, TreemapData,
    TreemapRect, TriMesh2D, TriMesh3DData, TriMeshError, UniformWaveform, VectorField2D,
    VectorField3D, ViolinData, ViolinEntry, ViolinHalf, ViolinOptions, ViolinScale,
    VolumeRenderMode, VolumeSliceData, VolumeTransferFunction, VoronoiData, Waterfall2DData,
    WaterfallColumn, WaterfallKind, WaveformError, WindowKind, WordCloudData, fft_arbitrary,
    fft_radix2, fft_real, poly_eval_complex, poly_roots_aberth, visible_tiles_for_bounds,
};
pub use diagnostics::{
    ContourDiagnostics, FlowDiagnostics, GeoDiagnostics, GeometryDiagnostics, ImageDiagnostics,
    NetworkDiagnostics, PatchDiagnostics, Plot3DDiagnostics, PlotDiagnostics, PolarDiagnostics,
    StatDiagnostics,
};
pub use interaction::{
    AxisLink, CameraGesture3D, CameraNavigation3D, CursorMeasurements, CursorReading,
    GeoGlobeState, GeoPlotState, HoverInfo, Hovered3DBar, Hovered3DPoint, Hovered3DRegion,
    HoveredBar, HoveredCell, HoveredGeoPoint, HoveredGlobeItem, HoveredPoint, HoveredPolarBin,
    HoveredPolarPoint, HoveredRegion, HoveredSkyItem, HoveredSmithPoint, HoveredVector, LinkError,
    LinkReport, Plot3DState, PlotState, PolarState, RegionKind, SharedCursors, SkyPlotState,
    SmithPlotState, VectorAnchor,
};
pub use layout::{
    BubbleLegend, BubbleLegendResponse, DecorationContent, FigureError, LegendComponent,
    LegendEntry, LegendPosition, PlotDecoration,
};
pub use model::{
    AlphaShape, AlphaShape3D, AnimatedLine, Annotation, AnnotationCoord, Area, Bar3D, Bars,
    BinScatter, Biplot, BoxChart, BoxStyle3D, Bubble3D, BubbleChart, BubbleCloud, Candlestick,
    CategoricalHeatmap, ColorMap, ColorScale, ColorScaleError, Comet, Comet3D, CompassPlot,
    ConePlot3D, ConfusionChart, ConstantPlane3D, Contour3D, Contours, ControlChart,
    CurvilinearPcolor, CustomLayer3D, CustomPlotLayer, CwtPlot, DendrogramPlot, DepthMode,
    EcdfPlot, ErrorBars, Feather, Fill, Fill3D, FilledContours, FunctionPlot, GanttChart,
    GeoBubbles, GeoDensity, GeoLayer, GeoLine, GeoScatter, GeometryBatch3D, Graph3D,
    GraphHighlight, GraphPlot, Heatmap, Histogram2, ImplicitCurve, Isocaps3D, Isosurface3D,
    Layer3D, LayerPaintContext, LightSource3D, Line, Line3D, LineInterpolation, LinePattern,
    LineSampling, MarkerSampling, MarkerShape, MarkerStyle, MeshSurface3D, MeshSurfaceStyle,
    NicholsPlot, NyquistPlot, OhlcBars, ParallelPlot, Patch, Patch3D, PatchColoring, Pcolor,
    PcolorShading, Pie3D, PieChart, PlotDefaults, PlotImage, PlotLayer, PlotStyle, PlotTheme,
    PointCloud3D, PolarBubble, PolarHistogram, PolarLayer, PolarLine, PolarRegionAnnotation,
    PolarRegionKind, PolarScatter, PolyShape, ProbabilityPlot, Quiver, Quiver3D, RadarChart,
    Raincloud, Rectangles, RenderBackend, Ribbon3D, RootLocus, SankeyChart, Scatter, Scatter3D,
    Series, SeriesError, SeriesStyle, Spy, Stem3D, StemOrientation, Stems, StreamLine,
    StreamParticles3D, StreamRibbon3D, StreamTube3D, Streamline3D, Streamlines, SunburstChart,
    Surface3D, SurfaceShading, Swarm3D, SwarmChart, TetraMesh3D, Text3D, TreemapChart, TriPlot,
    TriSurf3D, ViolinPlot, VolumeRender3D, VolumeSlice3D, Voronoi, Waterfall2D, WaveformLine,
    WordCloud, ZPlanePlot, plot_defaults, set_plot_defaults,
};
pub use render::PlotCache;
#[cfg(feature = "export")]
pub use render::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
#[cfg(feature = "wgpu")]
pub use render::{WgpuPlotConfig, register_wgpu, register_wgpu_with_config, set_wgpu_budget};
pub use widget::{
    BodePanel, BodePlot, BodePlotResponse, BodePlotState, Clustergram, ClustergramResponse,
    ClustergramState, ColorBar, ColorBarResponse, Figure, FigureCell, FigureResponse, GeoGlobePlot,
    GeoGlobeResponse, GeoPlot, GeoPlotResponse, ParetoChart, Plot, Plot3D, Plot3DResponse,
    PlotMatrix, PlotResponse, PlotYy, PolarPlot, PolarPlotResponse, ScatterHistogram,
    ScatterHistogramState, SkyPlot, SkyPlotResponse, SmithPlot, SmithPlotResponse, StackedPlot,
    StackedPlotResponse, StepResponsePlot,
};
