//! Plot allocation/margins and egui legend layout, not data geometry.

mod bubble_legend;
mod color_bar;
mod figure;
mod legend;
mod plot;
mod plot3d;
mod polar;

pub use bubble_legend::{BubbleLegend, BubbleLegendResponse};
pub(crate) use bubble_legend::{bubble_legend_size, bubble_pixel_radius, paint_bubble_legend};
pub(crate) use color_bar::allocate_color_bar;
pub use figure::FigureError;
pub(crate) use figure::FigureLayout;
pub use legend::LegendEntry;
pub(crate) use legend::{legend_rect, show_legend};
pub(crate) use plot::{allocate_plot, data_rect};
pub(crate) use plot3d::compute_plot3d_layout;
pub(crate) use polar::compute_polar_layout;
