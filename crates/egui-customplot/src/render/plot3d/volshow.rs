use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape};

use super::{layers::push_marker_data, scene::Scene3D, volshow_raycast::raycast_into_buffer};
use crate::{
    ColorScale, Layer3D, MarkerShape, Plot3DDiagnostics, Plot3DState, PointCloud3D, Transform3D,
};

pub(super) fn collect_point_cloud3d(
    scene: &mut Scene3D,
    pc: PointCloud3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let indices = pc.data.query_lod(transform, pc.max_points, pc.pixel_cell);
    let shape = pc.style.marker_shape.unwrap_or(MarkerShape::Circle);
    let radius = pc.style.marker_radius.unwrap_or(2.6);
    let base = pc.style.color.unwrap_or(fallback);
    let rgb = pc.data.rgb();
    let intensity = pc.data.intensity();
    let i_scale = pc.color_scale.or_else(|| {
        pc.data.intensity_range().and_then(|[lo, hi]| {
            ColorScale::new([f64::from(lo), f64::from(hi)])
                .ok()
                .map(|cs| cs.color_map(pc.color_map))
        })
    });
    let pts = pc.data.points();

    for idx in indices {
        diagnostics.visited_items += 1;
        let col = if let Some(cols) = rgb {
            let [r, g, b] = cols[idx];
            Color32::from_rgb(r, g, b)
        } else if let (Some(vals), Some(sc)) = (intensity, i_scale) {
            sc.color(f64::from(vals[idx])).unwrap_or(base)
        } else {
            base
        };
        push_marker_data(scene, pts[idx], radius, col, shape, transform);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn composite_volumes_zbuffer(
    layers: &[Layer3D<'_>],
    state: &Plot3DState,
    transform: Transform3D,
    depth: &[f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    plot_area: Rect,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (idx, &layer) in layers.iter().enumerate() {
        let Layer3D::VolumeRender(vr) = layer else {
            continue;
        };
        if !state.layer_visible(layer.id(idx)) {
            continue;
        }
        let max_dim = vr.max_resolution.clamp(16, 160) as f32;
        let pw = plot_area.width().max(2.0);
        let ph = plot_area.height().max(2.0);
        let scale = (max_dim / pw.max(ph)).min(1.0);
        let vw = (((pw * scale).round() as usize).clamp(2, 160)).min(w);
        let vh = (((ph * scale).round() as usize).clamp(2, 160)).min(h);
        if vw == w && vh == h {
            raycast_into_buffer(
                vr,
                transform,
                Some(depth),
                rgba,
                None,
                w,
                h,
                plot_area,
                diagnostics,
            );
            continue;
        }
        let mut c_rgba = vec![[0u8; 4]; vw * vh];
        let mut c_depth = vec![f32::INFINITY; vw * vh];
        let mut c_hit_t = vec![f32::INFINITY; vw * vh];
        for cy in 0..vh {
            let fy0 = cy * h / vh;
            let fy1 = (((cy + 1) * h).div_ceil(vh)).min(h);
            for cx in 0..vw {
                let fx0 = cx * w / vw;
                let fx1 = (((cx + 1) * w).div_ceil(vw)).min(w);
                let mut max_d = 0.0_f32;
                for fy in fy0..fy1 {
                    for fx in fx0..fx1 {
                        max_d = max_d.max(depth[fy * w + fx]);
                        if max_d.is_infinite() {
                            break;
                        }
                    }
                    if max_d.is_infinite() {
                        break;
                    }
                }
                c_depth[cy * vw + cx] = max_d;
            }
        }
        raycast_into_buffer(
            vr,
            transform,
            Some(&c_depth),
            &mut c_rgba,
            Some(&mut c_hit_t),
            vw,
            vh,
            plot_area,
            diagnostics,
        );
        composite_coarse_onto_zbuffer(&c_rgba, &c_hit_t, vw, vh, depth, rgba, w, h, diagnostics);
    }
}

pub(super) fn paint_volumes_painter(
    layers: &[Layer3D<'_>],
    state: &Plot3DState,
    transform: Transform3D,
    painter: &Painter,
    plot_area: Rect,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (idx, &layer) in layers.iter().enumerate() {
        let Layer3D::VolumeRender(vr) = layer else {
            continue;
        };
        if !state.layer_visible(layer.id(idx)) {
            continue;
        }
        let max_dim = vr.max_resolution.clamp(16, 160) as f32;
        let pw = plot_area.width().max(2.0);
        let ph = plot_area.height().max(2.0);
        let scale = (max_dim / pw.max(ph)).min(1.0);
        let w = ((pw * scale).round() as usize).clamp(2, 160);
        let h = ((ph * scale).round() as usize).clamp(2, 160);
        let mut rgba = vec![[0u8; 4]; w * h];
        raycast_into_buffer(
            vr,
            transform,
            None,
            &mut rgba,
            None,
            w,
            h,
            plot_area,
            diagnostics,
        );
        emit_rle_quads(&rgba, w, h, plot_area, painter, diagnostics);
    }
}

#[allow(clippy::too_many_arguments)]
fn composite_coarse_onto_zbuffer(
    c_rgba: &[[u8; 4]],
    c_hit_t: &[f32],
    vw: usize,
    vh: usize,
    depth: &[f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let sx = vw as f32 / w as f32;
    let sy = vh as f32 / h as f32;
    for py in 0..h {
        let cyf = ((py as f32 + 0.5) * sy - 0.5).clamp(0.0, (vh - 1) as f32);
        let cy0 = cyf.floor() as usize;
        let cy1 = (cy0 + 1).min(vh - 1);
        let wy = cyf - cy0 as f32;
        for px in 0..w {
            let cxf = ((px as f32 + 0.5) * sx - 0.5).clamp(0.0, (vw - 1) as f32);
            let cx0 = cxf.floor() as usize;
            let cx1 = (cx0 + 1).min(vw - 1);
            let wx = cxf - cx0 as f32;
            let idx4 = [
                cy0 * vw + cx0,
                cy0 * vw + cx1,
                cy1 * vw + cx0,
                cy1 * vw + cx1,
            ];
            let wt = [
                (1.0 - wx) * (1.0 - wy),
                wx * (1.0 - wy),
                (1.0 - wx) * wy,
                wx * wy,
            ];
            let full_idx = py * w + px;
            let z_buf = depth[full_idx];
            let (mut pr, mut pg, mut pb, mut pa) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
            for k in 0..4 {
                let c = c_rgba[idx4[k]];
                if c[3] == 0 || c_hit_t[idx4[k]] > z_buf + 0.02 {
                    continue;
                }
                let a = (f32::from(c[3]) / 255.0) * wt[k];
                pr += (f32::from(c[0]) / 255.0) * a;
                pg += (f32::from(c[1]) / 255.0) * a;
                pb += (f32::from(c[2]) / 255.0) * a;
                pa += a;
            }
            if pa > 0.01 {
                let inv = 1.0 / pa;
                let a = pa.clamp(0.0, 1.0);
                let bg = rgba[full_idx];
                let bg_a = f32::from(bg[3]) / 255.0;
                let rem = 1.0 - a;
                let out_a = (a + bg_a * rem).clamp(0.0, 1.0);
                let mix = |fg: f32, bg_u8: u8| {
                    let val = (fg * a + (f32::from(bg_u8) / 255.0) * bg_a * rem) / out_a.max(1e-5);
                    (val * 255.0).round().clamp(0.0, 255.0) as u8
                };
                rgba[full_idx] = [
                    mix((pr * inv).clamp(0.0, 1.0), bg[0]),
                    mix((pg * inv).clamp(0.0, 1.0), bg[1]),
                    mix((pb * inv).clamp(0.0, 1.0), bg[2]),
                    (out_a * 255.0).round() as u8,
                ];
                diagnostics.zbuffer_pixels += 1;
            }
        }
    }
}

fn emit_rle_quads(
    rgba: &[[u8; 4]],
    w: usize,
    h: usize,
    area: Rect,
    painter: &Painter,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let cw = area.width() / w as f32;
    let ch = area.height() / h as f32;
    let mut mesh = Mesh::default();
    let clipped = painter.with_clip_rect(area);
    for py in 0..h {
        let row = &rgba[py * w..(py + 1) * w];
        let mut x = 0usize;
        while x < w {
            let col = row[x];
            if col[3] == 0 {
                x += 1;
                continue;
            }
            let mut end = x + 1;
            while end < w && row[end] == col {
                end += 1;
            }
            let rect = Rect::from_min_max(
                Pos2::new(area.left() + x as f32 * cw, area.top() + py as f32 * ch),
                Pos2::new(
                    area.left() + end as f32 * cw,
                    area.top() + (py + 1) as f32 * ch,
                ),
            );
            mesh.add_colored_rect(
                rect,
                Color32::from_rgba_unmultiplied(col[0], col[1], col[2], col[3]),
            );
            diagnostics.zbuffer_spans += 1;
            x = end;
        }
    }
    if !mesh.is_empty() {
        clipped.add(Shape::mesh(mesh));
    }
}
