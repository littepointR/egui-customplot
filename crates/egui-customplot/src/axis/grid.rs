use std::ops::Range;

use crate::{GridData, PlotBounds, PlotTransform};
use egui::{Pos2, Rect};

/// Binary-search visible rectilinear cells. Domain-crossing log cells are not
/// partially invented; both edges must be valid in that coordinate system.
pub(crate) fn visible_grid(
    data: &GridData,
    transform: PlotTransform,
    clip: Rect,
) -> [Range<usize>; 2] {
    visible_edges(
        [data.x_edges(), data.y_edges()],
        [
            data.domain_start(0, transform.scales()[0]),
            data.domain_start(1, transform.scales()[1]),
        ],
        transform,
        clip,
    )
}

pub(crate) fn visible_edges(
    axes: [&[f64]; 2],
    domain_starts: [usize; 2],
    transform: PlotTransform,
    clip: Rect,
) -> [Range<usize>; 2] {
    let clip = clip.intersect(transform.rect());
    if !clip.is_positive() {
        return [0..0, 0..0];
    }
    let a = transform.point_from_position(clip.min);
    let b = transform.point_from_position(clip.max);
    std::array::from_fn(|axis| {
        let edges = axes[axis];
        let domain_start = domain_starts[axis];
        let lo = a[axis].min(b[axis]);
        let hi = a[axis].max(b[axis]);
        let start = edges
            .partition_point(|e| *e <= lo)
            .saturating_sub(1)
            .max(domain_start)
            .min(edges.len() - 1);
        let end = edges
            .partition_point(|e| *e < hi)
            .min(edges.len() - 1)
            .max(start);
        start..end
    })
}

pub(crate) fn grid_cell_at(
    data: &GridData,
    transform: PlotTransform,
    position: Pos2,
) -> Option<[usize; 2]> {
    edge_cell_at(
        [data.x_edges(), data.y_edges()],
        [
            data.domain_start(0, transform.scales()[0]),
            data.domain_start(1, transform.scales()[1]),
        ],
        transform,
        position,
    )
}

pub(crate) fn edge_cell_at(
    axes: [&[f64]; 2],
    domain_starts: [usize; 2],
    transform: PlotTransform,
    position: Pos2,
) -> Option<[usize; 2]> {
    if !transform.rect().contains(position) {
        return None;
    }
    let point = transform.point_from_position(position);
    let mut indices = [0; 2];
    for axis in 0..2 {
        let edges = axes[axis];
        if !point[axis].is_finite()
            || point[axis] < edges[0]
            || point[axis] > edges[edges.len() - 1]
        {
            return None;
        }
        let cell = edges
            .partition_point(|e| *e <= point[axis])
            .saturating_sub(1)
            .min(edges.len() - 2);
        if cell < domain_starts[axis] {
            return None;
        }
        indices[axis] = cell;
    }
    Some(indices)
}

/// Clip in f64 data space before f32 projection, including huge offscreen cells.
pub(crate) fn grid_cell_rect(bounds: PlotBounds, transform: PlotTransform) -> Option<Rect> {
    super::clipped_data_rect(bounds.min(), bounds.max(), transform)
}
