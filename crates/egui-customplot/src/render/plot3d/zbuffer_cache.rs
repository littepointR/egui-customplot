use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use egui::{Align2, Color32, Mesh, Painter, Pos2, Rect, Shape};

use crate::{
    BoxStyle3D, ConstantPlane3D, Layer3D, Plot3DDiagnostics, Plot3DState, Text3D, Transform3D,
    VolumeRenderMode, model::ResolvedPlotStyle,
};

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
    pub clipped_items: usize,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compute_zbuffer_key(
    state: &Plot3DState,
    layers: &[Layer3D<'_>],
    planes: &[ConstantPlane3D],
    texts: &[Text3D],
    transform: Transform3D,
    box_style: BoxStyle3D,
    style: &ResolvedPlotStyle<'_>,
    w: usize,
    h: usize,
) -> u64 {
    let mut hsh = DefaultHasher::new();
    w.hash(&mut hsh);
    h.hash(&mut hsh);
    let r = transform.rect();
    r.width().to_bits().hash(&mut hsh);
    r.height().to_bits().hash(&mut hsh);
    (box_style as u8).hash(&mut hsh);
    style.background.hash(&mut hsh);
    style.text_color.hash(&mut hsh);

    let cam = transform.camera();
    cam.azimuth_deg.to_bits().hash(&mut hsh);
    cam.elevation_deg.to_bits().hash(&mut hsh);
    cam.zoom.to_bits().hash(&mut hsh);
    cam.pan[0].to_bits().hash(&mut hsh);
    cam.pan[1].to_bits().hash(&mut hsh);
    match cam.projection {
        crate::Projection3D::Orthographic => 0u8.hash(&mut hsh),
        crate::Projection3D::Perspective { fov_y_rad } => {
            1u8.hash(&mut hsh);
            fov_y_rad.to_bits().hash(&mut hsh);
        }
    }
    let b = transform.bounds();
    for v in b.min().into_iter().chain(b.max()) {
        v.to_bits().hash(&mut hsh);
    }
    for s in transform.scales() {
        (s as u8).hash(&mut hsh);
    }
    transform.reversed().hash(&mut hsh);
    for bs in transform.box_scale() {
        bs.to_bits().hash(&mut hsh);
    }
    (planes.as_ptr() as usize, planes.len()).hash(&mut hsh);
    (texts.as_ptr() as usize, texts.len()).hash(&mut hsh);
    for t in texts {
        t.text.hash(&mut hsh);
    }
    layers.len().hash(&mut hsh);

    for (idx, &layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        state.layer_visible(id).hash(&mut hsh);
        std::mem::discriminant(&layer).hash(&mut hsh);
        layer.name().hash(&mut hsh);
        let rs = layer.resolved_style(idx, style);
        rs.stroke.color.hash(&mut hsh);
        rs.stroke.width.to_bits().hash(&mut hsh);
        rs.marker.radius.to_bits().hash(&mut hsh);
        (rs.marker.shape as u8).hash(&mut hsh);
        hash_layer_payload(layer, &mut hsh);
    }
    hsh.finish()
}

fn hash_layer_payload(layer: Layer3D<'_>, hsh: &mut DefaultHasher) {
    match layer {
        Layer3D::Line(s) => {
            s.data.allocation().hash(hsh);
            s.show_markers.hash(hsh);
        }
        Layer3D::Scatter(s) => s.data.allocation().hash(hsh),
        Layer3D::Bubble(s) => {
            s.data.allocation().hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Swarm(s) => s.data.allocation().hash(hsh),
        Layer3D::Comet(s) => {
            s.data.allocation().hash(hsh);
            s.playback.progress_points().to_bits().hash(hsh);
            s.playback.tail_fraction_value().to_bits().hash(hsh);
        }
        Layer3D::Bar(s) => s.data.allocation().hash(hsh),
        Layer3D::Stem(s) => {
            s.data.allocation().hash(hsh);
            s.show_base.hash(hsh);
        }
        Layer3D::Pie(s) => s.data.allocation().hash(hsh),
        Layer3D::Quiver(s) => s.data.allocation().hash(hsh),
        Layer3D::Fill(s) => {
            s.data.allocation().hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Surface(s) => {
            s.data.allocation().hash(hsh);
            (s.shading as u8).hash(hsh);
            s.floor_contours.hash(hsh);
            s.show_normals.hash(hsh);
            s.colormap_lighting.hash(hsh);
        }
        Layer3D::MeshSurface(s) => {
            s.data.allocation().hash(hsh);
            s.hidden_line.hash(hsh);
            s.floor_contours.hash(hsh);
        }
        Layer3D::Contour(s) => s.data.allocation().hash(hsh),
        Layer3D::Ribbon(s) => {
            s.data.allocation().hash(hsh);
            (s.shading as u8).hash(hsh);
        }
        Layer3D::TriSurf(s) => {
            s.data.map(|m| m.allocation()).hash(hsh);
            s.implicit.map(|i| i.allocation()).hash(hsh);
        }
        Layer3D::VolumeSlice(s) => {
            s.data.allocation().hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Isosurface(s) => s.data.mesh().map(|m| m.allocation()).hash(hsh),
        Layer3D::Isocaps(s) => s.data.mesh().map(|m| m.allocation()).hash(hsh),
        Layer3D::TetraMesh(s) => {
            s.data.allocation().hash(hsh);
            s.boundary_only.hash(hsh);
        }
        Layer3D::Streamline(s) => s.data.allocation().hash(hsh),
        Layer3D::StreamParticles(s) => {
            s.data.allocation().hash(hsh);
            s.playback.phase().to_bits().hash(hsh);
        }
        Layer3D::StreamRibbon(s) => s.data.allocation().hash(hsh),
        Layer3D::StreamTube(s) => s.data.allocation().hash(hsh),
        Layer3D::ConePlot(s) => s.data.allocation().hash(hsh),
        Layer3D::Patch(s) => {
            s.data.allocation().hash(hsh);
            (s.shading as u8).hash(hsh);
            s.light.shininess.to_bits().hash(hsh);
            s.light.specular.to_bits().hash(hsh);
        }
        Layer3D::AlphaShape(s) => {
            s.data.allocation().hash(hsh);
            (s.shading as u8).hash(hsh);
        }
        Layer3D::Graph(s) => {
            s.data.allocation().hash(hsh);
            s.show_node_labels.hash(hsh);
        }
        Layer3D::VolumeRender(s) => {
            (s.volume.values().as_ptr() as usize, s.volume.values().len()).hash(hsh);
            s.sample_steps.hash(hsh);
            s.max_resolution.hash(hsh);
            s.shading.hash(hsh);
            s.opacity_scale.to_bits().hash(hsh);
            s.transfer.lut_bytes().hash(hsh);
            match s.mode {
                VolumeRenderMode::DirectAlpha => 0u8.hash(hsh),
                VolumeRenderMode::MaximumIntensity => 1u8.hash(hsh),
                VolumeRenderMode::IsosurfaceRay { threshold } => {
                    2u8.hash(hsh);
                    threshold.to_bits().hash(hsh);
                }
            }
        }
        Layer3D::PointCloud(s) => {
            s.data.allocation().hash(hsh);
            s.max_points.hash(hsh);
            s.pixel_cell.to_bits().hash(hsh);
            (s.color_map as u8).hash(hsh);
            s.style.marker_radius.map(f32::to_bits).hash(hsh);
        }
    }
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
    diagnostics.output_triangles += cache.output_triangles;
    diagnostics.output_segments += cache.output_segments;
    diagnostics.output_markers += cache.output_markers;
    diagnostics.clipped_items += cache.clipped_items;
}
