use super::extents::CachedExtents;
use crate::{AxisScale, PlotPoint};
use std::sync::Arc;

/// Asymmetric nonnegative offsets, not absolute endpoints or confidence levels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErrorBar {
    pub point: PlotPoint,
    pub x_error: [f64; 2],
    pub y_error: [f64; 2],
}

impl ErrorBar {
    pub fn new(point: PlotPoint) -> Self {
        Self {
            point,
            x_error: [0.0; 2],
            y_error: [0.0; 2],
        }
    }
    pub fn x_error(mut self, lower_upper: [f64; 2]) -> Self {
        self.x_error = lower_upper;
        self
    }
    pub fn y_error(mut self, lower_upper: [f64; 2]) -> Self {
        self.y_error = lower_upper;
        self
    }
    pub(crate) fn extent(self) -> Option<(PlotPoint, PlotPoint)> {
        if self.point.iter().any(|v| !v.is_finite())
            || self
                .x_error
                .iter()
                .chain(&self.y_error)
                .any(|v| !v.is_finite() || *v < 0.0)
        {
            return None;
        }
        let lo = [
            self.point[0] - self.x_error[0],
            self.point[1] - self.y_error[0],
        ];
        let hi = [
            self.point[0] + self.x_error[1],
            self.point[1] + self.y_error[1],
        ];
        (0..2)
            .all(|i| lo[i].is_finite() && hi[i].is_finite() && (hi[i] - lo[i]).is_finite())
            .then_some((lo, hi))
    }
    pub(crate) fn has_error(self) -> bool {
        self.x_error.iter().chain(&self.y_error).any(|v| *v > 0.0)
    }
}

#[derive(Clone, Debug)]
pub struct ErrorBarData {
    bars: Arc<[ErrorBar]>,
    summary: CachedExtents,
}

impl ErrorBarData {
    pub fn new(bars: impl Into<Arc<[ErrorBar]>>) -> Self {
        let bars = bars.into();
        let mut summary = CachedExtents::default();
        for &bar in bars.iter() {
            summary.push(bar.extent());
        }
        Self { bars, summary }
    }
    pub fn bars(&self) -> &[ErrorBar] {
        &self.bars
    }
    pub fn len(&self) -> usize {
        self.bars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        self.bars.len() * std::mem::size_of::<ErrorBar>()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.bars.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.summary.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.summary.invalid(scales)
    }
}
