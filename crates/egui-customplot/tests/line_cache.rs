mod support;

use std::sync::Arc;

use egui::{Color32, Context, Shape};
use egui_customplot::{
    Axis, AxisScale, Line, LinePattern, LineSampling, Plot, PlotBounds, PlotCache, PlotData,
    PlotState,
};
use support::{capture, run};

#[test]
fn hits_skip_query_and_clip_work_but_keep_geometry_and_fresh_style() {
    let ctx = Context::default();
    let mut data = PlotData::new(
        (1..10_000)
            .map(|i| [i as f64, (i % 100 + 1) as f64])
            .collect::<Vec<_>>(),
    );
    data.build_line_index(1024 * 1024).unwrap();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(1024 * 1024);
    let mut reference = Vec::new();
    for frame in 0..3 {
        let color = if frame == 2 {
            Color32::RED
        } else {
            Color32::BLUE
        };
        let (response, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p").show_cached(
                ui,
                &mut state,
                &[Line::new(&data)
                    .sampling(LineSampling::MinMax)
                    .color(color)
                    .width(2.0 + frame as f32)],
                &mut cache,
            )
        });
        let paths = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                Shape::Path(p) => {
                    assert_eq!(p.stroke.color, egui::epaint::ColorMode::Solid(color));
                    Some(p.points.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if frame == 0 {
            assert_eq!(response.diagnostics.line_cache_misses, 1);
            assert!(response.diagnostics.visited_segments > 0);
            reference = paths;
        } else {
            assert_eq!(response.diagnostics.line_cache_hits, 1);
            assert_eq!(response.diagnostics.visited_segments, 0);
            assert_eq!(response.diagnostics.query_comparisons, 0);
            assert_eq!(paths, reference);
        }
        assert!(cache.retained_bytes() <= cache.budget_bytes());
        output.drop_without_applying_deltas();
    }
    // Pattern is post-cache and must change on an otherwise identical hit.
    let response = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_cached(
            ui,
            &mut state,
            &[Line::new(&data)
                .sampling(LineSampling::MinMax)
                .pattern(LinePattern::Dashed {
                    length: 4.0,
                    gap: 3.0,
                })],
            &mut cache,
        )
    });
    assert_eq!(response.diagnostics.line_cache_hits, 1);
    assert!(response.diagnostics.output_paths > 1);
}

#[test]
fn data_transform_dpi_sampling_and_index_changes_cannot_reuse_stale_geometry() {
    let ctx = Context::default();
    let mut cache = PlotCache::new(1024 * 1024);
    let mut state = PlotState::default();
    let points: Arc<[[f64; 2]]> = vec![[1.0; 2], [10.0; 2], [100.0; 2]].into();
    let mut data = PlotData::new(points.clone());
    let original_version = data.version();
    assert_eq!(data.clone().version(), original_version);
    for step in 0..9 {
        if step == 1 {
            data = PlotData::new(points.clone());
            assert_ne!(data.version(), original_version);
        }
        if step == 7 {
            data.build_line_index(4096).unwrap();
        }
        if step == 8 {
            data.clear_line_index();
        }
        state.set_bounds(
            PlotBounds::new([1.0; 2], [if step < 2 { 100.0 } else { 80.0 }; 2]).unwrap(),
        );
        let response = run(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .height(if step < 3 { 320.0 } else { 300.0 })
                .x_axis(Axis::new().reversed(step >= 4).scale(if step >= 5 {
                    AxisScale::Log10
                } else {
                    AxisScale::Linear
                }))
                .show_cached(
                    ui,
                    &mut state,
                    &[Line::new(&data).sampling(if step >= 6 {
                        LineSampling::MinMax
                    } else {
                        LineSampling::Exact
                    })],
                    &mut cache,
                )
        });
        // Returning to the earlier index-less key may legitimately hit.
        if step < 8 {
            assert_eq!(response.diagnostics.line_cache_misses, 1, "step {step}");
        } else {
            assert_eq!(response.diagnostics.line_cache_hits, 1);
        }
    }
    assert!(cache.entry_count() >= 8);
}

