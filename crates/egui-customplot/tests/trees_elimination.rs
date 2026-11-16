use egui::Context;
use egui_customplot::{
    EliminationTree, GraphError, GraphPlot, Plot, PlotLayer, PlotState, SpyData, TreeData,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn treeplot_forest_layout_centers_parents_and_rejects_cycles() {
    // MATLAB 1-based parents: root(1)=0, root(2)=0, children of 1 are nodes 2? No: [0, 1, 1, 2, 2, 0, 6].
    // Forest with two roots (index 0 and index 5).
    let tree = TreeData::from_parents(&[0, 1, 1, 2, 2, 0, 6]).unwrap();
    assert_eq!(tree.roots(), &[0, 5]);
    assert_eq!(tree.height(), 2);
    let coords = tree.coords();
    for &[x, y] in coords {
        assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y));
    }
    // Root 0 sits strictly above child 1 which sits strictly above grandchild 3.
    assert!(coords[0][1] > coords[1][1] && coords[1][1] > coords[3][1]);
    // Parent 1 (0-based idx 1) is centered halfway between leaves 3 and 4.
    let expected_mid = 0.5 * (coords[3][0] + coords[4][0]);
    assert!((coords[1][0] - expected_mid).abs() < 1e-12);

    assert_eq!(
        TreeData::from_parents(&[2, 1]).unwrap_err(),
        GraphError::CycleDetected
    );
    assert_eq!(
        TreeData::from_parents(&[1]).unwrap_err(),
        GraphError::CycleDetected
    );

    let g = tree.to_graph();
    let ctx = Context::default();
    let mut state = PlotState::default();
    let resp = support::run(&ctx, vec![], true, |ui| {
        Plot::new("treeplot").graph_axes().show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(GraphPlot::new(&g).name("forest"))],
        )
    });
    assert_eq!(resp.diagnostics.network.output_nodes, 7);
    assert_eq!(resp.diagnostics.network.output_edges, 5);
}

#[test]
fn etreeplot_liu_algorithm_matches_known_sparse_matrices_and_scales_to_10k_chain() {
    // 1. Tridiagonal 6x6 chain: edge (i, i+1) -> MATLAB parent vector [2, 3, 4, 5, 6, 0].
    let tri_edges: Vec<[usize; 2]> = (0..5).map(|i| [i, i + 1]).collect();
    let tri_etree = EliminationTree::from_symmetric_edges(6, &tri_edges).unwrap();
    assert_eq!(tri_etree.parents_matlab_one_based(), vec![2, 3, 4, 5, 6, 0]);

    // 2. Disconnected two-component matrix via SpyData: edges (0,1) and (2,3).
    let spy = SpyData::from_coords(4, 4, &[(0, 1), (1, 0), (2, 3), (3, 2)]).unwrap();
    let spy_etree = EliminationTree::from_spy(&spy).unwrap();
    assert_eq!(spy_etree.parents_matlab_one_based(), vec![2, 0, 4, 0]);
    assert_eq!(spy_etree.postorder().len(), 4);

    // 3. Arrow matrix (star center at 0 connected to 1..4): chained etree [5, 5, 5, 5, 0]?
    // Wait: node 0 is column 0, connected to 1..4 -> parent(0)=1, then fill-in connects 1 to 2..4 -> linear chain!
    let arrow_edges = [[0, 1], [0, 2], [0, 3], [0, 4]];
    let arrow_etree = EliminationTree::from_symmetric_edges(5, &arrow_edges).unwrap();
    assert_eq!(arrow_etree.parents_matlab_one_based(), vec![2, 3, 4, 5, 0]);

    // 4. Deep 10,000-node tridiagonal chain must complete iteratively without stack overflow.
    let chain_edges: Vec<[usize; 2]> = (0..9_999).map(|i| [i, i + 1]).collect();
    let big_etree = EliminationTree::from_symmetric_edges(10_000, &chain_edges).unwrap();
    assert_eq!(big_etree.parents_zero_based()[9_999], None);
    let tree_data = big_etree.to_tree_data();
    assert_eq!(tree_data.height(), 9_999);
}
