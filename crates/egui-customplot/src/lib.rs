//! Native egui plotting, with persistent data and explicitly owned view state.
//!
//! Supports mixed lines/markers/scatter, linear/log axes, legends and raw-point
//! hover. Optional explicit monotonic-X indexes, min/max line LOD, a host-owned
//! bounded geometry cache and a bounded streaming ring are available. Markers
//! still visit raw samples; there is no GPU callback or performance guarantee.
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
    Axis, Axis3D, AxisScale, AxisTick, Bounds3D, BoundsError, Camera3D, DataAspect3D,
    NegativeRadiusMode, PlotBounds, PlotTransform, Point3D, PolarTransform, Projection3D,
    ThetaDirection, ThetaZeroLocation, Transform3D, YAxis,
};
pub use data::{
    AlphaShapeData, AnimatedLine2D, AnimatedLine3D, AppendReport, AreaBandData, AreaError,
    AreaStackMode, Bar, Bar3DBox, Bar3DData, Bar3DMode, Bar3DOrientation, BarData, BarOrientation,
    BarSeriesError, BinCell, BinScatterData, BinShape, BivariateError, BivariateNormalization,
    BoxChartData, BoxSummary, Bubble3DData, Bubble3DPoint, BubbleCloudData, BubbleData,
    BubbleError, BubblePoint, CategoricalHeatmapData, CloudError, Comet3DPlayback, CometPlayback,
    CompassData, CompositeError, ContourBand, ContourData, ContourError, ContourLevels,
    ContourLine, DataRect, EliminationTree, ErrorBar, ErrorBarData, FeatherData, FilledContourData,
    FlowError, FunctionContourError, FunctionCurveData, FunctionPlotError, FunctionPlotOptions,
    GraphData, GraphEdge, GraphError, GraphLayout, GraphLayoutKind, GridData, GridError,
    Histogram2Data, HistogramData, HistogramError, HistogramNormalization, ImageData, ImageError,
    ImplicitCurveData, ImplicitOptions, LegacyHistData, LineIndexError, MarginalKind,
    MultivariateError, PackedBubble, ParallelData, ParallelNormalization, ParametricCurve3DData,
    ParetoData, PatchData, PatchError, Pie3DData, Pie3DSlice, PieData, PieError, PieSlice,
    PlacedWord, Plot3DError, PlotData, PlotMatrixData, PlotPoint, PlotStream, Point3DData,
    PolarBubbleData, PolarBubblePoint, PolarCurveData, PolarError, PolarFunctionData,
    PolarHistNormalization, PolarHistogramData, PolyRegion, PolyShapeData, Polygon3D,
    Polygon3DData, PolygonData, Quiver3DArrow, Quiver3DData, QuiverArrow, QuiverData,
    RaincloudData, RaincloudEntry, RectData, RoseData, ScalarField2D, ScatterHistogramData,
    SortOrder, SpyData, SpyError, StatError, Stem3DData, StreamError, StreamOptions,
    StreamlineData, Swarm3DData, Swarm3DPoint, SwarmData, SwarmPoint, TableAggregation, TreeData,
    TriMesh2D, TriMeshError, VectorField2D, ViolinData, ViolinEntry, ViolinHalf, ViolinOptions,
    ViolinScale, VoronoiData, WordCloudData,
};
pub use diagnostics::{
    ContourDiagnostics, FlowDiagnostics, GeometryDiagnostics, ImageDiagnostics, NetworkDiagnostics,
    PatchDiagnostics, Plot3DDiagnostics, PlotDiagnostics, PolarDiagnostics, StatDiagnostics,
};
pub use interaction::{
    AxisLink, HoverInfo, Hovered3DBar, Hovered3DPoint, Hovered3DRegion, HoveredBar, HoveredCell,
    HoveredPoint, HoveredPolarBin, HoveredPolarPoint, HoveredRegion, HoveredVector, LinkError,
    LinkReport, Plot3DState, PlotState, PolarState, RegionKind, VectorAnchor,
};
pub use layout::{BubbleLegend, BubbleLegendResponse, FigureError, LegendEntry};
pub use model::{
    AlphaShape, AnimatedLine, Annotation, AnnotationCoord, Area, Bar3D, Bars, BinScatter, BoxChart,
    BoxStyle3D, Bubble3D, BubbleChart, BubbleCloud, CategoricalHeatmap, ColorMap, ColorScale,
    ColorScaleError, Comet, Comet3D, CompassPlot, ConstantPlane3D, Contours, DepthMode, ErrorBars,
    Feather, Fill, Fill3D, FilledContours, FunctionPlot, GraphHighlight, GraphPlot, Heatmap,
    Histogram2, ImplicitCurve, Layer3D, Line, Line3D, LineInterpolation, LinePattern, LineSampling,
    MarkerShape, MarkerStyle, ParallelPlot, Patch, PatchColoring, Pcolor, PcolorShading, Pie3D,
    PieChart, PlotImage, PlotLayer, PlotStyle, PlotTheme, PolarBubble, PolarHistogram, PolarLayer,
    PolarLine, PolarRegionAnnotation, PolarRegionKind, PolarScatter, PolyShape, Quiver, Quiver3D,
    Raincloud, Rectangles, Scatter, Scatter3D, Series, SeriesError, SeriesStyle, Spy, Stem3D,
    StemOrientation, Stems, Streamlines, Swarm3D, SwarmChart, Text3D, TriPlot, ViolinPlot, Voronoi,
    WordCloud,
};
pub use render::PlotCache;
#[cfg(feature = "export")]
pub use render::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
pub use widget::{
    ColorBar, ColorBarResponse, Figure, FigureCell, FigureResponse, ParetoChart, Plot, Plot3D,
    Plot3DResponse, PlotMatrix, PlotResponse, PlotYy, PolarPlot, PolarPlotResponse,
    ScatterHistogram, ScatterHistogramState, StackedPlot, StackedPlotResponse,
};
