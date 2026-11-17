use super::*;

#[test]
fn surfaces3d_gallery_renders_across_layout_retries_and_toggles_modes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--surfaces3d"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Surfaces3D);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 6);
        assert!(out[0].diagnostics.plot3d.output_triangles > 200);
        assert!(out[0].diagnostics.plot3d.output_segments > 10);
        assert!(out[1].diagnostics.plot3d.output_segments > 100);
        assert!(out[2].diagnostics.plot3d.output_segments > 20);
        assert!(out[2].diagnostics.plot3d.output_triangles > 40);
        assert!(out[3].diagnostics.plot3d.output_triangles > 200);
        assert!(out[4].diagnostics.plot3d.output_triangles > 200);
        assert!(out[4].diagnostics.plot3d.zbuffer_pixels > 0);
        assert!(out[5].diagnostics.plot3d.output_triangles > 200);
        if passes == 0 {
            ui.ctx().request_discard("surfaces3d gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    demo.surfaces3d.shading = egui_customplot::SurfaceShading::Faceted;
    demo.surfaces3d.colormap_lighting = true;
    demo.surfaces3d.show_normals = true;
    demo.surfaces3d.hidden_line = false;
    demo.surfaces3d.zbuffer = false;
    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
