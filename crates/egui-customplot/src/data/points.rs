use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use super::{LineIndexError, index::LineIndex};
use crate::{AxisScale, BoundsError, PlotBounds};

pub type PlotPoint = [f64; 2];

static NEXT_VERSION: AtomicU64 = AtomicU64::new(1);

/// Minimum sample count at which [`PlotData::new`] automatically constructs a
/// monotonic-X tree index (within the default 64 MiB budget).
pub const AUTO_INDEX_MIN_POINTS: usize = 8_192;
const AUTO_INDEX_MAX_BYTES: usize = 64 * 1024 * 1024;

/// Immutable shared XY data. Cloning shares samples; construction scans once.
/// When `len >= AUTO_INDEX_MIN_POINTS` and finite runs are X-nondecreasing,
/// [`PlotData::new`] automatically builds an immutable index within 64 MiB so
/// default lines and raw picking run in sub-millisecond time.
///
/// 不可变共享二维 XY 数据源：克隆零拷贝共享样本，构建时单次扫描极值，
/// 并在单调非降 X 序列达到 [`AUTO_INDEX_MIN_POINTS`] 时自动建立 64 MiB 有界索引以实现亚毫秒 LOD 与拾取。
#[derive(Clone, Debug)]
pub struct PlotData {
    points: Arc<[PlotPoint]>,
    extents: [Option<(PlotPoint, PlotPoint)>; 4],
    valid_counts: [usize; 4],
    non_finite: usize,
    line_index: Option<Arc<LineIndex>>,
    version: u64,
}

impl PlotData {
    pub fn new(points: impl Into<Arc<[PlotPoint]>>) -> Self {
        Self::from_arc(points.into(), Some(AUTO_INDEX_MAX_BYTES))
    }

    /// Construct `PlotData` without building an automatic monotonic-X index.
    pub fn new_unindexed(points: impl Into<Arc<[PlotPoint]>>) -> Self {
        Self::from_arc(points.into(), None)
    }

    /// Construct `PlotData` and build a monotonic-X index bounded by `budget_bytes`.
    pub fn with_index_budget(
        points: impl Into<Arc<[PlotPoint]>>,
        budget_bytes: usize,
    ) -> Result<Self, LineIndexError> {
        let mut data = Self::from_arc(points.into(), None);
        data.build_line_index(budget_bytes)?;
        Ok(data)
    }

