use std::sync::Arc;

use super::radial_flow::ModernError;
use crate::{AxisScale, PlotBounds};

#[derive(Clone, Debug)]
pub struct GanttTask {
    pub label: String,
    pub start: f64,
    pub end: f64,
    pub progress: f64,
    pub group: usize,
    pub milestone: bool,
}

/// Schedule & dependency Gantt chart (`GanttChart`, `EXT-26`), compatible with `Axis::datetime` / `Axis::duration`.
#[derive(Clone, Debug)]
pub struct GanttData {
    tasks: Arc<[GanttTask]>,
    dependencies: Arc<[[usize; 2]]>,
}

impl GanttData {
    pub fn new(tasks: Vec<GanttTask>, dependencies: Vec<[usize; 2]>) -> Result<Self, ModernError> {
        if tasks.is_empty() {
            return Err(ModernError::Empty);
        }
        for t in &tasks {
            if !t.start.is_finite() || !t.end.is_finite() || !t.progress.is_finite() {
                return Err(ModernError::NonFinite);
            }
            if t.start > t.end || !(0.0..=1.0).contains(&t.progress) {
                return Err(ModernError::InvalidInterval);
            }
        }
        for &[a, b] in &dependencies {
            if a >= tasks.len() || b >= tasks.len() || a == b {
                return Err(ModernError::InvalidNode);
            }
        }
        Ok(Self {
            tasks: tasks.into(),
            dependencies: dependencies.into(),
        })
    }

    pub fn tasks(&self) -> &[GanttTask] {
        &self.tasks
    }
    pub fn dependencies(&self) -> &[[usize; 2]] {
        &self.dependencies
    }
    pub fn y_ticks(&self) -> Vec<(f64, String)> {
        self.tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (i as f64 + 1.0, t.label.clone()))
            .collect()
    }
    pub fn sample_bytes(&self) -> usize {
        self.tasks.len() * std::mem::size_of::<GanttTask>() + self.dependencies.len() * 16
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY, 0.5];
        let mut max = [f64::NEG_INFINITY, self.tasks.len() as f64 + 0.5];
        for t in self.tasks.iter() {
            if scales[0].forward(t.start).is_none() || scales[0].forward(t.end).is_none() {
                continue;
            }
            min[0] = min[0].min(t.start);
            max[0] = max[0].max(t.end);
        }
        (min[0].is_finite()
            && scales[1].forward(min[1]).is_some()
            && scales[1].forward(max[1]).is_some())
        .then(|| PlotBounds::new(min, max).ok())
        .flatten()
    }
}
