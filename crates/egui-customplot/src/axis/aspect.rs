use egui::Rect;

use super::{BoundsError, PlotBounds};

/// Expand, never crop, transformed bounds to the requested X/Y screen unit ratio.
pub(super) fn fit_aspect(
    bounds: PlotBounds,
    rect: Rect,
    aspect: f64,
) -> Result<PlotBounds, BoundsError> {
    if !aspect.is_finite() || aspect <= 0.0 {
        return Err(BoundsError);
    }
    let spans = bounds.span();
    let target = f64::from(rect.width()) / f64::from(rect.height()) / aspect;
    if !target.is_finite() || target <= 0.0 {
        return Err(BoundsError);
    }
    let current = spans[0] / spans[1];
    if (current / target - 1.0).abs() <= 1e-12 {
        return Ok(bounds);
    }
    let (axis, span) = if current < target {
        (0, spans[1] * target)
    } else {
        (1, spans[0] / target)
    };
    if !span.is_finite() || span <= 0.0 {
        return Err(BoundsError);
    }
    let center = bounds.min[axis] + spans[axis] * 0.5;
    let mut min = bounds.min;
    let mut max = bounds.max;
    min[axis] = center - span * 0.5;
    max[axis] = center + span * 0.5;
    PlotBounds::new(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_only_the_short_dimension_and_is_idempotent() {
        let bounds = PlotBounds::new([-1.0; 2], [1.0; 2]).unwrap();
        for size in [egui::vec2(800.0, 400.0), egui::vec2(200.0, 600.0)] {
            let rect = Rect::from_min_size(egui::Pos2::ZERO, size);
            let fitted = fit_aspect(bounds, rect, 1.0).unwrap();
            assert_eq!(fit_aspect(fitted, rect, 1.0).unwrap(), fitted);
            for axis in 0..2 {
                assert!(fitted.min()[axis] <= bounds.min()[axis]);
                assert!(fitted.max()[axis] >= bounds.max()[axis]);
            }
            assert!(
                (fitted.span()[0] / fitted.span()[1] - f64::from(size.x / size.y)).abs() < 1e-6
            );
        }
    }

    #[test]
    fn rejects_invalid_or_unrepresentable_aspect() {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 400.0));
        for ratio in [0.0, -1.0, f64::INFINITY, f64::NAN, f64::from_bits(1)] {
            assert!(fit_aspect(PlotBounds::default(), rect, ratio).is_err());
        }
    }
}