    fn from_arc(points: Arc<[PlotPoint]>, auto_index_budget: Option<usize>) -> Self {
        let mut extents: [Option<(PlotPoint, PlotPoint)>; 4] = [None; 4];
        let mut valid_counts = [0; 4];
        let mut non_finite = 0;
        let mut monotonic_x = true;
        let mut previous_x: Option<f64> = None;
        for p in points.iter() {
            if !p[0].is_finite() || !p[1].is_finite() {
                non_finite += 1;
                previous_x = None;
                continue;
            }
            if monotonic_x && previous_x.is_some_and(|x| p[0] < x) {
                monotonic_x = false;
            }
            previous_x = Some(p[0]);
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
        let line_index = auto_index_budget
            .filter(|_| monotonic_x && points.len() >= AUTO_INDEX_MIN_POINTS)
            .and_then(|budget| LineIndex::build(&points, budget).ok())
            .map(Arc::new);
        Self {
            points,
            extents,
            valid_counts,
            non_finite,
            line_index,
            version: next_version(),
        }
    }

    pub fn points(&self) -> &[PlotPoint] {
        &self.points
    }
    /// Process-local immutable snapshot version. Clones share it; constructing
    /// another PlotData creates a new version even from the same Arc samples.
    pub fn version(&self) -> u64 {
        self.version
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
    /// Explicit preprocessing, outside UI. Requires nondecreasing X within each
    /// finite XY run; equal X is allowed, nonfinite points break runs. The budget
    /// covers this index's run/tree payload and metadata, not samples, allocator
    /// headers, other snapshots or temporary rendering. Failure preserves the
    /// existing index. Clones share a successfully built immutable index.
    pub fn build_line_index(&mut self, budget_bytes: usize) -> Result<(), LineIndexError> {
        if let Some(index) = &self.line_index {
            return if index.bytes() <= budget_bytes {
                Ok(())
            } else {
                Err(LineIndexError::BudgetExceeded {
                    required_bytes: index.bytes(),
                    budget_bytes,
                })
            };
        }
        let index = LineIndex::build(self.points(), budget_bytes)?;
        self.line_index = Some(Arc::new(index));
        Ok(())
    }

    pub fn clear_line_index(&mut self) {
        self.line_index = None;
    }

    pub fn line_index_bytes(&self) -> usize {
        self.line_index.as_ref().map_or(0, |i| i.bytes())
    }

    pub(crate) fn line_index(&self) -> Option<&LineIndex> {
        self.line_index.as_deref()
    }

    pub(crate) fn index_identity(&self) -> Option<*const LineIndex> {
        self.line_index.as_ref().map(Arc::as_ptr)
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
        if !scales[0].is_probability() && !scales[1].is_probability() {
            return self.extents[Self::scale_mask(scales)];
        }
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut any = false;
        for &p in self.points.iter() {
            if scales[0].forward(p[0]).is_some() && scales[1].forward(p[1]).is_some() {
                min[0] = min[0].min(p[0]);
                min[1] = min[1].min(p[1]);
                max[0] = max[0].max(p[0]);
                max[1] = max[1].max(p[1]);
                any = true;
            }
        }
        any.then_some((min, max))
    }

    pub fn invalid_points(&self, scales: [AxisScale; 2]) -> usize {
        if !scales[0].is_probability() && !scales[1].is_probability() {
            return self.len() - self.valid_counts[Self::scale_mask(scales)];
        }
        self.points
            .iter()
            .filter(|p| scales[0].forward(p[0]).is_none() || scales[1].forward(p[1]).is_none())
            .count()
    }

    fn scale_mask(scales: [AxisScale; 2]) -> usize {
        usize::from(scales[0] == AxisScale::Log10)
            | (usize::from(scales[1] == AxisScale::Log10) << 1)
    }

    pub(crate) fn identity(&self) -> *const PlotPoint {
        self.points.as_ptr()
    }

    /// Collects sorted point indices whose `(x, y)` coordinates fall inside `bounds`.
    /// Uses the monotonic-X index when available to avoid scanning points outside X bounds.
    pub fn brushed_indices(&self, bounds: PlotBounds) -> Vec<usize> {
        let [lo, hi] = [bounds.min(), bounds.max()];
        let mut indices = Vec::new();
        if let Some(index) = self.line_index() {
            let mut stats = super::QueryStats::default();
            let ranges = index.candidates(
                &self.points,
                [AxisScale::Linear; 2],
                [lo[0], hi[0]],
                &mut stats,
            );
            for range in ranges {
                for (offset, p) in self.points[range.clone()].iter().enumerate() {
                    if p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1] {
                        indices.push(range.start + offset);
                    }
                }
            }
            return indices;
        }
        for (i, p) in self.points.iter().enumerate() {
            if p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1] {
                indices.push(i);
            }
        }
        indices
    }
}

pub(crate) fn next_version() -> u64 {
    let mut current = NEXT_VERSION.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .expect("PlotData snapshot version exhausted");
        match NEXT_VERSION.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(version) => return version,
            Err(version) => current = version,
        }
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
    fn index_is_released_by_last_owner_and_does_not_retain_samples() {
        let points: Arc<[PlotPoint]> = vec![[1.0; 2], [2.0; 2]].into();
        let weak_points = Arc::downgrade(&points);
        let mut data = PlotData::new(points);
        data.build_line_index(usize::MAX).unwrap();
        let weak_index = Arc::downgrade(data.line_index.as_ref().unwrap());
        let mut clone = data.clone();
        data.clear_line_index();
        assert!(weak_index.upgrade().is_some());
        clone.clear_line_index();
        assert!(weak_index.upgrade().is_none());
        drop(data);
        drop(clone);
        assert!(weak_points.upgrade().is_none());
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
