use std::{error::Error, fmt};

use egui::{Pos2, Rect, Vec2};

/// Invalid, degenerate, non-finite, or unrepresentable linear coordinate range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundsError;

impl fmt::Display for BoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounds must have finite endpoints and finite, positive spans")
    }
}

impl Error for BoundsError {}

/// A validated, nondegenerate rectangle in f64 data coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotBounds {
    min: [f64; 2],
    max: [f64; 2],
}

impl Default for PlotBounds {
    fn default() -> Self {
        Self {
            min: [-1.0; 2],
            max: [1.0; 2],
        }
    }
}

impl PlotBounds {
    pub fn new(min: [f64; 2], max: [f64; 2]) -> Result<Self, BoundsError> {
        for axis in 0..2 {
            let span = max[axis] - min[axis];
            if !min[axis].is_finite() || !max[axis].is_finite() || !span.is_finite() || span <= 0.0
            {
                return Err(BoundsError);
            }
        }
        Ok(Self { min, max })
    }

    pub fn min(self) -> [f64; 2] {
        self.min
    }
    pub fn max(self) -> [f64; 2] {
        self.max
    }
    pub fn span(self) -> [f64; 2] {
        [self.max[0] - self.min[0], self.max[1] - self.min[1]]
    }

    pub(crate) fn fit(min: [f64; 2], max: [f64; 2]) -> Result<Self, BoundsError> {
        let mut lo = min;
        let mut hi = max;
        for axis in 0..2 {
            let span = hi[axis] - lo[axis];
            let pad = if span == 0.0 {
                (lo[axis].abs() * 0.05).max(0.5)
            } else {
                span * 0.05
            };
            // Padding is best-effort; don't overflow otherwise valid data.
            let padded_lo = lo[axis] - pad;
            let padded_hi = hi[axis] + pad;
            if padded_lo.is_finite() && padded_hi.is_finite() && (padded_hi - padded_lo).is_finite()
            {
                lo[axis] = padded_lo;
                hi[axis] = padded_hi;
            }
        }
        Self::new(lo, hi)
    }

    /// Translate in data coordinates. Invalid/overflowing navigation is rejected.
    pub fn translated(self, delta: [f64; 2]) -> Result<Self, BoundsError> {
        Self::new(
            [self.min[0] + delta[0], self.min[1] + delta[1]],
            [self.max[0] + delta[0], self.max[1] + delta[1]],
        )
    }

    /// Scale the spans around an anchor; factors below one zoom in.
    pub fn zoomed(self, anchor: [f64; 2], factor: f64) -> Result<Self, BoundsError> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(BoundsError);
        }
        let mut min = self.min;
        let mut max = self.max;
        for axis in 0..2 {
            min[axis] = anchor[axis] + (min[axis] - anchor[axis]) * factor;
            max[axis] = anchor[axis] + (max[axis] - anchor[axis]) * factor;
        }
        Self::new(min, max)
    }
}

/// Linear coordinate mapping, keeping subtraction in f64 before screen conversion.
#[derive(Clone, Copy, Debug)]
pub struct PlotTransform {
    bounds: PlotBounds,
    rect: Rect,
}

impl PlotTransform {
    /// Returns `None` for non-finite or empty screen rectangles.
    pub fn new(bounds: PlotBounds, rect: Rect) -> Option<Self> {
        (rect.is_finite() && rect.size().is_finite() && rect.width() > 0.0 && rect.height() > 0.0)
            .then_some(Self { bounds, rect })
    }

    pub fn bounds(self) -> PlotBounds {
        self.bounds
    }
    pub fn rect(self) -> Rect {
        self.rect
    }

    pub fn position_from_point(self, point: [f64; 2]) -> Pos2 {
        let p = self.normalized(point);
        self.position_from_normalized(p)
    }

    pub fn point_from_position(self, position: Pos2) -> [f64; 2] {
        let span = self.bounds.span();
        [
            self.bounds.min[0]
                + f64::from(position.x - self.rect.left()) / f64::from(self.rect.width()) * span[0],
            self.bounds.min[1]
                + f64::from(self.rect.bottom() - position.y) / f64::from(self.rect.height())
                    * span[1],
        ]
    }

    pub(crate) fn pan(self, delta: Vec2) -> Result<PlotBounds, BoundsError> {
        let span = self.bounds.span();
        self.bounds.translated([
            -f64::from(delta.x) / f64::from(self.rect.width()) * span[0],
            f64::from(delta.y) / f64::from(self.rect.height()) * span[1],
        ])
    }

    pub(crate) fn normalized(self, point: [f64; 2]) -> [f64; 2] {
        let span = self.bounds.span();
        [
            (point[0] - self.bounds.min[0]) / span[0],
            (point[1] - self.bounds.min[1]) / span[1],
        ]
    }

    pub(crate) fn position_from_normalized(self, p: [f64; 2]) -> Pos2 {
        egui::pos2(
            self.rect.left() + p[0] as f32 * self.rect.width(),
            self.rect.bottom() - p[1] as f32 * self.rect.height(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_bounds() {
        for (min, max) in [
            ([0.0; 2], [0.0; 2]),
            ([f64::NAN, 0.0], [1.0; 2]),
            ([-f64::MAX; 2], [f64::MAX; 2]),
            ([1.0; 2], [0.0; 2]),
        ] {
            assert!(PlotBounds::new(min, max).is_err());
        }
    }

    #[test]
    fn mapping_roundtrip_preserves_large_local_origin() {
        let bounds = PlotBounds::new([1e12, -2.0], [1e12 + 1.0, 2.0]).unwrap();
        let t = PlotTransform::new(
            bounds,
            Rect::from_min_size(egui::pos2(20.0, 10.0), egui::vec2(800.0, 400.0)),
        )
        .unwrap();
        for p in [[1e12, -2.0], [1e12 + 0.25, 1.0], [1e12 + 1.0, 2.0]] {
            assert_eq!(t.point_from_position(t.position_from_point(p)), p);
        }
        assert_eq!(t.position_from_point(bounds.min()), t.rect().left_bottom());
    }

    #[test]
    fn zoom_keeps_anchor_and_pan_has_correct_direction() {
        let b = PlotBounds::new([0.0; 2], [10.0; 2]).unwrap();
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0));
        let t = PlotTransform::new(b, rect).unwrap();
        let anchor = [2.0, 7.0];
        let zoomed = PlotTransform::new(b.zoomed(anchor, 0.5).unwrap(), rect).unwrap();
        assert_eq!(
            zoomed.position_from_point(anchor),
            t.position_from_point(anchor)
        );
        assert_eq!(t.pan(egui::vec2(10.0, 10.0)).unwrap().min(), [-1.0, 1.0]);
        assert!(b.zoomed(anchor, 0.0).is_err());
    }
}
