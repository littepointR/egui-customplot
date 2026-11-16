//! Host-owned 2D graph, digraph, gplot spatial mesh, treeplot & etreeplot elimination tree gallery.
use eframe::egui::{self, Color32};
use egui_customplot::{
    ColorMap, ColorScale, EliminationTree, Figure, GraphData, GraphHighlight, GraphLayout,
    GraphPlot, Plot, PlotLayer, PlotResponse, PlotState, SpyData, TreeData,
};

use crate::i18n::{lang, tr};

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
        let l = lang(ui.ctx());
        ui.separator();
        ui.checkbox(
            &mut self.enabled,
            tr(
                l,
                "Networks, graphs & trees",
                "网络图与消去树",
                "ネットワーク・グラフ・木",
            ),
        );
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(
                &mut self.highlight_path,
                tr(
                    l,
                    "Highlight active subgraph",
                    "高亮当前子图路径",
                    "部分グラフを強調表示",
                ),
            );
            if ui
                .button(tr(
                    l,
                    "Reset network views",
                    "重置网络图视图",
                    "グラフビュー初期化",
                ))
                .clicked()
            {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small(tr(
                l,
                "plot(graph), plot(digraph) Layered DAG, gplot(Spy), treeplot & etreeplot.",
                "支持力导向无向图、分层 DAG、gplot 网格、treeplot 与消去树。",
                "無向 Force・階層 DAG・gplot・treeplot・消去木に対応。",
            ));
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let l = lang(ui.ctx());
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
                        Plot::new("net-force").height(height).graph_axes().title(tr(
                            l,
                            "plot(graph) Force + Multi-edges + Self-loop",
                            "无向力导向图 + 平行多边 + 自环",
                            "無向グラフ（多重辺＋自己ループ）",
                        )),
                        s0,
                        &[PlotLayer::from(layer)],
                    )
                }
                1 => cell.show_layers(
                    Plot::new("net-dag").height(height).graph_axes().title(tr(
                        l,
                        "plot(digraph) Layered DAG + Bidir Curves",
                        "有向分层 DAG + 双向曲边",
                        "有向階層 DAG＋双方向曲線",
                    )),
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
                    Plot::new("net-gplot").height(height).graph_axes().title(tr(
                        l,
                        "gplot(A, Coordinates) Finite-Element Mesh",
                        "gplot(A, XY) 空间有限元网格",
                        "gplot 空間メッシュ",
                    )),
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
