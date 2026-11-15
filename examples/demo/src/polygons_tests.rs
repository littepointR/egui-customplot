use super::*;

#[test]
fn polygon_gallery_preserves_host_samples_and_renders_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--polygons"));
    assert!(demo.polygons.enabled);
    let ids = demo.polygons.sample_identities();
    for (pos_neg, gouraud) in [(false, true), (true, false)] {
        demo.polygons.pos_neg_split = pos_neg;
        demo.polygons.gouraud_patch = gouraud;
        let mut passes = 0;
        ctx.run_ui(input(0.0), |ui| {
            let out = demo.show_charts(ui);
            assert_eq!(out.len(), 2);
            assert_eq!(demo.polygons.sample_identities(), ids);
            assert!(out[0].diagnostics.patches.output_triangles > 0);
            assert!(out[1].diagnostics.patches.output_triangles > 0);
            if passes == 0 {
                ui.ctx().request_discard("polygon gallery final layout");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
    }
    ctx.run_ui(input(1.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
