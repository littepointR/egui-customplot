use egui::{Color32, Mesh, Vec2};

use crate::{MarkerShape, MarkerStyle, PlotPoint, PlotTransform};

#[derive(Default)]
pub(super) struct PreparedMarkers {
    pub mesh: Mesh,
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
        if !push_marker(&mut result.mesh, center, radius, color, &outline, fringe) {
            result.numeric_rejections += 1;
            continue;
        }
        result.output_markers += 1;
    }
    result
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

pub(super) fn marker_outline(shape: MarkerShape) -> Vec<Vec2> {
    match shape {
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
            let direction = if shape == MarkerShape::TriangleUp {
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
            if shape == MarkerShape::Cross {
                for p in &mut points {
                    *p = egui::vec2(p.x - p.y, p.x + p.y) * std::f32::consts::FRAC_1_SQRT_2;
                }
            }
            points
        }
    }
}

#[cfg(test)]
#[path = "markers_tests.rs"]
mod tests;
