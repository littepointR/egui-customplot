#[path = "support/line.rs"]
mod line_support;
mod support;

use egui::{Color32, Context, Shape, Stroke};
use egui_customplot::{
    Axis, AxisScale, LinePattern, Plot, PlotBounds, PlotData, PlotState, StemOrientation, Stems,
};
use line_support::{assert_axis_aligned, paths};
use support::{capture, run};

#[test]
fn rods_keep_raw_order_and_baseline_without_connecting_neighbors_in_each_domain() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[3.0, 4.0], [1.0, 2.0], [f64::NAN; 2], [2.0, 1.0]]);
    for orientation in [StemOrientation::Vertical, StemOrientation::Horizontal] {
        let axis = usize::from(orientation == StemOrientation::Vertical);
        for scale in [AxisScale::Linear, AxisScale::Log10] {
            for reversed in [[false; 2], [true; 2], [true, false]] {
                let baseline = if scale == AxisScale::Log10 { 0.5 } else { -2.0 };
                let mut state = PlotState::default();
                let stems = Stems::new(&data)
                    .orientation(orientation)
                    .baseline(baseline)
                    .color(Color32::RED)
                    .baseline_stroke(Stroke::new(1.0, Color32::GREEN));
                assert!(std::ptr::eq(stems.data(), &data));
                let (out, output) = capture(&ctx, vec![], true, |ui| {
                    Plot::new("p")
                        .x_axis(Axis::new().scale(scale).reversed(reversed[0]))
                        .y_axis(Axis::new().scale(scale).reversed(reversed[1]))
                        .show_layers(ui, &mut state, &[stems.into()])
                });
                let actual = paths(&output);
                let baselines: Vec<_> = output
                    .shapes
                    .iter()
                    .filter_map(|s| match s.shape {
                        Shape::LineSegment { points, stroke } if stroke.color == Color32::GREEN => {
                            Some(points)
                        }
                        _ => None,
                    })
                    .collect();
                output.drop_without_applying_deltas();
                let expected: Vec<_> = data
                    .points()
                    .iter()
                    .filter(|p| p[0].is_finite())
                    .map(|&p| {
                        let mut base = p;
                        base[axis] = baseline;
                        vec![
                            out.transform.position_from_point(base),
                            out.transform.position_from_point(p),
                        ]
                    })
                    .collect();
                assert_eq!(actual, expected, "{orientation:?} {scale:?} {reversed:?}");
                assert_axis_aligned(&actual);
                let (mut min, mut max) = ([1.0; 2], [3.0, 4.0]);
                min[axis] = baseline;
                max[axis] = baseline;
                assert_eq!(
                    baselines,
                    vec![[
                        out.transform.position_from_point(min),
                        out.transform.position_from_point(max)
                    ]]
                );
                assert!(out.transform.bounds().min()[axis] < baseline);
                assert_eq!(out.diagnostics.visited_stems, data.len());
                assert_eq!(out.diagnostics.output_stem_segments, 3);
                assert_eq!(out.diagnostics.output_stem_baselines, 1);
                assert_eq!(out.diagnostics.output_markers, 3);
                assert_eq!(out.diagnostics.non_finite_points, 1);
            }
        }
    }
}

#[test]
fn huge_crossing_rods_clip_in_data_space_without_false_baseline_or_endpoint_markers() {
    let ctx = Context::default();
    for orientation in [StemOrientation::Vertical, StemOrientation::Horizontal] {
        let swap = |p: [f64; 2]| {
            if orientation == StemOrientation::Horizontal {
                [p[1], p[0]]
            } else {
                p
            }
        };
        let data = PlotData::new(
            vec![[0.4, f64::MAX], [0.6, f64::MAX], [2.0, 1.0]]
                .into_iter()
                .map(swap)
                .collect::<Vec<_>>(),
        );
        let mut state = PlotState::default();
        let bounds = if orientation == StemOrientation::Vertical {
            PlotBounds::new([0.0, -1.0], [1.0; 2]).unwrap()
        } else {
            PlotBounds::new([-1.0, 0.0], [1.0; 2]).unwrap()
        };
        state.set_bounds(bounds);
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(
                ui,
                &mut state,
                &[Stems::new(&data)
                    .orientation(orientation)
                    .baseline(-f64::MAX)
                    .into()],
            )
        });
        let actual = paths(&output);
        output.drop_without_applying_deltas();
        let expected: Vec<_> = [0.4, 0.6]
            .map(|fixed| {
                vec![
                    out.transform.position_from_point(swap([fixed, -1.0])),
                    out.transform.position_from_point(swap([fixed, 1.0])),
                ]
            })
            .to_vec();
        assert_eq!(actual, expected);
        assert_axis_aligned(&actual);
        assert_eq!(out.diagnostics.output_stem_baselines, 0);
        assert_eq!(out.diagnostics.output_markers, 0);
        // The rods clip successfully; the two raw marker centers overflow f32
        // offscreen and are explicitly rejected, never moved to the view edge.
        assert_eq!(out.diagnostics.numeric_rejections, 2);
    }
}

