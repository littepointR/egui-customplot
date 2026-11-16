use super::*;

#[test]
fn plot3d_gallery_renders_across_layout_retries_and_toggles_modes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--plot3d"));
    assert!(demo.plot3d.enabled);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 6);
        assert!(out[0].diagnostics.plot3d.output_segments > 50);
        assert!(out[1].diagnostics.plot3d.output_markers >= 48);
        assert!(out[2].diagnostics.plot3d.output_markers >= 50);
        assert!(out[3].diagnostics.plot3d.output_triangles >= 12 * 12);
        assert!(out[4].diagnostics.plot3d.output_planes == 2);
        assert!(out[4].diagnostics.plot3d.zbuffer_pixels > 0);
        assert!(out[5].diagnostics.plot3d.output_triangles > 40);
        if passes == 0 {
            ui.ctx().request_discard("plot3d gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    demo.plot3d.bar3h = true;
    demo.plot3d.zbuffer = false;
    demo.plot3d.perspective = true;
    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
