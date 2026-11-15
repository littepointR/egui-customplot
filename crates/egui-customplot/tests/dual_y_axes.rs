mod support;

use egui::{Color32, Context, Event, IdSalt, Modifiers, Shape};
use egui_customplot::{
    Axis, AxisScale, Line, LineSampling, Plot, PlotBounds, PlotCache, PlotData, PlotState, Scatter,
    Series, YAxis,
};
use support::{capture, run};

#[test]
fn independent_y_bounds_share_x_and_reach_the_actual_painter() {
    let ctx = Context::default();
    let left = PlotData::new(vec![[0.0, -1.0], [10.0, 1.0]]);
    let right = PlotData::new(vec![[0.0, 1000.0], [20.0, 3000.0]]);
    let mut state = PlotState::default();
    let series = [
        Line::new(&left).color(Color32::RED).into(),
        Line::new(&right)
            .y_axis(YAxis::Right)
            .color(Color32::BLUE)
            .into(),
    ];
    let (out, output) = capture(&ctx, vec![], true, |ui| {
        Plot::new("p")
            .right_y_axis(Axis::new().label("Pressure"))
            .show_series(ui, &mut state, &series)
    });
    let secondary = out.right_transform.unwrap();
    assert!(out.transform.bounds().max()[1] < 2.0);
    assert!(secondary.bounds().min()[1] > 800.0 && secondary.bounds().max()[1] > 3000.0);
    assert_eq!(out.transform.bounds().min()[0], secondary.bounds().min()[0]);
    assert_eq!(out.transform.bounds().max()[0], secondary.bounds().max()[0]);
    assert!(out.transform.bounds().max()[0] > 20.0);
    assert_eq!(out.transform.rect(), secondary.rect());
    assert!(!out.right_axis_ticks.is_empty());
    let paths = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(p) => Some(&p.points),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (path, (data, transform)) in paths
        .into_iter()
        .zip([(&left, out.transform), (&right, secondary)])
    {
        assert_eq!(
            path.as_slice(),
            data.points()
                .iter()
                .map(|p| transform.position_from_point(*p))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(out.diagnostics.output_paths, 2);
    assert!(state.bounds().is_none() && state.right_y_range().is_none());
    output.drop_without_applying_deltas();
}

#[test]
fn right_log_domain_indexed_pick_and_highlight_use_the_right_raw_coordinates() {
    let ctx = Context::default();
    let left = PlotData::new(vec![[1.0, -2.0], [2.0, -1.0]]);
    let mut right = PlotData::new(vec![[1.0, -100.0], [2.0, 10.0], [3.0, 1000.0]]);
    right.build_line_index(4096).unwrap();
    let mut state = PlotState::default();
    let series: [Series<'_>; 2] = [
        Line::new(&left).into(),
        Scatter::new(&right)
            .y_axis(YAxis::Right)
            .name("Pressure")
            .into(),
    ];
    let plot = || Plot::new("p").right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true));
    let first = run(&ctx, vec![], false, |ui| {
        plot().show_series(ui, &mut state, &series)
    });
    assert_eq!(
        first.diagnostics.domain_invalid_points, 1,
        "negative left Y stays valid"
    );
    let pos = first
        .right_transform
        .unwrap()
        .position_from_point(right.points()[2]);
    let (out, output) = capture(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        plot().show_series(ui, &mut state, &series)
    });
    let point = out.hovered_point.unwrap();
    assert_eq!(
        (point.y_axis, point.series_index, point.point_index),
        (YAxis::Right, 1, 2)
    );
    assert_eq!(point.point, [3.0, 1000.0]);
    assert!(out.hover_text.unwrap().contains("1.00000000e3"));
    assert_eq!(out.diagnostics.pick_index_runs, 1);
    assert!(
        output
            .shapes
            .iter()
            .any(|s| matches!(&s.shape, Shape::Circle(c) if c.center == pos && c.radius == 5.0))
    );
    output.drop_without_applying_deltas();
}

#[test]
fn right_only_series_fit_shared_x_and_hidden_series_do_not_expand_ranges() {
    let ctx = Context::default();
    let right = PlotData::new(vec![[100.0, 10_000.0], [200.0, 20_000.0]]);
    let mut state = PlotState::default();
    let lines = [Line::new(&right).id_salt("right").y_axis(YAxis::Right)];
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p")
            .right_y_axis(Axis::new())
            .show(ui, &mut state, &lines)
    });
    assert!(out.transform.bounds().min()[0] > 90.0 && out.transform.bounds().max()[0] > 200.0);
    assert_eq!(
        [
            out.transform.bounds().min()[1],
            out.transform.bounds().max()[1]
        ],
        [-1.0, 1.0]
    );
    state.set_series_visible(IdSalt::new("right"), false);
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p")
            .right_y_axis(Axis::new())
            .show(ui, &mut state, &lines)
    });
    assert_eq!(out.diagnostics.output_paths, 0);
    assert_eq!(out.transform.bounds(), PlotBounds::default());
    state.set_series_visible(IdSalt::new("right"), true);
    let fallback = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &lines)
    });
    assert_eq!(fallback.diagnostics.unconfigured_right_axis_series, 1);
    assert_eq!(fallback.diagnostics.output_paths, 1);
    assert!(fallback.right_transform.is_none());
    assert!(fallback.transform.bounds().max()[1] > 20_000.0);
}

