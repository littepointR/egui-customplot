use super::*;

#[test]
fn distributions_gallery_renders_all_eight_panels_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--distributions"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Distributions);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 8);
        assert!(out[0].diagnostics.input_bars > 0);
        assert!(out[1].diagnostics.input_bars > 0);
        assert!(out[2].diagnostics.stats.output_boxes > 0);
        assert!(out[2].diagnostics.stats.output_bubbles > 0);
        assert!(out[3].diagnostics.stats.output_violins > 0);
        assert!(out[4].diagnostics.stats.output_violins > 0);
        assert!(out[5].diagnostics.stats.output_slices > 0);
        assert!(out[6].diagnostics.stats.output_clouds > 0);
        assert!(out[7].diagnostics.stats.output_clouds > 0);
        if passes == 0 {
            ui.ctx()
                .request_discard("distributions gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
