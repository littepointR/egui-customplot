use egui::{Color32, Painter};

use super::mesh::{MeshBatch, paint_ring_border, stroke_valid};
use crate::{Patch, PatchColoring, PlotDiagnostics, PlotPoint, PlotTransform, axis::ClipVertex};

pub(super) fn paint_patch(
    painter: &Painter,
    patch: Patch<'_>,
    transform: PlotTransform,
    default_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let verts = patch.data.vertices();
    let scales = transform.scales();
    let mut batch = MeshBatch::default();
    let draw_border = stroke_valid(patch.border);
    let mut edge_pts: Vec<PlotPoint> = Vec::new();
    for face in 0..patch.data.face_count() {
        diagnostics.patches.visited_items += 1;
        let tris = patch.data.face_triangles(face, scales);
        if tris.is_empty() {
            continue;
        }
        let flat_color = match patch.coloring {
            PatchColoring::Uniform => Some(default_color),
            PatchColoring::FaceColors(colors) => Some(colors[face]),
            PatchColoring::FaceScalars { values, scale } => scale.color(values[face]),
            PatchColoring::VertexColors(_) => None,
            PatchColoring::VertexScalars { values, scale } => {
                if patch
                    .data
                    .face(face)
                    .iter()
                    .any(|&idx| scale.color(values[idx]).is_none())
                {
                    continue;
                }
                None
            }
        };
        let mut drew_face = true;
        if let Some(color) = flat_color {
            for &[i0, i1, i2] in tris {
                let norm = [
                    transform.normalized(verts[i0]),
                    transform.normalized(verts[i1]),
                    transform.normalized(verts[i2]),
                ];
                batch.add_flat_triangle(painter, norm, color, transform);
            }
        } else {
            for &[i0, i1, i2] in tris {
                let c0 = vertex_color(patch.coloring, i0);
                let c1 = vertex_color(patch.coloring, i1);
                let c2 = vertex_color(patch.coloring, i2);
                let (Some(c0), Some(c1), Some(c2)) = (c0, c1, c2) else {
                    drew_face = false;
                    continue;
                };
                let tri = [
                    ClipVertex::from_color(transform.normalized(verts[i0]), c0),
                    ClipVertex::from_color(transform.normalized(verts[i1]), c1),
                    ClipVertex::from_color(transform.normalized(verts[i2]), c2),
                ];
                batch.add_gouraud_triangle(painter, tri, transform);
            }
        }
        if draw_border && drew_face {
            edge_pts.clear();
            edge_pts.extend(patch.data.face(face).iter().map(|&idx| verts[idx]));
            paint_ring_border(
                painter,
                &edge_pts,
                true,
                patch.border,
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

fn vertex_color(coloring: PatchColoring<'_>, idx: usize) -> Option<Color32> {
    match coloring {
        PatchColoring::VertexColors(colors) => Some(colors[idx]),
        PatchColoring::VertexScalars { values, scale } => scale.color(values[idx]),
        _ => None,
    }
}
