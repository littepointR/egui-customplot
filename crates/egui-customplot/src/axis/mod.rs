//! Coordinate domains, validated bounds, transforms and tick configuration.

mod aspect;
mod bounds;
mod geo;
mod geoglobe;
mod grid;
mod linked;
mod plot3d;
mod polar;
mod polygon;
mod probability;
mod rect;
mod scale;
mod segment;
mod sky;
mod smith;
mod ticks;
mod time;
mod transform;
mod y_axis;

pub use bounds::{BoundsError, PlotBounds};
pub use geo::{
    GeoBounds, GeoError, GeoPoint, GeoProjection, GeoTransform, haversine_km, shortest_dlon,
    wrap_lon_180,
};
pub(crate) use geo::{format_lat, format_lon, geo_graticule_ticks, project_geo_polyline};
pub use geoglobe::GeoGlobeTransform;
pub(crate) use grid::{edge_cell_at, grid_cell_at, grid_cell_rect, visible_edges, visible_grid};
pub(crate) use linked::apply_axis_ranges;
pub(crate) use plot3d::edge_geometry;
pub use plot3d::{
    Axis3D, AxisDecoration3D, AxisEdge3D, AxisLabel3D, AxisLayoutContext3D, Bounds3D, Camera3D,
    CameraLens3D, DataAspect3D, Point3D, Projection3D, ScaleMapping3D, Transform3D,
};
pub(crate) use plot3d::{
    ClipVertex3D, clip_gouraud_tri_unit_cube, clip_segment_unit_cube, clip_triangle_unit_cube,
    intersect_plane_unit_cube,
};
pub use polar::{NegativeRadiusMode, PolarTransform, ThetaDirection, ThetaZeroLocation};
pub(crate) use polygon::{
    ClipVertex, clip_normalized_triangle, point_in_ring_transformed, project_boundary_segment,
};
pub use probability::{norm_cdf, norm_inv};
pub(crate) use probability::{
    norm_cdf_clamped, valid_probability, weibull_forward, weibull_inverse,
};
pub(crate) use rect::{clipped_axis_segment, clipped_data_rect, rect_edges, valid_extent};
pub use scale::AxisScale;
pub(crate) use segment::{
    clip_normalized_segment, graph_edge_screen_polyline, point_polyline_distance,
    project_vector_segment,
};
pub use sky::SkyTransform;
pub use smith::{SmithGridKind, SmithTransform};
pub(crate) use ticks::format_tick;
pub use ticks::{Axis, AxisTick};
pub use time::{
    CivilDateTime, TimeAxisMode, civil_to_unix_secs, format_duration_hms, unix_secs_to_civil,
};
pub use transform::PlotTransform;
pub use y_axis::YAxis;
pub(crate) use y_axis::{SeriesTransforms, secondary_auto};
