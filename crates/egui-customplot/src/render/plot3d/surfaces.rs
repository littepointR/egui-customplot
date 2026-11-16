use egui::{Color32, Stroke};

use super::{
    meshes_contours::collect_floor_contours,
    scene::{Scene3D, scale_rgb},
    solid_layers::push_quiver3_head,
};
use crate::{
    ColorMap, ColorScale, LightSource3D, Plot3DDiagnostics, Point3D, Ribbon3D, Surface3D,
    Surface3DData, SurfaceShading, Transform3D, TriSurf3D, model::ResolvedPlotStyle,
};

pub(super) struct SurfaceAppearance {
    pub(super) shading: SurfaceShading,
    pub(super) explicit_color: Option<Color32>,
    pub(super) color_scale: Option<ColorScale>,
    pub(super) light: Option<LightSource3D>,
    pub(super) colormap_lighting: bool,
    pub(super) wireframe_stroke: Option<Stroke>,
    pub(super) alpha: f32,
    pub(super) fallback: Color32,
}

pub(super) fn collect_surface3d(
    scene: &mut Scene3D,
    surf: Surface3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let app = SurfaceAppearance {
        shading: surf.shading,
        explicit_color: surf.style.color,
        color_scale: surf.color_scale,
        light: surf.light,
        colormap_lighting: surf.colormap_lighting,
        wireframe_stroke: surf.wireframe_stroke,
        alpha: surf.alpha,
        fallback,
    };
    emit_surface_grid(scene, surf.data, &app, transform, diagnostics);

    if surf.floor_contours
        && let Some(c) = surf.data.contours()
    {
        let z_floor = surf.floor_z.unwrap_or_else(|| transform.bounds().min()[2]);
        collect_floor_contours(scene, c, z_floor, surf.color_scale, fallback, transform);
    }

    if surf.show_normals {
        let q = surf
            .data
            .normals_quiver(surf.normal_stride, surf.normal_scale);
        let n_stroke = Stroke::new(1.1, Color32::from_rgb(35, 95, 165));
        for &arr in q.arrows() {
            if let Some(end) = arr.endpoint(1.0) {
                scene.push_data_segment(arr.origin, end, n_stroke, transform);
                push_quiver3_head(scene, arr.origin, end, 5.0, n_stroke.color, transform);
            }
        }
    }
}

pub(super) fn collect_ribbon3d(
    scene: &mut Scene3D,
    rib: Ribbon3D<'_>,
    style: &ResolvedPlotStyle<'_>,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (idx, strip) in rib.data.strips().iter().enumerate() {
        let col = style.resolve_series(idx, Default::default()).stroke.color;
        let app = SurfaceAppearance {
            shading: rib.shading,
            explicit_color: rib.color_scale.is_none().then_some(col),
            color_scale: rib.color_scale,
            light: Some(LightSource3D::default()),
            colormap_lighting: false,
            wireframe_stroke: rib.edge_stroke,
            alpha: rib.alpha,
            fallback: col,
        };
        emit_surface_grid(scene, strip, &app, transform, diagnostics);
    }
}

