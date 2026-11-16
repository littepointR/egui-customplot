use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape, Stroke, epaint};

use crate::{
    MarkerShape, Plot3DDiagnostics, Point3D, Transform3D,
    axis::{
        ClipVertex3D, clip_gouraud_tri_unit_cube, clip_segment_unit_cube, clip_triangle_unit_cube,
    },
};

#[derive(Clone, Copy)]
pub(super) struct Tri3D {
    pub pos: [Pos2; 3],
    pub depth: [f32; 3],
    pub colors: [Color32; 3],
    pub opaque: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Seg3D {
    pub p0: Pos2,
    pub p1: Pos2,
    pub depth: [f32; 2],
    pub stroke: Stroke,
}

#[derive(Clone, Copy)]
pub(super) struct Marker3D {
    pub pos: Pos2,
    pub depth: f32,
    pub radius: f32,
    pub color: Color32,
    pub shape: MarkerShape,
}

#[derive(Clone)]
pub(super) struct Label3D {
    pub pos: Pos2,
    pub text: String,
    pub color: Color32,
}

#[derive(Default)]
pub(super) struct Scene3D {
    pub tris: Vec<Tri3D>,
    pub segs: Vec<Seg3D>,
    pub markers: Vec<Marker3D>,
    pub labels: Vec<Label3D>,
    pub clipped: usize,
}

impl Scene3D {
    pub fn push_data_segment(
        &mut self,
        a: Point3D,
        b: Point3D,
        stroke: Stroke,
        transform: Transform3D,
    ) {
        if stroke.color == Color32::TRANSPARENT || stroke.width <= 0.0 {
            return;
        }
        let (Some(ua), Some(ub)) = (transform.data_to_unit(a), transform.data_to_unit(b)) else {
            return;
        };
        let Some((ca, cb)) = clip_segment_unit_cube(ua, ub) else {
            self.clipped += 1;
            return;
        };
        let (Some((pa, da)), Some((pb, db))) =
            (transform.project_unit(ca), transform.project_unit(cb))
        else {
            self.clipped += 1;
            return;
        };
        self.segs.push(Seg3D {
            p0: pa,
            p1: pb,
            depth: [da, db],
            stroke,
        });
    }

    pub fn push_shaded_tri_data(
        &mut self,
        verts: [Point3D; 3],
        base_color: Color32,
        transform: Transform3D,
    ) {
        let Some(u0) = transform.data_to_unit(verts[0]) else {
            return;
        };
        let Some(u1) = transform.data_to_unit(verts[1]) else {
            return;
        };
        let Some(u2) = transform.data_to_unit(verts[2]) else {
            return;
        };
        let w = [
            transform.unit_to_world(u0),
            transform.unit_to_world(u1),
            transform.unit_to_world(u2),
        ];
        let shade = face_shade(w[0], w[1], w[2]);
        let shaded = scale_rgb(base_color, shade);
        self.push_unit_tri([u0, u1, u2], shaded, transform);
    }

    pub fn push_unit_tri(&mut self, u: [[f64; 3]; 3], color: Color32, transform: Transform3D) {
        if color == Color32::TRANSPARENT {
            return;
        }
        let clipped = clip_triangle_unit_cube(u);
        if clipped.len() < 3 {
            self.clipped += 1;
            return;
        }
        let mut proj = Vec::with_capacity(clipped.len());
        for pt in clipped {
            let Some((p, d)) = transform.project_unit(pt) else {
                return;
            };
            proj.push((p, d));
        }
        let opaque = color.a() >= 250;
        for i in 1..(proj.len() - 1) {
            self.tris.push(Tri3D {
                pos: [proj[0].0, proj[i].0, proj[i + 1].0],
                depth: [proj[0].1, proj[i].1, proj[i + 1].1],
                colors: [color; 3],
                opaque,
            });
        }
    }

