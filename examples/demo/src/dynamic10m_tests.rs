use super::*;

#[test]
fn dynamic10m_gallery_is_lazy_by_default_and_updates_once_per_frame() {
    let mut demo = Demo::default();
    assert!(!demo.dynamic10m.enabled);
    assert_eq!(demo.dynamic10m.batch_size, 32_768);

    demo.dynamic10m.enabled = true;
    demo.dynamic10m.set_capacity_for_test(4_096);
    let ctx = egui::Context::default();
    ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(4).unwrap());

    for frame in 0..3 {
        let mut passes = 0;
        let mut resp_diag = None;
        ctx.run_ui(input(frame as f64 * 0.01), |ui| {
            let out = demo.show_charts(ui);
            assert_eq!(out.len(), 1);
            resp_diag = Some(out[0].diagnostics.clone());
            if passes == 0 {
                ui.ctx().request_discard("dynamic10m multipass");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
        let d = resp_diag.unwrap();
        assert_eq!(d.input_points, 4_096);
        assert!(d.output_points > 0);
    }

    demo.dynamic10m.mode = crate::dynamic10m::DynamicMode::StreamingRing;
    let out = ctx.run_ui(input(0.05), |ui| {
        let res = demo.show_charts(ui);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].diagnostics.input_points, 4_096);
    });
    out.drop_without_applying_deltas();
}
