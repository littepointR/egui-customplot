//! Explicit snapshots of already-painted geometry; never a raw-data re-render.

mod capture;
mod options;
mod png;
mod raster;
mod svg;
mod vector;
mod writer;

pub(crate) use capture::is_capturing_snapshot;
pub use capture::{CaptureLimits, PlotSnapshot};
pub use options::{ExportError, ExportOptions};
