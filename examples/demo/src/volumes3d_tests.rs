use super::*;

#[test]
fn volumes3d_gallery_renders_across_layout_retries_and_toggles_modes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--volumes3d"));
    assert!(demo.volumes3d.enabled);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 6);
        assert!(out[0].diagnostics.plot3d.output_triangles > 200);
        assert!(out[1].diagnostics.plot3d.output_segments > 20);
        assert!(out[2].diagnostics.plot3d.output_triangles > 200);
        assert!(out[2].diagnostics.plot3d.zbuffer_pixels > 0);
        assert!(out[3].diagnostics.plot3d.output_triangles >= 24);
        assert!(out[4].diagnostics.plot3d.output_markers >= 18);
        assert!(out[5].diagnostics.plot3d.output_triangles > 200);
        if passes == 0 {
            ui.ctx().request_discard("volumes3d gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    demo.volumes3d.boundary_only = false;
    demo.volumes3d.arbitrary_slice = false;
    demo.volumes3d.zbuffer = false;
    demo.volumes3d.animate_particles = true;
    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
