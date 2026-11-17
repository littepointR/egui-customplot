use super::*;
use crate::multivariate::MultivariateView;

#[test]
fn multivariate_gallery_renders_overview_and_composite_modes_cleanly() {
    let ctx = egui::Context::default();
    let mut demo = Demo::with_gallery(Some("--multivariate"));
    assert_eq!(demo.gallery, crate::gallery::ActiveGallery::Multivariate);

    let mut passes = 0;
    ctx.run_ui(input(0.0), |ui| {
        let out = demo.show_charts(ui);
        assert_eq!(out.len(), 7);
        assert!(out[0].diagnostics.stats.output_bubbles > 0);
        assert!(out[1].diagnostics.stats.output_bins > 0);
        assert!(out[2].diagnostics.stats.output_bins > 0);
        assert!(out[3].diagnostics.input_heatmap_cells > 0);
        assert!(out[4].diagnostics.stats.output_parallel_segments > 0);
        assert!(out[5].diagnostics.input_bars > 0);
        assert!(out[6].diagnostics.output_paths > 0);
        if passes == 0 {
            ui.ctx().request_discard("multivariate overview retry");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);

    for (mode, expected_plots) in [
        (MultivariateView::Stacked, 3usize),
        (MultivariateView::ScatterHistogram, 4usize),
        (MultivariateView::PlotMatrix, 9usize),
    ] {
        demo.multivariate.view = mode;
        for frame in 0..2 {
            ctx.run_ui(input(0.1 + frame as f64 * 0.016), |ui| {
                let out = demo.show_charts(ui);
                assert_eq!(out.len(), expected_plots);
            })
            .drop_without_applying_deltas();
        }
    }
}
