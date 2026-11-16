use super::*;

#[test]
fn geo_gallery_renders_across_layout_retries_and_toggles_modes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--geo"));
    assert!(demo.geo.enabled);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].diagnostics.geo.dateline_splits, 0);
        assert!(out[0].diagnostics.geo.output_paths >= 1);
        assert_eq!(out[1].diagnostics.geo.output_markers, 8);
        assert!(out[2].diagnostics.geo.output_density_cells > 0);
        assert_eq!(out[3].diagnostics.geo.output_bubbles, 8);
        if passes == 0 {
            ui.ctx().request_discard("geo gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    demo.geo.equirectangular = true;
    demo.geo.dark_basemap = true;
    demo.geo.use_host_tiles = true;
    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
