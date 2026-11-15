use crate::{PlotBounds, PlotTransform};

/// Overlay shared data ranges without freezing the other automatic axis.
/// Incompatible log domains or equal-aspect constraints explicitly fall back.
pub(crate) fn apply_axis_ranges(
    transform: PlotTransform,
    ranges: [Option<[f64; 2]>; 2],
) -> (PlotTransform, usize) {
    if transform.data_aspect().is_some() {
        return (transform, ranges.iter().filter(|r| r.is_some()).count());
    }
    let mut result = transform;
    let mut rejected = 0;
    for (axis, range) in ranges.into_iter().enumerate() {
        let Some(range) = range else { continue };
        let mut min = result.bounds().min();
        let mut max = result.bounds().max();
        min[axis] = range[0];
        max[axis] = range[1];
        match PlotBounds::new(min, max)
            .ok()
            .and_then(|b| result.with_bounds(b))
        {
            Some(next) => result = next,
            None => rejected += 1,
        }
    }
    (result, rejected)
}