pub(super) fn collect_trisurf3d(
    scene: &mut Scene3D,
    ts: TriSurf3D<'_>,
    bg_color: Color32,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let Some(mesh) = ts.data else {
        return;
    };
    let verts = mesh.vertices();
    let norms = mesh.vertex_normals();
    let v_scal = mesh.vertex_scalars();
    let f_scal = mesh.face_scalars();
    let s_range = mesh.scalar_range();
    let light = LightSource3D::default();

    if ts.wireframe {
        diagnostics.visited_items += mesh.edges().len();
        if ts.hidden_line {
            let occlude = Color32::from_rgb(bg_color.r(), bg_color.g(), bg_color.b());
            for &[i0, i1, i2] in mesh.triangles() {
                let p = [verts[i0 as usize], verts[i1 as usize], verts[i2 as usize]];
                scene.push_gouraud_tri_data(p, [occlude; 3], 1.5e-4, transform);
            }
        }
        let edge_stroke = ts
            .style
            .color
            .map(|c| Stroke::new(1.1, c))
            .unwrap_or_else(|| Stroke::new(1.1, fallback));
        for &[a, b] in mesh.edges() {
            let pa = verts[a as usize];
            let pb = verts[b as usize];
            let c = ts.style.color.unwrap_or_else(|| {
                map_scalar_color(
                    0.5 * (v_scal[a as usize] + v_scal[b as usize]),
                    ts.color_scale,
                    s_range,
                    edge_stroke.color,
                )
            });
            scene.push_data_segment(pa, pb, Stroke::new(edge_stroke.width, c), transform);
        }
        return;
    }

    for (fi, &[i0, i1, i2]) in mesh.triangles().iter().enumerate() {
        diagnostics.visited_items += 1;
        let idx = [i0 as usize, i1 as usize, i2 as usize];
        let p = [verts[idx[0]], verts[idx[1]], verts[idx[2]]];
        if ts.shading == SurfaceShading::Interp && f_scal.is_none() {
            let cols = idx.map(|vi| {
                let base = ts.style.color.unwrap_or_else(|| {
                    map_scalar_color(v_scal[vi], ts.color_scale, s_range, fallback)
                });
                let factor = light_factor(norms[vi], light, transform);
                scale_rgb(base, factor).gamma_multiply(ts.alpha)
            });
            scene.push_gouraud_tri_data(p, cols, 0.0, transform);
        } else {
            let scalar = f_scal
                .map(|fs| fs[fi])
                .unwrap_or_else(|| (v_scal[idx[0]] + v_scal[idx[1]] + v_scal[idx[2]]) / 3.0);
            let base = ts
                .style
                .color
                .unwrap_or_else(|| map_scalar_color(scalar, ts.color_scale, s_range, fallback));
            let navg = [
                norms[idx[0]][0] + norms[idx[1]][0] + norms[idx[2]][0],
                norms[idx[0]][1] + norms[idx[1]][1] + norms[idx[2]][1],
                norms[idx[0]][2] + norms[idx[1]][2] + norms[idx[2]][2],
            ];
            let c = scale_rgb(base, light_factor(navg, light, transform)).gamma_multiply(ts.alpha);
            scene.push_gouraud_tri_data(p, [c; 3], 0.0, transform);
        }
    }
    if ts.shading == SurfaceShading::Faceted
        && let Some(stroke) = ts.edge_stroke
    {
        for &[a, b] in mesh.edges() {
            scene.push_data_segment(verts[a as usize], verts[b as usize], stroke, transform);
        }
    }
}

