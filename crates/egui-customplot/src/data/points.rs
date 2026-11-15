use std::sync::Arc;

use crate::{AxisScale, BoundsError, PlotBounds};

pub type PlotPoint = [f64; 2];

/// Immutable shared XY data. Cloning shares samples; construction scans once.
/// A point with either non-finite component breaks the polyline.
#[derive(Clone, Debug)]
pub struct PlotData {
    points: Arc<[PlotPoint]>,
    extents: [Option<(PlotPoint, PlotPoint)>; 4],
    valid_counts: [usize; 4],
    non_finite: usize,
}

impl PlotData {
    pub fn new(points: impl Into<Arc<[PlotPoint]>>) -> Self {
        let points = points.into();
        let mut extents: [Option<(PlotPoint, PlotPoint)>; 4] = [None; 4];
        let mut valid_counts = [0; 4];
        let mut non_finite = 0;
        for p in points.iter() {
            if !p[0].is_finite() || !p[1].is_finite() {
                non_finite += 1;
                continue;
            }
            for mask in 0..4 {
                if (mask & 1 != 0 && p[0] <= 0.0) || (mask & 2 != 0 && p[1] <= 0.0) {
                    continue;
                }
                valid_counts[mask] += 1;
                let (min, max) = extents[mask].get_or_insert((*p, *p));
                for axis in 0..2 {
                    min[axis] = min[axis].min(p[axis]);
                    max[axis] = max[axis].max(p[axis]);
                }
            }
        }
        Self {
            points,
            extents,
            valid_counts,
            non_finite,
        }
    }

    pub fn points(&self) -> &[PlotPoint] {
        &self.points
    }
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    pub fn non_finite_points(&self) -> usize {
        self.non_finite
    }
    /// Sample payload only, excluding Arc headers, allocator, derived/GPU data.
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.points())
    }
    pub fn fitted_bounds(&self) -> Result<Option<PlotBounds>, BoundsError> {
        self.fitted_bounds_with_scales([AxisScale::Linear; 2])
    }

    pub fn fitted_bounds_with_scales(
        &self,
        scales: [AxisScale; 2],
    ) -> Result<Option<PlotBounds>, BoundsError> {
        self.extent(scales)
            .map(|(min, max)| PlotBounds::fit_scaled(min, max, scales))
            .transpose()
    }

    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents[Self::scale_mask(scales)]
    }

    pub fn invalid_points(&self, scales: [AxisScale; 2]) -> usize {
        self.len() - self.valid_counts[Self::scale_mask(scales)]
    }

    fn scale_mask(scales: [AxisScale; 2]) -> usize {
        usize::from(scales[0] == AxisScale::Log10)
            | (usize::from(scales[1] == AxisScale::Log10) << 1)
    }

    pub(crate) fn identity(&self) -> *const PlotPoint {
        self.points.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_bounds_are_cached_and_clones_share_samples() {
        let d = PlotData::new(vec![
            [2.0, 5.0],
            [f64::NAN, 100.0],
            [f64::INFINITY, 0.0],
            [4.0, 5.0],
        ]);
        let b = d.fitted_bounds().unwrap().unwrap();
        assert!(b.min()[0] < 2.0 && b.max()[0] > 4.0);
        assert!(b.min()[1] < 5.0 && b.max()[1] > 5.0);
        assert_eq!(d.non_finite_points(), 2);
        assert_eq!(d.sample_bytes(), 4 * 16);
        assert_eq!(d.identity(), d.clone().identity());
    }

    #[test]
    fn empty_and_constant_data() {
        assert_eq!(PlotData::new(Vec::new()).fitted_bounds(), Ok(None));
        assert_eq!(PlotData::new(vec![[f64::NAN; 2]]).fitted_bounds(), Ok(None));
        assert!(
            PlotData::new(vec![[0.0; 2]])
                .fitted_bounds()
                .unwrap()
                .is_some()
        );
        assert!(
            PlotData::new(vec![[-f64::MAX; 2], [f64::MAX; 2]])
                .fitted_bounds()
                .is_err()
        );
    }

    #[test]
    fn cached_log_bounds_exclude_invalid_pairs_not_independent_components() {
        let data = PlotData::new(vec![
            [-10.0, 100.0],
            [100.0, -10.0],
            [2.0, 3.0],
            [20.0, 30.0],
            [f64::NAN, 1.0],
        ]);
        assert_eq!(data.invalid_points([AxisScale::Log10; 2]), 3);
        assert_eq!(
            data.extent([AxisScale::Log10; 2]),
            Some(([2.0, 3.0], [20.0, 30.0]))
        );
        let bounds = data
            .fitted_bounds_with_scales([AxisScale::Log10; 2])
            .unwrap()
            .unwrap();
        assert!(bounds.min()[0] > 0.0 && bounds.min()[0] < 2.0);
        assert!(bounds.max()[0] > 20.0 && bounds.max()[0] < 100.0);
        assert!(
            PlotData::new(vec![[-1.0; 2]])
                .fitted_bounds_with_scales([AxisScale::Log10; 2])
                .unwrap()
                .is_none()
        );
    }
}
