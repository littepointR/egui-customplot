use egui::{Align2, Color32, Mesh, Painter, Pos2, Rect, Shape};

use crate::Plot3DDiagnostics;

pub(crate) const MAX_CACHED_ZBUF_SPANS: usize = 65_536;

#[derive(Clone, Debug)]
pub(crate) struct ZBufferCache {
    pub key: u64,
    pub norm_spans: Vec<([f32; 4], Color32)>,
    pub labels: Vec<([f32; 2], String, Color32)>,
    pub visited_items: usize,
    pub zbuffer_pixels: usize,
    pub zbuffer_spans: usize,
    pub output_triangles: usize,
    pub output_segments: usize,
    pub output_markers: usize,
    pub output_planes: usize,
    pub clipped_items: usize,
}

pub(super) fn emit_cached_zbuffer(
    cache: &ZBufferCache,
    painter: &Painter,
    plot_area: Rect,
    text_color: Color32,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let clipped = painter.with_clip_rect(plot_area);
    let pw = plot_area.width().max(1.0);
    let ph = plot_area.height().max(1.0);
    let mut mesh = Mesh::default();
    for &(uv, col) in &cache.norm_spans {
        let r = Rect::from_min_max(
            Pos2::new(plot_area.left() + uv[0] * pw, plot_area.top() + uv[1] * ph),
            Pos2::new(plot_area.left() + uv[2] * pw, plot_area.top() + uv[3] * ph),
        );
        mesh.add_colored_rect(r, col);
        if mesh.indices.len() >= 12_000 * 6 {
            clipped.add(Shape::mesh(std::mem::take(&mut mesh)));
        }
    }
    if !mesh.is_empty() {
        clipped.add(Shape::mesh(mesh));
    }
    for (uv, text, color) in &cache.labels {
        let pos = Pos2::new(plot_area.left() + uv[0] * pw, plot_area.top() + uv[1] * ph);
        if plot_area.contains(pos) {
            clipped.text(
                pos,
                Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(11.0),
                if *color == Color32::PLACEHOLDER {
                    text_color
                } else {
                    *color
                },
            );
        }
    }
    diagnostics.visited_items += cache.visited_items;
    diagnostics.zbuffer_pixels += cache.zbuffer_pixels;
    diagnostics.zbuffer_spans += cache.zbuffer_spans;
    diagnostics.output_planes += cache.output_planes;
    diagnostics.output_triangles += cache.output_triangles;
    diagnostics.output_segments += cache.output_segments;
    diagnostics.output_markers += cache.output_markers;
    diagnostics.clipped_items += cache.clipped_items;
}
