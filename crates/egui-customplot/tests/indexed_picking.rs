mod support;

use egui::{Context, Event};
use egui_customplot::{
    Axis, AxisScale, Line, LineSampling, Plot, PlotBounds, PlotData, PlotState, Scatter,
};
use support::run;

#[test]
fn indexed_picking_matches_raw_scan_with_log_reversal_duplicates_gaps_and_lod() {
    let points = (0..5000)
        .map(|i| {
            if i % 151 == 0 {
                [f64::NAN; 2]
            } else {
                [(i / 2 + 1) as f64, (i * 17 % 101) as f64 - 2.0]
            }
        })
        .collect::<Vec<_>>();
    let raw = PlotData::new(points);
    let mut indexed = raw.clone();
    indexed.build_line_index(1024 * 1024).unwrap();
    let ctx = Context::default();
    for scales in [[AxisScale::Linear; 2], [AxisScale::Log10; 2]] {
        for reversed in [[false; 2], [true; 2]] {
            for scatter in [false, true] {
                let mut state = PlotState::default();
                state.set_bounds(PlotBounds::new([1.0; 2], [2501.0, 100.0]).unwrap());
                let plot = || {
                    Plot::new("p")
                        .pick_radius(12.0)
                        .x_axis(Axis::new().scale(scales[0]).reversed(reversed[0]))
                        .y_axis(Axis::new().scale(scales[1]).reversed(reversed[1]))
                };
                let initial = run(&ctx, vec![], false, |ui| {
                    plot().show(ui, &mut state, &[Line::new(&raw)])
                });
                for i in [1, 51, 101, 202, 3000, 4999] {
                    if !initial.transform.is_valid_point(raw.points()[i]) {
                        continue;
                    }
                    let position = initial.transform.position_from_point(raw.points()[i]);
                    let mut expected = None;
                    for data in [&raw, &indexed] {
                        let out = run(&ctx, vec![Event::PointerMoved(position)], true, |ui| {
                            let series = if scatter {
                                Scatter::new(data).into()
                            } else {
                                Line::new(data).sampling(LineSampling::MinMax).into()
                            };
                            plot().show_series(ui, &mut state, &[series])
                        });
                        let actual = out
                            .hovered_point
                            .map(|p| (p.point_index, p.point, p.screen_distance));
                        if std::ptr::eq(data, &raw) {
                            expected = actual;
                        } else {
                            assert_eq!(actual, expected, "{scales:?} {reversed:?} {scatter} {i}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn just_outside_samples_that_round_to_screen_edges_remain_pickable() {
    let ctx = Context::default();
    for reversed in [false, true] {
        let mut data = PlotData::new(vec![[1.0 - 1e-12, 1.5]; 30]);
        for indexed in [false, true] {
            if indexed {
                data.build_line_index(4096).unwrap();
            }
            let mut state = PlotState::default();
            state.set_bounds(PlotBounds::new([1.0; 2], [2.0; 2]).unwrap());
            let plot = || Plot::new("p").x_axis(Axis::new().reversed(reversed));
            let first = run(&ctx, vec![], false, |ui| {
                plot().show(ui, &mut state, &[Line::new(&data)])
            });
            let pos = first.transform.position_from_point(data.points()[0]);
            let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
                plot().show(ui, &mut state, &[Line::new(&data)])
            });
            assert_eq!(out.hovered_point.unwrap().point_index, 0);
        }
    }
}

#[test]
fn indexed_hover_reduces_x_candidates_but_dense_equal_x_is_reported_honestly() {
    let ctx = Context::default();
    for dense in [false, true] {
        let mut data = PlotData::new(
            (0..100_000)
                .map(|i| [if dense { 50_000.0 } else { i as f64 }, (i % 100) as f64])
                .collect::<Vec<_>>(),
        );
        data.build_line_index(1024 * 1024).unwrap();
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([0.0, -1.0], [100_000.0, 101.0]).unwrap());
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show(
                ui,
                &mut state,
                &[Line::new(&data).sampling(LineSampling::MinMax)],
            )
        });
        let pos = first.transform.rect().center();
        let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p").show(
                ui,
                &mut state,
                &[Line::new(&data).sampling(LineSampling::MinMax)],
            )
        });
        assert!(out.hovered_point.is_some());
        assert_eq!(out.diagnostics.pick_index_runs, 1);
        if dense {
            assert_eq!(out.diagnostics.visited_pick_points, data.len());
        } else {
            assert!(out.diagnostics.visited_pick_points < data.len() / 20);
        }
        assert!(out.diagnostics.pick_index_comparisons < 40);
    }
}
