mod support;

use std::sync::Arc;

use egui::{Color32, Context, Event, IdSalt, Stroke};
use egui_customplot::{
    Axis, AxisScale, Line, LinePattern, MarkerStyle, Plot, PlotBounds, PlotCache, PlotData,
    PlotState, StemOrientation, Stems, YAxis,
};
use support::run;

#[test]
fn visibility_styles_zero_rods_and_baseline_only_match_raw_endpoint_picking() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0, 2.0], [3.0, 0.0]]);
    let base = Stems::new(&data).id_salt("s").show_baseline(false);
    let invalid_pattern = LinePattern::Dashed {
        length: 0.0,
        gap: 3.0,
    };
    for (stems, rods, markers, picked) in [
        (base, 1, 2, [true, true]),
        (base.show_markers(false), 1, 0, [true, false]),
        (base.width(0.0).show_markers(false), 0, 0, [false; 2]),
        (base.width(0.0), 0, 2, [true; 2]),
        (base.width(f32::NAN), 0, 2, [true; 2]),
        (base.color(Color32::TRANSPARENT), 0, 0, [false; 2]),
        (
            base.width(0.0).markers(MarkerStyle {
                radius: f32::NAN,
                ..Default::default()
            }),
            0,
            0,
            [false; 2],
        ),
        (
            base.pattern(invalid_pattern).show_markers(false),
            0,
            0,
            [false; 2],
        ),
        (base.pattern(invalid_pattern), 0, 2, [true; 2]),
        (
            base.width(0.0)
                .show_markers(false)
                .show_baseline(true)
                .baseline_stroke(Stroke::new(2.0, Color32::GREEN)),
            0,
            0,
            [false; 2],
        ),
        (base.baseline(f64::NAN), 0, 0, [false; 2]),
    ] {
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([0.0, -1.0], [4.0, 3.0]).unwrap());
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[stems.into()])
        });
        assert_eq!(first.diagnostics.output_stem_segments, rods);
        assert_eq!(first.diagnostics.output_markers, markers);
        for (index, &point) in data.points().iter().enumerate() {
            let pos = first.transform.position_from_point(point);
            let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
                Plot::new("p")
                    .pick_radius(2.0)
                    .show_layers(ui, &mut state, &[stems.into()])
            });
            assert_eq!(
                out.hovered_point.is_some(),
                picked[index],
                "{stems:?} {index}"
            );
            if let Some(point) = out.hovered_point {
                assert_eq!(point.point_index, index);
            }
        }
        // Neither the rod interior nor the independent baseline is a raw sample.
        for point in [[1.0, 1.0], [2.0, 0.0]] {
            let out = run(
                &ctx,
                vec![Event::PointerMoved(
                    first.transform.position_from_point(point),
                )],
                false,
                |ui| {
                    Plot::new("p")
                        .pick_radius(2.0)
                        .show_layers(ui, &mut state, &[stems.into()])
                },
            );
            assert!(out.hovered_point.is_none());
        }
    }
}

#[test]
fn indexed_right_log_hover_preserves_raw_indices_axes_and_submission_ties() {
    let ctx = Context::default();
    let left = PlotData::new(vec![[1.0, -1.0], [3.0, 1.0]]);
    let mut data = PlotData::new(vec![[1.0, 10.0], [2.0, -1.0], [2.0, 100.0], [3.0, 50.0]]);
    for indexed in [false, true] {
        if indexed {
            data.build_line_index(4096).unwrap();
        }
        let mut state = PlotState::default();
        let plot = || {
            Plot::new("p")
                .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
                .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
        };
        let stems = Stems::new(&data)
            .baseline(1.0)
            .y_axis(YAxis::Right)
            .id_salt("s");
        let layers = [
            Line::new(&left).into(),
            stems.into(),
            stems.id_salt("later").into(),
        ];
        let first = run(&ctx, vec![], false, |ui| {
            plot().show_layers(ui, &mut state, &layers)
        });
        assert!(first.transform.bounds().max()[1] < 2.0);
        let transform = first.right_transform.unwrap();
        assert!(transform.bounds().min()[1] < 1.0);
        assert_eq!(first.diagnostics.domain_invalid_points, 2); // Once per descriptor.
        assert_eq!(
            first.diagnostics.unique_sample_bytes,
            left.sample_bytes() + data.sample_bytes()
        );
        for index in [0, 2, 3] {
            let point = data.points()[index];
            let out = run(
                &ctx,
                vec![Event::PointerMoved(transform.position_from_point(point))],
                true,
                |ui| plot().show_layers(ui, &mut state, &layers),
            );
            let hit = out.hovered_point.unwrap();
            assert_eq!(
                (hit.point_index, hit.point, hit.y_axis),
                (index, point, YAxis::Right)
            );
            assert_eq!(hit.series_id, IdSalt::new("s"));
        }
    }
    // A requested but unconfigured right axis follows the existing left fallback.
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut PlotState::default(),
            &[Stems::new(&data).y_axis(YAxis::Right).into()],
        )
    });
    assert_eq!(out.diagnostics.unconfigured_right_axis_series, 1);
}

