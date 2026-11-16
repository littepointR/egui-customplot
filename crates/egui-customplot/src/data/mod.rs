//! Immutable shared samples and cached data-domain summaries; no UI state.

mod animation;
mod area;
mod bar_groups;
mod bars;
mod bivariate;
mod bubble;
mod clouds;
mod composite;
mod contour;
mod delaunay;
mod distribution;
mod error_bars;
mod extents;
mod field2d;
mod flow;
mod function;
mod graph;
mod grid;
mod hist_legacy;
mod histogram;
mod image;
mod index;
mod multivariate;
mod patch;
mod pie;
mod plot3d;
mod points;
mod polar;
mod polygon;
mod polyshape;
mod quiver;
mod shapes;
mod spy;
mod stream;
mod triangulate;

pub use animation::{AnimatedLine2D, CometPlayback};
pub use area::{AreaBandData, AreaError, AreaStackMode};
pub use bar_groups::BarSeriesError;
pub use bars::{Bar, BarData, BarOrientation};
pub use bivariate::{
    BinCell, BinScatterData, BinShape, BivariateError, BivariateNormalization, Histogram2Data,
};
pub use bubble::{BubbleData, BubbleError, BubblePoint, SwarmData, SwarmPoint};
pub use clouds::{BubbleCloudData, CloudError, PackedBubble, PlacedWord, WordCloudData};
pub use composite::{
    CompositeError, MarginalKind, ParetoData, PlotMatrixData, ScatterHistogramData,
};
pub use contour::{
    ContourBand, ContourData, ContourError, ContourLevels, ContourLine, FilledContourData,
};
pub use delaunay::{AlphaShapeData, TriMesh2D, TriMeshError, VoronoiData};
pub use distribution::{
    BoxChartData, BoxSummary, RaincloudData, RaincloudEntry, StatError, ViolinData, ViolinEntry,
    ViolinHalf, ViolinOptions, ViolinScale,
};
pub use error_bars::{ErrorBar, ErrorBarData};
pub use field2d::{FunctionContourError, ScalarField2D};
pub use flow::{FeatherData, FlowError, StreamOptions, StreamlineData, VectorField2D};
pub use function::{
    FunctionCurveData, FunctionPlotError, FunctionPlotOptions, ImplicitCurveData, ImplicitOptions,
};
pub use graph::{
    EliminationTree, GraphData, GraphEdge, GraphError, GraphLayout, GraphLayoutKind, TreeData,
};
pub use grid::{GridData, GridError};
pub use hist_legacy::LegacyHistData;
pub use histogram::{HistogramData, HistogramError, HistogramNormalization};
pub use image::{ImageData, ImageError};
pub use index::LineIndexError;
pub(crate) use index::QueryStats;
pub use multivariate::{
    CategoricalHeatmapData, MultivariateError, ParallelData, ParallelNormalization, SortOrder,
    TableAggregation,
};
pub use patch::{PatchData, PatchError};
pub use pie::{PieData, PieError, PieSlice};
pub(crate) use plot3d::union_bounds3;
pub use plot3d::{
    AnimatedLine3D, Bar3DBox, Bar3DData, Bar3DMode, Bar3DOrientation, Bubble3DData, Bubble3DPoint,
    Comet3DPlayback, Cone3DItem, ConePlot3DData, Contour3DData, Contour3DLine,
    ImplicitSurface3DData, IsocapThreshold, Isocaps3DData, Isosurface3DData, ParametricCurve3DData,
    Pie3DData, Pie3DSlice, Plot3DError, Point3DData, Polygon3D, Polygon3DData, Quiver3DArrow,
    Quiver3DData, Ribbon3DData, ScalarVolume3D, SlicePlane3D, Stem3DData, Stream3DOptions,
    StreamParticles3DData, StreamParticlesPlayback, StreamRibbon3DData, StreamTube3DData,
    Streamline3DData, Streamline3DLine, Surface3DData, Swarm3DData, Swarm3DPoint, TetraMesh3DData,
    TriMesh3DData, VectorField3D, VolumeSliceData,
};
pub use points::{PlotData, PlotPoint};
pub use polar::{
    CompassData, PolarBubbleData, PolarBubblePoint, PolarCurveData, PolarError, PolarFunctionData,
    PolarHistNormalization, PolarHistogramData, RoseData,
};
pub use polygon::PolygonData;
pub use polyshape::{PolyRegion, PolyShapeData};
pub use quiver::{QuiverArrow, QuiverData};
pub use shapes::{DataRect, RectData};
pub use spy::{SpyData, SpyError};
pub use stream::{AppendReport, PlotStream, StreamError};
