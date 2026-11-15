//! Borrowed series descriptors and typed styles. Samples live in `data`.

mod line;
mod line_pattern;
mod marker;
mod scatter;
mod series;

pub use line::Line;
pub use line_pattern::LinePattern;
pub use marker::{MarkerShape, MarkerStyle};
pub use scatter::{Scatter, SeriesError};
pub use series::Series;
