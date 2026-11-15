#[path = "support/line.rs"]
mod line_support;
mod support;

use egui::Context;
use egui_customplot::{
    Axis, AxisScale, Line, LineInterpolation as Interpolation, LineSampling, Plot, PlotBounds,
    PlotData, PlotState,
};
use line_support::{assert_axis_aligned, paths};
use support::{capture, run};

#[test]
fn nonuniform_steps_use_raw_corners_endpoints_order_log_and_reversal() {
    let data = PlotData::new(vec![[1.0, 2.0], [3.0, 8.0], [10.0, 4.0]]);
    let ctx = Context::default();
    for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
        for scales in [[AxisScale::Linear; 2], [AxisScale::Log10; 2]] {
            for reversed in [[false; 2], [true; 2], [true, false]] {
                let mut state = PlotState::default();
                state.set_bounds(PlotBounds::new([0.5, 1.0], [11.0, 10.0]).unwrap());
                let (out, output) = capture(&ctx, vec![], true, |ui| {
                    Plot::new("p")
                        .x_axis(Axis::new().scale(scales[0]).reversed(reversed[0]))
                        .y_axis(Axis::new().scale(scales[1]).reversed(reversed[1]))
                        .show(
                            ui,
                            &mut state,
                            &[Line::new(&data).interpolation(interpolation)],
                        )
                });
                let expected = if interpolation == Interpolation::StepAfter {
                    [[1.0, 2.0], [3.0, 2.0], [3.0, 8.0], [10.0, 8.0], [10.0, 4.0]]
                } else {
                    [[1.0, 2.0], [1.0, 8.0], [3.0, 8.0], [3.0, 4.0], [10.0, 4.0]]
                }
                .map(|p| out.transform.position_from_point(p));
                let actual = paths(&output);
                assert_eq!(
                    actual,
                    vec![expected.to_vec()],
                    "{interpolation:?} {scales:?} {reversed:?}"
                );
                assert_axis_aligned(&actual);
                assert_eq!(out.diagnostics.visited_segments, 2);
                assert_eq!(out.diagnostics.input_points, 3);
                output.drop_without_applying_deltas();
            }
        }
    }
}

#[test]
fn no_plateaus_are_fabricated_next_to_nonfinite_or_log_invalid_samples() {
    let ctx = Context::default();
    for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
        for gap in [[f64::NAN; 2], [5.0, f64::INFINITY], [0.0, 5.0], [5.0, -1.0]] {
            let mut data = PlotData::new(vec![[1.0; 2], [2.0; 2], gap, [8.0; 2], [9.0; 2]]);
            let indexed = gap[0] != 0.0;
            if indexed {
                data.build_line_index(16 * 1024).unwrap();
            } else {
                // Interior nonpositive X makes this finite run unsorted, so an
                // index must reject it even though the log renderer breaks it.
                assert!(data.build_line_index(16 * 1024).is_err());
            }
            for sampling in [
                LineSampling::Exact,
                LineSampling::Visible,
                LineSampling::MinMax,
            ] {
                let mut state = PlotState::default();
                state.set_bounds(PlotBounds::new([0.5; 2], [10.0; 2]).unwrap());
                let (out, output) = capture(&ctx, vec![], false, |ui| {
                    Plot::new("p")
                        .x_axis(Axis::new().scale(AxisScale::Log10))
                        .y_axis(Axis::new().scale(AxisScale::Log10))
                        .show(
                            ui,
                            &mut state,
                            &[Line::new(&data)
                                .interpolation(interpolation)
                                .sampling(sampling)],
                        )
                });
                let actual = paths(&output);
                assert_eq!(
                    out.diagnostics.indexed_line_series,
                    usize::from(indexed && sampling != LineSampling::Exact)
                );
                assert_eq!(actual.len(), 2, "{gap:?} {sampling:?}");
                assert_eq!(
                    actual[0].last(),
                    Some(&out.transform.position_from_point([2.0; 2]))
                );
                assert_eq!(
                    actual[1].first(),
                    Some(&out.transform.position_from_point([8.0; 2]))
                );
                assert_axis_aligned(&actual);
                output.drop_without_applying_deltas();
            }
        }
    }
}

#[test]
fn clipped_steps_do_not_connect_along_view_edges_or_create_diagonals() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[-1.0; 2], [0.5, 2.0], [2.0, -1.0]]);
    for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show(
                ui,
                &mut state,
                &[Line::new(&data).interpolation(interpolation)],
            )
        });
        let endpoints = if interpolation == Interpolation::StepAfter {
            [[0.5, 0.0], [0.5, 1.0]]
        } else {
            [[0.5, 1.0], [0.5, 0.0]]
        };
        assert_eq!(
            paths(&output),
            vec![
                endpoints
                    .map(|p| out.transform.position_from_point(p))
                    .to_vec()
            ]
        );
        output.drop_without_applying_deltas();
    }
}

