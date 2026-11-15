use std::collections::HashSet;

use crate::{AxisScale, PlotDiagnostics, PlotLayer, PlotPoint};

#[derive(Default)]
pub(super) struct SeenStage8bPayloads {
    geometry: HashSet<(usize, usize)>,
    flow: HashSet<(usize, usize)>,
}

pub(super) fn accumulate_stage8b_layer(
    layer: PlotLayer<'_>,
    scales: [AxisScale; 2],
    diagnostics: &mut PlotDiagnostics,
    seen: &mut SeenStage8bPayloads,
    seen_points: &mut HashSet<(usize, usize)>,
    seen_quivers: &mut HashSet<(usize, usize)>,
) -> Option<(PlotPoint, PlotPoint)> {
    match layer {
        PlotLayer::TriPlot(s) => {
            diagnostics.geometry.input_edges += s.data.edges().len();
            diagnostics.geometry.input_triangles += s.data.triangles().len();
            diagnostics.geometry.domain_invalid_items += s.data.invalid(scales);
            if seen.geometry.insert(s.data.allocation()) {
                diagnostics.geometry.unique_bytes += s.data.sample_bytes();
            }
            s.data.extent(scales)
        }
        PlotLayer::Voronoi(s) => {
            diagnostics.geometry.input_edges += s.data.edges().len();
            diagnostics.geometry.domain_invalid_items += s.data.invalid(scales);
            if seen.geometry.insert(s.data.allocation()) {
                diagnostics.geometry.unique_bytes += s.data.sample_bytes();
            }
            s.data.extent(scales)
        }
        PlotLayer::AlphaShape(s) => {
            diagnostics.geometry.input_triangles += s.data.patch().face_count();
            diagnostics.geometry.input_edges += s.data.boundary_edges().len();
            diagnostics.geometry.domain_invalid_items += s.data.invalid(scales);
            if seen.geometry.insert(s.data.allocation()) {
                diagnostics.geometry.unique_bytes += s.data.sample_bytes();
            }
            s.data.extent(scales)
        }
        PlotLayer::Spy(s) => {
            diagnostics.geometry.input_spy_nonzeros += s.data.nnz();
            if seen.geometry.insert(s.data.allocation()) {
                diagnostics.geometry.unique_bytes += s.data.sample_bytes();
            }
            let ext = s.data.extent(scales);
            if ext.is_none() {
                diagnostics.geometry.domain_invalid_items += s.data.nnz();
            }
            ext
        }
        PlotLayer::Streamlines(s) => {
            diagnostics.flow.input_streamlines += s.data.lines().len();
            diagnostics.flow.domain_invalid_items += s.data.invalid(scales);
            if seen.flow.insert(s.data.allocation()) {
                diagnostics.flow.unique_bytes += s.data.sample_bytes();
            }
            s.data.extent(scales)
        }
        PlotLayer::Feather(s) => {
            let q = s.data.quiver();
            diagnostics.flow.input_feather_vectors += q.len();
            diagnostics.flow.domain_invalid_items += q.invalid(scales);
            let allocation = q.allocation();
            if seen.flow.insert(allocation) && seen_quivers.insert(allocation) {
                diagnostics.flow.unique_bytes += s.data.sample_bytes();
            }
            q.extent(scales)
        }
        PlotLayer::Comet(s) => {
            diagnostics.flow.input_comet_points += s.data.len();
            diagnostics.flow.domain_invalid_items +=
                s.data.invalid_points(scales) - s.data.non_finite_points();
            if seen.flow.insert((s.data.identity() as usize, s.data.len()))
                && seen_points.insert((s.data.identity() as usize, s.data.len()))
            {
                diagnostics.flow.unique_bytes += s.data.sample_bytes();
            }
            s.data.extent(scales)
        }
        _ => unreachable!("non-Stage8B layer dispatched to Stage 8B accumulator"),
    }
}
