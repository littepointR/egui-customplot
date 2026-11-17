use super::*;

#[test]
fn functions_and_animation_gallery_renders_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--functions"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Functions);

    for frame in 0..2 {
        let mut passes = 0;
        ctx.run_ui(input(frame as f64 * 0.05 + 0.01), |ui| {
            let out = demo.show_charts(ui);
            assert_eq!(out.len(), 4);
            assert!(out[0].diagnostics.output_paths >= 2);
            assert_eq!(out[0].diagnostics.output_annotations, 3);
            assert!(out[1].diagnostics.output_paths >= 2);
            assert!(out[2].diagnostics.contours.output_paths >= 2);
            assert_eq!(out[3].diagnostics.flow.output_heads, 1);
            if passes == 0 {
                ui.ctx().request_discard("functions gallery final layout");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
    }
    ctx.run_ui(input(0.2), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
