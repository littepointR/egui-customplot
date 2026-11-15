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

pub use axis::{Axis, AxisScale, AxisTick, BoundsError, PlotBounds, PlotTransform, YAxis};
pub use data::{
    AlphaShapeData, AnimatedLine2D, AppendReport, AreaBandData, AreaError, AreaStackMode, Bar,
    BarData, BarOrientation, CometPlayback, ContourBand, ContourData, ContourError, ContourLevels,
    ContourLine, DataRect, ErrorBar, ErrorBarData, FeatherData, FilledContourData, FlowError,
    FunctionContourError, FunctionCurveData, FunctionPlotError, FunctionPlotOptions, GridData,
    GridError, HistogramData, HistogramError, HistogramNormalization, ImageData, ImageError,
    ImplicitCurveData, ImplicitOptions, LineIndexError, PatchData, PatchError, PlotData, PlotPoint,
    PlotStream, PolyRegion, PolyShapeData, PolygonData, QuiverArrow, QuiverData, RectData,
    ScalarField2D, SpyData, SpyError, StreamError, StreamOptions, StreamlineData, TriMesh2D,
    TriMeshError, VectorField2D, VoronoiData,
};
pub use diagnostics::{
    ContourDiagnostics, FlowDiagnostics, GeometryDiagnostics, ImageDiagnostics, PatchDiagnostics,
    PlotDiagnostics,
};
pub use interaction::{
    AxisLink, HoverInfo, HoveredBar, HoveredCell, HoveredPoint, HoveredRegion, HoveredVector,
    LinkError, LinkReport, PlotState, RegionKind, VectorAnchor,
};
pub use layout::{FigureError, LegendEntry};
pub use model::{
    AlphaShape, AnimatedLine, Annotation, AnnotationCoord, Area, Bars, ColorMap, ColorScale,
    ColorScaleError, Comet, Contours, ErrorBars, Feather, Fill, FilledContours, FunctionPlot,
    Heatmap, ImplicitCurve, Line, LineInterpolation, LinePattern, LineSampling, MarkerShape,
    MarkerStyle, Patch, PatchColoring, Pcolor, PcolorShading, PlotImage, PlotLayer, PlotStyle,
    PlotTheme, PolyShape, Quiver, Rectangles, Scatter, Series, SeriesError, SeriesStyle, Spy,
    StemOrientation, Stems, Streamlines, TriPlot, Voronoi,
};
pub use render::PlotCache;
#[cfg(feature = "export")]
pub use render::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
pub use widget::{
    ColorBar, ColorBarResponse, Figure, FigureCell, FigureResponse, Plot, PlotResponse,
};
