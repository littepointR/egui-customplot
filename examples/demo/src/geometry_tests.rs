use super::*;

#[test]
fn geometry_flow_and_spy_gallery_renders_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--geometry"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Geometry);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 5);
        assert!(out[0].diagnostics.geometry.output_paths > 0);
        assert!(out[1].diagnostics.geometry.output_meshes >= 1);
        assert!(out[2].diagnostics.flow.output_paths > 0);
        assert!(out[2].diagnostics.flow.output_heads > 0);
        assert!(out[3].diagnostics.flow.output_heads > 0);
        assert!(out[4].diagnostics.geometry.output_markers > 100);
        if passes == 0 {
            ui.ctx().request_discard("geometry gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
