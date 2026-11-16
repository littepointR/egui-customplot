//! Coordinate domains, validated bounds, transforms and tick configuration.

mod aspect;
mod bounds;
mod grid;
mod linked;
mod plot3d;
mod polar;
mod polygon;
mod rect;
mod scale;
mod segment;
mod ticks;
mod transform;
mod y_axis;

pub use bounds::{BoundsError, PlotBounds};
pub(crate) use grid::{edge_cell_at, grid_cell_at, grid_cell_rect, visible_edges, visible_grid};
pub(crate) use linked::apply_axis_ranges;
pub use plot3d::{Axis3D, Bounds3D, Camera3D, DataAspect3D, Point3D, Projection3D, Transform3D};
pub(crate) use plot3d::{
    clip_segment_unit_cube, clip_triangle_unit_cube, intersect_plane_unit_cube,
};
pub use polar::{NegativeRadiusMode, PolarTransform, ThetaDirection, ThetaZeroLocation};
pub(crate) use polygon::{
    ClipVertex, clip_normalized_triangle, point_in_ring_transformed, project_boundary_segment,
};
pub(crate) use rect::{clipped_axis_segment, clipped_data_rect, rect_edges, valid_extent};
pub use scale::AxisScale;
pub(crate) use segment::{
    clip_normalized_segment, graph_edge_screen_polyline, point_polyline_distance,
    project_vector_segment,
};
pub use ticks::{Axis, AxisTick};
pub use transform::PlotTransform;
pub use y_axis::YAxis;
pub(crate) use y_axis::{SeriesTransforms, secondary_auto};
