mod support;

use egui::Context;
use egui_customplot::{Line, Plot, PlotState, PlotStream, StreamError};
use support::run;

#[test]
fn ring_wrap_oversize_reports_and_immutable_snapshots_match_chronology() {
    let mut stream = PlotStream::new(3, 48).unwrap();
    let first = stream.append_once(1, &[[1.0; 2], [2.0; 2]]).unwrap();
    assert_eq!(
        (first.applied, first.revision, first.evicted_points),
        (true, 1, 0)
    );
    let snapshot = stream.snapshot();
    let report = stream.append_once(2, &[[3.0; 2], [4.0; 2]]).unwrap();
    assert_eq!(report.evicted_points, 1);
    assert_eq!(stream.snapshot().points(), &[[2.0; 2], [3.0; 2], [4.0; 2]]);
    assert_eq!(snapshot.points(), &[[1.0; 2], [2.0; 2]]);
    let report = stream
        .append_once(3, &[[5.0; 2], [6.0; 2], [7.0; 2], [8.0; 2], [9.0; 2]])
        .unwrap();
    assert_eq!(
        (
            report.retained_input_points,
            report.evicted_points,
            report.dropped_input_points
        ),
        (3, 3, 2)
    );
    assert_eq!(stream.snapshot().points(), &[[7.0; 2], [8.0; 2], [9.0; 2]]);
    assert_eq!(stream.allocated_bytes(), 48);
    stream.clear().unwrap();
    assert!(stream.is_empty());
    assert_eq!(stream.revision(), 4);
    assert!(!stream.append_once(3, &[[10.0; 2]]).unwrap().applied);
    assert!(matches!(
        stream.append_once(2, &[]),
        Err(StreamError::StaleBatch { .. })
    ));
    stream.append_once(4, &[]).unwrap();
    assert_eq!(stream.revision(), 4);
}

#[test]
fn ring_matches_reference_over_many_wraps_and_keeps_domain_breaks() {
    let mut stream = PlotStream::new(79, 79 * 16).unwrap();
    let mut reference = std::collections::VecDeque::new();
    for id in 0..10_000u64 {
        let batch = (0..(id as usize * 17 % 150))
            .map(|i| {
                if i % 29 == 0 {
                    [f64::NAN; 2]
                } else {
                    [(id as usize * 150 + i) as f64, (i % 7) as f64 - 1.0]
                }
            })
            .collect::<Vec<_>>();
        stream.append_once(id, &batch).unwrap();
        for p in batch {
            reference.push_back(p);
            if reference.len() > 79 {
                reference.pop_front();
            }
        }
        let (a, b) = stream.as_slices();
        assert_eq!(stream.len(), reference.len());
        for (actual, expected) in a.iter().chain(b).zip(&reference) {
            assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
        }
        assert_eq!(stream.allocated_bytes(), 79 * 16);
    }
    let mut snapshot = stream.snapshot();
    snapshot.build_line_index(64 * 1024).unwrap();
    assert!(snapshot.non_finite_points() > 0);
}

#[test]
fn invalid_capacity_and_budget_fail_before_large_allocation() {
    assert!(matches!(
        PlotStream::new(0, 0),
        Err(StreamError::ZeroCapacity)
    ));
    assert!(matches!(
        PlotStream::new(100, 1599),
        Err(StreamError::BudgetExceeded {
            required_bytes: 1600,
            ..
        })
    ));
    assert!(matches!(
        PlotStream::new(usize::MAX, usize::MAX),
        Err(StreamError::SizeOverflow)
    ));
}

#[test]
fn multipass_applies_batch_once_and_each_published_snapshot_has_a_new_version() {
    let ctx = Context::default();
    let mut stream = PlotStream::new(100, 1600).unwrap();
    let mut state = PlotState::default();
    let mut versions = Vec::new();
    let response = run(&ctx, vec![], true, |ui| {
        let report = stream.append_once(42, &[[1.0; 2], [2.0; 2]]).unwrap();
        assert_eq!(report.applied, versions.is_empty());
        let snapshot = stream.snapshot();
        versions.push(snapshot.version());
        Plot::new("p").show(ui, &mut state, &[Line::new(&snapshot)])
    });
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.revision(), 1);
    assert_ne!(versions[0], versions[1]);
    assert_eq!(response.diagnostics.input_points, 2);
}
