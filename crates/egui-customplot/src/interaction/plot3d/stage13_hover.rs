use egui::{IdSalt, Pos2};

use super::hover::{Hovered3DPoint, Hovered3DRegion, consider_pt, consider_tri_region};
use crate::{Layer3D, Transform3D, axis::point_polyline_distance};

pub(super) fn pick_stage13_points(
    layer: Layer3D<'_>,
    id: IdSalt,
    li: usize,
    transform: Transform3D,
    pointer: Pos2,
    radius: f32,
    best_pt: &mut Option<Hovered3DPoint>,
) {
    match layer {
        Layer3D::Graph(g) => {
            let hit_r = radius.max(g.style.marker_radius.unwrap_or(4.5) + 2.0);
            for (pi, &p) in g.data.coords().iter().enumerate() {
                consider_pt(best_pt, id, li, pi, p, transform, pointer, hit_r);
            }
        }
        Layer3D::PointCloud(pc) => {
            let hit_r = radius.max(pc.style.marker_radius.unwrap_or(2.6) + 2.5);
            if let Some((idx, pos, dist, depth)) = pc.data.pick_nearest(transform, pointer, hit_r)
                && best_pt.as_ref().is_none_or(|b| dist < b.screen_distance)
            {
                *best_pt = Some(Hovered3DPoint {
                    layer_id: id,
                    layer_index: li,
                    point_index: idx,
                    position: pos,
                    depth,
                    screen_distance: dist,
                });
            }
        }
        _ => {}
    }
}

pub(super) fn pick_stage13_regions(
    layer: Layer3D<'_>,
    id: IdSalt,
    li: usize,
    transform: Transform3D,
    pointer: Pos2,
    best_region: &mut Option<Hovered3DRegion>,
) {
    match layer {
        Layer3D::Patch(patch) => {
            let verts = patch.data.vertices();
            let faces = patch.data.triangle_face_indices();
            for (ti, &tri) in patch.data.triangles().iter().enumerate() {
                let face_idx = faces[ti] as usize;
                let pts = tri.map(|i| verts[i as usize]);
                consider_tri_region(
                    best_region,
                    id,
                    li,
                    face_idx,
                    || {
                        if patch.name.is_empty() {
                            format!("face #{face_idx}")
                        } else {
                            format!("{}\nface #{face_idx}", patch.name)
                        }
                    },
                    pts,
                    transform,
                    pointer,
                );
            }
        }
        Layer3D::AlphaShape(ash) => {
            if let Some(mesh) = ash.data.mesh() {
                let verts = mesh.vertices();
                for (ti, &tri) in mesh.triangles().iter().enumerate() {
                    let pts = tri.map(|i| verts[i as usize]);
                    consider_tri_region(
                        best_region,
                        id,
                        li,
                        ti,
                        || {
                            if ash.name.is_empty() {
                                format!("alpha triangle #{ti}")
                            } else {
                                format!("{}\nalpha triangle #{ti}", ash.name)
                            }
                        },
                        pts,
                        transform,
                        pointer,
                    );
                }
            }
        }
        Layer3D::Graph(g) => {
            let coords = g.data.coords();
            let e_labels = g.data.edge_labels();
            for (ei, e) in g.data.edges().iter().enumerate() {
                let (Some((pa, da)), Some((pb, db))) = (
                    transform.project_data(coords[e.source]),
                    transform.project_data(coords[e.target]),
                ) else {
                    continue;
                };
                let ab = pb - pa;
                let ab2 = ab.length_sq();
                let t = if ab2 > 1e-6 {
                    ((pointer - pa).dot(ab) / ab2).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                if point_polyline_distance(pointer, &[pa, pb]) <= 8.0 {
                    let d = da + (db - da) * t + 0.05;
                    if best_region.as_ref().is_none_or(|cur| d < cur.depth) {
                        let lbl = e_labels
                            .get(ei)
                            .filter(|s| !s.is_empty())
                            .cloned()
                            .unwrap_or_else(|| format!("edge {} -> {}", e.source, e.target));
                        *best_region = Some(Hovered3DRegion {
                            layer_id: id,
                            layer_index: li,
                            item_index: ei,
                            label: if g.name.is_empty() {
                                lbl
                            } else {
                                format!("{}\n{lbl}", g.name)
                            },
                            depth: d,
                        });
                    }
                }
            }
        }
        Layer3D::VolumeRender(vr) => {
            if let Some((u0, ud, t0, t1)) = transform.screen_ray_unit(pointer) {
                let steps = 24usize;
                let dt = (t1 - t0) / steps as f64;
                let mut peak = f64::NEG_INFINITY;
                let mut peak_t = t0;
                for s in 0..steps {
                    let t = t0 + (s as f64 + 0.5) * dt;
                    let u = [u0[0] + t * ud[0], u0[1] + t * ud[1], u0[2] + t * ud[2]];
                    if let Some(dp) = transform.unit_to_data(u)
                        && let Some(val) = vr.volume.sample(dp)
                        && val > peak
                    {
                        peak = val;
                        peak_t = t;
                    }
                }
                if peak.is_finite() {
                    let depth = peak_t as f32;
                    if best_region.as_ref().is_none_or(|cur| depth < cur.depth) {
                        *best_region = Some(Hovered3DRegion {
                            layer_id: id,
                            layer_index: li,
                            item_index: 0,
                            label: if vr.name.is_empty() {
                                format!("vol peak={peak:.3}")
                            } else {
                                format!("{}\npeak={peak:.3}", vr.name)
                            },
                            depth,
                        });
                    }
                }
            }
        }
        _ => {}
    }
}