pub(super) fn emit_surface_grid(
    scene: &mut Scene3D,
    data: &Surface3DData,
    app: &SurfaceAppearance,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let rows = data.rows();
    let cols = data.cols();
    let pts = data.points();
    let norms = data.normals();
    let scals = data.scalars();
    let s_range = data.scalar_range();

    let vert_color = |idx: usize| -> Option<Color32> {
        let s = scals[idx];
        let base = if app.colormap_lighting {
            let ls = app.light.unwrap_or_default();
            let refl =
                (f64::from(light_factor(norms[idx], ls, transform) - 0.1) / 1.05).clamp(0.0, 1.0);
            let cmap = app.color_scale.map_or(ColorMap::Gray, ColorScale::map);
            cmap.sample(refl)
        } else if let Some(explicit) = app.explicit_color {
            let ls = app.light.unwrap_or_default();
            let f = if app.light.is_some() {
                light_factor(norms[idx], ls, transform)
            } else {
                1.0
            };
            scale_rgb(explicit, f)
        } else {
            let c = map_scalar_opt(s, app.color_scale, s_range, app.fallback)?;
            if let Some(ls) = app.light {
                scale_rgb(c, light_factor(norms[idx], ls, transform))
            } else {
                c
            }
        };
        Some(base.gamma_multiply(app.alpha))
    };

    for r in 0..(rows - 1) {
        for c in 0..(cols - 1) {
            diagnostics.visited_items += 1;
            let i00 = r * cols + c;
            let i10 = i00 + 1;
            let i01 = (r + 1) * cols + c;
            let i11 = i01 + 1;
            let p00 = pts[i00];
            let p10 = pts[i10];
            let p01 = pts[i01];
            let p11 = pts[i11];
            if ![p00, p10, p01, p11].iter().all(|p| is_fin3(*p)) {
                continue;
            }
            if app.shading == SurfaceShading::Interp {
                let (Some(c00), Some(c10), Some(c01), Some(c11)) = (
                    vert_color(i00),
                    vert_color(i10),
                    vert_color(i01),
                    vert_color(i11),
                ) else {
                    continue;
                };
                scene.push_gouraud_tri_data([p00, p10, p11], [c00, c10, c11], 0.0, transform);
                scene.push_gouraud_tri_data([p00, p11, p01], [c00, c11, c01], 0.0, transform);
            } else {
                let avg_s = 0.25 * (scals[i00] + scals[i10] + scals[i01] + scals[i11]);
                let avg_n = [
                    norms[i00][0] + norms[i10][0] + norms[i01][0] + norms[i11][0],
                    norms[i00][1] + norms[i10][1] + norms[i01][1] + norms[i11][1],
                    norms[i00][2] + norms[i10][2] + norms[i01][2] + norms[i11][2],
                ];
                let cell_c = if app.colormap_lighting {
                    let ls = app.light.unwrap_or_default();
                    let r = (f64::from(light_factor(avg_n, ls, transform) - 0.1) / 1.05)
                        .clamp(0.0, 1.0);
                    let cmap = app.color_scale.map_or(ColorMap::Gray, ColorScale::map);
                    cmap.sample(r)
                } else {
                    let Some(base) = app
                        .explicit_color
                        .or_else(|| map_scalar_opt(avg_s, app.color_scale, s_range, app.fallback))
                    else {
                        continue;
                    };
                    if let Some(ls) = app.light {
                        scale_rgb(base, light_factor(avg_n, ls, transform))
                    } else {
                        base
                    }
                }
                .gamma_multiply(app.alpha);
                scene.push_gouraud_tri_data([p00, p10, p11], [cell_c; 3], 0.0, transform);
                scene.push_gouraud_tri_data([p00, p11, p01], [cell_c; 3], 0.0, transform);
            }
            if app.shading == SurfaceShading::Faceted
                && let Some(wire) = app.wireframe_stroke
            {
                scene.push_data_segment(p00, p10, wire, transform);
                scene.push_data_segment(p00, p01, wire, transform);
                if c + 2 == cols {
                    scene.push_data_segment(p10, p11, wire, transform);
                }
                if r + 2 == rows {
                    scene.push_data_segment(p01, p11, wire, transform);
                }
            }
        }
    }
}

pub(super) fn map_scalar_color(
    val: f64,
    scale: Option<ColorScale>,
    range: Option<[f64; 2]>,
    fallback: Color32,
) -> Color32 {
    map_scalar_opt(val, scale, range, fallback).unwrap_or(fallback)
}

fn map_scalar_opt(
    val: f64,
    scale: Option<ColorScale>,
    range: Option<[f64; 2]>,
    fallback: Color32,
) -> Option<Color32> {
    if let Some(cs) = scale {
        return cs.color(val);
    }
    if let Some([lo, hi]) = range
        && hi > lo
        && val.is_finite()
    {
        return Some(ColorMap::Viridis.sample(((val - lo) / (hi - lo)).clamp(0.0, 1.0)));
    }
    Some(fallback)
}

pub(super) fn light_factor(
    data_norm: [f64; 3],
    light: LightSource3D,
    transform: Transform3D,
) -> f32 {
    let nu = transform
        .data_normal_to_unit_normal(data_norm)
        .unwrap_or([0.0, 0.0, 1.0]);
    let (_, _, forward) = transform.camera().basis();
    let v = [-forward[0], -forward[1], -forward[2]];
    let l = light.direction();
    let h_raw = [l[0] + v[0], l[1] + v[1], l[2] + v[2]];
    let hl = (h_raw[0] * h_raw[0] + h_raw[1] * h_raw[1] + h_raw[2] * h_raw[2])
        .sqrt()
        .max(1e-12);
    let h = [h_raw[0] / hl, h_raw[1] / hl, h_raw[2] / hl];
    let ndotl = (nu[0] * l[0] + nu[1] * l[1] + nu[2] * l[2]).abs() as f32;
    let ndoth = (nu[0] * h[0] + nu[1] * h[1] + nu[2] * h[2]).abs() as f32;
    (light.ambient + light.diffuse * ndotl + light.specular * ndoth.powf(light.shininess))
        .clamp(0.15, 1.15)
}

pub(super) fn is_fin3(p: Point3D) -> bool {
    p[0].is_finite() && p[1].is_finite() && p[2].is_finite()
}
