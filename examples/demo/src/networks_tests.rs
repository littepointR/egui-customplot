use super::*;

#[test]
fn networks_graphs_and_trees_gallery_renders_across_layout_retries() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--networks"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Networks);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].diagnostics.network.output_nodes, 8);
        assert_eq!(out[0].diagnostics.network.output_edges, 13);
        assert_eq!(out[1].diagnostics.network.output_arrows, 11);
        assert_eq!(out[2].diagnostics.network.output_nodes, 16);
        assert!(out[3].diagnostics.network.output_nodes >= 16);
        if passes == 0 {
            ui.ctx().request_discard("networks gallery final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    ctx.run_ui(input(0.1), |ui| demo.show(ui))
        .drop_without_applying_deltas();
}
