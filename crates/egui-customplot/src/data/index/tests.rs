use super::*;
use crate::{PlotBounds, PlotData, PlotTransform};

fn transform(scales: [AxisScale; 2], reversed: [bool; 2]) -> PlotTransform {
    PlotTransform::with_axes(
        PlotBounds::new([1.0; 2], [100.0; 2]).unwrap(),
        egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(32.0, 80.0)),
        scales,
        reversed,
    )
    .unwrap()
}

#[test]
fn run_tables_match_each_domain_and_allow_restarts_only_at_nonfinite_points() {
    let points = [
        [-2.0, 1.0],
        [0.0, 1.0],
        [1.0, -1.0],
        [1.0, 2.0],
        [2.0, 0.0],
        [3.0, 3.0],
        [f64::NAN; 2],
        [1.0, 1.0],
    ];
    let index = LineIndex::build(&points, usize::MAX).unwrap();
    assert_eq!(index.runs[0].as_ref(), &[0..6, 7..8]);
    assert_eq!(index.runs[1].as_ref(), &[2..6, 7..8]);
    assert_eq!(index.runs[2].as_ref(), &[0..2, 3..4, 5..6, 7..8]);
    assert_eq!(index.runs[3].as_ref(), &[3..4, 5..6, 7..8]);
    // A log-domain break does not excuse decreasing X in a finite XY run.
    assert!(matches!(
        LineIndex::build(&[[2.0, 1.0], [1.0, -1.0]], usize::MAX),
        Err(LineIndexError::UnsortedX { point_index: 1 })
    ));
}

#[test]
fn budget_boundary_failure_preservation_sharing_and_release_are_explicit() {
    let mut data = PlotData::new((0..1000).map(|i| [i as f64, 1.0]).collect::<Vec<_>>());
    assert!(matches!(
        data.build_line_index(0),
        Err(LineIndexError::BudgetExceeded { .. })
    ));
    assert_eq!(data.line_index_bytes(), 0);
    data.build_line_index(usize::MAX).unwrap();
    let bytes = data.line_index_bytes();
    let identity = data.index_identity();
    assert_eq!(
        data.build_line_index(bytes - 1),
        Err(LineIndexError::BudgetExceeded {
            required_bytes: bytes,
            budget_bytes: bytes - 1,
        })
    );
    assert_eq!(identity, data.index_identity());
    let mut shared = data.clone();
    assert_eq!(identity, shared.index_identity());
    data.clear_line_index();
    data.build_line_index(bytes).unwrap();
    assert_ne!(identity, data.index_identity());
    shared.clear_line_index();
    assert_eq!(shared.line_index_bytes(), 0);
}

#[test]
fn empty_singleton_and_all_invalid_indexes_query_without_paths() {
    for points in [vec![], vec![[2.0; 2]], vec![[f64::NAN; 2]; 100]] {
        let index = LineIndex::build(&points, usize::MAX).unwrap();
        for reduce in [false, true] {
            assert!(
                index
                    .query(
                        &points,
                        transform([AxisScale::Linear; 2], [false; 2]),
                        1.0,
                        reduce
                    )
                    .paths
                    .is_empty()
            );
        }
    }
}

#[test]
fn tree_extrema_match_raw_scan_for_partial_blocks_ties_and_gaps() {
    let points = (0..1025)
        .map(|i| {
            if i % 79 == 0 {
                [f64::NAN; 2]
            } else {
                [i as f64, (i * 17 % 47) as f64]
            }
        })
        .collect::<Vec<_>>();
    let index = LineIndex::build(&points, usize::MAX).unwrap();
    for start in (0..points.len()).step_by(7) {
        for length in [1, 2, 63, 64, 65, 127, 500] {
            let end = (start + length).min(points.len());
            let expected = (start..end)
                .filter(|i| valid(points[*i], 0))
                .fold(Extrema::default(), |e, i| {
                    e.merge(Extrema::point(i as u32, points[i][1]))
                });
            let result = index.extrema(&points, start..end, &mut QueryStats::default());
            assert_eq!((result.min, result.max), (expected.min, expected.max));
        }
    }
}

