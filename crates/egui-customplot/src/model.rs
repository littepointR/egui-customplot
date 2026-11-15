use std::sync::Arc;

use egui::{Color32, Stroke};

use crate::{BoundsError, PlotBounds};

pub type PlotPoint = [f64; 2];

/// Immutable shared XY data. Cloning shares samples; construction scans once.
/// A point with either non-finite component breaks the polyline.
#[derive(Clone, Debug)]
pub struct PlotData {
    points: Arc<[PlotPoint]>,
    pub(crate) extent: Option<(PlotPoint, PlotPoint)>,
    non_finite: usize,
}

impl PlotData {
    pub fn new(points: impl Into<Arc<[PlotPoint]>>) -> Self {
        let points = points.into();
        let mut extent: Option<(PlotPoint, PlotPoint)> = None;
        let mut non_finite = 0;
        for p in points.iter() {
            if !p[0].is_finite() || !p[1].is_finite() {
                non_finite += 1;
                continue;
            }
            let (min, max) = extent.get_or_insert((*p, *p));
            for axis in 0..2 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        Self {
            points,
            extent,
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
        self.extent
            .map(|(min, max)| PlotBounds::fit(min, max))
            .transpose()
    }

    pub(crate) fn identity(&self) -> *const PlotPoint {
        self.points.as_ptr()
    }
}

/// A borrowed series description; samples remain in the application model.
#[derive(Clone, Copy, Debug)]
pub struct Line<'a> {
    pub(crate) data: &'a PlotData,
    pub(crate) stroke: Option<Stroke>,
}

impl<'a> Line<'a> {
    pub fn new(data: &'a PlotData) -> Self {
        Self { data, stroke: None }
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }
    pub fn color(self, color: Color32) -> Self {
        self.stroke(Stroke::new(self.stroke.map_or(1.5, |s| s.width), color))
    }
    pub fn width(self, width: f32) -> Self {
        self.stroke(Stroke::new(
            width,
            self.stroke.map_or(Color32::LIGHT_BLUE, |s| s.color),
        ))
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
}
