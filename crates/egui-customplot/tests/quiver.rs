mod support;

use egui::{Context, Pos2, Shape};
use egui_customplot::{
    Axis, AxisScale, LinePattern, Plot, PlotBounds, PlotState, Quiver, QuiverArrow, QuiverData,
};
use support::{capture, run};

fn paths(output: &egui::FullOutput) -> Vec<Vec<Pos2>> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(path) => Some(path.points.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn true_endpoints_bounds_and_screen_heads_follow_linear_log_reversed_axes() {
    let ctx = Context::default();
    let records = vec![
        QuiverArrow::new([2.0, 3.0], [6.0, 12.0]),
        QuiverArrow::new([8.0, 15.0], [-5.0, -8.0]),
    ];
    let data = QuiverData::new(records.clone());
    for scale in [AxisScale::Linear, AxisScale::Log10] {
        for reversed in [[false; 2], [true; 2], [false, true]] {
            let mut state = PlotState::default();
            let quiver = Quiver::new(&data).head_size(12.0, 10.0);
            assert!(std::ptr::eq(quiver.data(), &data));
            let (out, output) = capture(&ctx, vec![], true, |ui| {
                Plot::new("p")
                    .x_axis(Axis::new().scale(scale).reversed(reversed[0]))
                    .y_axis(Axis::new().scale(scale).reversed(reversed[1]))
                    .show_layers(ui, &mut state, &[quiver.into()])
            });
            let actual = paths(&output);
            output.drop_without_applying_deltas();
            assert_eq!(actual.len(), 4);
            for (i, &arrow) in records.iter().enumerate() {
                let end = arrow.endpoint().unwrap();
                let a = out.transform.position_from_point(arrow.origin);
                let b = out.transform.position_from_point(end);
                assert_eq!(actual[i], vec![a, b]);
                let head = &actual[i + records.len()];
                assert_eq!(head.len(), 3);
                assert_eq!(head[1], b);
                let center = head[0].lerp(head[2], 0.5);
                let direction = (b - a).normalized();
                assert!(((b - center) - direction * 12.0).length() < 0.0001);
                assert!((head[0].distance(head[2]) - 10.0).abs() < 0.0001);
                assert!((head[0] - center).dot(direction).abs() < 0.0001);
                for point in [arrow.origin, end] {
                    assert!((0..2).all(|j| point[j] > out.transform.bounds().min()[j]
                        && point[j] < out.transform.bounds().max()[j]));
                }
            }
            assert_eq!(out.diagnostics.input_quiver_arrows, 2);
            assert_eq!(out.diagnostics.visited_quiver_arrows, 2);
            assert_eq!(out.diagnostics.output_quiver_shafts, 2);
            assert_eq!(out.diagnostics.output_quiver_heads, 2);
            assert_eq!(out.diagnostics.numeric_rejections, 0);
        }
    }
    assert_eq!(data.arrows(), records);
}

#[test]
fn joint_domains_nonfinite_and_endpoint_overflow_reject_whole_records_without_reindexing() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![
        QuiverArrow::new([1.0, 1.0], [2.0, 3.0]),
        QuiverArrow::new([-1.0, 1.0], [2.0, 3.0]),
        QuiverArrow::new([1.0, 1.0], [2.0, -2.0]),
        QuiverArrow::new([f64::NAN, 1.0], [1.0; 2]),
        QuiverArrow::new([1.0; 2], [f64::INFINITY, 1.0]),
        QuiverArrow::new([f64::MAX, 1.0], [f64::MAX, 0.0]),
    ]);
    assert!(data.arrows()[5].endpoint().is_none());
    for x_log in [false, true] {
        for y_log in [false, true] {
            let mut state = PlotState::default();
            let out = run(&ctx, vec![], false, |ui| {
                Plot::new("p")
                    .x_axis(Axis::new().scale(if x_log {
                        AxisScale::Log10
                    } else {
                        AxisScale::Linear
                    }))
                    .y_axis(Axis::new().scale(if y_log {
                        AxisScale::Log10
                    } else {
                        AxisScale::Linear
                    }))
                    .show_layers(ui, &mut state, &[Quiver::new(&data).into()])
            });
            let invalid = usize::from(x_log) + usize::from(y_log);
            assert_eq!(out.diagnostics.invalid_quiver_arrows, 3);
            assert_eq!(out.diagnostics.domain_invalid_quiver_arrows, invalid);
            assert_eq!(out.diagnostics.output_quiver_shafts, 3 - invalid);
            assert_eq!(out.diagnostics.output_quiver_heads, 3 - invalid);
            assert_eq!(out.diagnostics.input_points, 0);
            assert_eq!(out.diagnostics.unique_quiver_bytes, data.len() * 32);
        }
    }
}

