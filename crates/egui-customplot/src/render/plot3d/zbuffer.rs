use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape};

use super::{
    scene::{Marker3D, Scene3D, Seg3D, Tri3D},
    volshow::composite_volumes_zbuffer,
    zbuffer_cache::{MAX_CACHED_ZBUF_SPANS, ZBufferCache},
};
use crate::{Layer3D, Plot3DDiagnostics, Plot3DState, Transform3D};

const MAX_ZBUF_DIM: usize = 360;

pub(super) fn zbuffer_dims(plot_area: Rect) -> (usize, usize) {
    let pw = plot_area.width().max(2.0);
    let ph = plot_area.height().max(2.0);
    let scale = (MAX_ZBUF_DIM as f32 / pw.max(ph)).min(1.0);
    (
        ((pw * scale).round() as usize).clamp(2, MAX_ZBUF_DIM),
        ((ph * scale).round() as usize).clamp(2, MAX_ZBUF_DIM),
    )
}

/// Software Z-buffer rasterizing 3D primitives into `WHITE_UV` horizontal RLE quad batches.
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_zbuffer(
    scene: &Scene3D,
    layers: &[Layer3D<'_>],
    state: &Plot3DState,
    transform: Transform3D,
    painter: &Painter,
    plot_area: Rect,
    text_color: Color32,
    key: u64,
    before_render: Plot3DDiagnostics,
    diagnostics: &mut Plot3DDiagnostics,
) -> Option<ZBufferCache> {
    let clipped = painter.with_clip_rect(plot_area);
    let pw = plot_area.width().max(2.0);
    let ph = plot_area.height().max(2.0);
    let (w, h) = zbuffer_dims(plot_area);
    let px_count = w * h;
    let mut depth = vec![f32::INFINITY; px_count];
    let mut rgba: Vec<[u8; 4]> = vec![[0; 4]; px_count];
    diagnostics.zbuffer_pixels += px_count;

    // 1. Opaque triangles
    for t in &scene.tris {
        if t.opaque {
            raster_tri(&mut depth, &mut rgba, w, h, plot_area, t, true);
            diagnostics.output_triangles += 1;
        }
    }
    // 2. Segments with depth test
    for s in &scene.segs {
        raster_seg(&mut depth, &mut rgba, w, h, plot_area, s);
        diagnostics.output_segments += 1;
    }
    // 3. Markers with depth test
    for m in &scene.markers {
        raster_marker(&mut depth, &mut rgba, w, h, plot_area, m);
        diagnostics.output_markers += 1;
    }
    // 4. Translucent triangles sorted back-to-front tested against opaque depth
    let mut trans: Vec<(usize, f32)> = scene
        .tris
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.opaque)
        .map(|(i, t)| (i, (t.depth[0] + t.depth[1] + t.depth[2]) / 3.0))
        .collect();
    trans.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (idx, _) in trans {
        raster_tri(
            &mut depth,
            &mut rgba,
            w,
            h,
            plot_area,
            &scene.tris[idx],
            false,
        );
        diagnostics.output_triangles += 1;
    }

    // 4b. Direct volume raycast (depth-occluded by opaque primitives)
    composite_volumes_zbuffer(
        layers,
        state,
        transform,
        &depth,
        &mut rgba,
        w,
        h,
        plot_area,
        diagnostics,
    );

    // 5. Horizontal run-length encode painted pixels into WHITE_UV Mesh quads
    let cw = pw / w as f32;
    let ch = ph / h as f32;
    let mut mesh = Mesh::default();
    let mut spans = 0usize;
    let mut norm_spans: Vec<([f32; 4], Color32)> = Vec::new();
    let mut cache_overflow = false;
    let inv_w = 1.0 / w as f32;
    let inv_h = 1.0 / h as f32;
    for py in 0..h {
        let row = &rgba[py * w..(py + 1) * w];
        let v0 = py as f32 * inv_h;
        let v1 = (py + 1) as f32 * inv_h;
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
            let r = Rect::from_min_max(
                Pos2::new(
                    plot_area.left() + x as f32 * cw,
                    plot_area.top() + py as f32 * ch,
                ),
                Pos2::new(
                    plot_area.left() + end as f32 * cw,
                    plot_area.top() + (py + 1) as f32 * ch,
                ),
            );
            let c32 = Color32::from_rgba_unmultiplied(col[0], col[1], col[2], col[3]);
            mesh.add_colored_rect(r, c32);
            spans += 1;
            if !cache_overflow {
                if norm_spans.len() < MAX_CACHED_ZBUF_SPANS {
                    norm_spans.push(([x as f32 * inv_w, v0, end as f32 * inv_w, v1], c32));
                } else {
                    norm_spans.clear();
                    cache_overflow = true;
                }
            }
            if mesh.indices.len() >= 12_000 * 6 {
                clipped.add(Shape::mesh(std::mem::take(&mut mesh)));
            }
            x = end;
        }
    }
    if !mesh.is_empty() {
        clipped.add(Shape::mesh(mesh));
    }
    diagnostics.zbuffer_spans += spans;
    diagnostics.clipped_items += scene.clipped;

    let mut labels = Vec::new();
    for lbl in &scene.labels {
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
            labels.push((
                [
                    (lbl.pos.x - plot_area.left()) / pw,
                    (lbl.pos.y - plot_area.top()) / ph,
                ],
                lbl.text.clone(),
                lbl.color,
            ));
        }
    }

    (!cache_overflow).then_some(ZBufferCache {
        key,
        norm_spans,
        labels,
        visited_items: diagnostics.visited_items - before_render.visited_items,
        output_planes: diagnostics.output_planes - before_render.output_planes,
        zbuffer_pixels: diagnostics.zbuffer_pixels - before_render.zbuffer_pixels,
        zbuffer_spans: diagnostics.zbuffer_spans - before_render.zbuffer_spans,
        output_triangles: diagnostics.output_triangles - before_render.output_triangles,
        output_segments: diagnostics.output_segments - before_render.output_segments,
        output_markers: diagnostics.output_markers - before_render.output_markers,
        clipped_items: diagnostics.clipped_items - before_render.clipped_items,
    })
}

