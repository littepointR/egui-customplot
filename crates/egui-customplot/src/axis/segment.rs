use egui::Pos2;

use crate::{PlotPoint, PlotTransform};

pub(crate) struct ProjectedSegment {
    pub shaft: [Pos2; 2],
    pub direction: [f64; 2],
    pub length: f64,
    pub origin_visible: bool,
    pub end_visible: bool,
}

/// Independent vector projection with a conservative cancellation guard. The
/// older reference line path is unchanged. Err means numerically rejected, not
/// merely offscreen. Never silently clamp an unrepresentable vector to a border.
pub(crate) fn project_vector_segment(
    origin: PlotPoint,
    end: PlotPoint,
    transform: PlotTransform,
) -> Result<Option<ProjectedSegment>, ()> {
    let bounds = transform.bounds();
    if origin == end
        || (0..2).any(|i| {
            origin[i].max(end[i]) < bounds.min()[i] || origin[i].min(end[i]) > bounds.max()[i]
        })
    {
        return Ok(None);
    }
    let a = transform.normalized(origin);
    let b = transform.normalized(end);
    let rect = transform.rect();
    let screen_span = [f64::from(rect.width()), f64::from(rect.height())];
    // Reserve 32 ulps of intermediate error, requiring <0.01 logical point.
    // This is an explicit conservative limit, not arbitrary extreme-value LOD.
    if (0..2).any(|i| {
        !a[i].is_finite()
            || !b[i].is_finite()
            || a[i].abs().max(b[i].abs()) * f64::EPSILON * 32.0 * screen_span[i] > 0.01
    }) {
        return Err(());
    }
    let delta = [
        (b[0] - a[0]) * screen_span[0],
        -(b[1] - a[1]) * screen_span[1],
    ];
    let length = delta[0].hypot(delta[1]);
    if !length.is_finite() || length == 0.0 {
        return Err(());
    }
    let Some((start, finish)) = clip_normalized_segment(a, b) else {
        return Ok(None);
    };
    let shaft = [
        transform.position_from_normalized(start),
        transform.position_from_normalized(finish),
    ];
    if shaft.iter().any(|p| !p.is_finite()) {
        return Err(());
    }
    if shaft[0] == shaft[1] {
        return Ok(None); // No representable shaft, so no invented head or hit.
    }
    let inside = |p: PlotPoint| (0..2).all(|i| p[i] >= bounds.min()[i] && p[i] <= bounds.max()[i]);
    Ok(Some(ProjectedSegment {
        shaft,
        direction: delta.map(|d| d / length),
        length,
        origin_visible: inside(origin),
        end_visible: inside(end),
    }))
}

/// Normalized Liang–Barsky clipping, preserving input direction. Callers decide
/// which numerical domain is acceptable before reaching this reference routine.
pub(crate) fn clip_normalized_segment(
    a: PlotPoint,
    b: PlotPoint,
) -> Option<(PlotPoint, PlotPoint)> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let mut enter: f64 = 0.0;
    let mut exit: f64 = 1.0;
    for (p, q) in [
        (-d[0], a[0]),
        (d[0], 1.0 - a[0]),
        (-d[1], a[1]),
        (d[1], 1.0 - a[1]),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let ratio = q / p;
            if p < 0.0 {
                enter = enter.max(ratio);
            } else {
                exit = exit.min(ratio);
            }
            if enter > exit {
                return None;
            }
        }
    }
    let at = |t: f64| std::array::from_fn(|i| (a[i] + t * d[i]).clamp(0.0, 1.0));
    Some((at(enter), at(exit)))
}
