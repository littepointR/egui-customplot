use std::sync::Arc;

use super::MultivariateError;
use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParallelNormalization {
    None,
    ZScore,
    Norm,
    Range,
    MedianIqr,
}

/// Parallel coordinates multivariate dataset (`ML-032` `parallelplot`).
#[derive(Clone, Debug)]
pub struct ParallelData {
    var_names: Arc<[String]>,
    raw: Arc<[f64]>,
    normalized: Arc<[f64]>,
    col_ranges: Arc<[[f64; 2]]>,
    groups: Option<Arc<[usize]>>,
    group_names: Arc<[String]>,
    n_vars: usize,
    n_rows: usize,
    y_range: [f64; 2],
    normalization: ParallelNormalization,
}

impl ParallelData {
    pub fn new(
        var_names: &[&str],
        rows: &[&[f64]],
        normalization: ParallelNormalization,
    ) -> Result<Self, MultivariateError> {
        if var_names.is_empty() || rows.is_empty() {
            return Err(MultivariateError::Empty);
        }
        let n_vars = var_names.len();
        let n_rows = rows.len();
        for row in rows {
            if row.len() != n_vars {
                return Err(MultivariateError::ShapeMismatch);
            }
        }
        let mut raw = Vec::with_capacity(n_rows * n_vars);
        for row in rows {
            raw.extend_from_slice(row);
        }
        let mut normalized = raw.clone();
        let mut col_ranges = vec![[f64::NAN, f64::NAN]; n_vars];

        for col in 0..n_vars {
            let mut vals: Vec<f64> = (0..n_rows)
                .map(|r| raw[r * n_vars + col])
                .filter(|v| v.is_finite())
                .collect();
            if vals.is_empty() {
                continue;
            }
            vals.sort_by(f64::total_cmp);
            let lo = vals[0];
            let hi = vals[vals.len() - 1];
            col_ranges[col] = [lo, hi];
            let n = vals.len() as f64;
            let mean = vals.iter().sum::<f64>() / n;
            let var = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n.max(1.0);
            let std = var.sqrt();
            let l2 = vals.iter().map(|v| v * v).sum::<f64>().sqrt();
            let med = vals[vals.len() / 2];
            let q1 = vals[vals.len() / 4];
            let q3 = vals[(vals.len() * 3) / 4];
            let iqr = q3 - q1;

            for r in 0..n_rows {
                let v = raw[r * n_vars + col];
                if !v.is_finite() {
                    continue;
                }
                normalized[r * n_vars + col] = match normalization {
                    ParallelNormalization::None => v,
                    ParallelNormalization::ZScore => {
                        if std > 0.0 {
                            (v - mean) / std
                        } else {
                            0.0
                        }
                    }
                    ParallelNormalization::Norm => {
                        if l2 > 0.0 {
                            v / l2
                        } else {
                            0.0
                        }
                    }
                    ParallelNormalization::Range => {
                        if (hi - lo) > 0.0 {
                            (v - lo) / (hi - lo)
                        } else {
                            0.5
                        }
                    }
                    ParallelNormalization::MedianIqr => {
                        if iqr > 0.0 {
                            (v - med) / iqr
                        } else {
                            0.0
                        }
                    }
                };
            }
        }

        let (mut y_lo, mut y_hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &v in &normalized {
            if v.is_finite() {
                y_lo = y_lo.min(v);
                y_hi = y_hi.max(v);
            }
        }
        if !y_lo.is_finite() || !y_hi.is_finite() {
            return Err(MultivariateError::NoFiniteValues);
        }
        if (y_hi - y_lo).abs() <= f64::EPSILON {
            y_lo -= 0.5;
            y_hi += 0.5;
        }
        Ok(Self {
            var_names: var_names.iter().map(|s| (*s).to_owned()).collect(),
            raw: raw.into(),
            normalized: normalized.into(),
            col_ranges: col_ranges.into(),
            groups: None,
            group_names: Arc::from([]),
            n_vars,
            n_rows,
            y_range: [y_lo, y_hi],
            normalization,
        })
    }

    pub fn with_groups(
        mut self,
        groups: &[usize],
        group_names: &[&str],
    ) -> Result<Self, MultivariateError> {
        if groups.len() != self.n_rows {
            return Err(MultivariateError::ShapeMismatch);
        }
        self.groups = Some(groups.into());
        self.group_names = group_names.iter().map(|s| (*s).to_owned()).collect();
        Ok(self)
    }

    pub fn var_names(&self) -> &[String] {
        &self.var_names
    }

    pub fn n_vars(&self) -> usize {
        self.n_vars
    }

    pub fn n_rows(&self) -> usize {
        self.n_rows
    }

    pub fn raw_row(&self, row: usize) -> &[f64] {
        let start = row * self.n_vars;
        &self.raw[start..start + self.n_vars]
    }

    pub fn normalized_row(&self, row: usize) -> &[f64] {
        let start = row * self.n_vars;
        &self.normalized[start..start + self.n_vars]
    }

    pub fn row_group(&self, row: usize) -> usize {
        self.groups.as_ref().map_or(0, |g| g[row])
    }

    pub fn group_names(&self) -> &[String] {
        &self.group_names
    }

    pub fn col_ranges(&self) -> &[[f64; 2]] {
        &self.col_ranges
    }

    pub fn normalization(&self) -> ParallelNormalization {
        self.normalization
    }

    pub fn x_ticks(&self) -> Vec<(f64, String)> {
        self.var_names
            .iter()
            .enumerate()
            .map(|(i, s)| (i as f64, s.clone()))
            .collect()
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales[0] != AxisScale::Linear
            || scales[1].forward(self.y_range[0]).is_none()
            || scales[1].forward(self.y_range[1]).is_none()
        {
            return None;
        }
        let max_x = (self.n_vars.saturating_sub(1)).max(1) as f64;
        PlotBounds::new([0.0, self.y_range[0]], [max_x, self.y_range[1]]).ok()
    }
}
