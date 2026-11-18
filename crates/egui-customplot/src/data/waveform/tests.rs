use egui::pos2;

use super::*;
use crate::{AxisScale, PlotBounds, PlotTransform};

#[test]
fn waveform_construction_update_extents_and_idempotence() {
    let vals: Vec<f32> = (0..500)
        .map(|i| {
            if i == 100 {
                f32::NAN
            } else {
                (i as f32 * 0.1).sin() * 4.0
            }
        })
        .collect();
    let mut wave = UniformWaveform::from_f32(-10.0, 0.5, vals, 1024 * 1024).unwrap();
    assert_eq!(wave.len(), 500);
    assert_eq!(wave.non_finite_points(), 1);
    let ext_lin = wave.extent([AxisScale::Linear; 2]).unwrap();
    assert!((ext_lin.0[0] - (-10.0)).abs() < 1e-12);
    assert!((ext_lin.1[0] - (-10.0 + 499.0 * 0.5)).abs() < 1e-12);
    assert!(wave.invalid_points([AxisScale::Log10; 2]) > 20);

    let v0 = wave.version();
    assert!(wave.write_range_once(1, 200, &[99.0, -50.0]).unwrap());
    assert!(!wave.write_range_once(1, 200, &[0.0]).unwrap());
    assert_ne!(wave.version(), v0);
    let ext2 = wave.extent([AxisScale::Linear; 2]).unwrap();
    assert_eq!(ext2.0[1], -50.0);
    assert_eq!(ext2.1[1], 99.0);

    let sv = wave.samples_version();
    assert!(wave.set_x_origin_once(2, 1.0).unwrap());
    assert_eq!(wave.samples_version(), sv);
    assert!(wave.extent([AxisScale::Log10, AxisScale::Linear]).is_some());
}

#[test]
fn waveform_query_and_pick_are_exact_and_o1_bounded() {
    let vals: Vec<f32> = (0..200_000)
        .map(|i| ((i as f32 * 0.01).sin() * 10.0) + 15.0)
        .collect();
    let wave = UniformWaveform::from_f32(0.0, 0.001, vals, 16 * 1024 * 1024).unwrap();
    let transform = PlotTransform::new(
        PlotBounds::new([10.0, 0.0], [180.0, 30.0]).unwrap(),
        egui::Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(500.0, 300.0)),
    )
    .unwrap();
    let q = wave.query(transform, 1.0, true);
    assert!(q.stats.reduced);
    assert_eq!(q.stats.comparisons, 0);
    assert!(q.indices.len() <= 500 * 4);

    let target_idx = 50_000;
    let target = wave.point_at(target_idx);
    let pos = transform.position_from_point(target);
    let mut stats = crate::data::QueryStats::default();
    let hit = wave.pick(transform, pos, 6.0, 6.0, &mut stats).unwrap();
    assert_eq!(hit.0, target_idx);
    assert!(
        stats.scanned_points < 2_000,
        "scanned {}",
        stats.scanned_points
    );

    let ramp: Vec<f32> = (0..200_000).map(|i| (i as f32 * 0.001) % 30.0).collect();
    let ramp_wave = UniformWaveform::from_f32(0.0, 0.001, ramp, 16 * 1024 * 1024).unwrap();
    let target = ramp_wave.point_at(target_idx);
    let pos = transform.position_from_point(target);
    let mut ramp_stats = crate::data::QueryStats::default();
    let ramp_hit = ramp_wave
        .pick(transform, pos, 6.0, 6.0, &mut ramp_stats)
        .unwrap();
    assert_eq!(ramp_hit.0, target_idx);
    assert!(
        ramp_stats.scanned_points < 512,
        "scanned {}",
        ramp_stats.scanned_points
    );
}

