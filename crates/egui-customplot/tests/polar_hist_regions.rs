use std::f64::consts::TAU;

use egui::{Color32, Context, Event};
use egui_customplot::{
    NegativeRadiusMode, PolarCurveData, PolarHistNormalization, PolarHistogram, PolarHistogramData,
    PolarLayer, PolarLine, PolarPlot, PolarRegionAnnotation, PolarScatter, PolarState, RoseData,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

#[test]
fn polarhistogram_normalizations_and_rose_petals_are_consistent() {
    let angles = [
        0.1,
        0.2,
        0.3 + TAU,
        -0.1, // wraps to 2*pi - 0.1 (last bin)
        1.7,
        f64::NAN,
    ];
    let hist = PolarHistogramData::new(&angles, 6).unwrap();
    assert_eq!(hist.non_finite(), 1);
    assert_eq!(hist.counts().iter().sum::<usize>(), 5);
    assert_eq!(hist.counts()[0], 3);
    assert_eq!(hist.counts()[5], 1);

    let probs = hist.values(PolarHistNormalization::Probability);
    let sum_p: f64 = probs.iter().sum();
    assert!((sum_p - 1.0).abs() < 1e-12);

    let pdf = hist.values(PolarHistNormalization::Pdf);
    let bin_w = TAU / 6.0;
    let integral: f64 = pdf.iter().map(|v| v * bin_w).sum();
    assert!((integral - 1.0).abs() < 1e-12);

    let cdf = hist.values(PolarHistNormalization::Cdf);
    assert!((cdf[5] - 1.0).abs() < 1e-12);

    let density = hist.values(PolarHistNormalization::CountDensity);
    assert!((density.iter().map(|d| d * bin_w).sum::<f64>() - 5.0).abs() < 1e-12);

    let cum = hist.values(PolarHistNormalization::CumCount);
    assert!((cum[5] - 5.0).abs() < 1e-12);

    let rose = RoseData::new(&angles, 8).unwrap();
    assert_eq!(rose.counts().iter().sum::<usize>(), 5);
    let curve = rose.to_polar_curve();
    assert!(curve.len() >= 8 * 4);
}

#[test]
fn polar_regions_wrap_around_and_hover_picking_hits_points_and_bins() {
    let angles = [0.15, 0.25, 0.35, 2.2, 4.1];
    let hist = PolarHistogramData::new(&angles, 6).unwrap();
    let pts = PolarCurveData::from_theta_r(&[2.0, 4.5], &[2.2, 1.8]).unwrap();

    // Cross-zero wrap sector: 330° (5.7596 rad) -> 30° (0.5236 rad).
    let cross_zero =
        PolarRegionAnnotation::theta_region(330.0_f64.to_radians(), 30.0_f64.to_radians())
            .fill(Color32::from_rgba_unmultiplied(80, 140, 220, 60))
            .label("Wrap Sector");
    let ring = PolarRegionAnnotation::radius_region(1.0, 2.0);
    let patch = PolarRegionAnnotation::polar_region([1.5, 2.5], [0.5, 2.5]);

    let ctx = Context::default();
    let mut state = PolarState::default();
    let mut initial = None;
    ctx.run_ui(support::input(vec![]), |ui| {
        initial = Some(
            PolarPlot::new("hist_reg")
                .regions([cross_zero.clone(), ring.clone(), patch.clone()])
                .show(
                    ui,
                    &mut state,
                    &[
                        PolarLayer::from(PolarHistogram::new(&hist).name("bins")),
                        PolarLayer::from(PolarScatter::new(&pts).name("pts")),
                        PolarLayer::from(PolarLine::new(&pts).name("line")),
                    ],
                ),
        );
    })
    .drop_without_applying_deltas();
    let base = initial.unwrap();
    // Only non-empty bins (bins 0, 2, 3) emit radial sectors.
    assert_eq!(base.diagnostics.polar.output_sectors, 3);
    assert_eq!(base.diagnostics.polar.output_regions, 3);

    // Hover directly over scatter point 0 (theta=2.0, r=2.2).
    let pt_pos = base
        .transform
        .position_from_polar(2.0, 2.2, NegativeRadiusMode::ShiftMinRadius)
        .unwrap();
    let mut pt_hover = None;
    ctx.run_ui(support::input(vec![Event::PointerMoved(pt_pos)]), |ui| {
        pt_hover = Some(
            PolarPlot::new("hist_reg")
                .regions([cross_zero.clone(), ring.clone(), patch.clone()])
                .show(
                    ui,
                    &mut state,
                    &[
                        PolarLayer::from(PolarHistogram::new(&hist).name("bins")),
                        PolarLayer::from(PolarScatter::new(&pts).name("pts")),
                    ],
                ),
        );
    })
    .drop_without_applying_deltas();
    let pt_hover = pt_hover.unwrap();
    assert_eq!(pt_hover.hovered_point.as_ref().unwrap().point_index, 0);

    // Hover inside bin 0 (0..60 deg, value=3).
    let bin_pos = base
        .transform
        .position_from_polar(0.25, 1.5, NegativeRadiusMode::ShiftMinRadius)
        .unwrap();
    let mut bin_hover = None;
    ctx.run_ui(support::input(vec![Event::PointerMoved(bin_pos)]), |ui| {
        bin_hover = Some(
            PolarPlot::new("hist_reg")
                .regions([cross_zero.clone(), ring.clone(), patch.clone()])
                .show(
                    ui,
                    &mut state,
                    &[
                        PolarLayer::from(PolarHistogram::new(&hist).name("bins")),
                        PolarLayer::from(PolarScatter::new(&pts).name("pts")),
                    ],
                ),
        );
    })
    .drop_without_applying_deltas();
    let bin_hover = bin_hover.unwrap();
    assert_eq!(bin_hover.hovered_bin.as_ref().unwrap().bin_index, 0);
    assert_eq!(bin_hover.hovered_bin.as_ref().unwrap().count, 3);
}