#[test]
fn optional_index_narrows_endpoint_picking_but_rendering_still_scans_all_rods() {
    let ctx = Context::default();
    let mut data = PlotData::new(
        (1..=100_000)
            .map(|i| [i as f64, (i % 9 + 1) as f64])
            .collect::<Vec<_>>(),
    );
    let mut reference = None;
    for indexed in [false, true] {
        if indexed {
            data.build_line_index(1024 * 1024).unwrap();
        }
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([4000.0, 0.0], [4020.0, 10.0]).unwrap());
        let stems = Stems::new(&data).show_markers(false);
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[stems.into()])
        });
        let point = data.points()[4004];
        let out = run(
            &ctx,
            vec![Event::PointerMoved(
                first.transform.position_from_point(point),
            )],
            false,
            |ui| {
                Plot::new("p")
                    .pick_radius(0.1)
                    .show_layers(ui, &mut state, &[stems.into()])
            },
        );
        let hit = out.hovered_point.unwrap();
        let actual = (hit.point_index, hit.point);
        if indexed {
            assert_eq!(Some(actual), reference);
            assert!(out.diagnostics.visited_pick_points < 10);
        } else {
            reference = Some(actual);
            assert_eq!(out.diagnostics.visited_pick_points, data.len());
        }
        assert_eq!(out.diagnostics.visited_stems, data.len());
        assert_eq!(out.diagnostics.output_stem_segments, 21);
    }
}

#[test]
fn samples_and_indexes_are_deduplicated_and_not_retained_by_state_context_or_line_cache() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(64 * 1024);
    for _ in 0..32 {
        let points: Arc<[[f64; 2]]> = vec![[1.0, 2.0], [3.0, 4.0]].into();
        let mut data = PlotData::new(points.clone());
        data.build_line_index(4096).unwrap();
        let clone = data.clone();
        let layers = [
            Stems::new(&data).id_salt("a").into(),
            Stems::new(&clone)
                .orientation(StemOrientation::Horizontal)
                .id_salt("b")
                .into(),
            Line::new(&clone).id_salt("c").into(),
        ];
        let out = run(&ctx, vec![], true, |ui| {
            Plot::new("p").show_layers_cached(ui, &mut state, &layers, &mut cache)
        });
        assert_eq!(out.diagnostics.unique_sample_bytes, data.sample_bytes());
        assert_eq!(out.diagnostics.unique_index_bytes, data.line_index_bytes());
        assert_eq!(out.diagnostics.input_points, 3 * data.len());
        drop((data, clone));
        assert_eq!(Arc::strong_count(&points), 1);
        assert!(cache.retained_bytes() <= cache.budget_bytes());
    }
    cache.clear();
    assert_eq!(cache.retained_bytes(), 0);
}

#[test]
fn hidden_and_parent_clipped_stems_skip_geometry_and_leave_no_stale_visibility() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0, 2.0], [3.0, 4.0]]);
    let mut state = PlotState::default();
    let stems = Stems::new(&data).id_salt("s").name("Stems");
    let clipped = run(&ctx, vec![], true, |ui| {
        ui.set_clip_rect(egui::Rect::NOTHING);
        Plot::new("p").show_layers(ui, &mut state, &[stems.into()])
    });
    assert!(clipped.diagnostics.painting_skipped);
    assert_eq!(
        (
            clipped.diagnostics.visited_stems,
            clipped.diagnostics.visited_marker_points
        ),
        (0, 0)
    );
    state.set_series_visible(IdSalt::new("s"), false);
    let hidden = run(&ctx, vec![], true, |ui| {
        Plot::new("p")
            .legend(true)
            .show_layers(ui, &mut state, &[stems.into()])
    });
    assert_eq!(hidden.diagnostics.visited_stems, 0);
    assert!(!hidden.legend_entries[0].visible);
    run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[])
    });
    assert!(state.series_visible(IdSalt::new("s")));
    let shown = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[stems.into()])
    });
    assert_eq!(shown.diagnostics.output_stem_segments, 2);
}
