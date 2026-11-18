use egui::{Color32, Stroke};

use super::meshes_contours::{collect_contour3d, collect_mesh_surface3d};
use super::scene::{Label3D, Marker3D, Scene3D};
use super::solid_layers::{collect_constant_plane, collect_fill3, collect_pie3, collect_quiver3};
use super::surfaces::{collect_ribbon3d, collect_surface3d, collect_trisurf3d};
use super::volume_flow::{
    collect_cone_plot3d, collect_isocaps3d, collect_isosurface3d, collect_stream_ribbon3d,
    collect_stream_tube3d, collect_streamline3d, collect_streamparticles3d, collect_tetramesh3d,
    collect_volume_slice3d,
};
use crate::{
    ConstantPlane3D, Layer3D, MarkerShape, Plot3DDiagnostics, Plot3DState, Point3D, Text3D,
    Transform3D, model::ResolvedPlotStyle,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_layers_3d(
    scene: &mut Scene3D,
    state: &Plot3DState,
    layers: &[Layer3D<'_>],
    planes: &[ConstantPlane3D],
    texts: &[Text3D],
    transform: Transform3D,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (idx, &layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        if !state.layer_visible(id) {
            continue;
        }
        let resolved = layer.resolved_style(idx, style);
        match layer {
            Layer3D::Custom(layer) => {
                super::custom::collect(scene, layer, resolved, transform, diagnostics)
            }
            Layer3D::Line(line) => {
                collect_polyline(
                    scene,
                    line.data.points(),
                    resolved.stroke,
                    line.show_markers
                        .then_some((resolved.marker.shape, resolved.marker.radius)),
                    transform,
                    diagnostics,
                );
            }
            Layer3D::Scatter(sc) => {
                let pts = sc.data.points();
                let colors = sc.data.colors();
                let radii = sc.data.radii();
                let scalars = sc.data.scalars();
                for (i, &p) in pts.iter().enumerate() {
                    diagnostics.visited_items += 1;
                    let c = colors
                        .and_then(|arr| arr.get(i).copied())
                        .or_else(|| {
                            sc.color_scale
                                .zip(scalars.and_then(|s| s.get(i).copied()))
                                .and_then(|(cs, v)| cs.color(v))
                        })
                        .unwrap_or(resolved.stroke.color);
                    let r = radii
                        .and_then(|arr| arr.get(i).copied())
                        .unwrap_or(resolved.marker.radius);
                    push_marker_data(scene, p, r, c, resolved.marker.shape, transform);
                }
            }
            Layer3D::Bubble(bub) => {
                for pt in bub.data.points() {
                    diagnostics.visited_items += 1;
                    let c = match (bub.color_scale, pt.scalar) {
                        (Some(cs), Some(v)) => cs.color(v).unwrap_or(resolved.stroke.color),
                        _ => resolved.stroke.color,
                    }
                    .gamma_multiply(bub.alpha);
                    push_marker_data(
                        scene,
                        pt.position,
                        bub.data.pixel_radius(pt.size),
                        c,
                        MarkerShape::Circle,
                        transform,
                    );
                }
            }
            Layer3D::Swarm(sw) => {
                for pt in sw.data.points() {
                    diagnostics.visited_items += 1;
                    let c = sw
                        .group_colors
                        .and_then(|gc| gc.get(pt.group_index).copied())
                        .unwrap_or_else(|| {
                            style
                                .resolve_series(pt.group_index, Default::default())
                                .stroke
                                .color
                        });
                    push_marker_data(
                        scene,
                        pt.position,
                        resolved.marker.radius,
                        c,
                        resolved.marker.shape,
                        transform,
                    );
                }
            }
            Layer3D::Comet(comet) => {
                let pts = comet.visible_points();
                let head = pts.len().saturating_sub(1);
                let tail_len = ((comet.data.points().len() as f64)
                    * comet.playback.tail_fraction_value())
                .round() as usize;
                let body_end = head.saturating_sub(tail_len.max(1));
                if body_end > 0 {
                    let body_stroke = Stroke::new(
                        resolved.stroke.width,
                        resolved.stroke.color.gamma_multiply(0.4),
                    );
                    collect_polyline(
                        scene,
                        &pts[..=body_end],
                        body_stroke,
                        None,
                        transform,
                        diagnostics,
                    );
                }
                if head > body_end {
                    let tail_c = comet.tail_color.unwrap_or(resolved.stroke.color);
                    collect_polyline(
                        scene,
                        &pts[body_end..=head],
                        Stroke::new(resolved.stroke.width.max(2.0), tail_c),
                        None,
                        transform,
                        diagnostics,
                    );
                }
                if let Some(&hp) = pts.get(head) {
                    push_marker_data(
                        scene,
                        hp,
                        comet.head_radius,
                        Color32::WHITE.lerp_to_gamma(resolved.stroke.color, 0.35),
                        MarkerShape::Circle,
                        transform,
                    );
                }
            }
            Layer3D::Bar(bar) => {
                for bx in bar.data.boxes() {
                    diagnostics.visited_items += 1;
                    let c = bar
                        .color_scale
                        .and_then(|cs| cs.color(bx.value))
                        .unwrap_or_else(|| {
                            style
                                .resolve_series(bx.col, Default::default())
                                .stroke
                                .color
                        })
                        .gamma_multiply(bar.alpha);
                    scene.push_box(bx.min, bx.max, c, bar.edge_stroke, transform);
                }
            }
            Layer3D::Stem(stem) => {
                let z0 = stem.data.base_z();
                for &p in stem.data.points() {
                    diagnostics.visited_items += 1;
                    scene.push_data_segment([p[0], p[1], z0], p, resolved.stroke, transform);
                    push_marker_data(
                        scene,
                        p,
                        resolved.marker.radius,
                        resolved.stroke.color,
                        resolved.marker.shape,
                        transform,
                    );
                    if stem.show_base {
                        push_marker_data(
                            scene,
                            [p[0], p[1], z0],
                            2.0,
                            resolved.stroke.color.gamma_multiply(0.6),
                            MarkerShape::Circle,
                            transform,
                        );
                    }
                }
            }
            Layer3D::Pie(pie) => {
                collect_pie3(scene, pie, transform, style, diagnostics);
            }
            Layer3D::Quiver(q) => {
                collect_quiver3(scene, q, resolved.stroke, transform, diagnostics);
            }
            Layer3D::Fill(fill) => {
                collect_fill3(scene, fill, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::Surface(surf) => {
                collect_surface3d(scene, surf, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::MeshSurface(mesh) => {
                collect_mesh_surface3d(
                    scene,
                    mesh,
                    style.background,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::Contour(c3) => {
                collect_contour3d(scene, c3, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::Ribbon(rib) => {
                collect_ribbon3d(scene, rib, style, transform, diagnostics);
            }
            Layer3D::TriSurf(ts) => {
                collect_trisurf3d(
                    scene,
                    ts,
                    style.background,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::VolumeSlice(vs) => {
                collect_volume_slice3d(scene, vs, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::Isosurface(iso) => {
                collect_isosurface3d(
                    scene,
                    iso,
                    style.background,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::Isocaps(caps) => {
                collect_isocaps3d(
                    scene,
                    caps,
                    style.background,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::TetraMesh(tm) => {
                collect_tetramesh3d(scene, tm, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::Streamline(sl) => {
                collect_streamline3d(scene, sl, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::StreamParticles(sp) => {
                collect_streamparticles3d(scene, sp, resolved.stroke.color, transform, diagnostics);
            }
            Layer3D::StreamRibbon(sr) => {
                collect_stream_ribbon3d(scene, sr, style, transform, diagnostics);
            }
            Layer3D::StreamTube(st) => {
                collect_stream_tube3d(scene, st, style, transform, diagnostics);
            }
            Layer3D::ConePlot(cp) => {
                collect_cone_plot3d(
                    scene,
                    cp,
                    style.background,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::Patch(p) => {
                super::stage13::collect_patch3d(
                    scene,
                    p,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::AlphaShape(ash) => {
                super::stage13::collect_alphashape3d(
                    scene,
                    ash,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::Graph(g) => {
                super::stage13::collect_graph3d(
                    scene,
                    g,
                    resolved.stroke.color,
                    resolved.marker.shape,
                    resolved.marker.radius,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::PointCloud(pc) => {
                super::volshow::collect_point_cloud3d(
                    scene,
                    pc,
                    resolved.stroke.color,
                    transform,
                    diagnostics,
                );
            }
            Layer3D::VolumeRender(_) => {}
        }
    }

    for plane in planes {
        collect_constant_plane(scene, plane, transform, diagnostics);
    }
    for t in texts {
        if let Some((pos, _)) = transform.project_data(t.position) {
            scene.labels.push(Label3D {
                pos,
                text: t.text.clone(),
                color: t.color.unwrap_or(Color32::PLACEHOLDER),
            });
        }
    }
}

pub(super) fn collect_polyline(
    scene: &mut Scene3D,
    pts: &[Point3D],
    stroke: Stroke,
    markers: Option<(MarkerShape, f32)>,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let mut prev: Option<Point3D> = None;
    for &p in pts {
        diagnostics.visited_items += 1;
        if p.iter().all(|v| v.is_finite()) {
            if let Some(a) = prev {
                scene.push_data_segment(a, p, stroke, transform);
            }
            if let Some((shape, r)) = markers {
                push_marker_data(scene, p, r, stroke.color, shape, transform);
            }
            prev = Some(p);
        } else {
            prev = None;
        }
    }
}

pub(super) fn push_marker_data(
    scene: &mut Scene3D,
    p: Point3D,
    radius: f32,
    color: Color32,
    shape: MarkerShape,
    transform: Transform3D,
) {
    let Some(u) = transform.data_to_unit(p) else {
        return;
    };
    if u.iter().any(|&v| !(-1e-4..=1.0001).contains(&v)) {
        scene.clipped += 1;
        return;
    }
    if let Some((pos, depth)) = transform.project_unit(u) {
        scene.markers.push(Marker3D {
            pos,
            depth,
            radius,
            color,
            shape,
        });
    }
}
