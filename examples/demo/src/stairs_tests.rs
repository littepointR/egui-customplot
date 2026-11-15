use super::*;

#[test]
fn stairs_preset_and_live_updates_keep_interpolation_and_samples_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--stairs"));
    assert_eq!(demo.data.signal.len(), 8);
    assert_eq!(demo.data.signal.non_finite_points(), 1);
    for interpolation in [
        egui_customplot::LineInterpolation::StepAfter,
        egui_customplot::LineInterpolation::StepBefore,
    ] {
        demo.options.interpolation = interpolation;
        let version = demo.data.signal.version();
        let mut passes = 0;
        let output = ctx.run_ui(input(0.0), |ui| {
            let out = demo.show_charts(ui);
            assert_eq!(out[0].diagnostics.input_points, 12);
            assert_eq!(demo.data.signal.version(), version);
            if passes == 0 {
                ui.ctx().request_discard("stairs gallery final layout");
            }
            passes += 1;
        });
        assert_eq!(passes, 2);
        let paths: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Path(p) => Some(p.points.clone()),
                _ => None,
            })
            .collect();
        output.drop_without_applying_deltas();
        assert!(!paths.is_empty());
        for p in paths {
            assert!(p.windows(2).all(|p| p[0].x == p[1].x || p[0].y == p[1].y));
        }
    }
    demo.options.performance.streaming = true;
    for frame in 1..=3 {
        ctx.run_ui(input(frame as f64 * 0.1), |ui| {
            demo.show(ui);
            ui.ctx().request_discard("stairs stream final layout");
        })
        .drop_without_applying_deltas();
        assert_eq!(demo.performance.live.data.len(), frame * 256);
        assert!(demo.performance.live.error.is_none());
    }
}
