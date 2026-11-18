//! Plot allocation/margins and egui legend layout, not data geometry.

mod axis_labels_3d;
mod bubble_legend;
mod color_bar;
mod decorations;
pub use decorations::{DecorationContent, PlotDecoration};
pub(crate) use decorations::{DecorationSlots, show_decorations};
mod figure;
mod geo;
mod legend;
mod legend_component;
pub use legend_component::LegendComponent;
pub(crate) use legend_component::reserve_component;
mod plot;
mod plot3d;
pub(crate) use axis_labels_3d::axis_labels;
mod polar;

pub use bubble_legend::{BubbleLegend, BubbleLegendResponse};
pub(crate) use bubble_legend::{bubble_legend_size, bubble_pixel_radius, paint_bubble_legend};
pub(crate) use color_bar::{allocate_color_bar, reserve_color_bar};
pub use figure::FigureError;
pub(crate) use figure::FigureLayout;
pub(crate) use geo::{compute_geo_layout, geo_legend_names};
pub use legend::{LegendEntry, LegendPosition};
pub(crate) use legend::{legend_item_button, legend_rect, reserve_legend, show_legend};
pub(crate) use plot::{allocate_plot, data_rect};
pub(crate) use plot3d::{compute_auto_bounds_3d, compute_plot3d_layout};
pub(crate) use polar::compute_polar_layout;
