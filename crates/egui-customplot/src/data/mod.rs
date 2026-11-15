//! Immutable shared samples and cached data-domain summaries; no UI state.

mod bars;
mod error_bars;
mod extents;
mod grid;
mod histogram;
mod index;
mod points;
mod stream;

pub use bars::{Bar, BarData, BarOrientation};
pub use error_bars::{ErrorBar, ErrorBarData};
pub use grid::{GridData, GridError};
pub use histogram::{HistogramData, HistogramError, HistogramNormalization};
pub use index::LineIndexError;
pub(crate) use index::QueryStats;
pub use points::{PlotData, PlotPoint};
pub use stream::{AppendReport, PlotStream, StreamError};
