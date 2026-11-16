mod support;

use std::f64::consts::TAU;

use egui::{Event, Pos2};
use egui_customplot::{
    AUTO_INDEX_MIN_POINTS, Axis, AxisScale, IndexedStream, Line, LineSampling, MarkerSampling,
    Plot, PlotBounds, PlotData, PlotPoint, PlotState, RenderBackend, Scatter, Series, StreamError,
    StreamLine, UniformWaveform, WaveformLine,
};

fn monotonic_wave(n: usize) -> Vec<PlotPoint> {
    (0..n)
        .map(|i| {
            let x = i as f64 / (n.saturating_sub(1).max(1)) as f64 * 100.0;
            let y = (x * 0.7).sin() + if i % 997 == 0 { 1.4 } else { 0.0 };
            [x, y]
        })
        .collect()
}

#[test]
fn plot_data_auto_indexes_large_monotonic_and_respects_opt_out() {
    let mono = PlotData::new(monotonic_wave(AUTO_INDEX_MIN_POINTS + 1_808));
    assert!(
        mono.line_index_bytes() > 0,
        "PlotData::new should auto-index monotonic X >= AUTO_INDEX_MIN_POINTS"
    );

    let small = PlotData::new(monotonic_wave(1_000));
    assert_eq!(small.line_index_bytes(), 0);

    let unindexed = PlotData::new_unindexed(monotonic_wave(10_000));
    assert_eq!(unindexed.line_index_bytes(), 0);

    // Non-monotonic parametric curve stays unindexed silently without error.
    let spiral: Vec<PlotPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64 * 0.01;
            [t * t.cos(), t * t.sin()]
        })
        .collect();
    let non_mono = PlotData::new(spiral);
    assert_eq!(non_mono.line_index_bytes(), 0);
}

#[test]
fn auto_sampling_and_state_auto_cache_work_transparently() {
    let ctx = egui::Context::default();
    let data = PlotData::new(monotonic_wave(100_000));
    let mut state = PlotState::default();

    // Frame 1 at full view: default LineSampling::Auto uses LOD + populates internal auto_cache.
    let first = support::run(&ctx, vec![], false, |ui| {
        Plot::new("auto_lod")
            .height(420.0)
            .show(ui, &mut state, &[Line::new(&data)])
    });
    assert_eq!(first.diagnostics.auto_lod_series, 1);
    assert!(first.diagnostics.output_points > 0);
    assert!(first.diagnostics.output_points <= 4_000);
    assert!(first.diagnostics.query_selected_points > 0);
    assert_eq!(first.diagnostics.line_cache_hits, 0);

    // Frame 2 with unchanged view hits PlotState's internal auto_cache without touching host PlotCache stats.
    let second = support::run(&ctx, vec![], false, |ui| {
        Plot::new("auto_lod")
            .height(420.0)
            .show(ui, &mut state, &[Line::new(&data)])
    });
    assert_eq!(second.diagnostics.query_selected_points, 0);
    assert_eq!(second.diagnostics.line_cache_hits, 0);
    assert_eq!(
        second.diagnostics.output_points,
        first.diagnostics.output_points
    );

    // Zooming closely drops below the exact threshold and disables LOD reduction automatically.
    state.set_bounds(PlotBounds::new([0.0, -3.0], [0.15, 3.0]).unwrap());
    let zoomed = support::run(&ctx, vec![], false, |ui| {
        Plot::new("auto_lod")
            .height(420.0)
            .show(ui, &mut state, &[Line::new(&data)])
    });
    assert_eq!(zoomed.diagnostics.auto_lod_series, 0);
    assert!(zoomed.diagnostics.query_selected_points > 0);
    assert!(zoomed.diagnostics.query_selected_points < 250);

    state.reset();
    let reset = support::run(&ctx, vec![], false, |ui| {
        Plot::new("auto_lod")
            .height(420.0)
            .show(ui, &mut state, &[Line::new(&data)])
    });
    assert_eq!(reset.diagnostics.auto_lod_series, 1);
}

