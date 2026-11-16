//! Borrowed series descriptors and typed styles. Samples live in `data`.

mod animation;
mod annotation;
mod area;
mod bars;
mod bubble_swarm;
mod color_scale;
mod contour;
mod distribution;
mod error_bars;
mod fast_series;
mod flow;
mod function;
mod geo;
mod graph;
mod heatmap;
mod image;
mod interpolation;
mod layer;
mod line;
mod line_pattern;
mod marker;
mod multivariate;
mod patch;
mod pcolor;
mod pie_clouds;
mod plot3d;
mod polar;
mod polar_region;
mod polygon;
mod polyshape;
mod quiver;
mod sampling;
mod scatter;
mod series;
mod shapes;
mod spy;
mod stems;
mod style;
mod summary;
mod theme;
mod triangulation;

pub use animation::{AnimatedLine, Comet};
pub use annotation::{Annotation, AnnotationCoord};
pub use area::Area;
pub(crate) use area::AreaSource;
pub use bars::Bars;
pub use bubble_swarm::{BinScatter, BubbleChart, Histogram2, SwarmChart};
pub use color_scale::{ColorMap, ColorScale, ColorScaleError};
pub use contour::{Contours, FilledContours};
pub use distribution::{BoxChart, Raincloud, ViolinPlot};
pub use error_bars::ErrorBars;
pub use fast_series::{StreamLine, WaveformLine};
pub use flow::{Feather, Streamlines};
pub use function::{FunctionPlot, ImplicitCurve};
pub(crate) use geo::summarize_geo;
pub use geo::{GeoBubbles, GeoDensity, GeoLayer, GeoLine, GeoScatter};
pub use graph::{GraphHighlight, GraphPlot};
pub use heatmap::Heatmap;
pub use image::PlotImage;
pub use interpolation::LineInterpolation;
pub use layer::PlotLayer;
pub use line::Line;
pub use line_pattern::LinePattern;
pub use marker::{MarkerShape, MarkerStyle};
pub use multivariate::{CategoricalHeatmap, ParallelPlot};
pub use patch::{Patch, PatchColoring};
pub use pcolor::{Pcolor, PcolorShading};
pub use pie_clouds::{BubbleCloud, PieChart, WordCloud};
pub(crate) use plot3d::summarize_plot3d;
pub use plot3d::{
    Bar3D, BoxStyle3D, Bubble3D, Comet3D, ConePlot3D, ConstantPlane3D, Contour3D, DepthMode,
    Fill3D, Isocaps3D, Isosurface3D, Layer3D, LightSource3D, Line3D, MeshSurface3D,
    MeshSurfaceStyle, Pie3D, Quiver3D, Ribbon3D, Scatter3D, Stem3D, StreamParticles3D,
    StreamRibbon3D, StreamTube3D, Streamline3D, Surface3D, SurfaceShading, Swarm3D, TetraMesh3D,
    Text3D, TriSurf3D, VolumeSlice3D,
};
pub use polar::{CompassPlot, PolarBubble, PolarHistogram, PolarLayer, PolarLine, PolarScatter};
pub use polar_region::{PolarRegionAnnotation, PolarRegionKind};
pub use polygon::Fill;
pub use polyshape::PolyShape;
pub use quiver::Quiver;
pub use sampling::{LineSampling, MarkerSampling, RenderBackend};
pub use scatter::{Scatter, SeriesError};
pub use series::Series;
pub use shapes::Rectangles;
pub use spy::Spy;
pub use stems::{StemOrientation, Stems};
pub use style::{PlotStyle, SeriesStyle};
pub(crate) use style::{ResolvedPlotStyle, ResolvedSeriesStyle, resolve_plot_style};
pub(crate) use summary::summarize_series;
pub use theme::PlotTheme;
pub use triangulation::{AlphaShape, TriPlot, Voronoi};
