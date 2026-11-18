use egui::{Pos2, Rect, Vec2};

use super::{AxisScale, BoundsError, PlotBounds};

/// Linear/log coordinate mapping, keeping subtraction in f64 before screen conversion.
#[derive(Clone, Copy, Debug)]
pub struct PlotTransform {
    bounds: PlotBounds,
    axis_bounds: PlotBounds,
    scales: [AxisScale; 2],
    reversed: [bool; 2],
    rect: Rect,
    data_aspect: Option<f64>,
}

impl PlotTransform {
    pub(crate) fn with_bounds(self, bounds: PlotBounds) -> Option<Self> {
        let transform = Self::with_axes(bounds, self.rect, self.scales, self.reversed)?;
        match self.data_aspect {
            Some(aspect) => transform.with_data_aspect(aspect),
            None => Some(transform),
        }
    }
    /// Returns `None` for non-finite or empty screen rectangles.
    pub fn new(bounds: PlotBounds, rect: Rect) -> Option<Self> {
        Self::with_axes(bounds, rect, [AxisScale::Linear; 2], [false; 2])
    }

    pub fn with_axes(
        bounds: PlotBounds,
        rect: Rect,
        scales: [AxisScale; 2],
        reversed: [bool; 2],
    ) -> Option<Self> {
        if !rect.is_finite()
            || !rect.size().is_finite()
            || rect.width() <= 0.0
            || rect.height() <= 0.0
        {
            return None;
        }
        let axis_bounds = PlotBounds::new(
            [
                scales[0].forward(bounds.min[0])?,
                scales[1].forward(bounds.min[1])?,
            ],
            [
                scales[0].forward(bounds.max[0])?,
                scales[1].forward(bounds.max[1])?,
            ],
        )
        .ok()?;
        Some(Self {
            bounds,
            axis_bounds,
            scales,
            reversed,
            rect,
            data_aspect: None,
        })
    }

    /// X-unit screen length divided by Y-unit screen length. One makes units
    /// equal. Uses transformed units (decades on log axes), expands bounds around
    /// their center and never crops data. Returns None if expansion overflows.
    pub fn with_data_aspect(self, aspect: f64) -> Option<Self> {
        let axis_bounds = super::aspect::fit_aspect(self.axis_bounds, self.rect, aspect).ok()?;
        let bounds = if axis_bounds == self.axis_bounds {
            self.bounds
        } else {
            self.convert_axis_bounds(axis_bounds).ok()?
        };
        let mut result = Self::with_axes(bounds, self.rect, self.scales, self.reversed)?;
        result.data_aspect = Some(aspect);
        Some(result)
    }

    pub fn data_aspect(self) -> Option<f64> {
        self.data_aspect
    }

    pub fn scales(self) -> [AxisScale; 2] {
        self.scales
    }

    pub fn reversed(self) -> [bool; 2] {
        self.reversed
    }

