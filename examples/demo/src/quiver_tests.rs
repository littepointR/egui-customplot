use super::*;

#[test]
fn vector_gallery_reuses_original_payloads_across_style_and_layout_passes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--quiver"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Quiver);
    let identities = demo.quiver.sample_identities();
    for heads in [false, true] {
        for dashed in [false, true] {
            for reversed in [false, true] {
                demo.quiver.heads = heads;
                demo.quiver.dashed = dashed;
                demo.quiver.reversed = reversed;
                let mut passes = 0;
                ctx.run_ui(input(0.0), |ui| {
                    let out = demo.show_charts(ui);
                    assert_eq!(out.len(), 2);
                    assert_eq!(out[0].diagnostics.input_quiver_arrows, 78);
                    assert_eq!(out[0].diagnostics.invalid_quiver_arrows, 1);
                    assert_eq!(out[0].diagnostics.visited_quiver_arrows, 78);
                    assert_eq!(
                        out[0].diagnostics.output_quiver_heads,
                        if heads { 76 } else { 0 }
                    );
                    assert_eq!(
                        out[1].diagnostics.output_quiver_heads,
                        if heads { 4 } else { 0 }
                    );
                    assert_eq!(out[1].diagnostics.domain_invalid_quiver_arrows, 0);
                    assert!(out[1].right_transform.is_some());
                    assert_eq!(demo.quiver.sample_identities(), identities);
                    if passes == 0 {
                        ui.ctx().request_discard("vector final layout");
                    }
                    passes += 1;
                })
                .drop_without_applying_deltas();
                assert_eq!(passes, 2);
            }
        }
    }
    ctx.run_ui(input(1.0), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
