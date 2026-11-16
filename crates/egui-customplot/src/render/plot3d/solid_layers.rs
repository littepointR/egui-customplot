use std::f64::consts::PI;

use egui::{Color32, Stroke};

use super::scene::{Label3D, Scene3D, Seg3D, scale_rgb};
use crate::{
    ConstantPlane3D, Fill3D, Pie3D, Plot3DDiagnostics, Point3D, Quiver3D, Transform3D,
    axis::intersect_plane_unit_cube, model::ResolvedPlotStyle,
};

pub(super) fn collect_quiver3(
    scene: &mut Scene3D,
    q: Quiver3D<'_>,
    stroke: Stroke,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let scale = q.data.scale_factor();
    for &arr in q.data.arrows() {
        diagnostics.visited_items += 1;
        let Some(end) = arr.endpoint(scale) else {
            continue;
        };
        scene.push_data_segment(arr.origin, end, stroke, transform);
        if q.show_heads {
            push_quiver3_head(scene, arr.origin, end, q.head_size, stroke.color, transform);
        }
    }
}

pub(super) fn collect_fill3(
    scene: &mut Scene3D,
    fill: Fill3D<'_>,
    fallback_color: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (pi, poly) in fill.data.polygons().iter().enumerate() {
        diagnostics.visited_items += 1;
        let c = fill
            .face_colors
            .and_then(|fc| fc.get(pi).copied())
            .unwrap_or(fallback_color)
            .gamma_multiply(fill.alpha);
        for &[i, j, k] in poly.triangles.iter() {
            scene.push_shaded_tri_data(
                [poly.vertices[i], poly.vertices[j], poly.vertices[k]],
                c,
                transform,
            );
        }
        if let Some(border) = fill.border {
            for i in 0..poly.vertices.len() {
                scene.push_data_segment(
                    poly.vertices[i],
                    poly.vertices[(i + 1) % poly.vertices.len()],
                    border,
                    transform,
                );
            }
        }
    }
}

pub(super) fn collect_pie3(
    scene: &mut Scene3D,
    pie: Pie3D<'_>,
    transform: Transform3D,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let data = pie.data;
    let h = data.thickness();
    let expl = data.offset();
    for slice in data.slices() {
        diagnostics.visited_items += 1;
        let color = style
            .resolve_series(slice.index, Default::default())
            .stroke
            .color;
        let mid_ang = 0.5 * (slice.start_angle_rad + slice.end_angle_rad);
        let shift = if slice.exploded {
            [expl * mid_ang.cos(), expl * mid_ang.sin()]
        } else {
            [0.0, 0.0]
        };
        let steps = (((slice.end_angle_rad - slice.start_angle_rad) / (PI / 18.0)).ceil() as usize)
            .clamp(4, 36);
        let c_bot = [shift[0], shift[1], 0.0];
        let c_top = [shift[0], shift[1], h];
        let rim = |i: usize| -> (Point3D, Point3D) {
            let a = slice.start_angle_rad
                + (slice.end_angle_rad - slice.start_angle_rad) * (i as f64 / steps as f64);
            let x = shift[0] + a.cos();
            let y = shift[1] + a.sin();
            ([x, y, 0.0], [x, y, h])
        };
        let (b0, t0) = rim(0);
        let (bn, tn) = rim(steps);
        for quad in [[c_bot, b0, t0, c_top], [c_bot, c_top, tn, bn]] {
            scene.push_shaded_tri_data(
                [quad[0], quad[1], quad[2]],
                scale_rgb(color, 0.78),
                transform,
            );
            scene.push_shaded_tri_data(
                [quad[0], quad[2], quad[3]],
                scale_rgb(color, 0.78),
                transform,
            );
        }
        for i in 0..steps {
            let (bi0, ti0) = rim(i);
            let (bi1, ti1) = rim(i + 1);
            scene.push_shaded_tri_data([c_top, ti0, ti1], color, transform);
            scene.push_shaded_tri_data([c_bot, bi1, bi0], scale_rgb(color, 0.65), transform);
            scene.push_shaded_tri_data([bi0, bi1, ti1], scale_rgb(color, 0.85), transform);
            scene.push_shaded_tri_data([bi0, ti1, ti0], scale_rgb(color, 0.85), transform);
            if let Some(stroke) = pie.border {
                scene.push_data_segment(ti0, ti1, stroke, transform);
            }
        }
        if let Some(stroke) = pie.border {
            scene.push_data_segment(c_top, t0, stroke, transform);
            scene.push_data_segment(c_top, tn, stroke, transform);
        }
        if pie.show_labels
            && let Some((pos, _)) = transform.project_data([
                shift[0] + 0.68 * mid_ang.cos(),
                shift[1] + 0.68 * mid_ang.sin(),
                h + 0.06,
            ])
        {
            scene.labels.push(Label3D {
                pos,
                text: slice.label.clone(),
                color: Color32::PLACEHOLDER,
            });
        }
    }
}