#[test]
fn dpi_changes_miss_even_with_identical_logical_transform() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(4096);
    let mut rect = None;
    for dpi in [1.0, 2.0] {
        ctx.set_pixels_per_point(dpi);
        let response = run(&ctx, vec![], false, |ui| {
            // Context zoom also changes the root's logical screen size. Fix
            // the plot width so only physical density changes in this test.
            ui.set_width(300.0);
            assert_eq!(ui.ctx().pixels_per_point(), dpi);
            Plot::new("p").show_cached(ui, &mut state, &[Line::new(&data)], &mut cache)
        });
        assert_eq!(response.diagnostics.line_cache_misses, 1);
        if let Some(rect) = rect {
            assert_eq!(response.transform.rect(), rect);
        }
        rect = Some(response.transform.rect());
    }
}

#[test]
fn budgets_evict_lru_reject_oversize_and_release_samples_independently() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(1024 * 1024);
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_cached(ui, &mut state, &[Line::new(&data)], &mut cache)
    });
    let one_entry = cache.retained_bytes();
    assert!(one_entry > 0);
    cache.set_budget(one_entry);
    let oversized = PlotData::new(
        (0..1000)
            .map(|i| [i as f64, (i % 10) as f64])
            .collect::<Vec<_>>(),
    );
    let response = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_cached(ui, &mut state, &[Line::new(&oversized)], &mut cache)
    });
    assert_eq!(response.diagnostics.line_cache_rejections, 1);
    assert_eq!(
        cache.entry_count(),
        1,
        "rejection must preserve earlier entries"
    );
    for _ in 0..100 {
        let points: Arc<[[f64; 2]]> = vec![[0.0; 2], [1.0; 2]].into();
        let weak = Arc::downgrade(&points);
        let data = PlotData::new(points);
        let response = run(&ctx, vec![], true, |ui| {
            Plot::new("p").show_cached(ui, &mut state, &[Line::new(&data)], &mut cache)
        });
        assert_eq!(response.diagnostics.line_cache_hits, 1);
        assert_eq!(cache.entry_count(), 1);
        assert!(cache.retained_bytes() <= one_entry);
        drop(data);
        assert!(
            weak.upgrade().is_none(),
            "geometry cache must not retain samples"
        );
    }
    cache.set_budget(0);
    assert_eq!(cache.retained_bytes(), 0);
    let response = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_cached(ui, &mut state, &[Line::new(&data)], &mut cache)
    });
    assert_eq!(response.diagnostics.line_cache_rejections, 1);
    assert_eq!(response.diagnostics.output_paths, 1);
    assert_eq!(cache.entry_count(), 0);
    cache.clear();
    assert_eq!(cache.retained_bytes(), 0);
}

#[test]
fn lru_promotes_hits_and_entry_count_caps_even_empty_results() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut cache = PlotCache::new(1024 * 1024);
    let data: [_; 3] = std::array::from_fn(|_| PlotData::new(vec![[0.0; 2], [1.0; 2]]));
    let mut show = |cache: &mut PlotCache, index: usize| {
        run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_cached(ui, &mut state, &[Line::new(&data[index])], cache)
        })
    };
    show(&mut cache, 0);
    let one_entry = cache.retained_bytes();
    cache.set_budget(one_entry * 2);
    show(&mut cache, 1);
    assert_eq!(show(&mut cache, 0).diagnostics.line_cache_hits, 1);
    show(&mut cache, 2);
    assert_eq!(show(&mut cache, 0).diagnostics.line_cache_hits, 1);
    assert_eq!(show(&mut cache, 1).diagnostics.line_cache_misses, 1);
    cache.set_budget(1024 * 1024);
    cache.clear();
    for _ in 0..100 {
        let data = PlotData::new(Vec::new());
        run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_cached(ui, &mut state, &[Line::new(&data)], &mut cache)
        });
        assert!(cache.retained_bytes() <= cache.budget_bytes());
        assert!(cache.entry_count() <= 64);
    }
    assert_eq!(cache.entry_count(), 64);
}
