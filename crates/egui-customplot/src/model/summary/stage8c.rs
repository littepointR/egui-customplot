use std::collections::HashSet;

use crate::{AxisScale, PlotBounds, PlotDiagnostics, PlotLayer};

#[derive(Default)]
pub(super) struct SeenStage8cPayloads {
    ptrs: HashSet<usize>,
}

impl SeenStage8cPayloads {
    fn first_seen<T>(&mut self, ptr: *const T) -> bool {
        self.ptrs.insert(ptr as usize)
    }
}

pub(super) fn accumulate_stage8c_layer(
    layer: PlotLayer<'_>,
    scales: [AxisScale; 2],
    diagnostics: &mut PlotDiagnostics,
    seen: &mut SeenStage8cPayloads,
) -> Option<(crate::PlotPoint, crate::PlotPoint)> {
    let bounds: Option<PlotBounds> = match layer {
        PlotLayer::BubbleChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_bubbles += s.data.points().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::SwarmChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_bubbles += s.data.points().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::Histogram2(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_bins += s.data.counts().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::BinScatter(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_bins += s.data.cells().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::BoxChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_boxes += s.data.boxes().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::ViolinPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_violins += s.data.entries().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::Raincloud(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_violins += s.data.entries().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::PieChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_slices += s.data.slices().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::WordCloud(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_clouds += s.data.words().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::BubbleCloud(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_clouds += s.data.bubbles().len();
            }
            s.data.bounds(scales)
        }
        PlotLayer::CategoricalHeatmap(s) => {
            if seen.first_seen(s.data) {
                let grid = s.data.grid();
                diagnostics.input_heatmap_cells +=
                    (grid.x_edges().len() - 1) * (grid.y_edges().len() - 1);
            }
            s.data.bounds(scales)
        }
        PlotLayer::ParallelPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_parallel_rows += s.data.n_rows();
            }
            s.data.bounds(scales)
        }
        _ => None,
    };
    bounds.map(|b| (b.min(), b.max()))
}
