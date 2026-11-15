#[path = "support/line.rs"]
mod line_support;
mod support;

use egui::{Color32, Context, Event, IdSalt, Modifiers};
use egui_customplot::{
    Axis, AxisScale, Line, LineInterpolation as Interpolation, LinePattern, LineSampling,
    MarkerStyle, Plot, PlotBounds, PlotCache, PlotData, PlotState, YAxis,
};
use line_support::{assert_axis_aligned, paths};
use support::{capture, run};

#[test]
fn raw_markers_and_right_axis_hover_never_treat_synthetic_corners_as_samples() {
    let ctx = Context::default();
    let mut data = PlotData::new(vec![[1.0, 10.0], [4.0, 1000.0], [10.0, 100.0]]);
    for indexed in [false, true] {
        if indexed {
            data.build_line_index(4096).unwrap();
        }
        for interpolation in [Interpolation::StepAfter, Interpolation::StepBefore] {
            let mut state = PlotState::default();
            state.set_bounds(PlotBounds::new([0.5, -1.0], [11.0, 1.0]).unwrap());
            state.set_right_y_range([1.0, 10_000.0]).unwrap();
            let plot = || {
                Plot::new("p")
                    .pick_radius(5.0)
                    .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
                    .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            };
            let line = Line::new(&data)
                .id_salt("stairs")
                .name("Steps")
                .y_axis(YAxis::Right)
                .interpolation(interpolation)
                .sampling(LineSampling::MinMax)
                .markers(MarkerStyle::default());
            let first = run(&ctx, vec![], false, |ui| {
                plot().show(ui, &mut state, &[line])
            });
            assert_eq!(first.diagnostics.output_markers, data.len());
            assert_eq!(first.diagnostics.visited_marker_points, data.len());
            let transform = first.right_transform.unwrap();
            for (index, point) in data.points().iter().enumerate() {
                let pos = transform.position_from_point(*point);
                let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
                    plot().show(ui, &mut state, &[line])
                });
                let picked = out.hovered_point.unwrap();
                assert_eq!(picked.point_index, index);
                assert_eq!(picked.point, *point);
                assert_eq!(picked.y_axis, YAxis::Right);
                assert_eq!(picked.series_id, IdSalt::new("stairs"));
                assert_eq!(out.diagnostics.pick_index_runs, usize::from(indexed));
            }
            let corner = if interpolation == Interpolation::StepAfter {
                [4.0, 10.0]
            } else {
                [1.0, 1000.0]
            };
            let out = run(
                &ctx,
                vec![Event::PointerMoved(transform.position_from_point(corner))],
                true,
                |ui| plot().show(ui, &mut state, &[line]),
            );
            assert!(
                out.hovered_point.is_none(),
                "a derived corner is not an input sample"
            );
        }
    }
}

#[test]
fn cache_keys_include_interpolation_while_styles_visibility_and_layout_remain_independent() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0, 2.0], [3.0, 1.0]]);
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(64 * 1024);
    let mut after = Vec::new();
    for (frame, interpolation) in [
        Interpolation::Linear,
        Interpolation::StepAfter,
        Interpolation::StepBefore,
        Interpolation::StepAfter,
    ]
    .into_iter()
    .enumerate()
    {
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").legend(true).show_cached(
                ui,
                &mut state,
                &[Line::new(&data)
                    .id_salt("s")
                    .name("Signal")
                    .interpolation(interpolation)
                    .color(if frame < 3 {
                        Color32::RED
                    } else {
                        Color32::BLUE
                    })
                    .width(1.0 + frame as f32)],
                &mut cache,
            )
        });
        if frame < 3 {
            assert_eq!(out.diagnostics.line_cache_misses, 1);
        } else {
            assert_eq!(out.diagnostics.line_cache_hits, 1);
            assert_eq!(out.diagnostics.visited_segments, 0);
            assert_eq!(paths(&output), after);
        }
        if interpolation != Interpolation::Linear {
            assert_axis_aligned(&paths(&output));
        }
        if frame == 1 {
            after = paths(&output);
        }
        assert_eq!(out.legend_entries.len(), 1);
        output.drop_without_applying_deltas();
    }
    let out = run(&ctx, vec![], true, |ui| {
        Plot::new("p").legend(true).show_cached(
            ui,
            &mut state,
            &[Line::new(&data)
                .id_salt("s")
                .name("Signal")
                .interpolation(Interpolation::StepAfter)
                .pattern(LinePattern::Dashed {
                    length: 7.0,
                    gap: 3.0,
                })],
            &mut cache,
        )
    });
    assert_eq!(out.diagnostics.line_cache_hits, 1);
    assert!(out.diagnostics.output_paths > 1);
    state.set_series_visible(IdSalt::new("s"), false);
    let out = run(&ctx, vec![], true, |ui| {
        Plot::new("p").legend(true).show_cached(
            ui,
            &mut state,
            &[Line::new(&data)
                .id_salt("s")
                .name("Signal")
                .interpolation(Interpolation::StepAfter)],
            &mut cache,
        )
    });
    assert_eq!(out.diagnostics.output_paths, 0);
    assert_eq!(out.diagnostics.line_cache_hits, 0);
    assert!(cache.retained_bytes() <= cache.budget_bytes());
    cache.clear();
    assert_eq!(cache.retained_bytes(), 0);
}