fn to_buf(p: Pos2, area: Rect, w: usize, h: usize) -> [f32; 2] {
    [
        (p.x - area.left()) / area.width() * w as f32,
        (p.y - area.top()) / area.height() * h as f32,
    ]
}

#[inline]
fn clamp_box(
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    w: usize,
    h: usize,
) -> Option<[usize; 4]> {
    if w == 0
        || h == 0
        || !min_x.is_finite()
        || !max_x.is_finite()
        || !min_y.is_finite()
        || !max_y.is_finite()
    {
        return None;
    }
    let (wf, hf) = (w as f32, h as f32);
    if max_x < 0.0 || min_x >= wf || max_y < 0.0 || min_y >= hf {
        return None;
    }
    let x0 = (min_x.floor().max(0.0) as usize).min(w - 1);
    let x1 = (max_x.ceil().max(0.0) as usize).min(w - 1);
    let y0 = (min_y.floor().max(0.0) as usize).min(h - 1);
    let y1 = (max_y.ceil().max(0.0) as usize).min(h - 1);
    (x0 <= x1 && y0 <= y1).then_some([x0, x1, y0, y1])
}

fn raster_tri(
    depth: &mut [f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    area: Rect,
    t: &Tri3D,
    opaque: bool,
) {
    let a = to_buf(t.pos[0], area, w, h);
    let b = to_buf(t.pos[1], area, w, h);
    let c = to_buf(t.pos[2], area, w, h);
    let area2 = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if !area2.is_finite() || area2.abs() < 1e-4 {
        return;
    }
    let sign = area2.signum();
    let inv = 1.0 / area2;
    let Some([x0, x1, y0, y1]) = clamp_box(
        a[0].min(b[0]).min(c[0]),
        a[0].max(b[0]).max(c[0]),
        a[1].min(b[1]).min(c[1]),
        a[1].max(b[1]).max(c[1]),
        w,
        h,
    ) else {
        return;
    };
    let c0 = t.colors[0].to_srgba_unmultiplied();
    let c1 = t.colors[1].to_srgba_unmultiplied();
    let c2 = t.colors[2].to_srgba_unmultiplied();
    let uniform = c0 == c1 && c1 == c2;

    for py in y0..=y1 {
        let fy = py as f32 + 0.5;
        for px in x0..=x1 {
            let fx = px as f32 + 0.5;
            let e0 = ((c[0] - b[0]) * (fy - b[1]) - (c[1] - b[1]) * (fx - b[0])) * sign;
            let e1 = ((a[0] - c[0]) * (fy - c[1]) - (a[1] - c[1]) * (fx - c[0])) * sign;
            let e2 = ((b[0] - a[0]) * (fy - a[1]) - (b[1] - a[1]) * (fx - a[0])) * sign;
            if e0 >= -0.01 && e1 >= -0.01 && e2 >= -0.01 {
                let l0 = (e0 * sign * inv).clamp(0.0, 1.0);
                let l1 = (e1 * sign * inv).clamp(0.0, 1.0);
                let l2 = (e2 * sign * inv).clamp(0.0, 1.0);
                let sum = (l0 + l1 + l2).max(1e-6);
                let (w0, w1, w2) = (l0 / sum, l1 / sum, l2 / sum);
                let z = w0 * t.depth[0] + w1 * t.depth[1] + w2 * t.depth[2];
                let col = if uniform {
                    c0
                } else {
                    std::array::from_fn(|k| {
                        (w0 * f32::from(c0[k]) + w1 * f32::from(c1[k]) + w2 * f32::from(c2[k]))
                            .round()
                            .clamp(0.0, 255.0) as u8
                    })
                };
                let idx = py * w + px;
                if opaque {
                    if z <= depth[idx] {
                        depth[idx] = z;
                        rgba[idx] = col;
                    }
                } else if z <= depth[idx] + 1e-4 {
                    rgba[idx] = blend_over(rgba[idx], col);
                }
            }
        }
    }
}

fn raster_seg(depth: &mut [f32], rgba: &mut [[u8; 4]], w: usize, h: usize, area: Rect, s: &Seg3D) {
    let a = to_buf(s.p0, area, w, h);
    let b = to_buf(s.p1, area, w, h);
    let (wf, hf) = (w as f32, h as f32);
    if !a[0].is_finite()
        || !a[1].is_finite()
        || !b[0].is_finite()
        || !b[1].is_finite()
        || (a[0] < 0.0 && b[0] < 0.0)
        || (a[0] >= wf && b[0] >= wf)
        || (a[1] < 0.0 && b[1] < 0.0)
        || (a[1] >= hf && b[1] >= hf)
    {
        return;
    }
    let steps = (a[0] - b[0]).hypot(a[1] - b[1]).ceil().clamp(1.0, 1024.0) as usize;
    let col = s.stroke.color.to_srgba_unmultiplied();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = (a[0] + (b[0] - a[0]) * t).floor() as i32;
        let y = (a[1] + (b[1] - a[1]) * t).floor() as i32;
        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let z = s.depth[0] + (s.depth[1] - s.depth[0]) * t - 2e-4;
            let idx = y as usize * w + x as usize;
            if z <= depth[idx] {
                depth[idx] = z;
                rgba[idx] = if col[3] >= 250 {
                    col
                } else {
                    blend_over(rgba[idx], col)
                };
            }
        }
    }
}

