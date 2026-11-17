use std::f64::consts::TAU;

use egui::{Color32, Stroke};

use super::{
    layers::{collect_polyline, push_marker_data},
    scene::{Label3D, Scene3D, scale_rgb},
    solid_layers::push_quiver3_head,
    surfaces::{is_fin3, light_factor, map_scalar_color},
};
use crate::{
    AlphaShape3D, Graph3D, MarkerShape, Patch3D, Plot3DDiagnostics, SurfaceShading, Transform3D,
};

pub(super) fn collect_patch3d(
    scene: &mut Scene3D,
    patch: Patch3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let data = patch.data;
    let verts = data.vertices();
    let v_norms = data.vertex_normals();
    let f_norms = data.face_normals();
    let v_scal = data.vertex_scalars();
    let f_scal = data.face_scalars();
    let v_cols = data.vertex_colors();
    let f_cols = data.face_colors();
    let s_range = data.scalar_range();
    let base_fb = patch.style.color.unwrap_or(fallback);

    for (ti, &tri) in data.triangles().iter().enumerate() {
        diagnostics.visited_items += 1;
        let idx = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        let p = [verts[idx[0]], verts[idx[1]], verts[idx[2]]];
        if !p.iter().all(|pt| is_fin3(*pt)) {
            continue;
        }
        let fi = data.triangle_face_indices()[ti] as usize;
        if patch.shading == SurfaceShading::Interp && f_cols.is_none() && f_scal.is_none() {
            let cols = idx.map(|vi| {
                let raw = v_cols
                    .map(|c| c[vi])
                    .or_else(|| {
                        v_scal.map(|s| map_scalar_color(s[vi], patch.color_scale, s_range, base_fb))
                    })
                    .unwrap_or(base_fb);
                scale_rgb(raw, light_factor(v_norms[vi], patch.light, transform))
                    .gamma_multiply(patch.alpha)
            });
            scene.push_gouraud_tri_data(p, cols, 0.0, transform);
        } else {
            let raw = f_cols
                .map(|c| c[fi])
                .or_else(|| {
                    f_scal.map(|s| map_scalar_color(s[fi], patch.color_scale, s_range, base_fb))
                })
                .or_else(|| v_cols.map(|vc| avg_color3(vc[idx[0]], vc[idx[1]], vc[idx[2]])))
                .or_else(|| {
                    v_scal.map(|vs| {
                        let m = (vs[idx[0]] + vs[idx[1]] + vs[idx[2]]) / 3.0;
                        map_scalar_color(m, patch.color_scale, s_range, base_fb)
                    })
                })
                .unwrap_or(base_fb);
            let lit = scale_rgb(raw, light_factor(f_norms[fi], patch.light, transform))
                .gamma_multiply(patch.alpha);
            scene.push_gouraud_tri_data(p, [lit; 3], 0.0, transform);
        }
    }

    if patch.shading == SurfaceShading::Faceted
        && let Some(stroke) = patch.edge_stroke
    {
        for &[a, b] in data.edges() {
            let pa = verts[a as usize];
            let pb = verts[b as usize];
            if is_fin3(pa) && is_fin3(pb) {
                scene.push_data_segment(pa, pb, stroke, transform);
            }
        }
    }
}

pub(super) fn collect_alphashape3d(
    scene: &mut Scene3D,
    ash: AlphaShape3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let Some(mesh) = ash.data.mesh() else {
        return;
    };
    let verts = mesh.vertices();
    let norms = mesh.vertex_normals();
    let v_scal = mesh.vertex_scalars();
    let s_range = mesh.scalar_range();
    let base_fb = ash.style.color.unwrap_or(fallback);

    for &[i0, i1, i2] in mesh.triangles() {
        diagnostics.visited_items += 1;
        let idx = [i0 as usize, i1 as usize, i2 as usize];
        let p = [verts[idx[0]], verts[idx[1]], verts[idx[2]]];
        if ash.shading == SurfaceShading::Interp {
            let cols = idx.map(|vi| {
                let raw = ash.style.color.unwrap_or_else(|| {
                    map_scalar_color(v_scal[vi], ash.color_scale, s_range, base_fb)
                });
                scale_rgb(raw, light_factor(norms[vi], ash.light, transform))
                    .gamma_multiply(ash.alpha)
            });
            scene.push_gouraud_tri_data(p, cols, 0.0, transform);
        } else {
            let navg = [
                norms[idx[0]][0] + norms[idx[1]][0] + norms[idx[2]][0],
                norms[idx[0]][1] + norms[idx[1]][1] + norms[idx[2]][1],
                norms[idx[0]][2] + norms[idx[1]][2] + norms[idx[2]][2],
            ];
            let savg = (v_scal[idx[0]] + v_scal[idx[1]] + v_scal[idx[2]]) / 3.0;
            let raw = ash
                .style
                .color
                .unwrap_or_else(|| map_scalar_color(savg, ash.color_scale, s_range, base_fb));
            let c =
                scale_rgb(raw, light_factor(navg, ash.light, transform)).gamma_multiply(ash.alpha);
            scene.push_gouraud_tri_data(p, [c; 3], 0.0, transform);
        }
    }

    if ash.shading == SurfaceShading::Faceted
        && let Some(stroke) = ash.edge_stroke
    {
        for &[a, b] in mesh.edges() {
            scene.push_data_segment(verts[a as usize], verts[b as usize], stroke, transform);
        }
    }
}

