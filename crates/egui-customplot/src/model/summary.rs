use std::collections::HashSet;

use egui::IdSalt;

use crate::{AxisScale, PlotBounds, PlotDiagnostics, PlotLayer, YAxis};

pub(crate) struct SeriesSummary {
    pub bounds: [PlotBounds; 2],
    pub diagnostics: PlotDiagnostics,
    pub present: HashSet<IdSalt>,
}

/// Merge cached extents only. Samples/index payloads are deduplicated separately
/// from descriptor counts and visibility; no per-show raw data scan occurs here.
pub(crate) fn summarize_series(
    series: &[PlotLayer<'_>],
    scales: [AxisScale; 2],
    right_scale: Option<AxisScale>,
    visible: impl Fn(IdSalt) -> bool,
) -> SeriesSummary {
    let mut diagnostics = PlotDiagnostics::default();
    let mut present = HashSet::new();
    let mut seen = HashSet::new();
    let mut seen_indexes = HashSet::new();
    let mut seen_grid = HashSet::new();
    let mut seen_bars = HashSet::new();
    let mut seen_errors = HashSet::new();
    let mut ranges: [[Option<[f64; 2]>; 2]; 2] = [[None; 2]; 2];
    for (index, s) in series.iter().enumerate() {
        if !present.insert(s.id(index)) {
            diagnostics.duplicate_series_ids += 1;
        }
        let right = s.y_axis() == YAxis::Right && right_scale.is_some();
        if s.y_axis() == YAxis::Right && right_scale.is_none() {
            diagnostics.unconfigured_right_axis_series += 1;
        }
        let scales = [
            scales[0],
            if right {
                right_scale.unwrap()
            } else {
                scales[1]
            },
        ];
        let extent = match *s {
            PlotLayer::Series(s) => {
                let data = s.data();
                diagnostics.input_points += data.len();
                diagnostics.non_finite_points += data.non_finite_points();
                diagnostics.domain_invalid_points +=
                    data.invalid_points(scales) - data.non_finite_points();
                if seen.insert((data.identity(), data.len())) {
                    diagnostics.unique_sample_bytes += data.sample_bytes();
                }
                if let Some(identity) = data.index_identity()
                    && seen_indexes.insert(identity)
                {
                    diagnostics.unique_index_bytes += data.line_index_bytes();
                }
                data.extent(scales)
            }
            PlotLayer::Heatmap(s) => {
                diagnostics.input_heatmap_cells += s.data.values().len();
                diagnostics.non_finite_heatmap_cells += s.data.non_finite_cells();
                diagnostics.color_invalid_heatmap_cells +=
                    s.data.invalid_color_cells(s.colors.scale());
                diagnostics.domain_invalid_heatmap_cells += s.data.invalid_domain_cells(scales);
                for allocation in s.data.allocations() {
                    if seen_grid.insert(allocation) {
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
                if seen_bars.insert(allocation) {
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
                if seen_errors.insert(allocation) {
                    diagnostics.unique_error_bar_bytes += allocation.1;
                }
                s.data.extent(scales)
            }
        };
        if !visible(s.id(index)) {
            continue;
        }
        diagnostics.visible_series += 1;
        if let Some((lo, hi)) = extent {
            for axis in 0..2 {
                merge(&mut ranges[usize::from(right)][axis], [lo[axis], hi[axis]]);
            }
        }
    }
    // A right-only plot still gets its X range from the right-hand data. Each Y
    // is fit independently, using that side's scale and finite XY domain.
    let mut x = ranges[0][0];
    if let Some(right_x) = ranges[1][0] {
        merge(&mut x, right_x);
    }
    ranges[0][0] = x;
    ranges[1][0] = x;
    let bounds = std::array::from_fn(|side| {
        let scales = [
            scales[0],
            if side == 1 {
                right_scale.unwrap_or(scales[1])
            } else {
                scales[1]
            },
        ];
        PlotBounds::fit_optional_ranges(ranges[side], scales).unwrap_or_else(|_| {
            diagnostics.auto_bounds_error = true;
            PlotBounds::default_scaled(scales)
        })
    });
    SeriesSummary {
        bounds,
        diagnostics,
        present,
    }
}

fn merge(target: &mut Option<[f64; 2]>, range: [f64; 2]) {
    let current = target.get_or_insert(range);
    current[0] = current[0].min(range[0]);
    current[1] = current[1].max(range[1]);
}
