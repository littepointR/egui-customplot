use egui::{IdSalt, Pos2};

use super::{
    hover::{Hovered3DPoint, Hovered3DRegion, consider_pt, consider_tri_region},
    surface_hover::pick_grid_tris,
};
use crate::{Layer3D, Point3D, Transform3D, TriMesh3DData};

pub(super) fn pick_volume_points(
    layer: Layer3D<'_>,
    id: IdSalt,
    li: usize,
    transform: Transform3D,
    pointer: Pos2,
    radius: f32,
    best_pt: &mut Option<Hovered3DPoint>,
) {
    match layer {
        Layer3D::Streamline(sl) => {
            let mut cursor = 0usize;
            for line in sl.data.lines() {
                for &p in line.points.iter() {
                    consider_pt(best_pt, id, li, cursor, p, transform, pointer, radius);
                    cursor += 1;
                }
            }
        }
        Layer3D::StreamParticles(sp) => {
            let rad = radius.max(sp.style.marker_radius.unwrap_or(4.0));
            for (pi, (p, _)) in sp.playback.positions(sp.data).into_iter().enumerate() {
                consider_pt(best_pt, id, li, pi, p, transform, pointer, rad);
            }
        }
        Layer3D::ConePlot(cp) => {
            for (ci, c) in cp.data.cones().iter().enumerate() {
                consider_pt(best_pt, id, li, ci, c.origin, transform, pointer, radius);
                consider_pt(best_pt, id, li, ci, c.apex, transform, pointer, radius);
            }
        }
        _ => {}
    }
}

pub(super) fn pick_volume_regions(
    layer: Layer3D<'_>,
    id: IdSalt,
    li: usize,
    transform: Transform3D,
    pointer: Pos2,
    best_region: &mut Option<Hovered3DRegion>,
) {
    match layer {
        Layer3D::VolumeSlice(vs) => {
            for (si, slice) in vs.data.slices().iter().enumerate() {
                pick_grid_tris(
                    best_region,
                    id,
                    li,
                    slice,
                    "slice",
                    Some(si),
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::Isosurface(iso) => {
            if let Some(m) = iso.data.mesh() {
                pick_mesh_tris(best_region, id, li, m, "isosurface", transform, pointer);
            }
        }
        Layer3D::Isocaps(caps) => {
            if let Some(m) = caps.data.mesh() {
                pick_mesh_tris(best_region, id, li, m, "isocaps", transform, pointer);
            }
        }
        Layer3D::TetraMesh(tm) => {
            let verts = tm.data.vertices();
            let (faces, tids) = if tm.boundary_only {
                (tm.data.boundary_faces(), tm.data.boundary_face_tet())
            } else {
                (tm.data.all_faces(), tm.data.all_face_tet())
            };
            for (&[a, b, c], &tid) in faces.iter().zip(tids.iter()) {
                consider_tri_region(
                    best_region,
                    id,
                    li,
                    tid as usize,
                    || format!("tet[{tid}]"),
                    [verts[a as usize], verts[b as usize], verts[c as usize]],
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::StreamRibbon(sr) => {
            for (si, strip) in sr.data.strips().iter().enumerate() {
                pick_grid_tris(
                    best_region,
                    id,
                    li,
                    strip,
                    "streamribbon",
                    Some(si),
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::StreamTube(st) => {
            for (si, tube) in st.data.tubes().iter().enumerate() {
                pick_grid_tris(
                    best_region,
                    id,
                    li,
                    tube,
                    "streamtube",
                    Some(si),
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::ConePlot(cp) => {
            if let Some(m) = cp.data.mesh() {
                pick_mesh_tris(best_region, id, li, m, "coneplot", transform, pointer);
            }
        }
        _ => {}
    }
}

fn pick_mesh_tris(
    best_region: &mut Option<Hovered3DRegion>,
    id: IdSalt,
    li: usize,
    mesh: &TriMesh3DData,
    prefix: &str,
    transform: Transform3D,
    pointer: Pos2,
) {
    let verts: &[Point3D] = mesh.vertices();
    for (ti, &[a, b, c]) in mesh.triangles().iter().enumerate() {
        consider_tri_region(
            best_region,
            id,
            li,
            ti,
            || format!("{prefix}[{ti}]"),
            [verts[a as usize], verts[b as usize], verts[c as usize]],
            transform,
            pointer,
        );
    }
}
