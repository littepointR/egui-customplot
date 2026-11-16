use std::collections::HashSet;

use crate::{AxisScale, PlotDiagnostics, PlotLayer, PlotPoint};

#[derive(Default)]
pub(super) struct SeenStage8dPayloads {
    ptrs: HashSet<(usize, usize)>,
}

pub(super) fn accumulate_stage8d_layer(
    layer: PlotLayer<'_>,
    scales: [AxisScale; 2],
    diagnostics: &mut PlotDiagnostics,
    seen: &mut SeenStage8dPayloads,
) -> Option<(PlotPoint, PlotPoint)> {
    match layer {
        PlotLayer::Graph(g) => {
            diagnostics.network.input_nodes += g.data.node_count();
            diagnostics.network.input_edges += g.data.edges().len();
            diagnostics.network.self_loops += g.data.self_loops();
            diagnostics.network.multi_edges += g.data.multi_edges();
            diagnostics.network.invalid_items += g.data.non_finite_nodes();
            let allocation = g.data.allocation();
            if seen.ptrs.insert(allocation) {
                diagnostics.network.unique_bytes += allocation.1;
            }
            g.data.extent(scales)
        }
        _ => None,
    }
}