    pub fn push_gouraud_tri_data(
        &mut self,
        verts: [Point3D; 3],
        colors: [Color32; 3],
        depth_bias: f32,
        transform: Transform3D,
    ) {
        let Some(u0) = transform.data_to_unit(verts[0]) else {
            return;
        };
        let Some(u1) = transform.data_to_unit(verts[1]) else {
            return;
        };
        let Some(u2) = transform.data_to_unit(verts[2]) else {
            return;
        };
        let to_f = |c: Color32| {
            let [r, g, b, a] = c.to_srgba_unmultiplied();
            [f32::from(r), f32::from(g), f32::from(b), f32::from(a)]
        };
        let clipped = clip_gouraud_tri_unit_cube([
            ClipVertex3D {
                u: u0,
                rgba: to_f(colors[0]),
            },
            ClipVertex3D {
                u: u1,
                rgba: to_f(colors[1]),
            },
            ClipVertex3D {
                u: u2,
                rgba: to_f(colors[2]),
            },
        ]);
        if clipped.len() < 3 {
            self.clipped += 1;
            return;
        }
        let mut proj = Vec::with_capacity(clipped.len());
        for cv in clipped {
            let Some((p, d)) = transform.project_unit(cv.u) else {
                return;
            };
            let c = Color32::from_rgba_unmultiplied(
                cv.rgba[0].round().clamp(0.0, 255.0) as u8,
                cv.rgba[1].round().clamp(0.0, 255.0) as u8,
                cv.rgba[2].round().clamp(0.0, 255.0) as u8,
                cv.rgba[3].round().clamp(0.0, 255.0) as u8,
            );
            proj.push((p, d + depth_bias, c));
        }
        let opaque = colors.iter().all(|c| c.a() >= 250);
        for i in 1..(proj.len() - 1) {
            self.tris.push(Tri3D {
                pos: [proj[0].0, proj[i].0, proj[i + 1].0],
                depth: [proj[0].1, proj[i].1, proj[i + 1].1],
                colors: [proj[0].2, proj[i].2, proj[i + 1].2],
                opaque,
            });
        }
    }

    pub fn push_box(
        &mut self,
        min: Point3D,
        max: Point3D,
        color: Color32,
        edge: Option<Stroke>,
        transform: Transform3D,
    ) {
        let corners = [
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [max[0], max[1], min[2]],
            [min[0], max[1], min[2]],
            [min[0], min[1], max[2]],
            [max[0], min[1], max[2]],
            [max[0], max[1], max[2]],
            [min[0], max[1], max[2]],
        ];
        let quads = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [1, 2, 6, 5],
            [0, 4, 7, 3],
        ];
        for q in quads {
            self.push_shaded_tri_data(
                [corners[q[0]], corners[q[1]], corners[q[2]]],
                color,
                transform,
            );
            self.push_shaded_tri_data(
                [corners[q[0]], corners[q[2]], corners[q[3]]],
                color,
                transform,
            );
        }
        if let Some(stroke) = edge
            && stroke.width > 0.0
            && stroke.color != Color32::TRANSPARENT
        {
            let edges = [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];
            for (i, j) in edges {
                self.push_data_segment(corners[i], corners[j], stroke, transform);
            }
        }
    }

