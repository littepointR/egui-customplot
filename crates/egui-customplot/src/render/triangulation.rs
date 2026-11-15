use egui::{Color32, Painter, Pos2, Shape, Stroke, epaint};

use super::{
    markers::prepare_markers,
    mesh::{MeshBatch, stroke_valid},
    pattern::apply_pattern,
};
use crate::{
    AlphaShape, PlotDiagnostics, PlotPoint, PlotTransform, TriPlot, Voronoi,
    axis::{clip_normalized_segment, project_boundary_segment},
    model::ResolvedSeriesStyle,
};

pub(super) fn paint_triplot(
    painter: &Painter,
    tri: TriPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let verts = tri.data.vertices();
    if let Some(fill) = tri.face_fill
        && fill != Color32::TRANSPARENT
    {
        let mut batch = MeshBatch::default();
        for &[i0, i1, i2] in tri.data.triangles() {
            let pts = [verts[i0 as usize], verts[i1 as usize], verts[i2 as usize]];
            if pts.iter().all(|&p| transform.is_valid_point(p)) {
                batch.add_flat_triangle(
                    painter,
                    pts.map(|p| transform.normalized(p)),
                    fill,
                    transform,
                );
            }
        }
        let stats = batch.finish(painter);
        diagnostics.geometry.output_meshes += stats.meshes;
        diagnostics.geometry_capacity_bytes += stats.bytes;
        diagnostics.numeric_rejections += stats.numeric_rejections;
    }

    if !style.stroke.width.is_finite() || style.stroke.width < 0.0 || !style.pattern.is_valid() {
        diagnostics.numeric_rejections += 1;
    } else if !style.stroke.is_empty() {
        if let Some((values, scale)) = tri.vertex_scalars {
            if values.len() != verts.len() {
                diagnostics.numeric_rejections += 1;
            } else {
                paint_scalar_edges(
                    painter,
                    tri.data.edges(),
                    verts,
                    values,
                    scale,
                    style,
                    transform,
                    diagnostics,
                );
            }
        } else {
            let mut segments = Vec::with_capacity(tri.data.edges().len());
            for &[u, v] in tri.data.edges() {
                diagnostics.geometry.visited_items += 1;
                push_projected_segment(
                    verts[u as usize],
                    verts[v as usize],
                    transform,
                    &mut segments,
                    diagnostics,
                );
            }
            paint_patterned_segments(painter, segments, style, diagnostics);
        }
    }

    if tri.show_markers {
        paint_point_markers(
            painter,
            verts,
            transform,
            style,
            pixels_per_point,
            diagnostics,
        );
    }
}

pub(super) fn paint_voronoi(
    painter: &Painter,
    vor: Voronoi<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if !style.stroke.width.is_finite() || style.stroke.width < 0.0 || !style.pattern.is_valid() {
        diagnostics.numeric_rejections += 1;
    } else if !style.stroke.is_empty() {
        let mut segments = Vec::with_capacity(vor.data.edges().len());
        for &[a, b] in vor.data.edges() {
            diagnostics.geometry.visited_items += 1;
            push_projected_segment(a, b, transform, &mut segments, diagnostics);
        }
        paint_patterned_segments(painter, segments, style, diagnostics);
    }

    if vor.show_sites {
        paint_point_markers(
            painter,
            vor.data.sites(),
            transform,
            style,
            pixels_per_point,
            diagnostics,
        );
    }
}

