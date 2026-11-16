use super::*;

#[test]
fn polar_gallery_renders_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--polar"));
    assert!(demo.polar.enabled);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 6);
        assert!(out[0].diagnostics.polar.output_paths >= 3);
        assert!(out[1].diagnostics.polar.output_markers >= 18);
        assert!(out[2].diagnostics.polar.output_sectors > 0);
        assert!(out[3].diagnostics.polar.output_arrows >= 12);
        assert_eq!(out[4].diagnostics.polar.output_regions, 3);
        assert!(out[5].diagnostics.polar.output_paths >= 1);
        if passes == 0 {
            ui.ctx().request_discard("polar gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
