//! CPU geometry preparation and ordered egui Painter submission.

mod animation;
mod annotation;
mod area;
mod axes;
mod bars;
mod bubble_bivariate;
mod cache;
mod color_bar;
mod contour;
mod distribution;
mod error_bars;
#[cfg(feature = "export")]
mod export;
mod flow;
mod graph;
mod heatmap;
mod image;
mod line;
mod line_query;
mod markers;
mod mesh;
mod multivariate;
mod overlay;
mod patch;
mod pattern;
mod pcolor;
mod pie_clouds;
mod plot3d;
mod polar;
mod polygon;
mod quiver;
mod right_axis;
mod series;
mod shapes;
mod spy;
mod stage8c;
mod stems;
mod triangulation;

pub(crate) use annotation::{paint_annotations, paint_figure_annotations};
pub(crate) use axes::{AxesStyle, paint_axes};
pub use cache::PlotCache;
pub(crate) use color_bar::paint_color_bar;
#[cfg(feature = "export")]
pub use export::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
pub(crate) use overlay::{paint_bar_hover, paint_cell_hover, paint_hover, paint_region_hover};
pub(crate) use plot3d::paint_plot3d;
pub(crate) use polar::paint_polar;
pub(crate) use right_axis::paint_right_axis;
pub(crate) use series::paint_series;