#[test]
fn swept_write_keeps_exact_column_peaks_without_rescanning_both_edges() {
    let n = 200_003;
    let samples: Vec<f32> = (0..n).map(|i| 2.0 + (i % 7) as f32).collect();
    let mut wave = UniformWaveform::from_f32(0.0, 1.0, samples, 8 << 20).unwrap();
    let patch: Vec<f32> = (0..32_768)
        .map(|i| if i % 79 == 0 { 12.0 } else { 4.0 })
        .collect();
    wave.write_range_once(1, 65_531, &patch).unwrap();
    let columns = 389;
    let bounds = PlotBounds::new([0.0, 1.0], [(n - 1) as f64, 13.0]).unwrap();
    for scale in [AxisScale::Linear, AxisScale::Log10] {
        let transform = PlotTransform::with_axes(
            bounds,
            egui::Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(columns as f32, 120.0)),
            [AxisScale::Linear, scale],
            [true, true],
        )
        .unwrap();
        let q = wave.query(transform, 1.0, true);
        let mut expected = Vec::new();
        let mut start = 0;
        for col in 1..=columns {
            let end = if col == columns {
                n
            } else {
                ((n - 1) as f64 * col as f64 / columns as f64).ceil() as usize
            };
            let mut lo = start;
            let mut hi = start;
            for i in start + 1..end {
                if wave.samples()[i] < wave.samples()[lo] {
                    lo = i;
                }
                if wave.samples()[i] > wave.samples()[hi] {
                    hi = i;
                }
            }
            let mut picks = [start, lo, hi, end - 1];
            picks.sort_unstable();
            expected.extend(picks.map(|i| i as u32));
            start = end;
        }
        expected.dedup();
        assert_eq!(q.indices, expected);
        assert!(
            q.stats.scanned_points < columns * WAVE_BLOCK_SIZE / 2,
            "{} raw edge samples",
            q.stats.scanned_points
        );
    }
}

#[test]
fn partial_writes_update_all_domain_summaries_across_leaf_edges() {
    let n = 385;
    let mut wave = UniformWaveform::new_zeroed(-11.0, 0.25, n, 1 << 20).unwrap();
    let patch = (0..187)
        .map(|i| [f32::NAN, 7.0, 7.0, -3.0, f32::INFINITY, -0.0, 1.0][i % 7])
        .collect::<Vec<_>>();
    for (batch, start) in [3, 127, 61, 198].into_iter().enumerate() {
        wave.write_range_once(batch as u64, start, &patch).unwrap();
        for start in (0..n).step_by(17) {
            for len in [1, 63, 64, 65, 188, n] {
                let range = start..(start + len).min(n);
                let summary = wave.range_node(range.clone(), None);
                for log in [false, true] {
                    let valid = range
                        .clone()
                        .filter(|&i| {
                            let y = wave.samples()[i];
                            y.is_finite() && (!log || y > 0.0)
                        })
                        .collect::<Vec<_>>();
                    let (count, first, last, low, high) = if log {
                        (
                            summary.valid_log_y,
                            summary.first_pos_valid,
                            summary.last_pos_valid,
                            summary.min_pos_idx,
                            summary.max_pos_idx,
                        )
                    } else {
                        (
                            summary.valid_linear,
                            summary.first_valid,
                            summary.last_valid,
                            summary.min_idx,
                            summary.max_idx,
                        )
                    };
                    assert_eq!(count as usize, valid.len());
                    if let Some(&first_valid) = valid.first() {
                        let (mut min, mut max) = (first_valid, first_valid);
                        for &i in &valid {
                            if wave.samples()[i] < wave.samples()[min] {
                                min = i;
                            }
                            if wave.samples()[i] > wave.samples()[max] {
                                max = i;
                            }
                        }
                        assert_eq!(
                            [first, last, low, high],
                            [first_valid, *valid.last().unwrap(), min, max].map(|i| i as u32)
                        );
                    } else {
                        assert_eq!([first, last, low, high], [u32::MAX; 4]);
                    }
                }
            }
        }
    }
}
