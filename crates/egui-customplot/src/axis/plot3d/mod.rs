mod camera;
mod clip;

pub use camera::{Camera3D, Projection3D, Transform3D};
pub(crate) use clip::{clip_segment_unit_cube, clip_triangle_unit_cube, intersect_plane_unit_cube};

use super::{Axis, AxisScale, AxisTick, BoundsError};

/// Immutable 3D data coordinate `[x, y, z]`.
pub type Point3D = [f64; 3];

/// Validated non-degenerate axis-aligned 3D data bounds `[x, y, z]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds3D {
    min: Point3D,
    max: Point3D,
}

impl Default for Bounds3D {
    fn default() -> Self {
        Self {
            min: [-1.0; 3],
            max: [1.0; 3],
        }
    }
}

impl Bounds3D {
    pub fn new(min: Point3D, max: Point3D) -> Result<Self, BoundsError> {
        for i in 0..3 {
            let span = max[i] - min[i];
            if !min[i].is_finite() || !max[i].is_finite() || !span.is_finite() || span <= 0.0 {
                return Err(BoundsError);
            }
        }
        Ok(Self { min, max })
    }

    pub fn from_points(min: Point3D, max: Point3D, pad_fraction: f64) -> Self {
        let mut lo = min;
        let mut hi = max;
        for i in 0..3 {
            let span = (hi[i] - lo[i]).abs();
            let pad = if span <= 1e-12 || !span.is_finite() {
                0.5
            } else {
                span * pad_fraction.max(0.0)
            };
            lo[i] -= pad;
            hi[i] += pad;
            if !lo[i].is_finite() || !hi[i].is_finite() || hi[i] <= lo[i] {
                lo[i] = -1.0;
                hi[i] = 1.0;
            }
        }
        Self { min: lo, max: hi }
    }

    pub fn min(self) -> Point3D {
        self.min
    }

    pub fn max(self) -> Point3D {
        self.max
    }

    pub fn span(self) -> Point3D {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    pub fn contains(self, p: Point3D) -> bool {
        (0..3).all(|i| p[i].is_finite() && p[i] >= self.min[i] && p[i] <= self.max[i])
    }

    pub(crate) fn validated_for_scales(
        min: Point3D,
        max: Point3D,
        scales: [AxisScale; 3],
    ) -> Result<Self, BoundsError> {
        let b = Self::new(min, max)?;
        for (i, scale) in scales.iter().enumerate() {
            let a = scale.forward(min[i]).ok_or(BoundsError)?;
            let z = scale.forward(max[i]).ok_or(BoundsError)?;
            if !a.is_finite() || !z.is_finite() || z <= a {
                return Err(BoundsError);
            }
        }
        Ok(b)
    }
}

/// 3D axis proportions (`daspect` semantics).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DataAspect3D {
    #[default]
    Auto,
    Equal,
    Manual([f64; 3]),
}

impl DataAspect3D {
    pub(crate) fn box_scales(self, bounds: Bounds3D) -> [f64; 3] {
        let span = bounds.span();
        let raw = match self {
            Self::Auto => [1.0, 1.0, 1.0],
            Self::Equal => span,
            Self::Manual(r) => {
                if r.iter().all(|v| v.is_finite() && *v > 0.0) {
                    [span[0] / r[0], span[1] / r[1], span[2] / r[2]]
                } else {
                    [1.0, 1.0, 1.0]
                }
            }
        };
        let max_comp = raw[0].max(raw[1]).max(raw[2]).max(1e-12);
        [
            (raw[0] / max_comp).clamp(0.12, 1.0),
            (raw[1] / max_comp).clamp(0.12, 1.0),
            (raw[2] / max_comp).clamp(0.12, 1.0),
        ]
    }
}

/// Configuration for one axis (`X`, `Y`, or `Z`) inside [`crate::Plot3D`].
#[derive(Clone, Debug)]
pub struct Axis3D {
    inner: Axis,
    bounds: Option<[f64; 2]>,
    grid: bool,
}

impl Default for Axis3D {
    fn default() -> Self {
        Self {
            inner: Axis::default(),
            bounds: None,
            grid: true,
        }
    }
}

impl Axis3D {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn scale(mut self, scale: AxisScale) -> Self {
        self.inner = self.inner.scale(scale);
        self
    }
    pub fn reversed(mut self, reversed: bool) -> Self {
        self.inner = self.inner.reversed(reversed);
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.label(label);
        self
    }
    pub fn max_ticks(mut self, count: usize) -> Self {
        self.inner = self.inner.max_ticks(count);
        self
    }
    pub fn visible(mut self, visible: bool) -> Self {
        self.inner = self.inner.visible(visible);
        self
    }
    pub fn tick_labels_visible(mut self, visible: bool) -> Self {
        self.inner = self.inner.tick_labels_visible(visible);
        self
    }
    pub fn grid(mut self, show: bool) -> Self {
        self.grid = show;
        self
    }
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() && max > min {
            self.bounds = Some([min, max]);
        }
        self
    }
    pub fn categories(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.inner = self.inner.categories(labels);
        self
    }
    pub fn explicit_ticks(
        mut self,
        ticks: impl IntoIterator<Item = (f64, impl Into<String>)>,
    ) -> Self {
        self.inner = self
            .inner
            .explicit_ticks(ticks.into_iter().map(|(v, s)| (v, s.into())));
        self
    }
    pub fn formatter(mut self, formatter: impl Fn(f64) -> String + Send + Sync + 'static) -> Self {
        self.inner = self.inner.formatter(formatter);
        self
    }

    pub fn axis_scale(&self) -> AxisScale {
        self.inner.scale
    }
    pub fn is_reversed(&self) -> bool {
        self.inner.reversed
    }
    pub fn axis_label(&self) -> &str {
        &self.inner.label
    }
    pub fn show_grid(&self) -> bool {
        self.grid && self.inner.visible
    }
    pub fn manual_bounds(&self) -> Option<[f64; 2]> {
        self.bounds
    }
    pub fn ticks(&self, range: [f64; 2], pixels: f32) -> Vec<AxisTick> {
        self.inner.ticks(range, pixels)
    }
}
