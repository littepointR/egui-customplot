use egui::{Color32, Context, Event};
use egui_customplot::{
    ColorMap, ColorScale, GraphData, GraphHighlight, GraphLayout, GraphPlot, Plot, PlotLayer,
    PlotState, RegionKind, SpyData,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn undirected_force_multi_edges_self_loops_and_gplot_render() {
    let edges = [[0, 1], [0, 1], [1, 2], [2, 0], [1, 1]];
    let g1 = GraphData::undirected(3, &edges, GraphLayout::force_default())
        .unwrap()
        .with_node_labels(&["A", "B", "C"])
        .unwrap()
        .with_edge_labels(&["e0", "e1", "e2", "e3", "loop"])
        .unwrap();
    let g2 = GraphData::undirected(3, &edges, GraphLayout::force_default()).unwrap();
    // Deterministic seeded Fruchterman-Reingold force layout.
    assert_eq!(g1.coords(), g2.coords());
    assert_eq!(g1.self_loops(), 1);
    assert_eq!(g1.multi_edges(), 1);

    let gplot_coords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [f64::NAN, 1.0]];
    let spy = SpyData::from_coords(4, 4, &[(0, 1), (1, 2), (2, 0), (0, 3)]).unwrap();
    let gplot = GraphData::from_spy(&spy, &gplot_coords).unwrap();
    assert_eq!(gplot.non_finite_nodes(), 1);

    let hl = GraphHighlight::new()
        .nodes(&[1])
        .edges(&[0, 4])
        .color(Color32::RED);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let resp = support::run(&ctx, vec![], true, |ui| {
        Plot::new("graph_test").graph_axes().show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(
                GraphPlot::new(&g1)
                    .name("net")
                    .show_node_labels(true)
                    .show_edge_labels(true)
                    .highlight(hl),
            )],
        )
    });
    assert_eq!(resp.diagnostics.network.output_nodes, 3);
    assert_eq!(resp.diagnostics.network.output_edges, 5);
    assert!(resp.diagnostics.network.output_labels >= 5);

    let mut gp_state = PlotState::default();
    let gp_resp = support::run(&ctx, vec![], false, |ui| {
        Plot::new("gplot_test").graph_axes().show_layers(
            ui,
            &mut gp_state,
            &[PlotLayer::from(GraphPlot::new(&gplot).name("spy_mesh"))],
        )
    });
    // Node 3 has NaN coordinates, so only nodes 0..=2 render and edges incident to NaN node skip.
    assert_eq!(gp_resp.diagnostics.network.output_nodes, 3);
}

#[test]
fn directed_layered_dag_renders_arrows_bidirectional_curves_and_picks_nodes_and_edges() {
    let edges = [[0, 1], [1, 0], [0, 2], [1, 3], [2, 3]];
    let dg = GraphData::directed(4, &edges, GraphLayout::Layered)
        .unwrap()
        .with_edge_weights(&[1.0, 2.0, 1.5, 2.5, 3.0])
        .unwrap();
    assert!(dg.edges()[0].bidirectional_pair);
    assert!(dg.edges()[1].bidirectional_pair);
    assert_eq!(dg.layout_kind(), egui_customplot::GraphLayoutKind::Layered);

    let cs = ColorScale::new([1.0, 3.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    let weights = [1.0, 2.0, 1.5, 2.5, 3.0];

    let ctx = Context::default();
    let mut state = PlotState::default();
    let base = support::run(&ctx, vec![], false, |ui| {
        Plot::new("digraph_test").graph_axes().show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(
                GraphPlot::new(&dg)
                    .name("dag")
                    .edge_scalars(&weights, cs)
                    .node_size(8.0),
            )],
        )
    });
    assert_eq!(base.diagnostics.network.output_arrows, 5);

    // Pick node 2 directly.
    let node_pos = base.transform.position_from_point(dg.coords()[2]);
    let node_hit = support::run(&ctx, vec![Event::PointerMoved(node_pos)], false, |ui| {
        Plot::new("digraph_test").graph_axes().show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(GraphPlot::new(&dg).node_size(8.0))],
        )
    });
    assert_eq!(node_hit.hovered_point.as_ref().unwrap().point_index, 2);

    // Pick edge 0->2 midpoint on a fixed coordinate graph so edge point is known.
    let straight_g = GraphData::undirected(
        2,
        &[[0, 1]],
        GraphLayout::Coordinates(std::sync::Arc::from([[0.0, 0.0], [2.0, 0.0]])),
    )
    .unwrap();
    let mut s_state = PlotState::default();
    let s_base = support::run(&ctx, vec![], false, |ui| {
        Plot::new("edge_pick").graph_axes().show_layers(
            ui,
            &mut s_state,
            &[PlotLayer::from(GraphPlot::new(&straight_g))],
        )
    });
    let mid_pos = s_base.transform.position_from_point([1.0, 0.0]);
    let edge_hit = support::run(&ctx, vec![Event::PointerMoved(mid_pos)], false, |ui| {
        Plot::new("edge_pick").graph_axes().show_layers(
            ui,
            &mut s_state,
            &[PlotLayer::from(GraphPlot::new(&straight_g))],
        )
    });
    assert_eq!(
        edge_hit.hovered_region.as_ref().expect("edge hover").kind,
        RegionKind::GraphEdge
    );
}
