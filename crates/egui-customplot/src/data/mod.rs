//! Immutable shared samples and cached data-domain summaries; no UI state.

mod animation;
mod area;
mod bar_groups;
mod bars;
mod biplot;
mod bivariate;
mod bubble;
mod clouds;
mod clustergram;
mod clustering;
mod composite;
mod confusion;
mod contour;
mod control;
mod curvilinear;
mod delaunay;
mod distribution;
mod dsp;
mod ecdf;
mod error_bars;
mod extents;
mod field2d;
mod financial;
mod flow;
mod function;
mod geo;
mod geoglobe;
mod graph;
mod grid;
mod hist_legacy;
mod histogram;
mod image;
mod index;
mod modern;
mod multivariate;
mod patch;
mod pie;
mod plot3d;
mod points;
mod polar;
mod polygon;
mod polyshape;
mod probplot;
mod quiver;
mod shapes;
mod sky;
mod smith;
mod spc;
mod spy;
mod stream;
mod stream_indexed;
mod triangulate;
mod waveform;
mod zplane;

pub use animation::{AnimatedLine2D, CometPlayback};
pub use area::{AreaBandData, AreaError, AreaStackMode};
pub use bar_groups::BarSeriesError;
pub use bars::{Bar, BarData, BarOrientation};
pub use biplot::BiplotData;
pub use bivariate::{
    BinCell, BinScatterData, BinShape, BivariateError, BivariateNormalization, Histogram2Data,
};
pub use bubble::{BubbleData, BubbleError, BubblePoint, SwarmData, SwarmPoint};
pub use clouds::{BubbleCloudData, CloudError, PackedBubble, PlacedWord, WordCloudData};
pub use clustergram::{ClusterStandardize, ClustergramData};
pub use clustering::{DendrogramBranch, DendrogramData, LinkageMethod, LinkageStep};
pub use composite::{
    CompositeError, MarginalKind, ParetoData, PlotMatrixData, ScatterHistogramData,
};
pub use confusion::{ConfusionError, ConfusionMatrixData, ConfusionNormalization};
pub use contour::{
    ContourBand, ContourData, ContourError, ContourLevels, ContourLine, FilledContourData,
};
pub use control::{
    ControlError, FrequencyResponseData, RootLocusData, StabilityMargins, TransferFunction,
    poly_eval_complex, poly_roots_aberth,
};
pub use curvilinear::CurvilinearField2D;
pub use delaunay::{AlphaShapeData, TriMesh2D, TriMeshError, VoronoiData};
pub use distribution::{
    BoxChartData, BoxSummary, RaincloudData, RaincloudEntry, StatError, ViolinData, ViolinEntry,
    ViolinHalf, ViolinOptions, ViolinScale,
};
pub use dsp::{
    CwtData, DspError, PspectrumData, SpectrogramData, WindowKind, fft_arbitrary, fft_radix2,
    fft_real,
};
pub use ecdf::{EcdfData, EcdfKind};
pub use error_bars::{ErrorBar, ErrorBarData};
pub use field2d::{FunctionContourError, ScalarField2D};
pub use financial::{
    FinancialError, OhlcBar, OhlcData, Waterfall2DData, WaterfallColumn, WaterfallKind,
};
pub use flow::{FeatherData, FlowError, StreamOptions, StreamlineData, VectorField2D};
pub use function::{
    FunctionCurveData, FunctionPlotError, FunctionPlotOptions, ImplicitCurveData, ImplicitOptions,
};
pub(crate) use geo::coastlines::OFFLINE_LAND_RINGS;
pub use geo::{
    GeoAttribution, GeoBasemapStyle, GeoBubbleData, GeoBubblePoint, GeoDensityData, GeoDensityGrid,
    GeoOfflinePolicy, GeoPathData, GeoRadiusMode, GeoRasterTile, GeoScatterData, GeoScatterPoint,
    GeoTileCache, GeoTileKey, visible_tiles_for_bounds,
};
pub use geoglobe::{GeoGlobeData, GlobeMarker3D, GlobeTrack3D};
pub use graph::{
    EliminationTree, GraphData, GraphEdge, GraphError, GraphLayout, GraphLayoutKind, TreeData,
};
pub use grid::{GridData, GridError};
pub use hist_legacy::LegacyHistData;
pub use histogram::{HistogramData, HistogramError, HistogramNormalization};
pub use image::{ImageData, ImageError};
pub use index::LineIndexError;
pub(crate) use index::{BLOCK_SIZE, LineIndex, QueryStats, project_axis_raw, screen_aabb};
pub use modern::{
    GanttData, GanttTask, ModernError, RadarData, RadarSeries, SankeyData, SankeyLink, SankeyNode,
    SankeyRibbon, SunburstData, SunburstInput, SunburstSector, TreemapData, TreemapRect,
};
pub use multivariate::{
    CategoricalHeatmapData, MultivariateError, ParallelData, ParallelNormalization, SortOrder,
    TableAggregation,
};
pub use patch::{PatchData, PatchError};
pub use pie::{PieData, PieError, PieSlice};
pub(crate) use plot3d::union_bounds3;
pub use plot3d::{
    AlphaShape3DData, AnimatedLine3D, Bar3DBox, Bar3DData, Bar3DMode, Bar3DOrientation,
    Bubble3DData, Bubble3DPoint, Comet3DPlayback, Cone3DItem, ConePlot3DData, Contour3DData,
    Contour3DLine, Graph3DData, GraphLayout3D, ImplicitSurface3DData, IsocapThreshold,
    Isocaps3DData, Isosurface3DData, ParametricCurve3DData, Patch3DData, Pie3DData, Pie3DSlice,
    Plot3DError, Point3DData, PointCloud3DData, Polygon3D, Polygon3DData, Quiver3DArrow,
    Quiver3DData, Ribbon3DData, ScalarVolume3D, SlicePlane3D, Stem3DData, Stream3DOptions,
    StreamParticles3DData, StreamParticlesPlayback, StreamRibbon3DData, StreamTube3DData,
    Streamline3DData, Streamline3DLine, Surface3DData, Swarm3DData, Swarm3DPoint, TetraMesh3DData,
    TransferStop, TriMesh3DData, VectorField3D, VolumeRenderMode, VolumeSliceData,
    VolumeTransferFunction,
};
pub use points::{AUTO_INDEX_MIN_POINTS, PlotData, PlotPoint};
pub use polar::{
    CompassData, PolarBubbleData, PolarBubblePoint, PolarCurveData, PolarError, PolarFunctionData,
    PolarHistNormalization, PolarHistogramData, RoseData,
};
pub use polygon::PolygonData;
pub use polyshape::{PolyRegion, PolyShapeData};
pub use probplot::{ProbabilityDist, ProbabilityPlotData};
pub use quiver::{QuiverArrow, QuiverData};
pub use shapes::{DataRect, RectData};
pub use sky::{SkyError, SkyPlotData, SkySatellite, SkyTrack};
pub use smith::{SmithError, SmithPlotData, SmithTrace};
pub use spc::{ControlChartData, SpcChartKind, SpcPoint, SpcViolationKind};
pub use spy::{SpyData, SpyError};
pub use stream::{AppendReport, PlotStream, StreamError};
pub use stream_indexed::IndexedStream;
pub(crate) use triangulate::triangulate_indexed_polygon;
pub use waveform::{UniformWaveform, WaveformError};
pub use zplane::{ComplexRoot, PoleZeroData};
