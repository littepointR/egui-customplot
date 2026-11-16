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
