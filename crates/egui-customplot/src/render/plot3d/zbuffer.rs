mod raster;
use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape};
use raster::{raster_marker, raster_seg, raster_tri};

use super::{
    scene::Scene3D,
    volshow::composite_volumes_zbuffer,
    zbuffer_cache::{MAX_CACHED_ZBUF_SPANS, ZBufferCache},
};
use crate::{Layer3D, Plot3DDiagnostics, Plot3DState, Transform3D};

const MAX_ZBUF_DIM: usize = 360;

pub(super) fn zbuffer_dims(plot_area: Rect, density: Option<f32>) -> (usize, usize) {
    let pw = plot_area.width().max(2.0);
    let ph = plot_area.height().max(2.0);
    let limit = if density.is_some() {
        2048
    } else {
        MAX_ZBUF_DIM
    };
    let scale = (limit as f32 / pw.max(ph)).min(density.unwrap_or(1.0));
    (
        ((pw * scale).round() as usize).clamp(2, limit),
        ((ph * scale).round() as usize).clamp(2, limit),
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
    dims: (usize, usize),
    before_render: Plot3DDiagnostics,
    diagnostics: &mut Plot3DDiagnostics,
) -> Option<ZBufferCache> {
    let clipped = painter.with_clip_rect(plot_area);
    let pw = plot_area.width().max(2.0);
    let ph = plot_area.height().max(2.0);
    let (w, h) = dims;
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
