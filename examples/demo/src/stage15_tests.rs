use super::*;

#[test]
fn stage15_gallery_renders_multipass_and_toggles_modes() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--stage15"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Stage15);

    let mut passes = 0;
    let out = ctx.run_ui(input(0.0), |ui| {
        let resps = demo.show_charts(ui);
        assert_eq!(resps.len(), 8);
        if passes == 0 {
            ui.ctx().request_discard("stage15 multipass");
        }
        passes += 1;
    });
    assert_eq!(passes, 2);
    assert!(!out.shapes.is_empty());
    out.drop_without_applying_deltas();

    demo.stage15.smith_admittance = true;
    demo.stage15.nyquist_mirror = false;
    demo.stage15.nichols_grid = false;
    demo.stage15.use_cwt = true;
    let mut snapshot = None;
    ctx.run_ui(input(0.1), |ui| {
        snapshot = Some(
            egui_customplot::PlotSnapshot::capture(ui, Default::default(), |ui| {
                demo.show_charts(ui)
                    .iter()
                    .fold(egui::Rect::NOTHING, |acc, r| acc.union(r.outer_rect))
            })
            .unwrap(),
        );
    })
    .drop_without_applying_deltas();

    let opts = egui_customplot::ExportOptions {
        pixels_per_point: 0.5,
        ..Default::default()
    };
    let snap = snapshot.unwrap();
    let svg = String::from_utf8(snap.to_svg(opts).unwrap()).unwrap();
    assert!(svg.contains("<svg") && !svg.contains("<text"));
    assert!(snap.to_png(opts).unwrap().starts_with(b"\x89PNG"));
}
