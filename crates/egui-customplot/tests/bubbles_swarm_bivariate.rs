use egui::{Color32, Context};
use egui_customplot::{
    BinScatter, BinScatterData, BinShape, BivariateNormalization as BivNorm, BubbleChart,
    BubbleData, BubbleLegend, ColorMap, ColorScale, Histogram2, Histogram2Data, Plot, PlotLayer,
    PlotState, RegionKind, SwarmChart, SwarmData,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn bubble_swarm_and_legend_render_and_pick_accurately() {
    let xy = [[1.0, 2.0], [2.0, 5.0], [3.0, 3.5], [f64::NAN, 1.0]];
    let sizes = [10.0, 40.0, 90.0, 20.0];
    let bubble = BubbleData::new(&xy, &sizes)
        .unwrap()
        .with_colors(&[1.0, 2.0, 3.0, 0.0])
        .unwrap()
        .radius_range(5.0, 20.0)
        .unwrap();
    assert_eq!(bubble.finite_count(), 3);
    assert!(bubble.pixel_radius(10.0) < bubble.pixel_radius(40.0));
    assert!(bubble.pixel_radius(40.0) < bubble.pixel_radius(90.0));

    let swarm = SwarmData::from_groups(
        &[1.0, 2.0],
        &[&[2.0, 2.05, 2.02, 2.04, 3.0], &[4.0, 4.01, 4.02]],
        0.6,
    )
    .unwrap();
    assert_eq!(swarm.points().len(), 8);
    assert!(
        swarm
            .points()
            .iter()
            .all(|p| (p.x - p.group_x).abs() <= 0.35)
    );

    let scale = ColorScale::new([0.0, 4.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    let legend = BubbleLegend::new(bubble.size_range(), bubble.radius_limits()).title("Size");
    let ctx = Context::default();
    let mut state = PlotState::default();

    let first = support::run(&ctx, vec![], true, |ui| {
        let lr = legend.clone().show(ui);
        assert_eq!(lr.samples.len(), 3);
        Plot::new("bubbles_swarm")
            .bubble_legend(legend.clone())
            .show_layers(
                ui,
                &mut state,
                &[
                    PlotLayer::from(BubbleChart::new(&bubble).color_scale(scale)),
                    PlotLayer::from(SwarmChart::new(&swarm).color(Color32::from_rgb(220, 90, 30))),
                ],
            )
    });
    assert!(first.diagnostics.stats.output_bubbles >= 10);
    let hover_pos = first.transform.position_from_point([2.0, 5.0]);
    let picked = support::run(
        &ctx,
        vec![egui::Event::PointerMoved(hover_pos)],
        false,
        |ui| {
            let _ = legend.clone().show(ui);
            Plot::new("bubbles_swarm")
                .bubble_legend(legend.clone())
                .show_layers(
                    ui,
                    &mut state,
                    &[
                        PlotLayer::from(BubbleChart::new(&bubble).color_scale(scale)),
                        PlotLayer::from(
                            SwarmChart::new(&swarm).color(Color32::from_rgb(220, 90, 30)),
                        ),
                    ],
                )
        },
    );
    let pt = picked.hovered_point.expect("hover bubble");
    assert_eq!(pt.point, [2.0, 5.0]);
}

#[test]
fn histogram2_normalizations_and_binscatter_shapes_render_and_pick() {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 0..40 {
        xs.push((i % 8) as f64 + 0.5);
        ys.push((i / 8) as f64 + 0.5);
    }
    let hist = Histogram2Data::new(&xs, &ys, 4, 4).unwrap();
    assert_eq!(hist.included_samples(), 40);
    for norm in [
        BivNorm::Count,
        BivNorm::Probability,
        BivNorm::Density,
        BivNorm::CumulativeCount,
        BivNorm::CumulativeProbability,
    ] {
        let g = hist.to_grid(norm).unwrap();
        assert_eq!(g.values().len(), 16);
    }
    let pts: Vec<[f64; 2]> = xs.iter().zip(ys.iter()).map(|(&a, &b)| [a, b]).collect();
    let hex = BinScatterData::new(&pts, 4, 4, BinShape::Hexagon).unwrap();
    let rect_bins = BinScatterData::new(&pts, 4, 4, BinShape::Rectangle).unwrap();
    assert!(!hex.cells().is_empty());

    let cscale = ColorScale::new([0.0, 10.0])
        .unwrap()
        .color_map(ColorMap::BlueRed);
    let ctx = Context::default();
    let mut h_state = PlotState::default();
    let mut b_state = PlotState::default();

    let h_res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("h2").show_layers(
            ui,
            &mut h_state,
            &[PlotLayer::from(
                Histogram2::new(&hist, cscale).show_counts(true),
            )],
        )
    });
    assert!(h_res.diagnostics.stats.output_bins > 0);
    let h_pos = h_res.transform.position_from_point([2.0, 2.0]);
    let h_hover = support::run(&ctx, vec![egui::Event::PointerMoved(h_pos)], false, |ui| {
        Plot::new("h2").show_layers(
            ui,
            &mut h_state,
            &[PlotLayer::from(Histogram2::new(&hist, cscale))],
        )
    });
    assert_eq!(
        h_hover.hovered_region.expect("hist2 region").kind,
        RegionKind::Histogram2Bin
    );

    let b_res = support::run(&ctx, vec![], true, |ui| {
        Plot::new("b2").show_layers(
            ui,
            &mut b_state,
            &[PlotLayer::from(BinScatter::new(&hex, cscale))],
        )
    });
    assert!(b_res.diagnostics.stats.output_bins > 0);
    let first_cell = &rect_bins.cells()[0];
    let mut r_state = PlotState::default();
    let r_res = support::run(&ctx, vec![], false, |ui| {
        Plot::new("r2").show_layers(
            ui,
            &mut r_state,
            &[PlotLayer::from(BinScatter::new(&rect_bins, cscale))],
        )
    });
    let r_pos = r_res
        .transform
        .position_from_point([first_cell.cx, first_cell.cy]);
    let r_hover = support::run(&ctx, vec![egui::Event::PointerMoved(r_pos)], false, |ui| {
        Plot::new("r2").show_layers(
            ui,
            &mut r_state,
            &[PlotLayer::from(BinScatter::new(&rect_bins, cscale))],
        )
    });
    assert_eq!(
        r_hover.hovered_region.expect("binscatter region").kind,
        RegionKind::BinScatterCell
    );
}