#[test]
fn invalid_baselines_reject_the_whole_layer_and_log_invalid_endpoints_stay_raw() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0, 2.0], [2.0, -1.0], [0.0, 4.0], [f64::NAN; 2]]);
    for orientation in [StemOrientation::Vertical, StemOrientation::Horizontal] {
        for baseline in [f64::NAN, f64::INFINITY, 0.0, -1.0, 0.5] {
            let mut state = PlotState::default();
            let (out, output) = capture(&ctx, vec![], false, |ui| {
                Plot::new("p")
                    .x_axis(Axis::new().scale(AxisScale::Log10))
                    .y_axis(Axis::new().scale(AxisScale::Log10))
                    .show_layers(
                        ui,
                        &mut state,
                        &[Stems::new(&data)
                            .orientation(orientation)
                            .baseline(baseline)
                            .into()],
                    )
            });
            let actual = paths(&output);
            output.drop_without_applying_deltas();
            let valid = baseline == 0.5;
            assert_eq!(actual.len(), usize::from(valid));
            assert_eq!(out.diagnostics.invalid_stem_baselines, usize::from(!valid));
            assert_eq!(
                out.diagnostics.visited_stems,
                if valid { data.len() } else { 0 }
            );
            assert_eq!(out.diagnostics.output_markers, usize::from(valid));
            assert_eq!(out.diagnostics.non_finite_points, 1);
            assert_eq!(out.diagnostics.domain_invalid_points, 2);
        }
    }
}

#[test]
fn signed_dashes_restart_at_each_baseline_and_never_join_rod_endpoints() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[-1.0, 2.0], [1.0, -2.0]]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([-2.0, -3.0], [2.0, 3.0]).unwrap());
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[Stems::new(&data)
                .show_markers(false)
                .show_baseline(false)
                .pattern(LinePattern::Dashed {
                    length: 10.0,
                    gap: 5.0,
                })
                .into()],
        )
    });
    let actual = paths(&output);
    output.drop_without_applying_deltas();
    assert_axis_aligned(&actual);
    assert!(out.diagnostics.output_stem_segments > 2);
    for point in data.points() {
        let base = out.transform.position_from_point([point[0], 0.0]);
        let rod: Vec<_> = actual.iter().filter(|p| p[0].x == base.x).collect();
        assert_eq!(rod[0][0], base);
        assert!((rod[0][0].distance(rod[0][1]) - 10.0).abs() < 0.001);
        assert!(rod.iter().all(|p| p.iter().all(|p| p.x == base.x)));
    }
}

#[test]
fn empty_singleton_duplicate_positions_and_hidden_baseline_have_defined_geometry() {
    let ctx = Context::default();
    for points in [
        vec![],
        vec![[1.0, 2.0]],
        vec![[1.0, 2.0], [1.0, -2.0]],
        vec![[1.0, 0.0]],
    ] {
        let data = PlotData::new(points);
        let mut state = PlotState::default();
        for show in [false, true] {
            let out = run(&ctx, vec![], false, |ui| {
                Plot::new("p").show_layers(
                    ui,
                    &mut state,
                    &[Stems::new(&data).show_baseline(show).into()],
                )
            });
            assert_eq!(
                out.diagnostics.output_stem_segments,
                data.points().iter().filter(|p| p[1] != 0.0).count()
            );
            assert_eq!(
                out.diagnostics.output_stem_baselines, 0,
                "no span for identical fixed coordinates"
            );
            assert_eq!(out.diagnostics.output_markers, data.len());
            if !data.is_empty() {
                assert!(out.transform.bounds().min()[1] < 0.0);
            }
        }
    }
}
