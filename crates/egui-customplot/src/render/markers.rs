use egui::{Color32, Mesh, Pos2, Vec2};

use crate::{MarkerShape, MarkerStyle, PlotPoint, PlotTransform};

#[derive(Default)]
pub(super) struct PreparedMarkers {
    pub mesh: Mesh,
    pub discs: Vec<egui::Shape>,
    pub visited_points: usize,
    pub output_markers: usize,
    pub numeric_rejections: usize,
    pub reduced: bool,
}

/// One mesh per series, not one Shape or widget per sample. A transparent outer
/// ring feathers the edge by one physical pixel. Raw samples/styles are borrowed.
pub(super) fn prepare_markers(
    points: &[PlotPoint],
    transform: PlotTransform,
    style: MarkerStyle,
    color: Color32,
    colors: Option<&[Color32]>,
    radii: Option<&[f32]>,
    pixels_per_point: f32,
) -> PreparedMarkers {
    let mut result = PreparedMarkers::default();
    let outline = marker_outline(style.shape);
    let fringe = 1.0 / pixels_per_point.max(0.01);
    for (index, point) in points.iter().enumerate() {
        result.visited_points += 1;
        if !transform.is_valid_point(*point) {
            continue;
        }
        let color = colors.map_or(color, |c| c[index]);
        let radius = radii.map_or(style.radius, |r| r[index]);
        if !radius.is_finite() || radius < 0.0 {
            result.numeric_rejections += 1;
            continue;
        }
        if radius == 0.0 || color == Color32::TRANSPARENT {
            continue;
        }
        let center = transform.position_from_point(*point);
        if !center.is_finite() {
            result.numeric_rejections += 1;
            continue;
        }
        if !transform.rect().expand(radius + fringe).contains(center) {
            continue;
        }
        if !push_shaped_marker(
            &mut result.mesh,
            center,
            radius,
            color,
            style.shape,
            &outline,
            fringe,
        ) {
            result.numeric_rejections += 1;
            continue;
        }
        result.output_markers += 1;
    }
    result
}

pub(crate) fn push_shaped_marker(
    mesh: &mut Mesh,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
    shape: MarkerShape,
    outline: &[Vec2],
    fringe: f32,
) -> bool {
    if !shape.is_hollow() {
        return push_marker(mesh, center, radius, color, outline, fringe);
    }
    let stroke_w = (radius * 0.38).clamp(1.1, 2.6);
    if radius <= stroke_w + 0.15 {
        return push_marker(mesh, center, radius, color, outline, fringe);
    }
    let inner_r = (radius - stroke_w).max(0.15);
    let inner_fade = (inner_r - fringe).max(0.0);
    let n = outline.len();
    if mesh.vertices.len() > (u32::MAX as usize).saturating_sub(n * 4 + 16) {
        return false;
    }
    let r0 = mesh.vertices.len() as u32;
    for unit in outline {
        mesh.colored_vertex(center + *unit * inner_fade, Color32::TRANSPARENT);
    }
    for unit in outline {
        mesh.colored_vertex(center + *unit * inner_r, color);
    }
    for unit in outline {
        mesh.colored_vertex(center + *unit * radius, color);
    }
    for unit in outline {
        mesh.colored_vertex(
            center + *unit * radius + unit.normalized() * fringe,
            Color32::TRANSPARENT,
        );
    }
    let n_u32 = n as u32;
    let r1 = r0 + n_u32;
    let r2 = r1 + n_u32;
    let r3 = r2 + n_u32;
    for edge in 0..n_u32 {
        let next = (edge + 1) % n_u32;
        for (a, b) in [(r0, r1), (r1, r2), (r2, r3)] {
            mesh.add_triangle(a + edge, b + edge, b + next);
            mesh.add_triangle(a + edge, b + next, a + next);
        }
    }
    true
}

pub(super) fn push_marker(
    mesh: &mut Mesh,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
    outline: &[Vec2],
    fringe: f32,
) -> bool {
    if mesh.vertices.len() > (u32::MAX as usize).saturating_sub(64) {
        return false;
    }
    let base = mesh.vertices.len() as u32;
    mesh.colored_vertex(center, color);
    for unit in outline {
        mesh.colored_vertex(center + *unit * radius, color);
    }
    for unit in outline {
        mesh.colored_vertex(
            center + *unit * radius + unit.normalized() * fringe,
            Color32::TRANSPARENT,
        );
    }
    let n = outline.len() as u32;
    for edge in 0..n {
        let next = (edge + 1) % n;
        mesh.add_triangle(base, base + 1 + edge, base + 1 + next);
        mesh.add_triangle(base + 1 + edge, base + 1 + n + edge, base + 1 + n + next);
        mesh.add_triangle(base + 1 + edge, base + 1 + n + next, base + 1 + next);
    }
    true
}

