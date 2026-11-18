//! Capture the current drawing or replay a host callback at a chosen size and density.

mod capture;
mod options;
mod png;
mod raster;
mod target;
pub(crate) use target::export_density;
mod svg;
mod vector;
mod writer;

pub(crate) use capture::is_capturing_snapshot;
pub use capture::{CaptureLimits, PlotSnapshot};
pub use options::{ExportError, ExportOptions};
