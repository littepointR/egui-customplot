use egui::{Color32, Mesh, Vec2};

use crate::{MarkerShape, MarkerStyle, PlotPoint, PlotTransform};

#[derive(Default)]
pub(super) struct PreparedMarkers {
    pub mesh: Mesh,
    pub visited_points: usize,
    pub output_markers: usize,
    pub numeric_rejections: usize,
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
mod tests {
    use super::*;
    use crate::{AxisScale, PlotBounds};

    fn transform() -> PlotTransform {
        PlotTransform::new(
            PlotBounds::new([0.0; 2], [1.0; 2]).unwrap(),
            egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(100.0)),
        )
        .unwrap()
    }

    #[test]
    fn per_point_styles_control_geometry_and_preserve_original_colors() {
        let points = [[0.2; 2], [0.4; 2], [0.6; 2], [0.8; 2]];
        let colors = [
            Color32::RED,
            Color32::TRANSPARENT,
            Color32::GREEN,
            Color32::BLUE,
        ];
        let radii = [2.0, 3.0, 0.0, 5.0];
        let p = prepare_markers(
            &points,
            transform(),
            MarkerStyle::default(),
            Color32::WHITE,
            Some(&colors),
            Some(&radii),
            1.0,
        );
        assert_eq!(p.output_markers, 2);
        assert_eq!(p.visited_points, 4);
        assert!(p.mesh.is_valid());
        assert!(p.mesh.vertices.iter().any(|v| v.color == Color32::RED));
        assert!(p.mesh.vertices.iter().any(|v| v.color == Color32::BLUE));
        assert!(!p.mesh.vertices.iter().any(|v| v.color == Color32::GREEN));
    }

    #[test]
    fn antialias_fringe_is_one_physical_pixel_not_one_logical_point() {
        for ppp in [1.0, 1.5, 2.0, 3.0] {
            let p = prepare_markers(
                &[[0.5; 2]],
                transform(),
                MarkerStyle {
                    shape: MarkerShape::Diamond,
                    radius: 5.0,
                },
                Color32::WHITE,
                None,
                None,
                ppp,
            );
            let center = transform().position_from_point([0.5; 2]);
            for (inner, outer) in p.mesh.vertices[1..5].iter().zip(&p.mesh.vertices[5..9]) {
                assert!(
                    ((outer.pos.distance(center) - inner.pos.distance(center)) * ppp - 1.0).abs()
                        < 1e-4
                );
                assert_eq!(outer.color, Color32::TRANSPARENT);
            }
        }
    }

    #[test]
    fn all_shapes_emit_finite_valid_triangles_with_positive_area() {
        for shape in [
            MarkerShape::Circle,
            MarkerShape::Square,
            MarkerShape::Diamond,
            MarkerShape::TriangleUp,
            MarkerShape::TriangleDown,
            MarkerShape::Plus,
            MarkerShape::Cross,
        ] {
            let p = prepare_markers(
                &[[0.5; 2]],
                transform(),
                MarkerStyle { shape, radius: 6.0 },
                Color32::WHITE,
                None,
                None,
                2.0,
            );
            assert_eq!(p.output_markers, 1);
            assert!(p.mesh.is_valid());
            assert!(p.mesh.vertices.iter().all(|v| v.pos.is_finite()));
            for triangle in p.mesh.indices.as_chunks::<3>().0 {
                let [a, b, c] = std::array::from_fn(|i| p.mesh.vertices[triangle[i] as usize].pos);
                assert!((b - a).rot90().dot(c - a).abs() > 0.0);
            }
        }
    }

    #[test]
    fn invalid_log_samples_and_far_offscreen_centers_do_not_emit_geometry() {
        let transform = PlotTransform::with_axes(
            PlotBounds::new([1.0; 2], [100.0; 2]).unwrap(),
            transform().rect(),
            [AxisScale::Log10; 2],
            [false; 2],
        )
        .unwrap();
        let p = prepare_markers(
            &[[1.0; 2], [0.0; 2], [-1.0; 2], [f64::NAN; 2], [1e20; 2]],
            transform,
            MarkerStyle::default(),
            Color32::WHITE,
            None,
            None,
            1.0,
        );
        assert_eq!(p.visited_points, 5);
        assert_eq!(p.output_markers, 1);
        assert_eq!(p.numeric_rejections, 0);
    }
}
