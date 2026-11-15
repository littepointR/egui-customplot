//! Coordinate domains, validated bounds, transforms and tick configuration.

mod aspect;
mod bounds;
mod grid;
mod linked;
mod rect;
mod scale;
mod ticks;
mod transform;
mod y_axis;

pub use bounds::{BoundsError, PlotBounds};
pub(crate) use grid::{grid_cell_at, grid_cell_rect, visible_grid};
pub(crate) use linked::apply_axis_ranges;
pub(crate) use rect::{clipped_axis_segment, clipped_data_rect, rect_edges, valid_extent};
pub use scale::AxisScale;
pub use ticks::{Axis, AxisTick};
pub use transform::PlotTransform;
pub use y_axis::YAxis;
pub(crate) use y_axis::{SeriesTransforms, secondary_auto};
