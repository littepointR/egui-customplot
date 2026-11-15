use std::collections::HashSet;

use crate::model::AreaSource;
use crate::{AxisScale, PlotDiagnostics, PlotLayer, PlotPoint};

#[derive(Default)]
pub(super) struct SeenPayloads {
    points: HashSet<(usize, usize)>,
    indexes: HashSet<usize>,
    grid: HashSet<(usize, usize)>,
    bars: HashSet<(usize, usize)>,
    errors: HashSet<(usize, usize)>,
    quivers: HashSet<(usize, usize)>,
    patches: HashSet<(usize, usize)>,
    contours: HashSet<(usize, usize)>,
    images: HashSet<(usize, usize)>,
    stage8b: super::stage8b::SeenStage8bPayloads,
    stage8c: super::stage8c::SeenStage8cPayloads,
}

pub(super) fn accumulate_layer(
    layer: PlotLayer<'_>,
    scales: [AxisScale; 2],
    diagnostics: &mut PlotDiagnostics,
    seen: &mut SeenPayloads,
) -> Option<(PlotPoint, PlotPoint)> {
    match layer {
        PlotLayer::Series(_) | PlotLayer::Stems(_) => {
            let data = layer.point_data().expect("point-data layer");
            diagnostics.input_points += data.len();
            diagnostics.non_finite_points += data.non_finite_points();
            diagnostics.domain_invalid_points +=
                data.invalid_points(scales) - data.non_finite_points();
            if seen.points.insert((data.identity() as usize, data.len())) {
                diagnostics.unique_sample_bytes += data.sample_bytes();
            }
            if let Some(identity) = data.index_identity()
                && seen.indexes.insert(identity as usize)
            {
                diagnostics.unique_index_bytes += data.line_index_bytes();
            }
            if let PlotLayer::Stems(stems) = layer {
                diagnostics.invalid_stem_baselines += usize::from(!stems.baseline_valid(scales));
                stems.extent(scales)
            } else {
                data.extent(scales)
            }
        }
        PlotLayer::Heatmap(s) => {
            diagnostics.input_heatmap_cells += s.data.values().len();
            diagnostics.non_finite_heatmap_cells += s.data.non_finite_cells();
            diagnostics.color_invalid_heatmap_cells += s.data.invalid_color_cells(s.colors.scale());
            diagnostics.domain_invalid_heatmap_cells += s.data.invalid_domain_cells(scales);
            for allocation in s.data.allocations() {
                if seen.grid.insert(allocation) {
                    diagnostics.unique_grid_bytes += allocation.1;
                }
            }
            s.data.extent(scales)
        }
        PlotLayer::Bars(s) => {
            let invalid = s.data.invalid([AxisScale::Linear; 2], s.orientation);
            diagnostics.input_bars += s.data.len();
            diagnostics.invalid_bars += invalid;
            diagnostics.domain_invalid_bars += s.data.invalid(scales, s.orientation) - invalid;
            let allocation = s.data.allocation();
            if seen.bars.insert(allocation) {
                diagnostics.unique_bar_bytes += allocation.1;
            }
            s.data.extent(scales, s.orientation)
        }
        PlotLayer::ErrorBars(s) => {
            let invalid = s.data.invalid([AxisScale::Linear; 2]);
            diagnostics.input_error_bars += s.data.len();
            diagnostics.invalid_error_bars += invalid;
            diagnostics.domain_invalid_error_bars += s.data.invalid(scales) - invalid;
            let allocation = s.data.allocation();
            if seen.errors.insert(allocation) {
                diagnostics.unique_error_bar_bytes += allocation.1;
            }
            s.data.extent(scales)
        }
        PlotLayer::Quiver(s) => {
            let invalid = s.data.invalid([AxisScale::Linear; 2]);
            diagnostics.input_quiver_arrows += s.data.len();
            diagnostics.invalid_quiver_arrows += invalid;
            diagnostics.domain_invalid_quiver_arrows += s.data.invalid(scales) - invalid;
            let allocation = s.data.allocation();
            if seen.quivers.insert(allocation) {
                diagnostics.unique_quiver_bytes += allocation.1;
            }
            s.data.extent(scales)
        }
        PlotLayer::Area(s) => {
            match s.source {
                AreaSource::Curve { data, .. } => {
                    diagnostics.input_points += data.len();
                    diagnostics.non_finite_points += data.non_finite_points();
                    diagnostics.domain_invalid_points +=
                        data.invalid_points(scales) - data.non_finite_points();
                    if seen.points.insert((data.identity() as usize, data.len())) {
                        diagnostics.unique_sample_bytes += data.sample_bytes();
                    }
                    if let Some(identity) = data.index_identity()
                        && seen.indexes.insert(identity as usize)
                    {
                        diagnostics.unique_index_bytes += data.line_index_bytes();
                    }
                    diagnostics.patches.invalid_area_baselines +=
                        usize::from(!s.baseline_valid(scales));
                }
                AreaSource::Band(band) => {
                    let invalid = band.invalid([AxisScale::Linear; 2]);
                    diagnostics.patches.input_items += band.len();
                    diagnostics.patches.invalid_items += invalid;
                    diagnostics.patches.domain_invalid_items += band.invalid(scales) - invalid;
                    for allocation in band.allocations() {
                        if seen.patches.insert(allocation) {
                            diagnostics.patches.unique_bytes += allocation.1;
                        }
                    }
                }
            }
            s.extent(scales)
        }
        PlotLayer::Fill(s) => {
            diagnostics.patches.input_items += s.data.ring_count();
            diagnostics.patches.invalid_items += s.data.invalid_linear();
            diagnostics.patches.domain_invalid_items += s.data.domain_invalid(scales);
            let allocation = s.data.allocation();
            if seen.patches.insert(allocation) {
                diagnostics.patches.unique_bytes += allocation.1;
            }
            s.data.extent(scales)
        }
        PlotLayer::Patch(s) => {
            diagnostics.patches.input_items += s.data.face_count();
            diagnostics.patches.invalid_items += s.data.invalid_linear();
            diagnostics.patches.domain_invalid_items += s.data.domain_invalid(scales);
            for allocation in s.data.allocations() {
                if seen.patches.insert(allocation) {
                    diagnostics.patches.unique_bytes += allocation.1;
                }
            }
            s.data.extent(scales)
        }
        PlotLayer::Rectangles(s) => {
            let invalid = s.data.invalid([AxisScale::Linear; 2]);
            diagnostics.patches.input_items += s.data.len();
            diagnostics.patches.invalid_items += invalid;
            diagnostics.patches.domain_invalid_items += s.data.invalid(scales) - invalid;
            let allocation = s.data.allocation();
            if seen.patches.insert(allocation) {
                diagnostics.patches.unique_bytes += allocation.1;
            }
            s.data.extent(scales)
        }
        PlotLayer::PolyShape(s) => {
            diagnostics.patches.input_items += s.data.region_count();
            diagnostics.patches.invalid_items += s.data.invalid_linear();
            diagnostics.patches.domain_invalid_items += s.data.domain_invalid(scales);
            let allocation = s.data.allocation();
            if seen.patches.insert(allocation) {
                diagnostics.patches.unique_bytes += allocation.1;
            }
            s.data.extent(scales)
        }
        PlotLayer::Contours(s) => {
            diagnostics.contours.input_lines += s.data.lines().len();
            diagnostics.contours.domain_invalid_items += s.data.domain_invalid(scales);
            let allocation = s.data.allocation();
            if seen.contours.insert(allocation) {
                diagnostics.contours.unique_bytes += allocation.1;
            }
            s.data.extent(scales)
        }
        PlotLayer::FilledContours(s) => {
            diagnostics.contours.input_bands += s.data.bands().len();
            diagnostics.contours.domain_invalid_items += s.data.domain_invalid(scales);
            let allocation = s.data.allocation();
            if seen.contours.insert(allocation) {
                diagnostics.contours.unique_bytes += allocation.1;
            }
            if let Some(lines) = s.border_lines
                && seen.contours.insert(lines.allocation())
            {
                diagnostics.contours.unique_bytes += lines.sample_bytes();
            }
            s.data.extent(scales)
        }
        PlotLayer::Pcolor(s) => {
            let cells = s.data.cell_cols() * s.data.cell_rows();
            diagnostics.images.input_cells += cells;
            diagnostics.images.non_finite_cells += s.data.pcolor_non_finite_cells();
            diagnostics.images.color_invalid_cells +=
                s.data.pcolor_color_invalid_cells(s.colors.scale());
            diagnostics.images.domain_invalid_cells += s.data.invalid_pcolor_cells(scales);
            for allocation in s.data.allocations() {
                if seen.images.insert(allocation) {
                    diagnostics.images.unique_bytes += allocation.1;
                }
            }
            s.data.extent(scales)
        }
        PlotLayer::Image(s) => {
            let pixels = s.data.width() * s.data.height();
            diagnostics.images.input_cells += pixels;
            diagnostics.images.domain_invalid_cells += s.data.invalid_domain_pixels(scales);
            for allocation in s.data.allocations() {
                if seen.images.insert(allocation) {
                    diagnostics.images.unique_bytes += allocation.1;
                }
            }
            s.data.extent(scales)
        }
        PlotLayer::TriPlot(_)
        | PlotLayer::Voronoi(_)
        | PlotLayer::AlphaShape(_)
        | PlotLayer::Spy(_)
        | PlotLayer::Streamlines(_)
        | PlotLayer::Feather(_)
        | PlotLayer::Comet(_) => super::stage8b::accumulate_stage8b_layer(
            layer,
            scales,
            diagnostics,
            &mut seen.stage8b,
            &mut seen.points,
            &mut seen.quivers,
        ),
        other => {
            super::stage8c::accumulate_stage8c_layer(other, scales, diagnostics, &mut seen.stage8c)
        }
    }
}