#[test]
fn clipping_never_fabricates_heads_at_intersections_and_uses_full_length_for_head_size() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![
        QuiverArrow::new([-2.0, 0.2], [4.0, 0.0]), // both ends outside, shaft only
        QuiverArrow::new([0.2, 0.4], [2.0, 0.0]),  // true tip outside, shaft only
        QuiverArrow::new([-20.0, 0.6], [20.01, 0.0]), // short visible shaft, full head
        QuiverArrow::new([0.5, 0.8], [0.5, 0.0]),  // real tip on boundary, real head
    ]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[Quiver::new(&data).head_size(10.0, 8.0).into()],
        )
    });
    let actual = paths(&output);
    assert!(
        output
            .shapes
            .iter()
            .filter(|s| matches!(s.shape, Shape::Path(_)))
            .all(|s| s.clip_rect == out.transform.rect())
    );
    output.drop_without_applying_deltas();
    assert_eq!(actual.len(), 6);
    assert_eq!(
        actual[0],
        vec![
            out.transform.position_from_point([0.0, 0.2]),
            out.transform.position_from_point([1.0, 0.2])
        ]
    );
    for (path, arrow) in actual[4..].iter().zip(&data.arrows()[2..]) {
        let tip = out.transform.position_from_point(arrow.endpoint().unwrap());
        assert_eq!(path[1], tip);
        assert!((tip.x - path[0].x - 10.0).abs() < 0.0001);
    }
    assert_eq!(out.diagnostics.output_quiver_heads, 2);
}

#[test]
fn subpixel_zero_empty_and_short_vectors_never_create_arbitrary_full_size_heads() {
    let ctx = Context::default();
    let empty = QuiverData::new(vec![]);
    assert!(empty.is_empty());
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("empty").show_layers(ui, &mut PlotState::default(), &[Quiver::new(&empty).into()])
    });
    assert_eq!(out.diagnostics.output_quiver_shafts, 0);
    let data = QuiverData::new(vec![
        QuiverArrow::new([0.5; 2], [0.0; 2]),
        QuiverArrow::new([0.5; 2], [f64::MIN_POSITIVE; 2]),
        QuiverArrow::new([0.5; 2], [0.001, 0.0]),
    ]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[Quiver::new(&data).head_size(1000.0, 800.0).into()],
        )
    });
    let actual = paths(&output);
    output.drop_without_applying_deltas();
    assert_eq!(actual.len(), 2);
    let head_length = actual[1][1].x - actual[1][0].x;
    let full_length = actual[0][1].distance(actual[0][0]);
    assert!((head_length / full_length - 0.4).abs() < 0.0001);
    assert_eq!(out.diagnostics.numeric_rejections, 0);
}

#[test]
fn ill_conditioned_extreme_crossings_diagnose_instead_of_inventing_edge_geometry() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![
        QuiverArrow::new([-f64::MAX, 0.5], [f64::MAX, 0.0]),
        QuiverArrow::new([0.5, 0.5], [1e100, 0.0]),
        QuiverArrow::new([1e100, 2.0], [1.0, 0.0]), // trivially outside, not a numeric error
        QuiverArrow::new([-1e5, 0.7], [2e5, 0.0]),  // representable offscreen crossing
    ]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Quiver::new(&data).into()])
    });
    let actual = paths(&output);
    output.drop_without_applying_deltas();
    assert_eq!(out.diagnostics.invalid_quiver_arrows, 0);
    assert_eq!(out.diagnostics.numeric_rejections, 2);
    assert_eq!(actual.len(), 1);
    assert_eq!(
        actual[0],
        vec![
            out.transform.position_from_point([0.0, 0.7]),
            out.transform.position_from_point([1.0, 0.7])
        ]
    );
}

#[test]
fn head_validation_and_dashes_are_explicit_and_heads_stay_solid() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![QuiverArrow::new([0.1, 0.2], [0.7, 0.6])]);
    for (size, expected_heads, numeric) in [
        ([10.0, 8.0], 1, 0),
        ([0.0, 8.0], 0, 0),
        ([-1.0, 8.0], 0, 1),
        ([f32::NAN, 8.0], 0, 1),
    ] {
        let mut state = PlotState::default();
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(
                ui,
                &mut state,
                &[Quiver::new(&data)
                    .head_size(size[0], size[1])
                    .pattern(LinePattern::Dashed {
                        length: 8.0,
                        gap: 4.0,
                    })
                    .into()],
            )
        });
        let actual = paths(&output);
        output.drop_without_applying_deltas();
        assert_eq!(
            actual.iter().filter(|p| p.len() == 3).count(),
            expected_heads
        );
        assert_eq!(out.diagnostics.output_quiver_heads, expected_heads);
        assert!(out.diagnostics.output_quiver_shafts > 1);
        assert_eq!(out.diagnostics.numeric_rejections, numeric);
    }
}
