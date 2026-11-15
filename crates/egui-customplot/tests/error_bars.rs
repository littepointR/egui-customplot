mod support;

use egui::{Color32, Context, Event, Shape};
use egui_customplot::{
    Axis, AxisScale, ErrorBar, ErrorBarData, ErrorBars, Line, Plot, PlotBounds, PlotData,
    PlotState, YAxis,
};
use support::{capture, run};

fn segments(output: &egui::FullOutput) -> Vec<[egui::Pos2; 2]> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::LineSegment { points, stroke } if stroke.color == Color32::RED => Some(*points),
            _ => None,
        })
        .collect()
}

#[test]
fn asymmetric_xy_whiskers_have_independent_stems_and_logical_point_caps() {
    for dpi in [1.0, 2.0] {
        let ctx = Context::default();
        ctx.set_pixels_per_point(dpi);
        let data = ErrorBarData::new(vec![
            ErrorBar::new([5.0; 2])
                .x_error([1.0, 2.0])
                .y_error([2.0, 3.0]),
        ]);
        let mut state = PlotState::default();
        let (out, output) = capture(&ctx, vec![], true, |ui| {
            Plot::new("p").show_layers(
                ui,
                &mut state,
                &[ErrorBars::new(&data)
                    .color(Color32::RED)
                    .cap_width(12.0)
                    .into()],
            )
        });
        let lines = segments(&output);
        assert_eq!(lines.len(), 6);
        assert_eq!(out.diagnostics.output_error_bar_segments, 6);
        assert_eq!(
            lines[0],
            [
                out.transform.position_from_point([4.0, 5.0]),
                out.transform.position_from_point([7.0, 5.0])
            ]
        );
        assert_eq!(
            lines[3],
            [
                out.transform.position_from_point([5.0, 3.0]),
                out.transform.position_from_point([5.0, 8.0])
            ]
        );
        for i in [1, 2, 4, 5] {
            assert!((lines[i][0].distance(lines[i][1]) - 12.0).abs() < 0.001);
        }
        output.drop_without_applying_deltas();
    }
}

#[test]
fn large_crossing_stems_clip_before_projection_without_false_boundary_caps() {
    let ctx = Context::default();
    let data = ErrorBarData::new(vec![
        ErrorBar::new([0.0; 2])
            .x_error([1e300; 2])
            .y_error([1e300; 2]),
    ]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([-1.0; 2], [1.0; 2]).unwrap());
    let (out, output) = capture(&ctx, vec![], true, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[ErrorBars::new(&data).color(Color32::RED).into()],
        )
    });
    assert_eq!(out.diagnostics.output_error_bar_segments, 2);
    assert!(
        segments(&output)
            .iter()
            .flatten()
            .all(|p| p.is_finite() && out.transform.rect().contains(*p))
    );
    output.drop_without_applying_deltas();
}

#[test]
fn invalid_offsets_overflow_and_log_crossings_are_omitted_as_whole_records() {
    let ctx = Context::default();
    let data = ErrorBarData::new(vec![
        ErrorBar::new([2.0; 2]).y_error([-1.0, 1.0]),
        ErrorBar::new([2.0; 2]).x_error([f64::INFINITY, 1.0]),
        ErrorBar::new([f64::MAX; 2]).y_error([0.0, f64::MAX]),
        ErrorBar::new([2.0; 2]).y_error([2.0, 1.0]),
        ErrorBar::new([10.0; 2])
            .x_error([2.0, 4.0])
            .y_error([1.0, 3.0]),
    ]);
    let mut state = PlotState::default();
    let plot = || {
        Plot::new("p")
            .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
    };
    let bars = ErrorBars::new(&data)
        .color(Color32::RED)
        .y_axis(YAxis::Right);
    let first = run(&ctx, vec![], false, |ui| {
        plot().show_layers(ui, &mut state, &[bars.into()])
    });
    assert_eq!(
        (
            first.diagnostics.invalid_error_bars,
            first.diagnostics.domain_invalid_error_bars
        ),
        (3, 1)
    );
    assert_eq!(first.diagnostics.output_error_bar_segments, 6);
    let pos = first
        .right_transform
        .unwrap()
        .position_from_point([10.0; 2]);
    let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        plot().show_layers(ui, &mut state, &[bars.into()])
    });
    let point = out.hovered_point.unwrap();
    assert_eq!(
        (point.point_index, point.point, point.y_axis),
        (4, [10.0; 2], YAxis::Right)
    );
}

#[test]
fn center_picking_keeps_submission_ties_and_ignores_zero_or_invisible_records() {
    let ctx = Context::default();
    let errors = ErrorBarData::new(vec![ErrorBar::new([1.0; 2]).y_error([0.5; 2])]);
    let points = PlotData::new(vec![[1.0; 2]]);
    for errors_first in [false, true] {
        let mut layers = [Line::new(&points).into(), ErrorBars::new(&errors).into()];
        if errors_first {
            layers.reverse();
        }
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        let pos = first.transform.position_from_point([1.0; 2]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        assert_eq!(out.hovered_point.unwrap().series_index, 0);
    }
    for width in [0.0, -1.0, f32::NAN] {
        let mut state = PlotState::default();
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(
                ui,
                &mut state,
                &[ErrorBars::new(&errors)
                    .width(width)
                    .color(Color32::RED)
                    .into()],
            )
        });
        assert_eq!(out.diagnostics.output_error_bar_segments, 0);
        assert!(segments(&output).is_empty());
        output.drop_without_applying_deltas();
    }
}

#[test]
fn invalid_cap_width_omits_caps_not_valid_stems_and_never_connects_centers() {
    let ctx = Context::default();
    let data = ErrorBarData::new(vec![
        ErrorBar::new([1.0; 2]).y_error([0.5; 2]),
        ErrorBar::new([3.0; 2]).y_error([0.5; 2]),
        ErrorBar::new([2.0; 2]),
    ]);
    let mut state = PlotState::default();
    let (out, output) = capture(&ctx, vec![], true, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[ErrorBars::new(&data)
                .color(Color32::RED)
                .cap_width(f32::NAN)
                .into()],
        )
    });
    assert_eq!(out.diagnostics.output_error_bar_segments, 2);
    assert_eq!(out.diagnostics.numeric_rejections, 1);
    assert!(segments(&output).iter().all(|[a, b]| a.x == b.x));
    output.drop_without_applying_deltas();
}
