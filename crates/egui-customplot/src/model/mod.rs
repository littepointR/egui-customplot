//! Borrowed series descriptors and typed styles. Samples live in `data`.

mod annotation;
mod line;
mod line_pattern;
mod marker;
mod scatter;
mod series;
mod style;
mod theme;

pub use annotation::Annotation;
pub use line::Line;
pub use line_pattern::LinePattern;
pub use marker::{MarkerShape, MarkerStyle};
pub use scatter::{Scatter, SeriesError};
pub use series::Series;
pub use style::{PlotStyle, SeriesStyle};
pub(crate) use style::{ResolvedPlotStyle, ResolvedSeriesStyle, resolve_plot_style};
pub use theme::PlotTheme;
