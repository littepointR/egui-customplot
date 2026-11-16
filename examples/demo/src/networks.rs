//! Host-owned 2D graph, digraph, gplot spatial mesh, treeplot & etreeplot elimination tree gallery.
use eframe::egui::{self, Color32};
use egui_customplot::{
    ColorMap, ColorScale, EliminationTree, Figure, GraphData, GraphHighlight, GraphLayout,
    GraphPlot, Plot, PlotLayer, PlotResponse, PlotState, SpyData, TreeData,
};

pub(super) struct NetworksDemo {
    pub enabled: bool,
    pub highlight_path: bool,
    force_graph: GraphData,
    layered_digraph: GraphData,
    gplot_mesh: GraphData,
    tree_plot: GraphData,
    etree_plot: GraphData,
    states: [PlotState; 4],
}

impl Default for NetworksDemo {
    fn default() -> Self {
        let force_edges = [
            [0, 1],
            [0, 1],
            [1, 2],
            [2, 3],
            [3, 4],
            [4, 5],
            [5, 0],
            [0, 3],
            [2, 2],
            [1, 4],
            [5, 6],
            [6, 7],
            [7, 4],
        ];
        let nlabels: Vec<String> = (0..8).map(|i| format!("N{i}")).collect();
        let force_graph = GraphData::undirected(8, &force_edges, GraphLayout::force_default())
            .expect("force graph")
            .with_node_labels(&nlabels)
            .expect("labels");

        let dag_edges = [
            [0, 1],
            [0, 2],
            [1, 3],
            [1, 4],
            [2, 4],
            [4, 2], // bidirectional pair
            [3, 5],
            [4, 5],
            [4, 6],
            [5, 7],
            [6, 7],
        ];
        let dag_labels: Vec<String> = [
            "Src", "Parse", "Cache", "Plan", "Opt", "Code", "Link", "Out",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let dag_weights = [1.0, 2.0, 2.5, 3.0, 1.5, 1.2, 3.5, 4.0, 2.0, 4.5, 3.0];
        let layered_digraph = GraphData::directed(8, &dag_edges, GraphLayout::Layered)
            .expect("layered digraph")
            .with_edge_weights(&dag_weights)
            .expect("weights")
            .with_node_labels(&dag_labels)
            .expect("dag labels");

        // 4x4 finite-element grid adjacency via SpyData (gplot).
        let mut coords = Vec::with_capacity(16);
        let mut entries = Vec::new();
        for iy in 0..4 {
            for ix in 0..4 {
                let idx = iy * 4 + ix;
                coords.push([ix as f64 + 0.1 * (iy as f64).sin(), iy as f64]);
                if ix + 1 < 4 {
                    entries.push((idx, idx + 1));
                    entries.push((idx + 1, idx));
                }
                if iy + 1 < 4 {
                    entries.push((idx, idx + 4));
                    entries.push((idx + 4, idx));
                }
                if ix + 1 < 4 && iy + 1 < 4 && (ix + iy) % 2 == 0 {
                    entries.push((idx, idx + 5));
                    entries.push((idx + 5, idx));
                }
            }
        }
        let spy = SpyData::from_coords(16, 16, &entries).expect("mesh spy");
        let gplot_mesh = GraphData::from_spy(&spy, &coords).expect("gplot mesh");

        let tree = TreeData::from_parents(&[0, 1, 1, 2, 2, 3, 3, 4]).expect("treeplot");
        let tree_plot = tree
            .to_graph()
            .with_node_labels(&["0", "1", "2", "3", "4", "5", "6", "7"])
            .expect("tree labels");
        let etree = EliminationTree::from_spy(&spy).expect("etree");
        let etree_plot = etree.to_tree_data().to_graph();

        Self {
            enabled: false,
            highlight_path: true,
            force_graph,
            layered_digraph,
            gplot_mesh,
            tree_plot,
            etree_plot,
            states: Default::default(),
        }
    }
}

impl NetworksDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Networks, graphs & trees");
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(&mut self.highlight_path, "Highlight active subgraph");
            if ui.button("Reset network views").clicked() {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small("plot(graph), plot(digraph) Layered DAG, gplot(Spy), treeplot & etreeplot.");
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 120.0) * 0.5).max(110.0);
        let cscale = ColorScale::new([1.0, 4.5])
            .expect("valid scale")
            .color_map(ColorMap::Viridis);
        let weights: Vec<f64> = self
            .layered_digraph
            .edges()
            .iter()
            .map(|e| e.weight)
            .collect();
        let hl = GraphHighlight::new()
            .nodes(&[0, 1, 2, 3])
            .edges(&[0, 2, 8])
            .color(Color32::from_rgb(220, 45, 45));

        let [s0, s1, s2, s3] = &mut self.states;
        Figure::new("networks-gallery")
            .columns(2)
            .cell_height(height)
            .show(ui, 4, |cell| match cell.index() {
                0 => {
                    let mut layer = GraphPlot::new(&self.force_graph)
                        .name("Undirected Force")
                        .show_node_labels(true);
                    if self.highlight_path {
                        layer = layer.highlight(hl);
                    }
                    cell.show_layers(
                        Plot::new("net-force")
                            .height(height)
                            .graph_axes()
                            .title("plot(graph) Force + Multi-edges + Self-loop"),
                        s0,
                        &[PlotLayer::from(layer)],
                    )
                }
                1 => cell.show_layers(
                    Plot::new("net-dag")
                        .height(height)
                        .graph_axes()
                        .title("plot(digraph) Layered DAG + Bidir Curves"),
                    s1,
                    &[PlotLayer::from(
                        GraphPlot::new(&self.layered_digraph)
                            .name("Pipeline DAG")
                            .edge_scalars(&weights, cscale)
                            .show_node_labels(true)
                            .node_size(8.0),
                    )],
                ),
                2 => cell.show_layers(
                    Plot::new("net-gplot")
                        .height(height)
                        .graph_axes()
                        .title("gplot(A, Coordinates) Finite-Element Mesh"),
                    s2,
                    &[PlotLayer::from(
                        GraphPlot::new(&self.gplot_mesh)
                            .name("gplot")
                            .node_color(Color32::from_rgb(46, 160, 67))
                            .edge_color(Color32::from_gray(140)),
                    )],
                ),
                _ => cell.show_layers(
                    Plot::new("net-tree")
                        .height(height)
                        .graph_axes()
                        .legend(true)
                        .title("treeplot & etreeplot(Spy)"),
                    s3,
                    &[
                        PlotLayer::from(
                            GraphPlot::new(&self.etree_plot)
                                .name("etreeplot (16-node mesh)")
                                .node_color(Color32::from_rgb(0, 114, 189))
                                .edge_color(Color32::from_rgb(100, 160, 210)),
                        ),
                        PlotLayer::from(
                            GraphPlot::new(&self.tree_plot)
                                .name("treeplot (offset overlay)")
                                .node_color(Color32::from_rgb(217, 83, 25))
                                .show_node_labels(true)
                                .node_size(4.5),
                        ),
                    ],
                ),
            })
            .expect("networks gallery figure")
            .inner
    }
}
