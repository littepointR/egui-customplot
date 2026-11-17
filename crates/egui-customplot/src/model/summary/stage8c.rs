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
            s.data.bounds_oriented(scales, s.orientation)
        }
        PlotLayer::ViolinPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_violins += s.data.entries().len();
            }
            s.data.bounds_oriented(scales, s.orientation)
        }
        PlotLayer::Raincloud(s) => {
            if seen.first_seen(s.data) {
                diagnostics.stats.input_violins += s.data.entries().len();
            }
            s.data.bounds_oriented(scales, s.orientation)
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
        PlotLayer::ConfusionChart(s) => {
            if seen.first_seen(s.data) {
                let n = s.data.n_classes();
                diagnostics.input_heatmap_cells += n * n;
            }
            s.data.bounds(scales, s.show_summaries)
        }
        PlotLayer::CurvilinearPcolor(s) => {
            if seen.first_seen(s.data) {
                diagnostics.images.input_cells += s.data.cell_rows() * s.data.cell_cols();
                diagnostics.images.non_finite_cells += s.data.non_finite_nodes();
                diagnostics.images.domain_invalid_cells += s.data.domain_invalid(scales);
                let alloc = s.data.allocation();
                diagnostics.images.unique_bytes += alloc.1;
            }
            s.data.bounds(scales)
        }
        PlotLayer::ProbabilityPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.len();
                diagnostics.domain_invalid_points += s.data.invalid(scales);
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::EcdfPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.len();
                diagnostics.domain_invalid_points += s.data.invalid(scales);
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::DendrogramPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.network.input_edges += s.data.branches().len();
                diagnostics.network.unique_bytes += s.data.sample_bytes();
            }
            s.data.bounds(s.orientation, scales)
        }
        PlotLayer::Biplot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_quiver_arrows += s.data.n_vars();
                diagnostics.input_points += s.data.n_obs();
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::ControlChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.points().len();
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::Candlestick(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_bars += s.data.len();
                diagnostics.unique_bar_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::OhlcBars(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_bars += s.data.len();
                diagnostics.unique_bar_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::Waterfall2D(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_bars += s.data.len();
                diagnostics.unique_bar_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::ZPlanePlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.zeros().len() + s.data.poles().len();
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::NyquistPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.nyquist_curve().len();
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data
                .nyquist_curve()
                .extent(scales)
                .and_then(|(mut min, mut max)| {
                    if scales[0] == AxisScale::Linear && scales[1] == AxisScale::Linear {
                        min[0] = min[0].min(-1.15);
                        max[0] = max[0].max(1.15);
                        min[1] = min[1].min(-1.15);
                        max[1] = max[1].max(1.15);
                    }
                    PlotBounds::new(min, max).ok()
                })
        }
        PlotLayer::NicholsPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.nichols_curve().len();
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data
                .nichols_curve()
                .extent(scales)
                .and_then(|(mut min, mut max)| {
                    if scales[0] == AxisScale::Linear && scales[1] == AxisScale::Linear {
                        min[0] = min[0].min(-210.0);
                        max[0] = max[0].max(0.0);
                        min[1] = min[1].min(-6.0);
                        max[1] = max[1].max(6.0);
                    }
                    PlotBounds::new(min, max).ok()
                })
        }
        PlotLayer::RootLocus(s) => {
            if seen.first_seen(s.data) {
                diagnostics.input_points += s.data.open_loop_poles().len();
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::CwtPlot(s) => {
            if seen.first_seen(s.data) {
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::RadarChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::SankeyChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::TreemapChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::SunburstChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        PlotLayer::GanttChart(s) => {
            if seen.first_seen(s.data) {
                diagnostics.unique_sample_bytes += s.data.sample_bytes();
            }
            s.data.bounds(scales)
        }
        _ => None,
    };
    bounds.map(|b| (b.min(), b.max()))
}
