//! Immutable shared samples and cached data-domain summaries; no UI state.

mod animation;
mod area;
mod bars;
mod contour;
mod delaunay;
mod error_bars;
mod extents;
mod field2d;
mod flow;
mod function;
mod grid;
mod histogram;
mod image;
mod index;
mod patch;
mod points;
mod polygon;
mod polyshape;
mod quiver;
mod shapes;
mod spy;
mod stream;
mod triangulate;

pub use animation::{AnimatedLine2D, CometPlayback};
pub use area::{AreaBandData, AreaError, AreaStackMode};
pub use bars::{Bar, BarData, BarOrientation};
pub use contour::{
    ContourBand, ContourData, ContourError, ContourLevels, ContourLine, FilledContourData,
};
pub use delaunay::{AlphaShapeData, TriMesh2D, TriMeshError, VoronoiData};
pub use error_bars::{ErrorBar, ErrorBarData};
pub use field2d::{FunctionContourError, ScalarField2D};
pub use flow::{FeatherData, FlowError, StreamOptions, StreamlineData, VectorField2D};
pub use function::{
    FunctionCurveData, FunctionPlotError, FunctionPlotOptions, ImplicitCurveData, ImplicitOptions,
};
pub use grid::{GridData, GridError};
pub use histogram::{HistogramData, HistogramError, HistogramNormalization};
pub use image::{ImageData, ImageError};
pub use index::LineIndexError;
pub(crate) use index::QueryStats;
pub use patch::{PatchData, PatchError};
pub use points::{PlotData, PlotPoint};
pub use polygon::PolygonData;
pub use polyshape::{PolyRegion, PolyShapeData};
pub use quiver::{QuiverArrow, QuiverData};
pub use shapes::{DataRect, RectData};
pub use spy::{SpyData, SpyError};
pub use stream::{AppendReport, PlotStream, StreamError};