#[test]
fn minmax_keeps_order_boundaries_peaks_and_domain_breaks_at_multiple_dpis() {
    let points = (0..30_000)
        .map(|i| {
            let y = match i {
                1234 => 99.0,
                12_345 => 1.0,
                16_789 => -1.0,
                22_222 => f64::NAN,
                _ => 50.0,
            };
            [i as f64 / 300.0 + 1.0, y]
        })
        .collect::<Vec<_>>();
    let index = LineIndex::build(&points, usize::MAX).unwrap();
    for scales in [[AxisScale::Linear; 2], [AxisScale::Log10; 2]] {
        for reversed in [[false; 2], [true; 2]] {
            for dpi in [1.0, 2.0, 3.0] {
                let query = index.query(&points, transform(scales, reversed), dpi, true);
                let mask = usize::from(scales[0] == AxisScale::Log10)
                    | (usize::from(scales[1] == AxisScale::Log10) << 1);
                for path in &query.paths {
                    assert!(path.windows(2).all(|p| p[0] < p[1]));
                    assert!((path[0]..=*path.last().unwrap()).all(|i| valid(points[i], mask)));
                    assert!(path.len() <= 4 * (32.0 * dpi) as usize);
                }
                for peak in [1234, 12_345] {
                    assert!(query.paths.iter().flatten().any(|i| *i == peak));
                }
                assert!(query.stats.scanned_points < 3 * 32 * dpi as usize * 128);
                assert!(query.stats.visited_nodes > 0);
            }
        }
    }
}

#[test]
fn queries_keep_crossing_neighbors_duplicate_x_and_report_break_heavy_cost() {
    let points = [
        [0.0, 50.0],
        [101.0, 50.0],
        [f64::NAN; 2],
        [1.0, 3.0],
        [1.0, 4.0],
        [101.0, 50.0],
    ];
    let index = LineIndex::build(&points, usize::MAX).unwrap();
    assert_eq!(
        index
            .query(
                &points,
                transform([AxisScale::Linear; 2], [false; 2]),
                1.0,
                false
            )
            .paths,
        vec![vec![0, 1], vec![3, 4, 5]]
    );
    let points = (0..10_000)
        .flat_map(|_| [[1.0, 50.0], [100.0, 50.0], [f64::NAN; 2]])
        .collect::<Vec<_>>();
    let index = LineIndex::build(&points, usize::MAX).unwrap();
    let query = index.query(
        &points,
        transform([AxisScale::Linear; 2], [false; 2]),
        1.0,
        true,
    );
    assert_eq!(query.paths.len(), 10_000);
    assert_eq!(query.stats.visited_runs, 10_000);
    assert_eq!(query.stats.selected_points, 20_000);
}

#[test]
fn gallop_partition_point_matches_std_across_uniform_and_clustered_inputs() {
    let uniform: Vec<PlotPoint> = (0..5000).map(|i| [i as f64 * 0.25, 0.0]).collect();
    let clustered: Vec<PlotPoint> = (0..5000)
        .map(|i| {
            let x = if i < 100 {
                0.0
            } else if i < 4000 {
                (i / 50) as f64
            } else {
                10_000.0 + (i - 4000) as f64
            };
            [x, 0.0]
        })
        .collect();
    for data in [&uniform, &clustered] {
        for start in [0, 1, 63, 64, 500, 4900] {
            for end in [start, (start + 17).min(data.len()), data.len()] {
                for hint in [1, 32, 128, 1024] {
                    for boundary in [-1.0, 0.0, 10.5, 80.0, 500.0, 10_500.0, 20_000.0] {
                        let expected =
                            start + data[start..end].partition_point(|p| p[0] < boundary);
                        let mut cmp = 0;
                        let actual = query::gallop_partition_point(
                            data, start, end, hint, boundary, &mut cmp,
                        );
                        assert_eq!(actual, expected, "{start}..{end} hint={hint} b={boundary}");
                    }
                }
            }
        }
    }
}
