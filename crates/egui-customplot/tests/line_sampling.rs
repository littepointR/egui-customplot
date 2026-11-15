mod support;

use egui::{Context, Shape};
use egui_customplot::{Axis, AxisScale, Line, LineSampling, Plot, PlotBounds, PlotData, PlotState};
use support::{capture, run};

fn paths(output: &egui::FullOutput) -> Vec<Vec<egui::Pos2>> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(p) => Some(p.points.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn indexed_visible_paths_equal_exact_across_domains_bounds_and_reversal() {
    let mut points = (0..3000)
        .map(|i| [i as f64 / 30.0 - 1.0, (i as f64 / 13.0).sin() * 50.0])
        .collect::<Vec<_>>();
    for i in [200, 500, 1000, 2500] {
        points[i] = [f64::NAN; 2];
    }
    // Include duplicate-X vertical segments and a restart in a new finite run.
    points.extend([[f64::NAN; 2], [1.0, 50.0], [1.0, 60.0], [200.0, 70.0]]);
    let mut data = PlotData::new(points);
    data.build_line_index(1024 * 1024).unwrap();
    let ctx = Context::default();
    for scales in [
        [AxisScale::Linear; 2],
        [AxisScale::Log10; 2],
        [AxisScale::Linear, AxisScale::Log10],
    ] {
        for reversed in [[false; 2], [true; 2], [true, false]] {
            for x in [[0.1, 80.0], [10.123, 10.126], [201.0, 202.0], [1.0, 1.01]] {
                let mut reference = Vec::new();
                for sampling in [LineSampling::Exact, LineSampling::Visible] {
                    let mut state = PlotState::default();
                    state.set_bounds(PlotBounds::new([x[0], 1.0], [x[1], 100.0]).unwrap());
                    let (_, output) = capture(&ctx, vec![], true, |ui| {
                        Plot::new("p")
                            .x_axis(Axis::new().scale(scales[0]).reversed(reversed[0]))
                            .y_axis(Axis::new().scale(scales[1]).reversed(reversed[1]))
                            .show(ui, &mut state, &[Line::new(&data).sampling(sampling)])
                    });
                    let actual = paths(&output);
                    if sampling == LineSampling::Exact {
                        reference = actual;
                    } else {
                        assert_eq!(actual, reference, "{scales:?} {reversed:?} {x:?}");
                    }
                    output.drop_without_applying_deltas();
                }
            }
        }
    }
}

#[test]
fn fallback_is_exact_and_cloned_index_bytes_are_deduplicated() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut data = PlotData::new(vec![[0.0; 2], [1.0; 2], [2.0, 0.0]]);
    let mut reference = Vec::new();
    for sampling in [
        LineSampling::Exact,
        LineSampling::Visible,
        LineSampling::MinMax,
    ] {
        let (response, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show(ui, &mut state, &[Line::new(&data).sampling(sampling)])
        });
        let actual = paths(&output);
        if sampling == LineSampling::Exact {
            reference = actual;
        } else {
            assert_eq!(response.diagnostics.lod_fallback_series, 1);
            assert_eq!(actual, reference);
        }
        output.drop_without_applying_deltas();
    }
    data.build_line_index(4096).unwrap();
    let clone = data.clone();
    let response = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data), Line::new(&clone)])
    });
    assert_eq!(
        response.diagnostics.unique_index_bytes,
        data.line_index_bytes()
    );
    assert_eq!(
        response.diagnostics.unique_sample_bytes,
        data.sample_bytes()
    );
}

#[test]
fn million_point_minmax_has_bounded_work_while_markers_still_visit_raw_data() {
    let ctx = Context::default();
    let mut data = PlotData::new(
        (0..1_000_000)
            .map(|i| [i as f64, (i % 100) as f64])
            .collect::<Vec<_>>(),
    );
    data.build_line_index(4 * 1024 * 1024).unwrap();
    let mut state = PlotState::default();
    let (response, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_hover(false).show(
            ui,
            &mut state,
            &[Line::new(&data).sampling(LineSampling::MinMax)],
        )
    });
    let columns = response.transform.rect().width().ceil() as usize;
    let d = response.diagnostics;
    assert_eq!(d.indexed_line_series, 1);
    assert!(d.visited_segments < columns * 4);
    assert!(d.query_comparisons < columns * 22);
    assert!(d.query_scanned_points < columns * 128);
    assert!(d.query_index_nodes < columns * 32);
    output.drop_without_applying_deltas();
    let (response, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_hover(false).show(
            ui,
            &mut state,
            &[Line::new(&data)
                .sampling(LineSampling::MinMax)
                .markers(egui_customplot::MarkerStyle::default())],
        )
    });
    assert_eq!(response.diagnostics.visited_marker_points, data.len());
    output.drop_without_applying_deltas();
}

#[test]
fn unsorted_input_rejects_index_without_disabling_the_reference_renderer() {
    let ctx = Context::default();
    let mut data = PlotData::new(vec![[2.0, 0.0], [1.0, 1.0], [3.0, 0.0]]);
    assert_eq!(
        data.build_line_index(4096),
        Err(egui_customplot::LineIndexError::UnsortedX { point_index: 1 })
    );
    let mut state = PlotState::default();
    let response = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(
            ui,
            &mut state,
            &[Line::new(&data).sampling(LineSampling::MinMax)],
        )
    });
    assert_eq!(response.diagnostics.lod_fallback_series, 1);
    assert_eq!(response.diagnostics.visited_segments, 2);
    assert_eq!(response.diagnostics.output_points, 3);
}
