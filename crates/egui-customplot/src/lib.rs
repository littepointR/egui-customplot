//! Native egui plotting, with persistent data and explicitly owned view state.
//!
//! This prototype implements linear 2D polylines only. Data is scanned once on
//! construction for bounds, but rendering still visits every segment each pass:
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
mod model;
mod render;
mod widget;

pub use axis::{BoundsError, PlotBounds, PlotTransform};
pub use model::{Line, PlotData, PlotPoint};
pub use widget::{Plot, PlotDiagnostics, PlotResponse, PlotState};
