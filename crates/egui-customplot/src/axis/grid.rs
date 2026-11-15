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
    let clip = clip.intersect(transform.rect());
    if !clip.is_positive() {
        return [0..0, 0..0];
    }
    let a = transform.point_from_position(clip.min);
    let b = transform.point_from_position(clip.max);
    std::array::from_fn(|axis| {
        let edges = if axis == 0 {
            data.x_edges()
        } else {
            data.y_edges()
        };
        let domain_start = data.domain_start(axis, transform.scales()[axis]);
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
    if !transform.rect().contains(position) {
        return None;
    }
    let point = transform.point_from_position(position);
    let mut indices = [0; 2];
    for axis in 0..2 {
        let edges = if axis == 0 {
            data.x_edges()
        } else {
            data.y_edges()
        };
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
        if cell < data.domain_start(axis, transform.scales()[axis]) {
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
