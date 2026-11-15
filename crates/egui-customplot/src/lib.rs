//! Native egui plotting, with persistent data and explicitly owned view state.
//!
//! Supports mixed lines/markers/scatter, linear/log axes, legends and raw-point
//! hover. Data is scanned once for bounds, but rendering still visits all samples:
//! there is no LOD, geometry cache, GPU callback, or performance guarantee yet.
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

mod axis;
mod data;
mod diagnostics;
mod interaction;
mod layout;
mod model;
mod render;
mod widget;

pub use axis::{Axis, AxisScale, AxisTick, BoundsError, PlotBounds, PlotTransform};
pub use data::{PlotData, PlotPoint};
pub use diagnostics::PlotDiagnostics;
pub use interaction::{HoveredPoint, PlotState};
pub use layout::LegendEntry;
pub use model::{Line, LinePattern, MarkerShape, MarkerStyle, Scatter, Series, SeriesError};
pub use widget::{Plot, PlotResponse};
