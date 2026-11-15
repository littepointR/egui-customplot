use egui::{Color32, Mesh, Painter, Pos2, Shape, Stroke, epaint};

use crate::{
    PlotDiagnostics, PlotPoint, PlotTransform,
    axis::{ClipVertex, clip_normalized_triangle, project_boundary_segment},
};

const TRIANGLES_PER_MESH: usize = 16_384;

#[derive(Default)]
pub(super) struct MeshStats {
    pub meshes: usize,
    pub triangles: usize,
    pub bytes: usize,
    pub numeric_rejections: usize,
}

#[derive(Default)]
pub(super) struct MeshBatch {
    mesh: Mesh,
    tri_count: usize,
    stats: MeshStats,
}

impl MeshBatch {
    pub fn add_flat_triangle(
        &mut self,
        painter: &Painter,
        verts: [[f64; 2]; 3],
        color: Color32,
        transform: PlotTransform,
    ) {
        if color == Color32::TRANSPARENT {
            return;
        }
        let tri = verts.map(|norm| ClipVertex::from_color(norm, color));
        self.clip_and_push(painter, tri, Some(color), transform);
    }

    pub fn add_gouraud_triangle(
        &mut self,
        painter: &Painter,
        tri: [ClipVertex; 3],
        transform: PlotTransform,
    ) {
        let uniform =
            (tri[0].rgba == tri[1].rgba && tri[1].rgba == tri[2].rgba).then(|| tri[0].color());
        if uniform == Some(Color32::TRANSPARENT) {
            return;
        }
        self.clip_and_push(painter, tri, uniform, transform);
    }

    fn clip_and_push(
        &mut self,
        painter: &Painter,
        tri: [ClipVertex; 3],
        uniform: Option<Color32>,
        transform: PlotTransform,
    ) {
        let rect = transform.rect();
        let span = [f64::from(rect.width()), f64::from(rect.height())];
        let Ok(poly) = clip_normalized_triangle(tri, span) else {
            self.stats.numeric_rejections += 1;
            return;
        };
        if poly.len() < 3 {
            return;
        }
        let mut projected = Vec::with_capacity(poly.len());
        for v in poly {
            let pos = transform.position_from_normalized(v.norm);
            if !pos.is_finite() {
                self.stats.numeric_rejections += 1;
                return;
            }
            let color = uniform.unwrap_or_else(|| v.color());
            projected.push(epaint::Vertex {
                pos,
                uv: epaint::WHITE_UV,
                color,
            });
        }
        let base = self.mesh.vertices.len() as u32;
        let tris = projected.len() - 2;
        self.mesh.vertices.extend(projected);
        for k in 1..=tris {
            self.mesh.indices.push(base);
            self.mesh.indices.push(base + k as u32);
            self.mesh.indices.push(base + k as u32 + 1);
        }
        self.tri_count += tris;
        self.stats.triangles += tris;
        if self.tri_count >= TRIANGLES_PER_MESH {
            self.flush(painter);
        }
    }

    fn flush(&mut self, painter: &Painter) {
        if self.mesh.is_empty() {
            return;
        }
        self.stats.bytes += self.mesh.vertices.capacity() * std::mem::size_of::<epaint::Vertex>()
            + self.mesh.indices.capacity() * std::mem::size_of::<u32>();
        self.stats.meshes += 1;
        painter.add(Shape::mesh(std::mem::take(&mut self.mesh)));
        self.tri_count = 0;
    }

    pub fn finish(mut self, painter: &Painter) -> MeshStats {
        self.flush(painter);
        self.stats
    }
}

pub(super) fn stroke_valid(stroke: Stroke) -> bool {
    stroke.width.is_finite() && stroke.width > 0.0 && stroke.color != Color32::TRANSPARENT
}

pub(super) fn paint_ring_border(
    painter: &Painter,
    points: &[PlotPoint],
    closed: bool,
    stroke: Stroke,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    if !stroke_valid(stroke) || points.len() < 2 {
        return;
    }
    let edge_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    let mut path: Vec<Pos2> = Vec::new();
    for i in 0..edge_count {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        match project_boundary_segment(a, b, transform) {
            Ok(Some([pa, pb])) => {
                if path.last().is_some_and(|prev| *prev == pa) {
                    path.push(pb);
                } else {
                    flush_path(painter, &mut path, stroke, diagnostics);
                    path.push(pa);
                    path.push(pb);
                }
            }
            Ok(None) => flush_path(painter, &mut path, stroke, diagnostics),
            Err(()) => {
                diagnostics.numeric_rejections += 1;
                flush_path(painter, &mut path, stroke, diagnostics);
            }
        }
    }
    flush_path(painter, &mut path, stroke, diagnostics);
}

fn flush_path(
    painter: &Painter,
    path: &mut Vec<Pos2>,
    stroke: Stroke,
    diagnostics: &mut PlotDiagnostics,
) {
    if path.len() >= 2 {
        diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        diagnostics.patches.output_strokes += 1;
        painter.add(Shape::line(std::mem::take(path), stroke));
    } else {
        path.clear();
    }
}
