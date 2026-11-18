use super::{
    layers::{collect_polyline, push_marker_data},
    scene::Scene3D,
};
use crate::{CustomLayer3D, Plot3DDiagnostics, Transform3D, model::ResolvedSeriesStyle};

pub(super) fn collect(
    scene: &mut Scene3D,
    layer: &dyn CustomLayer3D,
    resolved: ResolvedSeriesStyle,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    layer.batches(transform, &mut |batch| match batch {
        crate::GeometryBatch3D::Polyline(points) => {
            collect_polyline(scene, points, resolved.stroke, None, transform, diagnostics)
        }
        crate::GeometryBatch3D::Points(points) => {
            diagnostics.visited_items += points.len();
            for &p in points {
                push_marker_data(
                    scene,
                    p,
                    resolved.marker.radius,
                    resolved.stroke.color,
                    resolved.marker.shape,
                    transform,
                );
            }
        }
        crate::GeometryBatch3D::Triangles(tris) => {
            diagnostics.visited_items += tris.len();
            for &tri in tris {
                scene.push_shaded_tri_data(tri, resolved.stroke.color, transform);
            }
        }
    });
}
