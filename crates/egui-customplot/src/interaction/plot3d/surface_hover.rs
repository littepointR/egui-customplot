use egui::{IdSalt, Pos2};

use super::hover::{Hovered3DPoint, Hovered3DRegion, consider_pt, consider_tri_region};
use crate::{Layer3D, Point3D, Surface3DData, Transform3D};

pub(super) fn pick_surface_points(
    layer: Layer3D<'_>,
    id: IdSalt,
    li: usize,
    transform: Transform3D,
    pointer: Pos2,
    radius: f32,
    best_pt: &mut Option<Hovered3DPoint>,
) {
    match layer {
        Layer3D::Surface(s) => {
            if s.primitive {
                return;
            }
            for (pi, &p) in s.data.points().iter().enumerate() {
                consider_pt(best_pt, id, li, pi, p, transform, pointer, radius);
            }
        }
        Layer3D::MeshSurface(m) => {
            for (pi, &p) in m.data.points().iter().enumerate() {
                consider_pt(best_pt, id, li, pi, p, transform, pointer, radius);
            }
        }
        Layer3D::Contour(c) => {
            let mut cursor = 0usize;
            for line in c.data.lines() {
                for &p in line.points.iter() {
                    consider_pt(best_pt, id, li, cursor, p, transform, pointer, radius);
                    cursor += 1;
                }
            }
        }
        Layer3D::Ribbon(r) => {
            let mut cursor = 0usize;
            for strip in r.data.strips() {
                for &p in strip.points() {
                    consider_pt(best_pt, id, li, cursor, p, transform, pointer, radius);
                    cursor += 1;
                }
            }
        }
        Layer3D::TriSurf(ts) => {
            if let Some(m) = ts.data {
                for (pi, &p) in m.vertices().iter().enumerate() {
                    consider_pt(best_pt, id, li, pi, p, transform, pointer, radius);
                }
            }
        }
        _ => {}
    }
}

pub(super) fn pick_extra_regions(
    layer: Layer3D<'_>,
    id: IdSalt,
    li: usize,
    transform: Transform3D,
    pointer: Pos2,
    best_region: &mut Option<Hovered3DRegion>,
) {
    match layer {
        Layer3D::Pie(pie) => {
            let h = pie.data.thickness();
            let expl = pie.data.offset();
            for slice in pie.data.slices() {
                let mid = 0.5 * (slice.start_angle_rad + slice.end_angle_rad);
                let shift = if slice.exploded {
                    [expl * mid.cos(), expl * mid.sin()]
                } else {
                    [0.0, 0.0]
                };
                let c_top = [shift[0], shift[1], h];
                let steps = (((slice.end_angle_rad - slice.start_angle_rad)
                    / (std::f64::consts::PI / 18.0))
                    .ceil() as usize)
                    .clamp(4, 36);
                let rim = |i: usize| -> (Point3D, Point3D) {
                    let a = slice.start_angle_rad
                        + (slice.end_angle_rad - slice.start_angle_rad) * (i as f64 / steps as f64);
                    let x = shift[0] + a.cos();
                    let y = shift[1] + a.sin();
                    ([x, y, 0.0], [x, y, h])
                };
                for i in 0..steps {
                    let (bi0, ti0) = rim(i);
                    let (bi1, ti1) = rim(i + 1);
                    for tri in [[c_top, ti0, ti1], [bi0, bi1, ti1], [bi0, ti1, ti0]] {
                        consider_tri_region(
                            best_region,
                            id,
                            li,
                            slice.index,
                            || slice.label.clone(),
                            tri,
                            transform,
                            pointer,
                        );
                    }
                }
            }
        }
        Layer3D::Surface(s) => {
            if !s.primitive {
                pick_grid_tris(
                    best_region,
                    id,
                    li,
                    s.data,
                    "surf",
                    None,
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::Ribbon(r) => {
            for (si, strip) in r.data.strips().iter().enumerate() {
                pick_grid_tris(
                    best_region,
                    id,
                    li,
                    strip,
                    "ribbon",
                    Some(si),
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::TriSurf(ts) => {
            if !ts.wireframe
                && let Some(m) = ts.data
            {
                let verts = m.vertices();
                for (ti, &[a, b, c]) in m.triangles().iter().enumerate() {
                    consider_tri_region(
                        best_region,
                        id,
                        li,
                        ti,
                        || format!("trisurf[{ti}]"),
                        [verts[a as usize], verts[b as usize], verts[c as usize]],
                        transform,
                        pointer,
                    );
                }
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn pick_grid_tris(
    best_region: &mut Option<Hovered3DRegion>,
    id: IdSalt,
    li: usize,
    grid: &Surface3DData,
    prefix: &str,
    strip_idx: Option<usize>,
    transform: Transform3D,
    pointer: Pos2,
) {
    let rows = grid.rows();
    let cols = grid.cols();
    let pts = grid.points();
    let fin = |p: Point3D| p[0].is_finite() && p[1].is_finite() && p[2].is_finite();
    for r in 0..(rows - 1) {
        for c in 0..(cols - 1) {
            let p00 = pts[r * cols + c];
            let p10 = pts[r * cols + c + 1];
            let p01 = pts[(r + 1) * cols + c];
            let p11 = pts[(r + 1) * cols + c + 1];
            if ![p00, p10, p01, p11].iter().all(|p| fin(*p)) {
                continue;
            }
            let item_index = strip_idx.unwrap_or(r * (cols - 1) + c);
            let label = || match strip_idx {
                Some(si) => format!("{prefix}[{si}, {r}]"),
                None => format!("{prefix}[{r}, {c}]"),
            };
            for tri in [[p00, p10, p11], [p00, p11, p01]] {
                consider_tri_region(
                    best_region,
                    id,
                    li,
                    item_index,
                    label,
                    tri,
                    transform,
                    pointer,
                );
            }
        }
    }
}
