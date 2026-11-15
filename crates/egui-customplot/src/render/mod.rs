//! CPU geometry preparation and ordered egui Painter submission.

mod annotation;
mod axes;
mod bars;
mod cache;
mod color_bar;
mod error_bars;
#[cfg(feature = "export")]
mod export;
mod heatmap;
mod line;
mod line_query;
mod markers;
mod overlay;
mod pattern;
mod quiver;
mod right_axis;
mod series;
mod stems;

pub(crate) use annotation::paint_annotations;
pub(crate) use axes::{AxesStyle, paint_axes};
pub use cache::PlotCache;
pub(crate) use color_bar::paint_color_bar;
#[cfg(feature = "export")]
pub use export::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
pub(crate) use overlay::{paint_bar_hover, paint_cell_hover, paint_hover};
pub(crate) use right_axis::paint_right_axis;
pub(crate) use series::paint_series;