fn raster_marker(
    depth: &mut [f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    area: Rect,
    m: &Marker3D,
) {
    let c = to_buf(m.pos, area, w, h);
    let r_px = (m.radius * (w as f32 / area.width().max(1.0))).max(1.2);
    if !r_px.is_finite() {
        return;
    }
    let Some([x0, x1, y0, y1]) =
        clamp_box(c[0] - r_px, c[0] + r_px, c[1] - r_px, c[1] + r_px, w, h)
    else {
        return;
    };
    let r2 = r_px * r_px;
    let col = m.color.to_srgba_unmultiplied();
    let z = m.depth - 3e-4;
    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px as f32 + 0.5 - c[0];
            let dy = py as f32 + 0.5 - c[1];
            if dx * dx + dy * dy <= r2 {
                let idx = py * w + px;
                if z <= depth[idx] {
                    if col[3] >= 250 {
                        depth[idx] = z;
                        rgba[idx] = col;
                    } else {
                        rgba[idx] = blend_over(rgba[idx], col);
                    }
                }
            }
        }
    }
}

fn blend_over(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let sa = f32::from(src[3]) / 255.0;
    let da = f32::from(dst[3]) / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 1e-4 {
        return [0; 4];
    }
    let mix = |s: u8, d: u8| -> u8 {
        ((f32::from(s) * sa + f32::from(d) * da * (1.0 - sa)) / out_a).clamp(0.0, 255.0) as u8
    };
    [
        mix(src[0], dst[0]),
        mix(src[1], dst[1]),
        mix(src[2], dst[2]),
        (out_a * 255.0) as u8,
    ]
}