#[test]
fn indexed_picking_matches_raw_scan_with_y_branch_and_bound_pruning() {
    let ctx = egui::Context::default();
    let n = 200_000;
    let target_idx = 123_456;
    let pts: Vec<PlotPoint> = (0..n)
        .map(|i| {
            let x = i as f64 / (n - 1) as f64 * 100.0;
            let base = (x * 0.6).sin() * 3.0;
            let y = if i == target_idx { base + 2.8 } else { base };
            [x, y]
        })
        .collect();
    let target_pt = pts[target_idx];

    let indexed = PlotData::new(pts.clone());
    let unindexed = PlotData::new_unindexed(pts.clone());
    let mut stream = IndexedStream::new(n, 64 * 1024 * 1024).unwrap();
    stream.append_once(1, &pts).unwrap();

    let ys: Vec<f32> = pts.iter().map(|p| p[1] as f32).collect();
    let wave =
        UniformWaveform::from_f32(0.0, 100.0 / (n - 1) as f64, ys, 32 * 1024 * 1024).unwrap();

    let mut s_ref = PlotState::default();
    let setup = support::run(&ctx, vec![], false, |ui| {
        Plot::new("pick_parity").height(420.0).show(
            ui,
            &mut s_ref,
            &[Line::new(&unindexed).sampling(LineSampling::Exact)],
        )
    });
    let cursor: Pos2 = setup.transform.position_from_point(target_pt);
    assert!(setup.transform.rect().contains(cursor));

    let raw_hover = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
        Plot::new("pick_parity").height(420.0).show(
            ui,
            &mut s_ref,
            &[Line::new(&unindexed).sampling(LineSampling::Exact)],
        )
    });
    let expected = raw_hover.hovered_point.expect("raw hover hits target");

    let mut s_idx = PlotState::default();
    let idx_hover = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
        Plot::new("pick_parity")
            .height(420.0)
            .show(ui, &mut s_idx, &[Line::new(&indexed)])
    });
    let got_idx = idx_hover.hovered_point.expect("indexed hover hits");
    assert_eq!(got_idx.point_index, expected.point_index);
    assert!(idx_hover.diagnostics.visited_pick_points < 2_000);

    let mut s_stream = PlotState::default();
    let stream_hover = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
        Plot::new("pick_parity").height(420.0).show_stream(
            ui,
            &mut s_stream,
            &[StreamLine::new(&stream)],
        )
    });
    let got_stream = stream_hover.hovered_point.expect("stream hover hits");
    assert_eq!(got_stream.point_index, expected.point_index);
    assert!(stream_hover.diagnostics.visited_pick_points < 2_000);

    let mut s_wave = PlotState::default();
    let wave_hover = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
        Plot::new("pick_parity").height(420.0).show_waveform(
            ui,
            &mut s_wave,
            &[WaveformLine::new(&wave)],
        )
    });
    let got_wave = wave_hover.hovered_point.expect("waveform hover hits");
    assert!((got_wave.point[0] - expected.point[0]).abs() < 1e-9);
    assert!((got_wave.point[1] - expected.point[1]).abs() < 1e-4);
    assert!(wave_hover.diagnostics.visited_pick_points < 2_000);
}

#[test]
fn indexed_stream_ring_wrap_multipass_and_axes_work() {
    let ctx = egui::Context::default();
    let cap = 100_000;
    let mut stream = IndexedStream::new(cap, 32 * 1024 * 1024).unwrap();
    let make_batch = |start: usize, count: usize| -> Vec<PlotPoint> {
        (start..start + count)
            .map(|i| {
                let x = 1.0 + i as f64 * 0.05;
                let y = if i % 40_000 == 123 {
                    f64::NAN
                } else {
                    1.0 + (x * 0.1).sin().abs() * 50.0
                };
                [x, y]
            })
            .collect()
    };

    // Append across wrap boundary cleanly and enforce batch idempotency.
    let rep1 = stream.append_once(1, &make_batch(0, 70_000)).unwrap();
    assert!(rep1.applied);
    assert_eq!(rep1.evicted_points, 0);
    let rep_dup = stream.append_once(1, &make_batch(0, 70_000)).unwrap();
    assert!(!rep_dup.applied);
    assert!(matches!(
        stream.append_once(0, &[]),
        Err(StreamError::StaleBatch { .. })
    ));

    let rep2 = stream.append_once(2, &make_batch(70_000, 60_000)).unwrap();
    assert!(rep2.applied);
    assert_eq!(rep2.evicted_points, 30_000);
    assert_eq!(stream.len(), cap);
    let (a, b) = stream.as_slices();
    assert!(!b.is_empty() && a.len() + b.len() == cap);

    let mut state = PlotState::default();
    let resp = support::run(&ctx, vec![], false, |ui| {
        Plot::new("stream_wrap")
            .height(420.0)
            .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show_stream(ui, &mut state, &[StreamLine::new(&stream).name("ring")])
    });
    assert_eq!(resp.diagnostics.indexed_line_series, 1);
    assert_eq!(resp.diagnostics.input_points, cap);
    assert!(resp.diagnostics.non_finite_points >= 1);
    assert!(resp.diagnostics.output_paths >= 2);

    // Multi-pass layout keeps append_once idempotent and preserves output geometry.
    let rep_multi = support::run(&ctx, vec![], true, |ui| {
        let rep = stream.append_once(3, &make_batch(130_000, 128)).unwrap();
        let _ = rep;
        Plot::new("stream_wrap")
            .height(420.0)
            .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show_stream(ui, &mut state, &[StreamLine::new(&stream).name("ring")])
    });
    assert!(rep_multi.diagnostics.output_paths >= 2);
}

