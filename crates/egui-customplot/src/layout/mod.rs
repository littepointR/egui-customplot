//! Plot allocation/margins and egui legend layout, not data geometry.

mod legend;
mod plot;

pub use legend::LegendEntry;
pub(crate) use legend::{legend_rect, show_legend};
pub(crate) use plot::{allocate_plot, data_rect};