    pub fn is_valid_point(self, point: [f64; 2]) -> bool {
        self.scales[0].forward(point[0]).is_some() && self.scales[1].forward(point[1]).is_some()
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

    /// Project an in-domain linear batch with the same rounding as scalar projection.
    /// Keeping the loop free of occupancy checks lets the compiler vectorize it.
    pub(crate) fn project_linear_batch(self, points: &[[f64; 2]], positions: &mut [Pos2]) {
        debug_assert_eq!(self.scales, [AxisScale::Linear; 2]);
        debug_assert_eq!(points.len(), positions.len());
        let min = self.axis_bounds.min;
        let span = self.axis_bounds.span();
        for (point, pos) in points.iter().zip(positions) {
            let x = (point[0] - min[0]) / span[0];
            let y = (point[1] - min[1]) / span[1];
            *pos = self.position_from_normalized([
                if self.reversed[0] { 1.0 - x } else { x },
                if self.reversed[1] { 1.0 - y } else { y },
            ]);
        }
    }

    pub fn point_from_position(self, position: Pos2) -> [f64; 2] {
        let span = self.axis_bounds.span();
        let mut fraction = [
            f64::from(position.x - self.rect.left()) / f64::from(self.rect.width()),
            f64::from(self.rect.bottom() - position.y) / f64::from(self.rect.height()),
        ];
        for axis in 0..2 {
            if self.reversed[axis] {
                fraction[axis] = 1.0 - fraction[axis];
            }
        }
        std::array::from_fn(|axis| {
            self.scales[axis]
                .inverse(self.axis_bounds.min[axis] + fraction[axis] * span[axis])
                .unwrap_or(f64::NAN)
        })
    }

    pub(crate) fn pan(self, delta: Vec2) -> Result<PlotBounds, BoundsError> {
        let span = self.axis_bounds.span();
        let mut delta = [
            -f64::from(delta.x) / f64::from(self.rect.width()) * span[0],
            f64::from(delta.y) / f64::from(self.rect.height()) * span[1],
        ];
        for (axis, d) in delta.iter_mut().enumerate() {
            if self.reversed[axis] {
                *d = -*d;
            }
        }
        let mut result = self.convert_axis_bounds(self.axis_bounds.translated(delta)?)?;
        for (axis, d) in delta.iter().enumerate() {
            if *d == 0.0 {
                result.min[axis] = self.bounds.min[axis];
                result.max[axis] = self.bounds.max[axis];
            }
        }
        Ok(result)
    }

    /// Zoom in transformed coordinates. A factor of one leaves that axis alone.
    pub fn zoom_at(self, position: Pos2, factors: [f64; 2]) -> Result<PlotBounds, BoundsError> {
        let anchor = self.point_from_position(position);
        let mut min = self.axis_bounds.min;
        let mut max = self.axis_bounds.max;
        for axis in 0..2 {
            if !factors[axis].is_finite() || factors[axis] <= 0.0 {
                return Err(BoundsError);
            }
            let center = self.scales[axis].forward(anchor[axis]).ok_or(BoundsError)?;
            min[axis] = center + (min[axis] - center) * factors[axis];
            max[axis] = center + (max[axis] - center) * factors[axis];
        }
        let mut result = self.convert_axis_bounds(PlotBounds::new(min, max)?)?;
        for (axis, factor) in factors.iter().enumerate() {
            if *factor == 1.0 {
                result.min[axis] = self.bounds.min[axis];
                result.max[axis] = self.bounds.max[axis];
            }
        }
        Ok(result)
    }

    fn convert_axis_bounds(self, bounds: PlotBounds) -> Result<PlotBounds, BoundsError> {
        PlotBounds::new(
            [
                self.scales[0].inverse(bounds.min[0]).ok_or(BoundsError)?,
                self.scales[1].inverse(bounds.min[1]).ok_or(BoundsError)?,
            ],
            [
                self.scales[0].inverse(bounds.max[0]).ok_or(BoundsError)?,
                self.scales[1].inverse(bounds.max[1]).ok_or(BoundsError)?,
            ],
        )
    }

    pub(crate) fn normalized(self, point: [f64; 2]) -> [f64; 2] {
        let span = self.axis_bounds.span();
        std::array::from_fn(|axis| {
            let fraction = (self.scales[axis].forward(point[axis]).unwrap_or(f64::NAN)
                - self.axis_bounds.min[axis])
                / span[axis];
            if self.reversed[axis] {
                1.0 - fraction
            } else {
                fraction
            }
        })
    }

    pub(crate) fn position_from_normalized(self, p: [f64; 2]) -> Pos2 {
        egui::pos2(
            self.rect.left() + p[0] as f32 * self.rect.width(),
            self.rect.bottom() - p[1] as f32 * self.rect.height(),
        )
    }

    #[inline]
    pub(crate) fn axis_min(self, axis: usize) -> f64 {
        self.axis_bounds.min[axis]
    }

    #[inline]
    pub(crate) fn axis_span(self, axis: usize) -> f64 {
        self.axis_bounds.span()[axis]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_batch_matches_scalar_rounding_at_cell_boundaries() {
        let origin = 1e12;
        let bounds = PlotBounds::new([origin, -1.0], [origin + 64.0, 1.0]).unwrap();
        let rect = Rect::from_min_size(egui::pos2(3.25, 7.5), egui::vec2(128.0, 57.0));
        let points: Vec<_> = (-5..135)
            .flat_map(|i| {
                let x = origin + i as f64 * 0.5;
                [x.next_down(), x, x.next_up()].map(|v| [v, (i as f64 - 18.0) / 57.0])
            })
            .collect();
        for reversed in [[false, false], [true, false], [false, true], [true, true]] {
            let t =
                PlotTransform::with_axes(bounds, rect, [AxisScale::Linear; 2], reversed).unwrap();
            let mut positions = vec![Pos2::ZERO; points.len()];
            t.project_linear_batch(&points, &mut positions);
            for (point, projected) in points.iter().zip(positions) {
                assert_eq!(projected, t.position_from_point(*point));
            }
        }
    }

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

    #[test]
    fn log_transform_roundtrips_and_respects_reversal() {
        let b = PlotBounds::new([0.01, -2.0], [1000.0, 8.0]).unwrap();
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 500.0));
        for reversed in [[false; 2], [true; 2], [true, false]] {
            let t =
                PlotTransform::with_axes(b, rect, [AxisScale::Log10, AxisScale::Linear], reversed)
                    .unwrap();
            let p = [3.0, 2.5];
            let q = t.point_from_position(t.position_from_point(p));
            for axis in 0..2 {
                assert!((p[axis] - q[axis]).abs() < 1e-5);
            }
            let min_pos = t.position_from_point(b.min());
            assert_eq!(
                min_pos.x,
                if reversed[0] {
                    rect.right()
                } else {
                    rect.left()
                }
            );
            assert_eq!(
                min_pos.y,
                if reversed[1] {
                    rect.top()
                } else {
                    rect.bottom()
                }
            );
            let anchor = t.position_from_point(p);
            let zoomed = PlotTransform::with_axes(
                t.zoom_at(anchor, [0.5, 1.0]).unwrap(),
                rect,
                t.scales(),
                reversed,
            )
            .unwrap();
            assert!((zoomed.position_from_point(p) - anchor).length() < 0.001);
            assert_eq!(zoomed.bounds().min()[1], b.min()[1]);
            assert_eq!(zoomed.bounds().max()[1], b.max()[1]);
            assert!(!t.is_valid_point([0.0, 1.0]));
        }
        assert!(
            PlotTransform::with_axes(
                PlotBounds::default(),
                rect,
                [AxisScale::Log10; 2],
                [false; 2]
            )
            .is_none()
        );
    }

    #[test]
    fn logarithmic_pan_is_multiplicative_and_reversed_pan_flips_direction() {
        let bounds = PlotBounds::new([1.0, 1.0], [100.0, 100.0]).unwrap();
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0));
        for reversed in [false, true] {
            let t = PlotTransform::with_axes(bounds, rect, [AxisScale::Log10; 2], [reversed; 2])
                .unwrap();
            let panned = t.pan(egui::vec2(50.0, 0.0)).unwrap();
            let expected = if reversed { 10.0 } else { 0.1 };
            assert!((panned.min()[0] - expected).abs() < 1e-10);
            assert!((panned.max()[0] / panned.min()[0] - 100.0).abs() < 1e-10);
            assert_eq!(panned.min()[1], 1.0);
            assert_eq!(panned.max()[1], 100.0);
        }
    }
}
