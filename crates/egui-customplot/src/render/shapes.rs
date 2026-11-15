use egui::{Color32, Painter};

use super::mesh::{MeshBatch, paint_ring_border};
use crate::{PlotDiagnostics, PlotTransform, Rectangles, axis::valid_extent};

pub(super) fn paint_rectangles(
    painter: &Painter,
    rects: Rectangles<'_>,
    transform: PlotTransform,
    color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let view = transform.bounds();
    let vx_lo = view.min()[0].min(view.max()[0]);
    let vx_hi = view.min()[0].max(view.max()[0]);
    let vy_lo = view.min()[1].min(view.max()[1]);
    let vy_hi = view.min()[1].max(view.max()[1]);
    let mut batch = MeshBatch::default();
    for &rect in rects.data.rects() {
        diagnostics.patches.visited_items += 1;
        let Some((lo, hi)) = rect.extent() else {
            continue;
        };
        if !valid_extent(lo, hi, transform) {
            continue;
        }
        if hi[0] < vx_lo || lo[0] > vx_hi || hi[1] < vy_lo || lo[1] > vy_hi {
            continue;
        }
        let outline = rect.outline();
        if outline.len() < 3 {
            continue;
        }
        if color != Color32::TRANSPARENT {
            let n0 = transform.normalized(outline[0]);
            for k in 1..outline.len() - 1 {
                let tri = [
                    n0,
                    transform.normalized(outline[k]),
                    transform.normalized(outline[k + 1]),
                ];
                batch.add_flat_triangle(painter, tri, color, transform);
            }
        }
        paint_ring_border(
            painter,
            &outline,
            true,
            rects.border,
            transform,
            diagnostics,
        );
    }
    let stats = batch.finish(painter);
    diagnostics.patches.output_meshes += stats.meshes;
    diagnostics.patches.output_triangles += stats.triangles;
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}
