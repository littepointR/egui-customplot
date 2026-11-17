use super::{DendrogramData, GridData, LinkageMethod, MultivariateError};
use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterStandardize {
    None,
    RowZScore,
    ColZScore,
}

#[derive(Clone, Debug)]
pub struct ClustergramData {
    row_dendrogram: DendrogramData,
    col_dendrogram: DendrogramData,
    grid: GridData,
    row_labels: Vec<String>,
    col_labels: Vec<String>,
    value_range: [f64; 2],
}

impl ClustergramData {
    pub fn new(
        matrix: &[&[f64]],
        method: LinkageMethod,
        standardize: ClusterStandardize,
    ) -> Result<Self, MultivariateError> {
        let nrows = matrix.len();
        if nrows < 2 {
            return Err(MultivariateError::Empty);
        }
        let ncols = matrix[0].len();
        if ncols < 2 || matrix.iter().any(|r| r.len() != ncols) {
            return Err(MultivariateError::ShapeMismatch);
        }
        if matrix.iter().flat_map(|r| r.iter()).any(|v| !v.is_finite()) {
            return Err(MultivariateError::NoFiniteValues);
        }

        let mut norm: Vec<Vec<f64>> = matrix.iter().map(|r| r.to_vec()).collect();
        match standardize {
            ClusterStandardize::None => {}
            ClusterStandardize::RowZScore => {
                for row in &mut norm {
                    zscore_slice(row);
                }
            }
            ClusterStandardize::ColZScore => {
                for c in 0..ncols {
                    let mut col: Vec<f64> = (0..nrows).map(|r| norm[r][c]).collect();
                    zscore_slice(&mut col);
                    for r in 0..nrows {
                        norm[r][c] = col[r];
                    }
                }
            }
        }

        let row_refs: Vec<&[f64]> = norm.iter().map(Vec::as_slice).collect();
        let row_dendrogram = DendrogramData::from_observations(&row_refs, method)?;

        let cols_data: Vec<Vec<f64>> = (0..ncols)
            .map(|c| (0..nrows).map(|r| norm[r][c]).collect())
            .collect();
        let col_refs: Vec<&[f64]> = cols_data.iter().map(Vec::as_slice).collect();
        let col_dendrogram = DendrogramData::from_observations(&col_refs, method)?;

        let row_order = row_dendrogram.leaf_order();
        let col_order = col_dendrogram.leaf_order();
        let mut values = Vec::with_capacity(nrows * ncols);
        let mut vmin = f64::INFINITY;
        let mut vmax = f64::NEG_INFINITY;
        for &r in row_order {
            for &c in col_order {
                let v = norm[r][c];
                vmin = vmin.min(v);
                vmax = vmax.max(v);
                values.push(v);
            }
        }
        let x_edges: Vec<f64> = (0..=ncols).map(|i| i as f64 + 0.5).collect();
        let y_edges: Vec<f64> = (0..=nrows).map(|i| i as f64 + 0.5).collect();
        let grid = GridData::new(x_edges, y_edges, values)
            .map_err(|_| MultivariateError::ShapeMismatch)?;
        let value_range = if (vmax - vmin).abs() > 1e-12 {
            [vmin, vmax]
        } else {
            [vmin - 0.5, vmax + 0.5]
        };

        let row_labels = (0..nrows).map(|r| format!("R{}", r + 1)).collect();
        let col_labels = (0..ncols).map(|c| format!("C{}", c + 1)).collect();
        Ok(Self {
            row_dendrogram,
            col_dendrogram,
            grid,
            row_labels,
            col_labels,
            value_range,
        })
    }

    pub fn with_row_labels(mut self, labels: impl Into<Vec<String>>) -> Self {
        let labels = labels.into();
        if labels.len() == self.row_dendrogram.n_leaves() {
            self.row_dendrogram = self.row_dendrogram.with_labels(labels.clone());
            self.row_labels = labels;
        }
        self
    }

    pub fn with_col_labels(mut self, labels: impl Into<Vec<String>>) -> Self {
        let labels = labels.into();
        if labels.len() == self.col_dendrogram.n_leaves() {
            self.col_dendrogram = self.col_dendrogram.with_labels(labels.clone());
            self.col_labels = labels;
        }
        self
    }

    pub fn row_dendrogram(&self) -> &DendrogramData {
        &self.row_dendrogram
    }
    pub fn col_dendrogram(&self) -> &DendrogramData {
        &self.col_dendrogram
    }
    pub fn grid(&self) -> &GridData {
        &self.grid
    }
    pub fn value_range(&self) -> [f64; 2] {
        self.value_range
    }
    pub fn n_rows(&self) -> usize {
        self.row_dendrogram.n_leaves()
    }
    pub fn n_cols(&self) -> usize {
        self.col_dendrogram.n_leaves()
    }
    pub fn ordered_row_ticks(&self) -> Vec<(f64, String)> {
        self.row_dendrogram.leaf_ticks()
    }
    pub fn ordered_col_ticks(&self) -> Vec<(f64, String)> {
        self.col_dendrogram.leaf_ticks()
    }
    pub fn heatmap_bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (min, max) = self.grid.extent(scales)?;
        PlotBounds::new(min, max).ok()
    }
}

fn zscore_slice(vals: &mut [f64]) {
    let n = vals.len() as f64;
    let mean = vals.iter().sum::<f64>() / n;
    let var = vals
        .iter()
        .map(|&v| {
            let d = v - mean;
            d * d
        })
        .sum::<f64>()
        / (n - 1.0).max(1.0);
    let std = var.sqrt();
    if std > 1e-12 {
        for v in vals {
            *v = (*v - mean) / std;
        }
    } else {
        for v in vals {
            *v = 0.0;
        }
    }
}