#[test]
fn uniform_waveform_partial_updates_and_export_work() {
    let ctx = egui::Context::default();
    let n = 200_000;
    let initial: Vec<f32> = (0..n)
        .map(|i| {
            if i == n / 2 {
                f32::NAN
            } else {
                2.0 + ((i as f32) * 0.002).sin()
            }
        })
        .collect();
    let mut wave = UniformWaveform::from_f32(1.0, 0.01, initial, 16 * 1024 * 1024).unwrap();
    let patch: Vec<f32> = (0..4_096)
        .map(|i| 3.0 + (i as f32 * TAU as f32 / 1024.0).cos())
        .collect();

    // Single pass checks query diagnostics; multi-pass verifies update_once idempotency.
    let mut state = PlotState::default();
    let resp = support::run(&ctx, vec![], false, |ui| {
        wave.update_once(10, Some(2.0), 10_000, &patch).unwrap();
        Plot::new("wave_test")
            .height(400.0)
            .x_axis(Axis::new().scale(AxisScale::Log10))
            .show_waveform(ui, &mut state, &[WaveformLine::new(&wave).name("wf")])
    });
    assert_eq!(resp.diagnostics.input_points, n);
    assert_eq!(resp.diagnostics.non_finite_points, 1);
    assert!(resp.diagnostics.auto_lod_series == 1);
    assert!(resp.diagnostics.output_paths >= 2);
    assert_eq!(wave.x0(), 2.0);
    assert!((wave.point_at(10_000)[1] - 4.0).abs() < 1e-4);

    let multi = support::run(&ctx, vec![], true, |ui| {
        wave.update_once(11, Some(2.0), 12_000, &patch).unwrap();
        Plot::new("wave_test")
            .height(400.0)
            .x_axis(Axis::new().scale(AxisScale::Log10))
            .show_waveform(ui, &mut state, &[WaveformLine::new(&wave).name("wf")])
    });
    assert!(multi.diagnostics.output_paths >= 2);

    #[cfg(feature = "export")]
    {
        let mut snapshot = None;
        ctx.run_ui(support::input(vec![]), |ui| {
            snapshot = Some(
                egui_customplot::PlotSnapshot::capture(ui, Default::default(), |ui| {
                    Plot::new("wave_export")
                        .height(320.0)
                        .show_waveform(ui, &mut state, &[WaveformLine::new(&wave)])
                        .outer_rect
                })
                .unwrap(),
            );
        })
        .drop_without_applying_deltas();
        drop(wave);
        let snap = snapshot.unwrap();
        let svg = String::from_utf8(snap.to_svg(Default::default()).unwrap()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(
            snap.to_png(Default::default())
                .unwrap()
                .starts_with(b"\x89PNG")
        );
    }
}

#[test]
fn marker_lod_and_wgpu_headless_fallback_behave_cleanly() {
    let ctx = egui::Context::default();
    let pts = monotonic_wave(100_000);
    let data = PlotData::new(pts);
    let mut state = PlotState::default();

    // Default Scatter uses MarkerSampling::Auto -> pixel-grid reduction at 100k points.
    let full = support::run(&ctx, vec![], false, |ui| {
        Plot::new("marker_lod").height(400.0).show_series(
            ui,
            &mut state,
            &[Series::from(Scatter::new(&data))],
        )
    });
    assert_eq!(full.diagnostics.marker_lod_series, 1);
    assert!(full.diagnostics.output_markers < 20_000);

    // Explicit PixelGrid also reduces.
    let explicit_grid = support::run(&ctx, vec![], false, |ui| {
        Plot::new("marker_lod").height(400.0).show_series(
            ui,
            &mut state,
            &[Series::from(
                Scatter::new(&data).sampling(MarkerSampling::PixelGrid),
            )],
        )
    });
    assert!(explicit_grid.diagnostics.output_markers < 20_000);

    // Zooming to ~50 visible points renders exact raw markers without LOD flag.
    state.set_bounds(PlotBounds::new([0.0, -3.0], [0.049, 3.0]).unwrap());
    let zoomed = support::run(&ctx, vec![], false, |ui| {
        Plot::new("marker_lod").height(400.0).show_series(
            ui,
            &mut state,
            &[Series::from(Scatter::new(&data))],
        )
    });
    assert_eq!(zoomed.diagnostics.marker_lod_series, 0);
    assert_eq!(zoomed.diagnostics.output_markers, 50);

    // Explicit RenderBackend::Wgpu falls back cleanly without GPU context and reports fallback.
    state.reset();
    let wave =
        UniformWaveform::from_f32(0.0, 0.01, vec![0.5_f32; 50_000], 8 * 1024 * 1024).unwrap();
    let wf_wgpu = support::run(&ctx, vec![], false, |ui| {
        Plot::new("wgpu_fb_wave").height(320.0).show_waveform(
            ui,
            &mut state,
            &[WaveformLine::new(&wave).backend(RenderBackend::Wgpu)],
        )
    });
    assert_eq!(wf_wgpu.diagnostics.wgpu_series, 0);
    assert_eq!(wf_wgpu.diagnostics.wgpu_fallback_series, 1);
    assert!(wf_wgpu.diagnostics.output_points > 0);

    let sc_wgpu = support::run(&ctx, vec![], false, |ui| {
        Plot::new("wgpu_fb_scatter").height(320.0).show_series(
            ui,
            &mut state,
            &[Series::from(
                Scatter::new(&data).backend(RenderBackend::Wgpu),
            )],
        )
    });
    assert_eq!(sc_wgpu.diagnostics.wgpu_series, 0);
    assert_eq!(sc_wgpu.diagnostics.wgpu_fallback_series, 1);
    assert!(sc_wgpu.diagnostics.output_markers > 0);

    // Auto backend never records wgpu_fallback_series when GPU isn't registered.
    let wf_auto = support::run(&ctx, vec![], false, |ui| {
        Plot::new("auto_fb_wave").height(320.0).show_waveform(
            ui,
            &mut state,
            &[WaveformLine::new(&wave)],
        )
    });
    assert_eq!(wf_auto.diagnostics.wgpu_fallback_series, 0);
}

#[test]
fn deep_zoom_out_picking_queries_and_markers_stay_sublinear_and_exact() {
    let ctx = egui::Context::default();
    let n = 250_000usize;
    let dx = 100.0 / (n - 1) as f64;
    let ys_f32: Vec<f32> = (0..n)
        .map(|i| {
            if (120_000..120_500).contains(&i) {
                f32::NAN
            } else {
                let x = i as f64 * dx;
                ((x * 0.7).sin() * 2.5 + (x * 11.3).cos() * 1.2) as f32
            }
        })
        .collect();
    let pts: Vec<PlotPoint> = ys_f32
        .iter()
        .enumerate()
        .map(|(i, &y)| [i as f64 * dx, f64::from(y)])
        .collect();

    let indexed = PlotData::new(pts.clone());
    let unindexed = PlotData::new_unindexed(pts.clone());
    let mut stream = IndexedStream::new(n, 64 * 1024 * 1024).unwrap();
    stream.append_once(1, &pts).unwrap();
    let wave = UniformWaveform::from_f32(0.0, dx, ys_f32, 32 * 1024 * 1024).unwrap();

    let bounds_list = [
        (
            "200x",
            PlotBounds::new([-10_000.0, -500.0], [10_000.0, 500.0]).unwrap(),
        ),
        (
            "1e7x",
            PlotBounds::new([-5.0e8, -5.0e8], [5.0e8, 5.0e8]).unwrap(),
        ),
        (
            "1e100x",
            PlotBounds::new([-1.0e100, -1.0e100], [1.0e100, 1.0e100]).unwrap(),
        ),
    ];

    for (tag, bounds) in bounds_list {
        let mut s_setup = PlotState::default();
        s_setup.set_bounds(bounds);
        let setup = support::run(&ctx, vec![], false, |ui| {
            Plot::new("z_pick")
                .height(420.0)
                .show(ui, &mut s_setup, &[Line::new(&indexed)])
        });
        let cluster = setup.transform.position_from_point([50.0, 0.0]);
        assert!(setup.transform.rect().contains(cluster), "{tag}");

        let probes: Vec<(&str, Pos2)> = match tag {
            "200x" => vec![
                ("on_cluster", cluster),
                ("5px_above", egui::pos2(cluster.x, cluster.y - 5.0)),
            ],
            "1e7x" => vec![
                ("on_cluster", cluster),
                ("40px_away", egui::pos2(cluster.x + 40.0, cluster.y)),
            ],
            _ => vec![("on_cluster", cluster)],
        };

        for (probe_name, cursor) in probes {
            let mut s_ref = PlotState::default();
            s_ref.set_bounds(bounds);
            let r_ref = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
                Plot::new("z_pick").height(420.0).show(
                    ui,
                    &mut s_ref,
                    &[Line::new(&unindexed).sampling(LineSampling::Exact)],
                )
            });
            let expected = r_ref
                .hovered_point
                .as_ref()
                .map(|p| (p.point_index, p.screen_distance));
            if probe_name == "40px_away" {
                assert!(expected.is_none(), "{tag} {probe_name}");
                assert_eq!(r_ref.diagnostics.visited_pick_points, 0, "{tag} unindexed");
            } else {
                assert!(expected.is_some(), "{tag} {probe_name}");
            }

            let mut s_idx = PlotState::default();
            s_idx.set_bounds(bounds);
            let r_idx = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
                Plot::new("z_pick")
                    .height(420.0)
                    .show(ui, &mut s_idx, &[Line::new(&indexed)])
            });
            assert_eq!(
                r_idx
                    .hovered_point
                    .as_ref()
                    .map(|p| (p.point_index, p.screen_distance)),
                expected,
                "{tag} {probe_name} PlotData"
            );

            let mut s_stream = PlotState::default();
            s_stream.set_bounds(bounds);
            let r_stream = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
                Plot::new("z_pick").height(420.0).show_stream(
                    ui,
                    &mut s_stream,
                    &[StreamLine::new(&stream)],
                )
            });
            assert_eq!(
                r_stream
                    .hovered_point
                    .as_ref()
                    .map(|p| (p.point_index, p.screen_distance)),
                expected,
                "{tag} {probe_name} IndexedStream"
            );

            let mut s_wave = PlotState::default();
            s_wave.set_bounds(bounds);
            let r_wave = support::run(&ctx, vec![Event::PointerMoved(cursor)], false, |ui| {
                Plot::new("z_pick").height(420.0).show_waveform(
                    ui,
                    &mut s_wave,
                    &[WaveformLine::new(&wave)],
                )
            });
            assert_eq!(
                r_wave
                    .hovered_point
                    .as_ref()
                    .map(|p| (p.point_index, p.screen_distance)),
                expected,
                "{tag} {probe_name} UniformWaveform"
            );

            for (label, diag) in [
                ("PlotData", &r_idx.diagnostics),
                ("IndexedStream", &r_stream.diagnostics),
                ("UniformWaveform", &r_wave.diagnostics),
            ] {
                if probe_name == "40px_away" {
                    assert_eq!(diag.visited_pick_points, 0, "{tag} {probe_name} {label}");
                } else if tag == "200x" {
                    assert!(
                        diag.visited_pick_points < 2_500,
                        "{tag} {probe_name} {label} visited={}",
                        diag.visited_pick_points
                    );
                } else {
                    assert!(
                        diag.visited_pick_points <= 16,
                        "{tag} {probe_name} {label} visited={}",
                        diag.visited_pick_points
                    );
                }
                assert!(
                    diag.query_scanned_points < 2_500,
                    "{tag} {probe_name} {label} query_scanned={}",
                    diag.query_scanned_points
                );
            }
        }

        if tag != "1e100x" {
            let mut s_sc = PlotState::default();
            s_sc.set_bounds(bounds);
            let r_sc = support::run(&ctx, vec![], false, |ui| {
                Plot::new("z_scatter")
                    .height(420.0)
                    .show_hover(false)
                    .show_series(ui, &mut s_sc, &[Series::from(Scatter::new(&indexed))])
            });
            assert!(r_sc.diagnostics.output_markers > 0, "{tag}");
            if tag == "200x" {
                assert!(
                    r_sc.diagnostics.visited_marker_points < 2_500,
                    "{tag} scatter visited={}",
                    r_sc.diagnostics.visited_marker_points
                );
            } else {
                assert!(
                    r_sc.diagnostics.visited_marker_points <= 4,
                    "{tag} scatter visited={}",
                    r_sc.diagnostics.visited_marker_points
                );
            }
        }
    }

    // Oscillating Log10 Y crossing y <= 0 thousands of times inside 1 pixel column.
    let osc_n = 200_000usize;
    let osc_ys: Vec<f32> = (0..osc_n)
        .map(|i| ((i + 7) % 16) as f32 * 0.7 - 5.0)
        .collect();
    let osc_pts: Vec<PlotPoint> = osc_ys
        .iter()
        .enumerate()
        .map(|(i, &y)| [1.0 + i as f64 * 0.001, f64::from(y)])
        .collect();
    let osc_data = PlotData::new(osc_pts.clone());
    let mut osc_stream = IndexedStream::new(osc_n, 64 * 1024 * 1024).unwrap();
    osc_stream.append_once(1, &osc_pts).unwrap();
    let osc_wave = UniformWaveform::from_f32(1.0, 0.001, osc_ys, 32 * 1024 * 1024).unwrap();
    let log_bounds = PlotBounds::new([-1.0e5, 1.0e-6], [1.0e5, 1.0e12]).unwrap();
    let log_pos = {
        let mut s = PlotState::default();
        s.set_bounds(log_bounds);
        support::run(&ctx, vec![], false, |ui| {
            Plot::new("osc_pick")
                .height(420.0)
                .y_axis(Axis::new().scale(AxisScale::Log10))
                .show(ui, &mut s, &[Line::new(&osc_data)])
        })
        .transform
        .position_from_point([100.0, 5.5])
    };
    let mut sl = PlotState::default();
    sl.set_bounds(log_bounds);
    let r_osc_line = support::run(&ctx, vec![Event::PointerMoved(log_pos)], false, |ui| {
        Plot::new("osc_pick")
            .height(420.0)
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show(ui, &mut sl, &[Line::new(&osc_data)])
    });
    let mut ss = PlotState::default();
    ss.set_bounds(log_bounds);
    let r_osc_stream = support::run(&ctx, vec![Event::PointerMoved(log_pos)], false, |ui| {
        Plot::new("osc_pick")
            .height(420.0)
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show_stream(ui, &mut ss, &[StreamLine::new(&osc_stream)])
    });
    let mut sw = PlotState::default();
    sw.set_bounds(log_bounds);
    let r_osc_wave = support::run(&ctx, vec![Event::PointerMoved(log_pos)], false, |ui| {
        Plot::new("osc_pick")
            .height(420.0)
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .show_waveform(ui, &mut sw, &[WaveformLine::new(&osc_wave)])
    });
    assert!(
        r_osc_line.diagnostics.query_scanned_points < 2_500,
        "osc line query_scanned={}",
        r_osc_line.diagnostics.query_scanned_points
    );
    assert!(
        r_osc_stream.diagnostics.query_scanned_points < 2_500,
        "osc stream query_scanned={}",
        r_osc_stream.diagnostics.query_scanned_points
    );
    assert!(
        r_osc_wave.diagnostics.query_scanned_points < 2_500,
        "osc wave query_scanned={}",
        r_osc_wave.diagnostics.query_scanned_points
    );
    assert!(
        r_osc_stream.diagnostics.visited_pick_points < 2_500,
        "osc stream visited_pick={}",
        r_osc_stream.diagnostics.visited_pick_points
    );
    assert!(
        r_osc_wave.diagnostics.visited_pick_points < 2_500,
        "osc wave visited_pick={}",
        r_osc_wave.diagnostics.visited_pick_points
    );
}
