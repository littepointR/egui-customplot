use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, PlotPoint};

/// One independent data-space vector. No implicit normalization or autoscaling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuiverArrow {
    pub origin: PlotPoint,
    pub vector: PlotPoint,
}

impl QuiverArrow {
    pub fn new(origin: PlotPoint, vector: PlotPoint) -> Self {
        Self { origin, vector }
    }
    /// True endpoint, or None for nonfinite inputs or addition overflow.
    pub fn endpoint(self) -> Option<PlotPoint> {
        if self
            .origin
            .iter()
            .chain(&self.vector)
            .any(|v| !v.is_finite())
        {
            return None;
        }
        let end = std::array::from_fn(|i| self.origin[i] + self.vector[i]);
        end.iter().all(|v| v.is_finite()).then_some(end)
    }
    pub(crate) fn extent(self) -> Option<(PlotPoint, PlotPoint)> {
        let end = self.endpoint()?;
        Some((
            std::array::from_fn(|i| self.origin[i].min(end[i])),
            std::array::from_fn(|i| self.origin[i].max(end[i])),
        ))
    }
}

/// Immutable shared records with four joint-coordinate-domain extent summaries.
/// Construct outside the per-pass UI closure. Clones share the raw Arc payload.
#[derive(Clone, Debug)]
pub struct QuiverData {
    arrows: Arc<[QuiverArrow]>,
    summary: CachedExtents,
}

impl QuiverData {
    pub fn new(arrows: impl Into<Arc<[QuiverArrow]>>) -> Self {
        let arrows = arrows.into();
        let mut summary = CachedExtents::default();
        for &arrow in arrows.iter() {
            summary.push(arrow.extent());
        }
        Self { arrows, summary }
    }
    pub fn arrows(&self) -> &[QuiverArrow] {
        &self.arrows
    }
    pub fn len(&self) -> usize {
        self.arrows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.arrows.is_empty()
    }
    /// Raw records only, excluding Arc headers, summaries and render geometry.
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.arrows())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.arrows.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.summary.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.summary.invalid(scales)
    }
}
