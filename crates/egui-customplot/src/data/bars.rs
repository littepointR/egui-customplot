use super::extents::CachedExtents;
use crate::{AxisScale, PlotPoint};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// A single unstacked bar. Width is in data units on the category-position axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub center: f64,
    pub value: f64,
    pub width: f64,
    pub baseline: f64,
}

impl Bar {
    pub fn new(center: f64, value: f64) -> Self {
        Self {
            center,
            value,
            width: 0.8,
            baseline: 0.0,
        }
    }
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }
    pub fn baseline(mut self, baseline: f64) -> Self {
        self.baseline = baseline;
        self
    }
    pub(crate) fn extent(self, orientation: BarOrientation) -> Option<(PlotPoint, PlotPoint)> {
        if !self.center.is_finite()
            || !self.value.is_finite()
            || !self.baseline.is_finite()
            || !self.width.is_finite()
            || self.width <= 0.0
        {
            return None;
        }
        let mut lo = [
            self.center - self.width * 0.5,
            self.value.min(self.baseline),
        ];
        let mut hi = [
            self.center + self.width * 0.5,
            self.value.max(self.baseline),
        ];
        if !lo[0].is_finite()
            || !hi[0].is_finite()
            || hi[0] <= lo[0]
            || !(hi[1] - lo[1]).is_finite()
        {
            return None;
        }
        if orientation == BarOrientation::Horizontal {
            lo.swap(0, 1);
            hi.swap(0, 1);
        }
        Some((lo, hi))
    }
}

/// Immutable host-owned bar records with two orientations × four domain summaries.
/// Invalid records are retained for raw indexing, omitted and diagnosed at show.
#[derive(Clone, Debug)]
pub struct BarData {
    bars: Arc<[Bar]>,
    summaries: [CachedExtents; 2],
}

impl BarData {
    pub fn new(bars: impl Into<Arc<[Bar]>>) -> Self {
        let bars = bars.into();
        let mut summaries = [CachedExtents::default(), CachedExtents::default()];
        for &bar in bars.iter() {
            summaries[0].push(bar.extent(BarOrientation::Vertical));
            summaries[1].push(bar.extent(BarOrientation::Horizontal));
        }
        Self { bars, summaries }
    }
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }
    pub fn len(&self) -> usize {
        self.bars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        self.bars.len() * std::mem::size_of::<Bar>()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.bars.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(
        &self,
        scales: [AxisScale; 2],
        orientation: BarOrientation,
    ) -> Option<(PlotPoint, PlotPoint)> {
        self.summaries[usize::from(orientation == BarOrientation::Horizontal)].extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2], orientation: BarOrientation) -> usize {
        self.summaries[usize::from(orientation == BarOrientation::Horizontal)].invalid(scales)
    }
}
