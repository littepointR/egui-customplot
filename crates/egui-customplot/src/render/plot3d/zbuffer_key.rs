use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use egui::{Color32, Stroke};

use crate::{
    BoxStyle3D, ColorScale, ConstantPlane3D, Layer3D, LightSource3D, Plot3DState, Text3D,
    Transform3D, VolumeRenderMode, model::ResolvedPlotStyle,
};

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

    if let Some(lens) = transform.lens() {
        for v in lens.coefficients() {
            v.to_bits().hash(&mut hsh);
        }
    }
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
    transform
        .mappings()
        .map(|m| m.function_key())
        .hash(&mut hsh);
    transform.reversed().hash(&mut hsh);
    for bs in transform.box_scale() {
        bs.to_bits().hash(&mut hsh);
    }
    style.palette.hash(&mut hsh);
    style.series.color.hash(&mut hsh);
    planes.len().hash(&mut hsh);
    for plane in planes {
        plane.normal.map(f64::to_bits).hash(&mut hsh);
        plane.offset.to_bits().hash(&mut hsh);
        plane.fill.hash(&mut hsh);
        plane.border.map(stroke_key).hash(&mut hsh);
        plane.grid_divisions.hash(&mut hsh);
        plane.label.hash(&mut hsh);
    }
    texts.len().hash(&mut hsh);
    for t in texts {
        t.position.map(f64::to_bits).hash(&mut hsh);
        t.color.hash(&mut hsh);
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
        // Some layers branch on an explicit override even if its resolved value
        // equals the inherited style (for example, solid color vs colormap).
        let local = layer.local_style();
        local.color.hash(&mut hsh);
        local.line_width.map(f32::to_bits).hash(&mut hsh);
        local.marker_radius.map(f32::to_bits).hash(&mut hsh);
        local.marker_shape.map(|s| s as u8).hash(&mut hsh);
        hash_layer_payload(layer, &mut hsh);
    }
    hsh.finish()
}

fn stroke_key(stroke: Stroke) -> (u32, Color32) {
    (stroke.width.to_bits(), stroke.color)
}

fn scale_key(scale: ColorScale) -> ([u64; 2], u8, u8) {
    (
        scale.range().map(f64::to_bits),
        scale.scale() as u8,
        scale.map() as u8,
    )
}

fn light_key(light: LightSource3D) -> ([u64; 2], [u32; 4]) {
    (
        [light.azimuth_deg, light.elevation_deg].map(f64::to_bits),
        [
            light.ambient,
            light.diffuse,
            light.specular,
            light.shininess,
        ]
        .map(f32::to_bits),
    )
}

// Dataset versions identify immutable snapshots, including scalar, color and
// label builders. Borrowed per-layer overrides are compared by value.
fn hash_layer_payload(layer: Layer3D<'_>, hsh: &mut DefaultHasher) {
    match layer {
        Layer3D::Custom(s) => {
            s.id().hash(hsh);
            s.revision().hash(hsh);
        }
        Layer3D::Line(s) => {
            s.data.version.hash(hsh);
            s.show_markers.hash(hsh);
            s.primitive.hash(hsh);
        }
        Layer3D::Scatter(s) => {
            s.data.version.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
        }
        Layer3D::Bubble(s) => {
            s.data.version.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
            s.border.map(stroke_key).hash(hsh);
        }
        Layer3D::Swarm(s) => {
            s.data.version.hash(hsh);
            s.group_colors.hash(hsh);
        }
        Layer3D::Comet(s) => {
            s.data.version.hash(hsh);
            s.playback.progress_points().to_bits().hash(hsh);
            s.playback.tail_fraction_value().to_bits().hash(hsh);
            s.tail_color.hash(hsh);
            s.head_radius.to_bits().hash(hsh);
        }
        Layer3D::Bar(s) => {
            s.data.version.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Stem(s) => {
            s.data.version.hash(hsh);
            s.show_base.hash(hsh);
        }
        Layer3D::Pie(s) => {
            s.data.version.hash(hsh);
            s.show_labels.hash(hsh);
            s.border.map(stroke_key).hash(hsh);
        }
        Layer3D::Quiver(s) => {
            s.data.version.hash(hsh);
            s.head_size.to_bits().hash(hsh);
            s.show_heads.hash(hsh);
        }
        Layer3D::Fill(s) => {
            s.data.version.hash(hsh);
            s.alpha.to_bits().hash(hsh);
            s.border.map(stroke_key).hash(hsh);
            s.face_colors.hash(hsh);
        }
        Layer3D::Surface(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.light.map(light_key).hash(hsh);
            s.wireframe_stroke.map(stroke_key).hash(hsh);
            s.colormap_lighting.hash(hsh);
            s.alpha.to_bits().hash(hsh);
            s.primitive.hash(hsh);
            s.floor_z.map(f64::to_bits).hash(hsh);
            s.floor_contours.hash(hsh);
            s.show_normals.hash(hsh);
            s.normal_scale.to_bits().hash(hsh);
            s.normal_stride.hash(hsh);
        }
        Layer3D::MeshSurface(s) => {
            s.data.version.hash(hsh);
            (s.mesh_style as u8).hash(hsh);
            s.hidden_line.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.floor_contours.hash(hsh);
            s.floor_z.map(f64::to_bits).hash(hsh);
            s.base_z.map(f64::to_bits).hash(hsh);
        }
        Layer3D::Contour(s) => {
            s.data.version.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
        }
        Layer3D::Ribbon(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::TriSurf(s) => {
            (s.data.map(|d| d.version), s.implicit.map(|d| d.version)).hash(hsh);
            (s.shading as u8).hash(hsh);
            s.wireframe.hash(hsh);
            s.hidden_line.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::VolumeSlice(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.wireframe_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Isosurface(s) => {
            s.data.mesh().map(|d| d.version).hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Isocaps(s) => {
            s.data.mesh().map(|d| d.version).hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::TetraMesh(s) => {
            s.data.version.hash(hsh);
            s.boundary_only.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Streamline(s) => {
            s.data.version.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
        }
        Layer3D::StreamParticles(s) => {
            s.data.version.hash(hsh);
            s.playback.phase().to_bits().hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.show_lines.hash(hsh);
        }
        Layer3D::StreamRibbon(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::StreamTube(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::ConePlot(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.alpha.to_bits().hash(hsh);
        }
        Layer3D::Patch(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            light_key(s.light).hash(hsh);
            s.alpha.to_bits().hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
        }
        Layer3D::AlphaShape(s) => {
            s.data.version.hash(hsh);
            (s.shading as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            light_key(s.light).hash(hsh);
            s.alpha.to_bits().hash(hsh);
            s.edge_stroke.map(stroke_key).hash(hsh);
        }
        Layer3D::Graph(s) => {
            s.data.version.hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            stroke_key(s.edge_stroke).hash(hsh);
            s.arrow_size.to_bits().hash(hsh);
            s.show_node_labels.hash(hsh);
            s.show_edge_labels.hash(hsh);
        }
        Layer3D::PointCloud(s) => {
            s.data.version.hash(hsh);
            (s.color_map as u8).hash(hsh);
            s.color_scale.map(scale_key).hash(hsh);
            s.max_points.hash(hsh);
            s.pixel_cell.to_bits().hash(hsh);
        }
        Layer3D::VolumeRender(s) => {
            s.volume.version.hash(hsh);
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
    }
}