pub(crate) fn marker_outline(shape: MarkerShape) -> Vec<Vec2> {
    match shape.to_filled() {
        MarkerShape::Circle => (0..16)
            .map(|i| Vec2::angled(i as f32 * std::f32::consts::TAU / 16.0))
            .collect(),
        MarkerShape::Square => vec![
            egui::vec2(-1.0, -1.0),
            egui::vec2(1.0, -1.0),
            egui::vec2(1.0, 1.0),
            egui::vec2(-1.0, 1.0),
        ],
        MarkerShape::Diamond => vec![
            egui::vec2(0.0, -1.0),
            egui::vec2(1.0, 0.0),
            egui::vec2(0.0, 1.0),
            egui::vec2(-1.0, 0.0),
        ],
        MarkerShape::TriangleUp | MarkerShape::TriangleDown => {
            let direction = if shape.to_filled() == MarkerShape::TriangleUp {
                1.0
            } else {
                -1.0
            };
            vec![
                egui::vec2(0.0, -1.0) * direction,
                egui::vec2(0.866_025_4, 0.5) * direction,
                egui::vec2(-0.866_025_4, 0.5) * direction,
            ]
        }
        MarkerShape::Plus | MarkerShape::Cross => {
            let r = 0.3;
            let mut points = vec![
                egui::vec2(-r, -1.0),
                egui::vec2(r, -1.0),
                egui::vec2(r, -r),
                egui::vec2(1.0, -r),
                egui::vec2(1.0, r),
                egui::vec2(r, r),
                egui::vec2(r, 1.0),
                egui::vec2(-r, 1.0),
                egui::vec2(-r, r),
                egui::vec2(-1.0, r),
                egui::vec2(-1.0, -r),
                egui::vec2(-r, -r),
            ];
            if shape.to_filled() == MarkerShape::Cross {
                for p in &mut points {
                    *p = egui::vec2(p.x - p.y, p.x + p.y) * std::f32::consts::FRAC_1_SQRT_2;
                }
            }
            points
        }
        MarkerShape::Asterisk => {
            let w = 0.14_f32;
            let r_in = 0.28_f32;
            let mut points = Vec::with_capacity(18);
            for k in 0..6 {
                let a = k as f32 * std::f32::consts::TAU / 6.0;
                let u = Vec2::angled(a);
                let v = egui::vec2(-u.y, u.x);
                let valley_a = a - std::f32::consts::TAU / 12.0;
                points.push(Vec2::angled(valley_a) * r_in);
                points.push(u - v * w);
                points.push(u + v * w);
            }
            points
        }
        MarkerShape::Pentagram => (0..10)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 10.0;
                let rad = if i % 2 == 0 { 1.0 } else { 0.42 };
                Vec2::angled(a) * rad
            })
            .collect(),
        MarkerShape::Hexagram => (0..12)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 12.0;
                let rad = if i % 2 == 0 { 1.0 } else { 0.54 };
                Vec2::angled(a) * rad
            })
            .collect(),
        MarkerShape::CircleHollow
        | MarkerShape::SquareHollow
        | MarkerShape::DiamondHollow
        | MarkerShape::TriangleUpHollow
        | MarkerShape::TriangleDownHollow => unreachable!("handled by to_filled"),
    }
}

pub(super) fn prepare_selected_markers(
    chosen: Vec<(Pos2, f32, Color32)>,
    visited_points: usize,
    numeric_rejections: usize,
    reduced: bool,
    style: MarkerStyle,
    radii: Option<&[f32]>,
    ctx: &egui::Context,
) -> PreparedMarkers {
    let mut result = PreparedMarkers {
        visited_points,
        numeric_rejections,
        reduced,
        ..PreparedMarkers::default()
    };
    let ppp = ctx.pixels_per_point().max(0.01);
    let fringe = 1.0 / ppp;
    let small_circles = chosen.len() > 2_048
        && style.shape == MarkerShape::Circle
        && style.radius * ppp <= 5.0
        && radii.is_none();
    if small_circles
        && ctx.tessellation_options(|options| options.prerasterized_discs)
        && !super::is_capturing_snapshot(ctx)
    {
        // Let egui resolve the disc UVs after the host has finished growing its
        // font atlas. The previous outer fringe has half coverage at this radius.
        let radius = style.radius + fringe * 0.5;
        result.output_markers = chosen.len();
        result.discs = chosen
            .into_iter()
            .map(|(center, _, color)| egui::Shape::circle_filled(center, radius, color))
            .collect();
        return result;
    }
    let outline = if small_circles {
        (0..8)
            .map(|i| Vec2::angled(i as f32 * std::f32::consts::TAU / 8.0))
            .collect()
    } else {
        marker_outline(style.shape)
    };
    let n = outline.len();
    result.mesh.vertices.reserve(chosen.len() * (1 + 2 * n));
    result.mesh.indices.reserve(chosen.len() * 9 * n);
    if radii.is_none() && !style.shape.is_hollow() {
        let inner: Vec<Vec2> = outline.iter().map(|u| *u * style.radius).collect();
        let outer: Vec<Vec2> = outline
            .iter()
            .map(|u| *u * style.radius + u.normalized() * fringe)
            .collect();
        let ring = n as u32;
        let triangles: Vec<u32> = (0..ring)
            .flat_map(|edge| {
                let i = 1 + edge;
                let j = 1 + (edge + 1) % ring;
                [0, i, j, i, ring + i, ring + j, i, ring + j, j]
            })
            .collect();
        for (center, _, c) in chosen {
            if result.mesh.vertices.len() > (u32::MAX as usize).saturating_sub(64) {
                result.numeric_rejections += 1;
                continue;
            }
            let base = result.mesh.vertices.len() as u32;
            result.mesh.colored_vertex(center, c);
            for off in &inner {
                result.mesh.colored_vertex(center + *off, c);
            }
            for off in &outer {
                result
                    .mesh
                    .colored_vertex(center + *off, Color32::TRANSPARENT);
            }
            result
                .mesh
                .indices
                .extend(triangles.iter().map(|index| base + index));
            result.output_markers += 1;
        }
    } else {
        for (center, r, c) in chosen {
            if !push_shaped_marker(
                &mut result.mesh,
                center,
                r,
                c,
                style.shape,
                &outline,
                fringe,
            ) {
                result.numeric_rejections += 1;
                continue;
            }
            result.output_markers += 1;
        }
    }
    result
}

#[cfg(test)]
#[path = "markers_tests.rs"]
mod tests;