#[test]
fn visible_matches_exact_with_nonuniform_duplicate_x_and_boundary_neighbors() {
    let ctx = Context::default();
    let mut data = PlotData::new(vec![
        [1.0, 2.0],
        [2.0, 8.0],
        [2.0, 3.0],
        [4.0, 7.0],
        [10.0, 1.0],
        [f64::NAN; 2],
        [1.0, 5.0],
        [12.0, 9.0],
    ]);
    data.build_line_index(16 * 1024).unwrap();
    for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
        for scale in [AxisScale::Linear, AxisScale::Log10] {
            for reversed in [false, true] {
                for x in [
                    [1.5, 3.5],
                    [2.0, 2.1],
                    [2.1, 2.2],
                    [9.9, 10.0],
                    [13.0, 14.0],
                ] {
                    let mut reference = Vec::new();
                    for sampling in [LineSampling::Exact, LineSampling::Visible] {
                        let mut state = PlotState::default();
                        state.set_bounds(PlotBounds::new([x[0], 1.0], [x[1], 10.0]).unwrap());
                        let (_, output) = capture(&ctx, vec![], true, |ui| {
                            Plot::new("p")
                                .x_axis(Axis::new().scale(scale).reversed(reversed))
                                .show(
                                    ui,
                                    &mut state,
                                    &[Line::new(&data)
                                        .interpolation(interpolation)
                                        .sampling(sampling)],
                                )
                        });
                        let actual = paths(&output);
                        if sampling == LineSampling::Exact {
                            reference = actual;
                        } else {
                            assert_eq!(
                                actual, reference,
                                "{interpolation:?} {scale:?} {reversed} {x:?}"
                            );
                        }
                        output.drop_without_applying_deltas();
                    }
                }
            }
        }
    }
}

#[test]
fn empty_singleton_duplicates_and_unsorted_steps_keep_the_raw_series_contract() {
    let ctx = Context::default();
    for points in [
        vec![],
        vec![[2.0, 3.0]],
        vec![[2.0, 3.0], [2.0, 8.0]],
        vec![[4.0, 2.0], [1.0, 6.0], [3.0, 3.0]],
    ] {
        let data = PlotData::new(points);
        for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
            let mut state = PlotState::default();
            let line = Line::new(&data)
                .interpolation(interpolation)
                .sampling(LineSampling::Visible);
            let series = egui_customplot::Series::from(line);
            assert_eq!(series.data().points(), data.points());
            let (out, output) = capture(&ctx, vec![], false, |ui| {
                Plot::new("p").show(ui, &mut state, &[line])
            });
            assert_eq!(out.diagnostics.lod_fallback_series, 1);
            let actual = paths(&output);
            assert_eq!(actual.is_empty(), data.len() < 2);
            assert_axis_aligned(&actual);
            if data.len() >= 2 {
                assert_eq!(
                    actual[0].first(),
                    Some(&out.transform.position_from_point(data.points()[0]))
                );
                assert_eq!(
                    actual[0].last(),
                    Some(
                        &out.transform
                            .position_from_point(*data.points().last().unwrap())
                    )
                );
            }
            output.drop_without_applying_deltas();
        }
    }
}

#[test]
fn minmax_reduces_raw_selection_before_step_expansion_without_diagonal_shortcuts() {
    let ctx = Context::default();
    let mut data = PlotData::new(
        (0..100_000)
            .map(|i| [i as f64, (i % 101) as f64])
            .collect::<Vec<_>>(),
    );
    data.build_line_index(1024 * 1024).unwrap();
    let mut state = PlotState::default();
    for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show_hover(false).show(
                ui,
                &mut state,
                &[Line::new(&data)
                    .interpolation(interpolation)
                    .sampling(LineSampling::MinMax)],
            )
        });
        let actual = paths(&output);
        assert_axis_aligned(&actual);
        assert_eq!(actual.len(), 1);
        let columns = out.transform.rect().width().ceil() as usize;
        assert!(out.diagnostics.query_selected_points <= columns * 4);
        assert_eq!(
            out.diagnostics.output_points,
            2 * out.diagnostics.query_selected_points - 1
        );
        assert_eq!(
            actual[0].first(),
            Some(&out.transform.position_from_point(data.points()[0]))
        );
        assert_eq!(
            actual[0].last(),
            Some(
                &out.transform
                    .position_from_point(*data.points().last().unwrap())
            )
        );
        output.drop_without_applying_deltas();
    }
    // Interpolation never changes autoscale summaries or raw input ownership.
    let linear = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let stepped = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(
            ui,
            &mut state,
            &[Line::new(&data).interpolation(Interpolation::StepAfter)],
        )
    });
    assert_eq!(linear.transform.bounds(), stepped.transform.bounds());
    assert_eq!(stepped.diagnostics.unique_sample_bytes, data.sample_bytes());
}
