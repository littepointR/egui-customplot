use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, PlotPoint};

/// Data-space rectangle / rounded rectangle / ellipse (`curvature` in `[0, 1]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub curvature: [f64; 2],
}

impl DataRect {
    pub fn new(origin: [f64; 2], size: [f64; 2]) -> Self {
        Self {
            x: origin[0],
            y: origin[1],
            width: size[0],
            height: size[1],
            curvature: [0.0, 0.0],
        }
    }

    pub fn ellipse(center: [f64; 2], radii: [f64; 2]) -> Self {
        Self {
            x: center[0] - radii[0],
            y: center[1] - radii[1],
            width: radii[0] * 2.0,
            height: radii[1] * 2.0,
            curvature: [1.0, 1.0],
        }
    }

    pub fn curvature(mut self, curvature: [f64; 2]) -> Self {
        self.curvature = curvature;
        self
    }

    pub(crate) fn extent(self) -> Option<(PlotPoint, PlotPoint)> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.width.is_finite()
            || !self.height.is_finite()
            || self.width <= 0.0
            || self.height <= 0.0
            || self
                .curvature
                .iter()
                .any(|c| !c.is_finite() || *c < 0.0 || *c > 1.0)
        {
            return None;
        }
        let max_x = self.x + self.width;
        let max_y = self.y + self.height;
        if !max_x.is_finite() || !max_y.is_finite() || max_x <= self.x || max_y <= self.y {
            return None;
        }
        Some(([self.x, self.y], [max_x, max_y]))
    }

    pub(crate) fn contains_data(self, point: PlotPoint) -> bool {
        let Some((lo, hi)) = self.extent() else {
            return false;
        };
        if point[0] < lo[0] || point[0] > hi[0] || point[1] < lo[1] || point[1] > hi[1] {
            return false;
        }
        let rx = 0.5 * self.curvature[0] * self.width;
        let ry = 0.5 * self.curvature[1] * self.height;
        if rx <= 0.0 || ry <= 0.0 {
            return true;
        }
        let dx = if point[0] < lo[0] + rx {
            (lo[0] + rx) - point[0]
        } else if point[0] > hi[0] - rx {
            point[0] - (hi[0] - rx)
        } else {
            0.0
        };
        let dy = if point[1] < lo[1] + ry {
            (lo[1] + ry) - point[1]
        } else if point[1] > hi[1] - ry {
            point[1] - (hi[1] - ry)
        } else {
            0.0
        };
        if dx == 0.0 || dy == 0.0 {
            return true;
        }
        (dx / rx).powi(2) + (dy / ry).powi(2) <= 1.0
    }

    /// Generate closed CCW data-space perimeter points for tessellation & stroke.
    pub(crate) fn outline(self) -> Vec<PlotPoint> {
        let Some((lo, hi)) = self.extent() else {
            return Vec::new();
        };
        let rx = 0.5 * self.curvature[0] * self.width;
        let ry = 0.5 * self.curvature[1] * self.height;
        if rx <= 0.0 || ry <= 0.0 {
            return vec![lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]];
        }
        const ARC_STEPS: usize = 16;
        let corners = [
            ([hi[0] - rx, lo[1] + ry], -std::f64::consts::FRAC_PI_2, 0.0),
            ([hi[0] - rx, hi[1] - ry], 0.0, std::f64::consts::FRAC_PI_2),
            (
                [lo[0] + rx, hi[1] - ry],
                std::f64::consts::FRAC_PI_2,
                std::f64::consts::PI,
            ),
            (
                [lo[0] + rx, lo[1] + ry],
                std::f64::consts::PI,
                std::f64::consts::PI * 1.5,
            ),
        ];
        let mut pts = Vec::with_capacity(ARC_STEPS * 4 + 4);
        for (center, a0, a1) in corners {
            for s in 0..=ARC_STEPS {
                let t = s as f64 / ARC_STEPS as f64;
                let a = a0 + (a1 - a0) * t;
                let p = [center[0] + rx * a.cos(), center[1] + ry * a.sin()];
                if pts.last().is_none_or(|prev| *prev != p) {
                    pts.push(p);
                }
            }
        }
        if pts.len() > 1 && pts[0] == *pts.last().unwrap() {
            pts.pop();
        }
        pts
    }
}

/// Immutable host-owned data-space rectangles / rounded rectangles / ellipses.
#[derive(Clone, Debug)]
pub struct RectData {
    rects: Arc<[DataRect]>,
    extents: CachedExtents,
}

impl RectData {
    pub fn new(rects: impl Into<Arc<[DataRect]>>) -> Self {
        let rects = rects.into();
        let mut extents = CachedExtents::default();
        for &rect in rects.iter() {
            extents.push(rect.extent());
        }
        Self { rects, extents }
    }

    pub fn rects(&self) -> &[DataRect] {
        &self.rects
    }
    pub fn len(&self) -> usize {
        self.rects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        self.rects.len() * std::mem::size_of::<DataRect>()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.rects.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }
}