pub(super) fn paint_alphashape(
    painter: &Painter,
    alpha: AlphaShape<'_>,
    transform: PlotTransform,
    default_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let fill = alpha.style.color.unwrap_or(default_color);
    let patch = alpha.data.patch();
    let verts = patch.vertices();
    let scales = transform.scales();
    if fill != Color32::TRANSPARENT {
        let mut batch = MeshBatch::default();
        for face in 0..patch.face_count() {
            diagnostics.geometry.visited_items += 1;
            for &[i0, i1, i2] in patch.face_triangles(face, scales) {
                let norm = [
                    transform.normalized(verts[i0]),
                    transform.normalized(verts[i1]),
                    transform.normalized(verts[i2]),
                ];
                batch.add_flat_triangle(painter, norm, fill, transform);
            }
        }
        let stats = batch.finish(painter);
        diagnostics.geometry.output_meshes += stats.meshes;
        diagnostics.geometry_capacity_bytes += stats.bytes;
        diagnostics.numeric_rejections += stats.numeric_rejections;
    }

    if alpha.show_boundary && stroke_valid(alpha.border) {
        for &[a, b] in alpha.data.boundary_edges() {
            diagnostics.geometry.visited_items += 1;
            match project_boundary_segment(a, b, transform) {
                Ok(Some([pa, pb])) => {
                    diagnostics.geometry.output_paths += 1;
                    painter.add(Shape::line_segment([pa, pb], alpha.border));
                }
                Ok(None) => {}
                Err(()) => diagnostics.numeric_rejections += 1,
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_scalar_edges(
    painter: &Painter,
    edges: &[[u32; 2]],
    verts: &[PlotPoint],
    values: &[f64],
    scale: crate::ColorScale,
    style: ResolvedSeriesStyle,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let mut uniform_segments = Vec::new();
    for &[u, v] in edges {
        diagnostics.geometry.visited_items += 1;
        let a = verts[u as usize];
        let b = verts[v as usize];
        let (Some(ca), Some(cb)) = (
            scale.color(values[u as usize]),
            scale.color(values[v as usize]),
        ) else {
            continue;
        };
        if ca == cb {
            push_projected_segment(a, b, transform, &mut uniform_segments, diagnostics);
            continue;
        }
        if !transform.is_valid_point(a) || !transform.is_valid_point(b) {
            continue;
        }
        let na = transform.normalized(a);
        let nb = transform.normalized(b);
        if !na[0].is_finite() || !na[1].is_finite() || !nb[0].is_finite() || !nb[1].is_finite() {
            diagnostics.numeric_rejections += 1;
            continue;
        }
        let nmid = [0.5 * (na[0] + nb[0]), 0.5 * (na[1] + nb[1])];
        for ([p0, p1], col) in [([na, nmid], ca), ([nmid, nb], cb)] {
            if col == Color32::TRANSPARENT {
                continue;
            }
            if let Some((sa, sb)) = clip_normalized_segment(p0, p1) {
                let pa = transform.position_from_normalized(sa);
                let pb = transform.position_from_normalized(sb);
                if pa != pb && pa.is_finite() && pb.is_finite() {
                    let stroke = Stroke::new(style.stroke.width, col);
                    let patterned = apply_pattern(vec![vec![pa, pb]], style.pattern);
                    diagnostics.pattern_truncated |= patterned.truncated;
                    for path in patterned.paths {
                        diagnostics.geometry.output_paths += 1;
                        diagnostics.geometry_capacity_bytes +=
                            path.capacity() * std::mem::size_of::<Pos2>();
                        painter.add(Shape::line(path, stroke));
                    }
                }
            }
        }
    }
    paint_patterned_segments(painter, uniform_segments, style, diagnostics);
}

fn push_projected_segment(
    a: PlotPoint,
    b: PlotPoint,
    transform: PlotTransform,
    segments: &mut Vec<Vec<Pos2>>,
    diagnostics: &mut PlotDiagnostics,
) {
    match project_boundary_segment(a, b, transform) {
        Ok(Some([pa, pb])) => segments.push(vec![pa, pb]),
        Ok(None) => {}
        Err(()) => diagnostics.numeric_rejections += 1,
    }
}

fn paint_patterned_segments(
    painter: &Painter,
    segments: Vec<Vec<Pos2>>,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    if segments.is_empty() {
        return;
    }
    let patterned = apply_pattern(segments, style.pattern);
    diagnostics.pattern_truncated |= patterned.truncated;
    diagnostics.geometry.output_paths += patterned.paths.len();
    for path in patterned.paths {
        diagnostics.geometry_capacity_bytes += path.capacity() * std::mem::size_of::<Pos2>();
        painter.add(Shape::line(path, style.stroke));
    }
}

fn paint_point_markers(
    painter: &Painter,
    points: &[PlotPoint],
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let prepared = prepare_markers(
        points,
        transform,
        style.marker,
        style.stroke.color,
        None,
        None,
        pixels_per_point,
    );
    diagnostics.geometry.visited_items += prepared.visited_points;
    diagnostics.geometry.output_markers += prepared.output_markers;
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    diagnostics.geometry_capacity_bytes += prepared.mesh.vertices.capacity()
        * std::mem::size_of::<epaint::Vertex>()
        + prepared.mesh.indices.capacity() * std::mem::size_of::<u32>();
    if !prepared.mesh.is_empty() {
        diagnostics.geometry.output_meshes += 1;
        painter.add(Shape::mesh(prepared.mesh));
    }
}
