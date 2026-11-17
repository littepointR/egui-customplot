use super::*;

#[test]
fn stems_gallery_keeps_raw_samples_and_final_geometry_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--stems"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Stems);
    for horizontal in [false, true] {
        for markers in [false, true] {
            for dashed in [false, true] {
                demo.stems.horizontal = horizontal;
                demo.stems.markers = markers;
                demo.stems.dashed = dashed;
                demo.stems.baseline_visible = !dashed;
                demo.stems.baseline = -0.5;
                let mut passes = 0;
                let versions = demo.stems.data_versions();
                ctx.run_ui(input(0.0), |ui| {
                    let out = demo.show_charts(ui);
                    assert_eq!(out.len(), 2);
                    assert_eq!(out[0].diagnostics.input_points, 8);
                    assert_eq!(out[0].diagnostics.non_finite_points, 1);
                    assert_eq!(out[0].diagnostics.visited_stems, 8);
                    assert_eq!(
                        out[0].diagnostics.output_markers,
                        if markers { 7 } else { 0 }
                    );
                    assert_eq!(
                        out[1].diagnostics.output_markers,
                        if markers { 5 } else { 0 }
                    );
                    assert_eq!(out[1].diagnostics.invalid_stem_baselines, 0);
                    assert!(out[1].right_transform.is_some());
                    assert_eq!(demo.stems.data_versions(), versions);
                    if passes == 0 {
                        ui.ctx().request_discard("stem gallery final geometry");
                    }
                    passes += 1;
                })
                .drop_without_applying_deltas();
                assert_eq!(passes, 2);
            }
        }
    }
    ctx.run_ui(input(1.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
