use egui::{Color32, Painter};

use super::mesh::{MeshBatch, paint_ring_border};
use crate::{Fill, PlotDiagnostics, PlotTransform, PolyShape};

pub(super) fn paint_fill(
    painter: &Painter,
    fill: Fill<'_>,
    transform: PlotTransform,
    color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let scales = transform.scales();
    let mut batch = MeshBatch::default();
    for i in 0..fill.data.ring_count() {
        diagnostics.patches.visited_items += 1;
        let Some(tris) = fill.data.ring_triangles(i, scales) else {
            continue;
        };
        let ring = fill.data.ring(i);
        if color != Color32::TRANSPARENT {
            for &[a, b, c] in tris {
                let tri = [
                    transform.normalized(ring[a]),
                    transform.normalized(ring[b]),
                    transform.normalized(ring[c]),
                ];
                batch.add_flat_triangle(painter, tri, color, transform);
            }
        }
        paint_ring_border(painter, ring, true, fill.border, transform, diagnostics);
    }
    let stats = batch.finish(painter);
    diagnostics.patches.output_meshes += stats.meshes;
    diagnostics.patches.output_triangles += stats.triangles;
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}

pub(super) fn paint_polyshape(
    painter: &Painter,
    poly: PolyShape<'_>,
    transform: PlotTransform,
    color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let scales = transform.scales();
    let mut batch = MeshBatch::default();
    for i in 0..poly.data.region_count() {
        diagnostics.patches.visited_items += 1;
        let Some(tris) = poly.data.region_triangles(i, scales) else {
            continue;
        };
        let verts = poly.data.region_vertices(i);
        if color != Color32::TRANSPARENT {
            for &[a, b, c] in tris {
                let tri = [
                    transform.normalized(verts[a]),
                    transform.normalized(verts[b]),
                    transform.normalized(verts[c]),
                ];
                batch.add_flat_triangle(painter, tri, color, transform);
            }
        }
        paint_ring_border(
            painter,
            poly.data.region_outer(i),
            true,
            poly.border,
            transform,
            diagnostics,
        );
        for h in 0..poly.data.region_hole_count(i) {
            paint_ring_border(
                painter,
                poly.data.region_hole(i, h),
                true,
                poly.border,
                transform,
                diagnostics,
            );
        }
    }
    let stats = batch.finish(painter);
    diagnostics.patches.output_meshes += stats.meshes;
    diagnostics.patches.output_triangles += stats.triangles;
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}
