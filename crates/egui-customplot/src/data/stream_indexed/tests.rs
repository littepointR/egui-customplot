use std::collections::VecDeque;

use egui::pos2;

use super::*;
use crate::{AxisScale, PlotBounds, PlotTransform, StreamError};

#[test]
fn ring_wrap_extents_monotonicity_and_idempotence_match_reference() {
    let mut stream = IndexedStream::new(150, 256 * 1024).unwrap();
    let mut ref_q = VecDeque::new();
    for batch_id in 1..=25u64 {
        let batch = (0..37)
            .map(|i| {
                let x = (batch_id as usize * 37 + i) as f64 * 0.5;
                let y = if i == 11 {
                    f64::NAN
                } else {
                    (x * 0.3).sin() * 10.0 + 20.0
                };
                [x, y]
            })
            .collect::<Vec<_>>();
        let rep = stream.append_once(batch_id, &batch).unwrap();
        assert!(rep.applied);
        assert!(!stream.append_once(batch_id, &batch).unwrap().applied);
        for p in batch {
            ref_q.push_back(p);
            if ref_q.len() > 150 {
                ref_q.pop_front();
            }
        }
        assert_eq!(stream.len(), ref_q.len());
        assert!(stream.is_monotonic_x());
        let (a, b) = stream.as_slices();
        for (actual, expected) in a.iter().chain(b).zip(&ref_q) {
            assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
        }
        let mut expected_min = [f64::INFINITY; 2];
        let mut expected_max = [f64::NEG_INFINITY; 2];
        for p in &ref_q {
            if p[0].is_finite() && p[1].is_finite() {
                expected_min[0] = expected_min[0].min(p[0]);
                expected_min[1] = expected_min[1].min(p[1]);
                expected_max[0] = expected_max[0].max(p[0]);
                expected_max[1] = expected_max[1].max(p[1]);
            }
        }
        assert_eq!(
            stream.extent([AxisScale::Linear; 2]),
            Some((expected_min, expected_max))
        );
    }
}

#[test]
fn query_and_pick_handle_ring_wraps_and_evicted_unsorted_glitches() {
    let mut stream = IndexedStream::new(300, 512 * 1024).unwrap();
    let unsorted = vec![[10.0, 5.0], [2.0, 9.0], [12.0, 1.0]];
    stream.append_once(1, &unsorted).unwrap();
    assert!(!stream.is_monotonic_x());
    let sorted: Vec<PlotPoint> = (0..300)
        .map(|i| [20.0 + i as f64, ((i * 7) % 50) as f64 + 1.0])
        .collect();
    stream.append_once(2, &sorted).unwrap();
    assert!(stream.is_monotonic_x());

    let transform = PlotTransform::new(
        PlotBounds::new([50.0, 0.0], [200.0, 55.0]).unwrap(),
        egui::Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(32.0, 100.0)),
    )
    .unwrap();
    let q = stream.query(transform, 1.0, true);
    assert!(q.monotonic);
    assert!(!q.slots.is_empty());

    let target = sorted[120];
    let pos = transform.position_from_point(target);
    let mut stats = crate::data::QueryStats::default();
    let hit = stream.pick(transform, pos, 8.0, 8.0, &mut stats).unwrap();
    assert_eq!(hit.0, 120);
    assert_eq!(hit.1, target);
    assert!(matches!(
        stream.append_once(1, &sorted),
        Err(StreamError::StaleBatch { .. })
    ));
}