pub(super) fn collect_constant_plane(
    scene: &mut Scene3D,
    plane: &ConstantPlane3D,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let bounds = transform.bounds();
    let rev = transform.reversed();
    let span = bounds.span();
    let bmin = bounds.min();
    let bmax = bounds.max();
    let mut coef = [0.0; 3];
    let mut origin_dot = 0.0;
    for i in 0..3 {
        if !plane.normal[i].is_finite() {
            return;
        }
        origin_dot += plane.normal[i] * if rev[i] { bmax[i] } else { bmin[i] };
        coef[i] = plane.normal[i] * if rev[i] { -span[i] } else { span[i] };
    }
    let rhs = plane.offset - origin_dot;
    let poly = intersect_plane_unit_cube(coef, rhs);
    if poly.len() < 3 {
        return;
    }
    diagnostics.output_planes += 1;
    for i in 1..(poly.len() - 1) {
        scene.push_unit_tri([poly[0], poly[i], poly[i + 1]], plane.fill, transform);
    }
    if let Some(border) = plane.border {
        for i in 0..poly.len() {
            let j = (i + 1) % poly.len();
            if let (Some((pa, da)), Some((pb, db))) = (
                transform.project_unit(poly[i]),
                transform.project_unit(poly[j]),
            ) {
                scene.segs.push(Seg3D {
                    p0: pa,
                    p1: pb,
                    depth: [da, db],
                    stroke: border,
                });
            }
        }
    }
    if !plane.label.is_empty() {
        let inv = 1.0 / poly.len() as f64;
        let center = [
            poly.iter().map(|p| p[0]).sum::<f64>() * inv,
            poly.iter().map(|p| p[1]).sum::<f64>() * inv,
            poly.iter().map(|p| p[2]).sum::<f64>() * inv,
        ];
        if let Some((pos, _)) = transform.project_unit(center) {
            scene.labels.push(Label3D {
                pos,
                text: plane.label.clone(),
                color: Color32::PLACEHOLDER,
            });
        }
    }
}

pub(super) fn push_quiver3_head(
    scene: &mut Scene3D,
    origin: Point3D,
    end: Point3D,
    head_px: f32,
    color: Color32,
    transform: Transform3D,
) {
    let (Some((p0, _)), Some((p1, _))) =
        (transform.project_data(origin), transform.project_data(end))
    else {
        return;
    };
    let len = (p1 - p0).length();
    if len <= 1.0 {
        return;
    }
    let span = transform.bounds().span();
    let dvec = [end[0] - origin[0], end[1] - origin[1], end[2] - origin[2]];
    let dlen = (dvec[0] * dvec[0] + dvec[1] * dvec[1] + dvec[2] * dvec[2])
        .sqrt()
        .max(1e-12);
    let frac = (f64::from(head_px) / f64::from(len) * 1.1).min(0.35);
    let dir = [
        dvec[0] / dlen * frac * span[0] * 0.25,
        dvec[1] / dlen * frac * span[1] * 0.25,
        dvec[2] / dlen * frac * span[2] * 0.25,
    ];
    let back = [end[0] - dir[0], end[1] - dir[1], end[2] - dir[2]];
    let up = if (dvec[2] / dlen).abs() < 0.85 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let u = norm_scaled(cross(dvec, up), span, frac * 0.12);
    let v = norm_scaled(cross(dvec, u), span, frac * 0.12);
    let base = [
        [back[0] + u[0], back[1] + u[1], back[2] + u[2]],
        [back[0] + v[0], back[1] + v[1], back[2] + v[2]],
        [back[0] - u[0], back[1] - u[1], back[2] - u[2]],
        [back[0] - v[0], back[1] - v[1], back[2] - v[2]],
    ];
    for i in 0..4 {
        scene.push_shaded_tri_data([end, base[i], base[(i + 1) % 4]], color, transform);
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm_scaled(v: [f64; 3], span: [f64; 3], scale: f64) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-12);
    [
        v[0] / l * scale * span[0],
        v[1] / l * scale * span[1],
        v[2] / l * scale * span[2],
    ]
}
