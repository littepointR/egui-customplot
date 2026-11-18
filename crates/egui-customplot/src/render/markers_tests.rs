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

#[test]
fn dense_disc_and_mesh_paths_keep_the_same_pixel_grid_representatives() {
    use super::super::markers_lod::prepare_markers_with_sampling;
    use crate::{MarkerSampling, PlotData};

    let points = (0..30_000)
        .map(|i| [i as f64 / 30_000.0, (i % 97) as f64 / 97.0])
        .collect::<Vec<_>>();
    let data = PlotData::new(points);
    let ctx = egui::Context::default();
    let color = Color32::from_rgba_unmultiplied(10, 20, 30, 128);
    let output = ctx.run_ui(Default::default(), |_| {
        let prepare = |enabled| {
            ctx.tessellation_options_mut(|o| o.prerasterized_discs = enabled);
            prepare_markers_with_sampling(
                &data,
                transform(),
                MarkerStyle::default(),
                color,
                None,
                None,
                &ctx,
                MarkerSampling::Auto,
            )
        };
        let native = prepare(true);
        let mesh = prepare(false);
        assert!(native.reduced && native.output_markers > 2_048);
        assert_eq!(native.output_markers, mesh.output_markers);
        assert_eq!(native.discs.len(), mesh.output_markers);
        assert!(native.mesh.is_empty() && mesh.discs.is_empty());
        assert!(mesh.mesh.is_valid());
        assert_eq!(mesh.mesh.vertices.len(), 17 * native.discs.len());
        let (markers, remainder) = mesh.mesh.vertices.as_chunks::<17>();
        assert!(remainder.is_empty());
        for (disc, vertices) in native.discs.iter().zip(markers) {
            let egui::Shape::Circle(disc) = disc else {
                panic!("expected a filled disc")
            };
            assert_eq!(disc.center, vertices[0].pos);
            assert_eq!(disc.fill, vertices[0].color);
            assert_eq!(
                disc.radius,
                MarkerStyle::default().radius + 0.5 / ctx.pixels_per_point()
            );
        }
    });
    output.drop_without_applying_deltas();
}