#[test]
fn wheel_preserves_each_y_anchor_reversal_and_multipass_baseline() {
    let data = PlotData::new(vec![[1.0, 1.0], [100.0, 100.0]]);
    let mut results = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([1.0; 2], [100.0; 2]).unwrap());
        state.set_right_y_range([1.0, 10_000.0]).unwrap();
        let plot = || {
            Plot::new("p")
                .allow_scroll_zoom(true)
                .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
        };
        let initial = run(&ctx, vec![], false, |ui| {
            plot().show(ui, &mut state, &[Line::new(&data)])
        });
        let pos = initial.transform.rect().center() + egui::vec2(50.0, -30.0);
        let left_anchor = initial.transform.point_from_position(pos);
        let right_anchor = initial.right_transform.unwrap().point_from_position(pos);
        run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            plot().show(ui, &mut state, &[Line::new(&data)])
        });
        let out = run(
            &ctx,
            vec![
                Event::PointerMoved(pos),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 40.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Unknown,
                    modifiers: Modifiers::NONE,
                },
            ],
            multipass,
            |ui| plot().show(ui, &mut state, &[Line::new(&data)]),
        );
        let right = out.right_transform.unwrap();
        assert!((out.transform.point_from_position(pos)[1] - left_anchor[1]).abs() < 1e-10);
        assert!((right.point_from_position(pos)[1] - right_anchor[1]).abs() < 1e-8);
        assert_eq!(out.transform.bounds().min()[0], right.bounds().min()[0]);
        assert_eq!(out.transform.bounds().max()[0], right.bounds().max()[0]);
        assert!(out.view_changed && state.right_y_range().unwrap()[1] < 10_000.0);
        results.push((state.bounds(), state.right_y_range()));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn cached_geometry_keys_follow_each_series_axis_and_right_scale_changes_only_its_side() {
    let ctx = Context::default();
    let mut data = PlotData::new(vec![[1.0; 2], [10.0; 2], [100.0; 2]]);
    data.build_line_index(4096).unwrap();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(64 * 1024);
    let lines = [
        Line::new(&data).sampling(LineSampling::MinMax),
        Line::new(&data)
            .sampling(LineSampling::MinMax)
            .y_axis(YAxis::Right),
    ];
    for frame in 0..3 {
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .right_y_axis(Axis::new().reversed(frame == 2))
                .show_cached(ui, &mut state, &lines, &mut cache)
        });
        assert_eq!(out.diagnostics.unique_sample_bytes, data.sample_bytes());
        assert_eq!(out.diagnostics.unique_index_bytes, data.line_index_bytes());
        if frame == 1 {
            assert_eq!(out.diagnostics.line_cache_hits, 2);
        }
        if frame == 2 {
            assert_eq!(
                (
                    out.diagnostics.line_cache_hits,
                    out.diagnostics.line_cache_misses
                ),
                (1, 1)
            );
        }
    }
}

#[test]
fn invalid_manual_secondary_range_falls_back_and_home_resets_both_sides() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    assert!(state.set_right_y_range([2.0, 1.0]).is_err());
    state.set_right_y_range([-10.0, -1.0]).unwrap();
    state.set_bounds(PlotBounds::new([1.0; 2], [10.0; 2]).unwrap());
    let plot = || Plot::new("p").right_y_axis(Axis::new().scale(AxisScale::Log10));
    let out = run(&ctx, vec![], false, |ui| plot().show(ui, &mut state, &[]));
    assert!(out.diagnostics.invalid_right_y_range);
    assert!(out.right_transform.unwrap().bounds().min()[1] > 0.0);
    state.set_right_y_range([10.0, 100.0]).unwrap();
    ctx.memory_mut(|m| m.request_focus(out.response.id));
    let out = run(
        &ctx,
        vec![Event::Key {
            key: egui::Key::Home,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        true,
        |ui| plot().show(ui, &mut state, &[]),
    );
    assert!(out.view_changed);
    assert!(state.bounds().is_none() && state.right_y_range().is_none());
}
