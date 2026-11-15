use egui::{Pos2, Rect};

use crate::{PlotPoint, PlotTransform};

/// Independent primitives are rejected as a whole across coordinate domains.
pub(crate) fn valid_extent(lo: PlotPoint, hi: PlotPoint, transform: PlotTransform) -> bool {
    transform.is_valid_point(lo) && transform.is_valid_point(hi)
}

/// Clip in f64 data space before f32 projection, including huge offscreen cells.
pub(crate) fn clipped_data_rect(
    lo: PlotPoint,
    hi: PlotPoint,
    transform: PlotTransform,
) -> Option<Rect> {
    let view = transform.bounds();
    let min = std::array::from_fn(|i| lo[i].max(view.min()[i]));
    let max = std::array::from_fn(|i| hi[i].min(view.max()[i]));
    if min[0] >= max[0] || min[1] >= max[1] {
        return None;
    }
    let rect = Rect::from_two_pos(
        transform.position_from_point(min),
        transform.position_from_point(max),
    );
    (rect.is_finite() && rect.is_positive()).then_some(rect)
}

/// Axis-aligned data segment. Clamp before normalization to avoid overflow in
/// huge error extents. Does not create a new cap/edge at the clipping boundary.
pub(crate) fn clipped_axis_segment(
    mut a: PlotPoint,
    mut b: PlotPoint,
    varying_axis: usize,
    transform: PlotTransform,
) -> Option<[Pos2; 2]> {
    let fixed = 1 - varying_axis;
    let view = transform.bounds();
    if a[fixed] < view.min()[fixed] || a[fixed] > view.max()[fixed] {
        return None;
    }
    if a[varying_axis] > b[varying_axis] {
        std::mem::swap(&mut a, &mut b);
    }
    a[varying_axis] = a[varying_axis].max(view.min()[varying_axis]);
    b[varying_axis] = b[varying_axis].min(view.max()[varying_axis]);
    if a[varying_axis] >= b[varying_axis] {
        return None;
    }
    let pair = [
        transform.position_from_point(a),
        transform.position_from_point(b),
    ];
    pair.iter().all(|p| p.is_finite()).then_some(pair)
}

pub(crate) fn rect_edges(lo: PlotPoint, hi: PlotPoint) -> [(PlotPoint, PlotPoint, usize); 4] {
    [
        (lo, [hi[0], lo[1]], 0),
        ([lo[0], hi[1]], hi, 0),
        (lo, [lo[0], hi[1]], 1),
        ([hi[0], lo[1]], hi, 1),
    ]
}
