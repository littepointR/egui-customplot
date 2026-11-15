//! Plot allocation/margins and egui legend layout, not data geometry.

mod color_bar;
mod figure;
mod legend;
mod plot;

pub(crate) use color_bar::allocate_color_bar;
pub use figure::FigureError;
pub(crate) use figure::FigureLayout;
pub use legend::LegendEntry;
pub(crate) use legend::{legend_rect, show_legend};
pub(crate) use plot::{allocate_plot, data_rect};
