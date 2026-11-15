use std::{collections::HashMap, sync::Arc};

use super::MultivariateError;
use crate::{AxisScale, GridData, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAggregation {
    Count,
    Mean,
    Sum,
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    None,
    Ascending,
    Descending,
}

/// Categorical matrix/table-aggregated heatmap backed by [`GridData`] (`ML-031`).
#[derive(Clone, Debug)]
pub struct CategoricalHeatmapData {
    x_categories: Arc<[String]>,
    y_categories: Arc<[String]>,
    grid: GridData,
    counts: Arc<[u64]>,
}

impl CategoricalHeatmapData {
    /// Builds from dense row-major `values` (`y_labels.len() * x_labels.len()`).
    pub fn from_matrix(
        x_labels: &[&str],
        y_labels: &[&str],
        values: &[f64],
    ) -> Result<Self, MultivariateError> {
        if x_labels.is_empty() || y_labels.is_empty() {
            return Err(MultivariateError::Empty);
        }
        if x_labels.len() * y_labels.len() != values.len() {
            return Err(MultivariateError::ShapeMismatch);
        }
        let x_edges: Vec<f64> = (0..=x_labels.len()).map(|i| i as f64).collect();
        let y_edges: Vec<f64> = (0..=y_labels.len()).map(|j| j as f64).collect();
        let counts: Arc<[u64]> = values.iter().map(|v| u64::from(v.is_finite())).collect();
        let grid = GridData::new(x_edges, y_edges, values)
            .map_err(|_| MultivariateError::ShapeMismatch)?;
        Ok(Self {
            x_categories: x_labels.iter().map(|s| (*s).to_owned()).collect(),
            y_categories: y_labels.iter().map(|s| (*s).to_owned()).collect(),
            grid,
            counts,
        })
    }

    /// Aggregates `(x_sample, y_sample, optional value)` tuples like MATLAB `heatmap(tbl, xvar, yvar)`.
    pub fn from_table(
        x_samples: &[&str],
        y_samples: &[&str],
        values: Option<&[f64]>,
        agg: TableAggregation,
        sort: SortOrder,
    ) -> Result<Self, MultivariateError> {
        if x_samples.is_empty() {
            return Err(MultivariateError::Empty);
        }
        if x_samples.len() != y_samples.len() || values.is_some_and(|v| v.len() != x_samples.len())
        {
            return Err(MultivariateError::ShapeMismatch);
        }
        let mut x_cats = collect_categories(x_samples, sort);
        let mut y_cats = collect_categories(y_samples, sort);
        if x_cats.is_empty() || y_cats.is_empty() {
            return Err(MultivariateError::Empty);
        }
        let x_map: HashMap<&str, usize> = x_cats
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();
        let y_map: HashMap<&str, usize> = y_cats
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();
        let nx = x_cats.len();
        let ny = y_cats.len();
        let n = nx * ny;
        let mut counts = vec![0u64; n];
        let mut sums = vec![0.0_f64; n];
        let mut mins = vec![f64::INFINITY; n];
        let mut maxs = vec![f64::NEG_INFINITY; n];

        for i in 0..x_samples.len() {
            let ix = x_map[x_samples[i]];
            let iy = y_map[y_samples[i]];
            let cell = iy * nx + ix;
            let val = values.map_or(1.0, |v| v[i]);
            if agg == TableAggregation::Count {
                counts[cell] += 1;
            } else if val.is_finite() {
                counts[cell] += 1;
                sums[cell] += val;
                mins[cell] = mins[cell].min(val);
                maxs[cell] = maxs[cell].max(val);
            }
        }
        let mut out = vec![f64::NAN; n];
        for i in 0..n {
            if counts[i] == 0 {
                if agg == TableAggregation::Count {
                    out[i] = 0.0;
                }
                continue;
            }
            out[i] = match agg {
                TableAggregation::Count => counts[i] as f64,
                TableAggregation::Sum => sums[i],
                TableAggregation::Mean => sums[i] / counts[i] as f64,
                TableAggregation::Min => mins[i],
                TableAggregation::Max => maxs[i],
            };
        }
        let x_edges: Vec<f64> = (0..=nx).map(|i| i as f64).collect();
        let y_edges: Vec<f64> = (0..=ny).map(|j| j as f64).collect();
        let grid =
            GridData::new(x_edges, y_edges, out).map_err(|_| MultivariateError::ShapeMismatch)?;
        Ok(Self {
            x_categories: x_cats.drain(..).collect(),
            y_categories: y_cats.drain(..).collect(),
            grid,
            counts: counts.into(),
        })
    }

    pub fn grid(&self) -> &GridData {
        &self.grid
    }

    pub fn counts(&self) -> &[u64] {
        &self.counts
    }

    pub fn x_categories(&self) -> &[String] {
        &self.x_categories
    }

    pub fn y_categories(&self) -> &[String] {
        &self.y_categories
    }

    pub fn x_ticks(&self) -> Vec<(f64, String)> {
        self.x_categories
            .iter()
            .enumerate()
            .map(|(i, s)| (i as f64 + 0.5, s.clone()))
            .collect()
    }

    pub fn y_ticks(&self) -> Vec<(f64, String)> {
        self.y_categories
            .iter()
            .enumerate()
            .map(|(i, s)| (i as f64 + 0.5, s.clone()))
            .collect()
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let nx = self.x_categories.len() as f64;
        let ny = self.y_categories.len() as f64;
        if scales[0].forward(0.0).is_none()
            || scales[1].forward(0.0).is_none()
            || scales[0].forward(nx).is_none()
            || scales[1].forward(ny).is_none()
        {
            return None;
        }
        PlotBounds::new([0.0, 0.0], [nx, ny]).ok()
    }
}

fn collect_categories(samples: &[&str], sort: SortOrder) -> Vec<String> {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut list = Vec::new();
    for &s in samples {
        if !seen.contains_key(s) {
            seen.insert(s, list.len());
            list.push(s.to_owned());
        }
    }
    match sort {
        SortOrder::None => {}
        SortOrder::Ascending => list.sort(),
        SortOrder::Descending => list.sort_by(|a, b| b.cmp(a)),
    }
    list
}