pub(super) fn collect_graph3d(
    scene: &mut Scene3D,
    g: Graph3D<'_>,
    node_fallback: Color32,
    node_shape: MarkerShape,
    node_radius: f32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let data = g.data;
    let coords = data.coords();
    let span = transform.bounds().span();
    let loop_r = span[0].min(span[1]).min(span[2]).max(1e-6) * 0.09;
    let directed = data.is_directed();
    let e_labels = data.edge_labels();

    for (ei, edge) in data.edges().iter().enumerate() {
        diagnostics.visited_items += 1;
        let pa = coords[edge.source];
        let pb = coords[edge.target];
        if !is_fin3(pa) || !is_fin3(pb) {
            continue;
        }
        let stroke = Stroke::new(
            (g.edge_stroke.width * (edge.weight.abs().sqrt() as f32).clamp(0.7, 2.2))
                .clamp(0.6, 4.5),
            g.edge_stroke.color,
        );
        let label_pos = if edge.source == edge.target {
            let steps = 16usize;
            let mut pts = Vec::with_capacity(steps + 1);
            for k in 0..=steps {
                let ang = k as f64 / steps as f64 * TAU;
                pts.push([
                    pa[0] + loop_r * ang.cos(),
                    pa[1] + loop_r * 0.5 * ang.sin(),
                    pa[2] + loop_r * (1.0 - ang.cos()),
                ]);
            }
            collect_polyline(scene, &pts, stroke, None, transform, diagnostics);
            pts[steps / 2]
        } else if edge.parallel_count > 1 || edge.bidirectional_pair {
            let mut rank = edge.parallel_index as f64 - 0.5 * (edge.parallel_count - 1) as f64;
            if edge.bidirectional_pair {
                rank += if edge.source < edge.target {
                    0.55
                } else {
                    -0.55
                };
            }
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let up =
                if d[2].abs() < 0.85 * (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9) {
                    [0.0, 0.0, 1.0]
                } else {
                    [1.0, 0.0, 0.0]
                };
            let ox = d[1] * up[2] - d[2] * up[1];
            let oy = d[2] * up[0] - d[0] * up[2];
            let oz = d[0] * up[1] - d[1] * up[0];
            let ol = (ox * ox + oy * oy + oz * oz).sqrt().max(1e-9);
            let bend = 0.14 * rank * (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(loop_r);
            let ctrl = [
                0.5 * (pa[0] + pb[0]) + (ox / ol) * bend,
                0.5 * (pa[1] + pb[1]) + (oy / ol) * bend,
                0.5 * (pa[2] + pb[2]) + (oz / ol) * bend,
            ];
            let steps = 12usize;
            let mut pts = Vec::with_capacity(steps + 1);
            for k in 0..=steps {
                let t = k as f64 / steps as f64;
                let omt = 1.0 - t;
                pts.push([
                    omt * omt * pa[0] + 2.0 * omt * t * ctrl[0] + t * t * pb[0],
                    omt * omt * pa[1] + 2.0 * omt * t * ctrl[1] + t * t * pb[1],
                    omt * omt * pa[2] + 2.0 * omt * t * ctrl[2] + t * t * pb[2],
                ]);
            }
            collect_polyline(scene, &pts, stroke, None, transform, diagnostics);
            if directed {
                push_quiver3_head(
                    scene,
                    pts[steps - 1],
                    pb,
                    g.arrow_size,
                    stroke.color,
                    transform,
                );
            }
            pts[steps / 2]
        } else {
            scene.push_data_segment(pa, pb, stroke, transform);
            if directed {
                push_quiver3_head(scene, pa, pb, g.arrow_size, stroke.color, transform);
            }
            [
                0.5 * (pa[0] + pb[0]),
                0.5 * (pa[1] + pb[1]),
                0.5 * (pa[2] + pb[2]),
            ]
        };

        if g.show_edge_labels
            && let Some(lbl) = e_labels.get(ei)
            && !lbl.is_empty()
            && let Some((pos, _)) = transform.project_data(label_pos)
        {
            scene.labels.push(Label3D {
                pos,
                text: lbl.clone(),
                color: Color32::PLACEHOLDER,
            });
        }
    }

    let n_scals = data.node_scalars();
    let s_range = data.scalar_range();
    let n_labels = data.node_labels();
    let base_c = g.style.color.unwrap_or(node_fallback);
    for (ni, &p) in coords.iter().enumerate() {
        diagnostics.visited_items += 1;
        let c = n_scals
            .map(|s| map_scalar_color(s[ni], g.color_scale, s_range, base_c))
            .unwrap_or(base_c);
        push_marker_data(scene, p, node_radius, c, node_shape, transform);
        if g.show_node_labels
            && let Some(lbl) = n_labels.get(ni)
            && !lbl.is_empty()
            && let Some((mut pos, _)) = transform.project_data(p)
        {
            pos.x += node_radius + 4.0;
            scene.labels.push(Label3D {
                pos,
                text: lbl.clone(),
                color: Color32::PLACEHOLDER,
            });
        }
    }
}

fn avg_color3(a: Color32, b: Color32, c: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        ((u16::from(a.r()) + u16::from(b.r()) + u16::from(c.r())) / 3) as u8,
        ((u16::from(a.g()) + u16::from(b.g()) + u16::from(c.g())) / 3) as u8,
        ((u16::from(a.b()) + u16::from(b.b()) + u16::from(c.b())) / 3) as u8,
        ((u16::from(a.a()) + u16::from(b.a()) + u16::from(c.a())) / 3) as u8,
    )
}