#[test]
fn minmax_omitted_raw_sample_is_still_pickable_and_snapshots_invalidate_step_geometry() {
    let ctx = Context::default();
    let mut data = PlotData::new(
        (0..100_000)
            .map(|i| [i as f64, (i % 3) as f64 * 5.0])
            .collect::<Vec<_>>(),
    );
    // Four coordinate-domain run tables include many log-Y breaks at zero.
    data.build_line_index(2 * 1024 * 1024).unwrap();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(1024 * 1024);
    let line = Line::new(&data)
        .interpolation(Interpolation::StepAfter)
        .sampling(LineSampling::MinMax);
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_cached(ui, &mut state, &[line], &mut cache)
    });
    let displayed = paths(&output);
    let (index, point) = data
        .points()
        .iter()
        .enumerate()
        .skip(10_000)
        .find(|(_, p)| {
            p[1] == 5.0 && !displayed[0].contains(&out.transform.position_from_point(**p))
        })
        .unwrap();
    let pos = out.transform.position_from_point(*point);
    output.drop_without_applying_deltas();
    let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        Plot::new("p")
            .pick_radius(0.001)
            .show_cached(ui, &mut state, &[line], &mut cache)
    });
    assert_eq!(out.hovered_point.unwrap().point_index, index);
    assert_eq!(out.diagnostics.line_cache_hits, 1);
    assert!(out.diagnostics.visited_pick_points < data.len() / 10);
    let replacement = PlotData::new(vec![[0.0, 10.0], [99_999.0, 0.0]]);
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_cached(
            ui,
            &mut state,
            &[Line::new(&replacement)
                .interpolation(Interpolation::StepAfter)
                .sampling(LineSampling::MinMax)],
            &mut cache,
        )
    });
    assert_eq!(out.diagnostics.line_cache_misses, 1);
}

#[test]
fn zooming_steps_keeps_single_and_repeated_layout_navigation_identical() {
    let data = PlotData::new(vec![[1.0; 2], [10.0, 100.0], [100.0, 10.0]]);
    let mut results = Vec::new();
    for multipass in [false, true] {
        let ctx = Context::default();
        let mut state = PlotState::default();
        let plot = || {
            Plot::new("p")
                .allow_scroll_zoom(true)
                .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
        };
        let line = Line::new(&data).interpolation(Interpolation::StepAfter);
        let first = run(&ctx, vec![], false, |ui| {
            plot().show(ui, &mut state, &[line])
        });
        let pos = first.transform.rect().center();
        run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            plot().show(ui, &mut state, &[line])
        });
        let (out, output) = capture(
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
            |ui| plot().show(ui, &mut state, &[line]),
        );
        assert!(out.view_changed);
        assert_axis_aligned(&paths(&output));
        results.push((out.transform.bounds(), paths(&output)));
        output.drop_without_applying_deltas();
    }
    assert_eq!(results[0], results[1]);
}