    pub fn emit_sorted(
        &self,
        painter: &Painter,
        plot_area: Rect,
        text_color: Color32,
        diagnostics: &mut Plot3DDiagnostics,
    ) {
        let clipped = painter.with_clip_rect(plot_area);
        enum ItemRef {
            Tri(usize, f32),
            Seg(usize, f32),
            Marker(usize, f32),
        }
        let mut order = Vec::with_capacity(self.tris.len() + self.segs.len() + self.markers.len());
        for (i, t) in self.tris.iter().enumerate() {
            let d = (t.depth[0] + t.depth[1] + t.depth[2]) / 3.0;
            order.push(ItemRef::Tri(i, d));
        }
        for (i, s) in self.segs.iter().enumerate() {
            let d = 0.5 * (s.depth[0] + s.depth[1]) - 1e-4;
            order.push(ItemRef::Seg(i, d));
        }
        for (i, m) in self.markers.iter().enumerate() {
            order.push(ItemRef::Marker(i, m.depth - 2e-4));
        }
        order.sort_by(|a, b| {
            let da = match a {
                ItemRef::Tri(_, d) | ItemRef::Seg(_, d) | ItemRef::Marker(_, d) => *d,
            };
            let db = match b {
                ItemRef::Tri(_, d) | ItemRef::Seg(_, d) | ItemRef::Marker(_, d) => *d,
            };
            db.total_cmp(&da) // largest depth first (back-to-front)
        });

        let mut mesh = Mesh::default();
        for item in order {
            match item {
                ItemRef::Tri(idx, _) => {
                    let t = &self.tris[idx];
                    let base = mesh.vertices.len() as u32;
                    for k in 0..3 {
                        mesh.vertices.push(epaint::Vertex {
                            pos: t.pos[k],
                            uv: epaint::WHITE_UV,
                            color: t.colors[k],
                        });
                    }
                    mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
                    diagnostics.output_triangles += 1;
                    if mesh.indices.len() >= 16_384 * 3 {
                        clipped.add(Shape::mesh(std::mem::take(&mut mesh)));
                    }
                }
                ItemRef::Seg(idx, _) => {
                    flush_mesh(&clipped, &mut mesh);
                    let s = &self.segs[idx];
                    clipped.line_segment([s.p0, s.p1], s.stroke);
                    diagnostics.output_segments += 1;
                }
                ItemRef::Marker(idx, _) => {
                    flush_mesh(&clipped, &mut mesh);
                    let m = &self.markers[idx];
                    draw_marker_3d(&clipped, *m);
                    diagnostics.output_markers += 1;
                }
            }
        }
        flush_mesh(&clipped, &mut mesh);
        for lbl in &self.labels {
            if plot_area.contains(lbl.pos) {
                clipped.text(
                    lbl.pos,
                    egui::Align2::CENTER_CENTER,
                    &lbl.text,
                    egui::FontId::proportional(11.0),
                    if lbl.color == Color32::PLACEHOLDER {
                        text_color
                    } else {
                        lbl.color
                    },
                );
            }
        }
        diagnostics.clipped_items += self.clipped;
    }
}

fn flush_mesh(painter: &Painter, mesh: &mut Mesh) {
    if !mesh.is_empty() {
        painter.add(Shape::mesh(std::mem::take(mesh)));
    }
}

pub(super) fn draw_marker_3d(painter: &Painter, m: Marker3D) {
    match m.shape {
        MarkerShape::Circle => {
            painter.circle_filled(m.pos, m.radius, m.color);
        }
        MarkerShape::Square => {
            let r = Rect::from_center_size(m.pos, egui::vec2(m.radius * 1.8, m.radius * 1.8));
            painter.rect_filled(r, 0.0, m.color);
        }
        MarkerShape::Diamond => {
            let pts = vec![
                Pos2::new(m.pos.x, m.pos.y - m.radius),
                Pos2::new(m.pos.x + m.radius, m.pos.y),
                Pos2::new(m.pos.x, m.pos.y + m.radius),
                Pos2::new(m.pos.x - m.radius, m.pos.y),
            ];
            painter.add(Shape::convex_polygon(pts, m.color, Stroke::NONE));
        }
        _ => {
            painter.circle_filled(m.pos, m.radius, m.color);
        }
    }
}

fn face_shade(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = ab[1] * ac[2] - ab[2] * ac[1];
    let ny = ab[2] * ac[0] - ab[0] * ac[2];
    let nz = ab[0] * ac[1] - ab[1] * ac[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-12);
    let l = [0.42, -0.52, 0.74]; // normalized oblique directional light
    let dot = (nx / len * l[0] + ny / len * l[1] + nz / len * l[2]).abs();
    (0.60 + 0.40 * dot) as f32
}

pub(super) fn scale_rgb(c: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    let f = factor.clamp(0.15, 1.15);
    Color32::from_rgba_unmultiplied(
        (f32::from(r) * f).min(255.0) as u8,
        (f32::from(g) * f).min(255.0) as u8,
        (f32::from(b) * f).min(255.0) as u8,
        a,
    )
}
