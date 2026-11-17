use super::*;

#[test]
fn contour_and_image_gallery_survives_layout_retries_in_contourf_and_pcolor_modes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--contours"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Contours);
    let ids = demo.contours.sample_identities();

    for (show_pcolor, interp) in [(false, false), (true, true)] {
        demo.contours.show_pcolor = show_pcolor;
        demo.contours.interp_shading = interp;
        let mut passes = 0;
        ctx.run_ui(input(0.0), |ui| {
            let out = demo.show_charts(ui);
            assert_eq!(out.len(), 2);
            assert_eq!(demo.contours.sample_identities(), ids);
            if show_pcolor {
                assert!(out[0].diagnostics.images.output_cells > 0);
                assert!(out[0].diagnostics.images.output_meshes > 0);
            } else {
                assert!(out[0].diagnostics.contours.output_meshes > 0);
            }
            assert!(out[0].diagnostics.contours.output_paths > 0);
            assert_eq!(out[1].diagnostics.images.input_cells, 32 * 24);
            assert!(out[1].diagnostics.images.output_cells > 700);
            if passes == 0 {
                ui.ctx().request_discard("contour gallery final layout");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
    }
    ctx.run_ui(input(1.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
