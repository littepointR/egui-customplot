use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::{AxisScale, PlotBounds};

const MAX_CLASSES: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConfusionNormalization {
    #[default]
    Absolute,
    RowNormalized,
    ColumnNormalized,
    TotalNormalized,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfusionError {
    EmptyClasses,
    TooManyClasses,
    DimensionMismatch,
    LabelLengthMismatch,
    DuplicateClassKey,
    UnknownClass,
}

impl std::fmt::Display for ConfusionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyClasses => f.write_str("confusion matrix requires at least one class"),
            Self::TooManyClasses => f.write_str("confusion matrix supports at most 256 classes"),
            Self::DimensionMismatch => f.write_str("counts length must equal class_count^2"),
            Self::LabelLengthMismatch => {
                f.write_str("true/predicted labels and class_names lengths must match")
            }
            Self::DuplicateClassKey => f.write_str("class keys must be unique"),
            Self::UnknownClass => f.write_str("label slice contains an unlisted class"),
        }
    }
}

impl std::error::Error for ConfusionError {}

/// Confusion matrix counts and summary statistics (`confusionchart`, `EXT-06`).
/// Rows represent true classes; columns represent predicted classes.
#[derive(Clone, Debug)]
pub struct ConfusionMatrixData {
    class_names: Arc<[String]>,
    counts: Arc<[u64]>,
    row_totals: Box<[u64]>,
    col_totals: Box<[u64]>,
    grand_total: u64,
    max_diag: u64,
    max_off_diag: u64,
}

impl ConfusionMatrixData {
    pub fn from_counts(
        classes: &[impl AsRef<str>],
        counts: &[u64],
    ) -> Result<Self, ConfusionError> {
        let n = classes.len();
        if n == 0 {
            return Err(ConfusionError::EmptyClasses);
        }
        if n > MAX_CLASSES {
            return Err(ConfusionError::TooManyClasses);
        }
        let expected = n.checked_mul(n).ok_or(ConfusionError::DimensionMismatch)?;
        if counts.len() != expected {
            return Err(ConfusionError::DimensionMismatch);
        }
        let mut row_totals = vec![0u64; n].into_boxed_slice();
        let mut col_totals = vec![0u64; n].into_boxed_slice();
        let mut grand_total = 0u64;
        let mut max_diag = 0u64;
        let mut max_off_diag = 0u64;
        for row in 0..n {
            for col in 0..n {
                let c = counts[row * n + col];
                row_totals[row] = row_totals[row].saturating_add(c);
                col_totals[col] = col_totals[col].saturating_add(c);
                grand_total = grand_total.saturating_add(c);
                if row == col {
                    max_diag = max_diag.max(c);
                } else {
                    max_off_diag = max_off_diag.max(c);
                }
            }
        }
        let names: Arc<[String]> = classes
            .iter()
            .map(|s| s.as_ref().to_owned())
            .collect::<Vec<_>>()
            .into();
        Ok(Self {
            class_names: names,
            counts: Arc::from(counts),
            row_totals,
            col_totals,
            grand_total,
            max_diag,
            max_off_diag,
        })
    }

    pub fn from_labels<T: Eq + Hash>(
        classes: &[T],
        class_names: &[impl AsRef<str>],
        true_labels: &[T],
        predicted_labels: &[T],
    ) -> Result<Self, ConfusionError> {
        if classes.is_empty() {
            return Err(ConfusionError::EmptyClasses);
        }
        if classes.len() > MAX_CLASSES {
            return Err(ConfusionError::TooManyClasses);
        }
        if classes.len() != class_names.len() || true_labels.len() != predicted_labels.len() {
            return Err(ConfusionError::LabelLengthMismatch);
        }
        let n = classes.len();
        let mut index = HashMap::with_capacity(n);
        for (i, c) in classes.iter().enumerate() {
            if index.insert(c, i).is_some() {
                return Err(ConfusionError::DuplicateClassKey);
            }
        }
        let mut counts = vec![0u64; n * n];
        for (t, p) in true_labels.iter().zip(predicted_labels.iter()) {
            let &r = index.get(t).ok_or(ConfusionError::UnknownClass)?;
            let &c = index.get(p).ok_or(ConfusionError::UnknownClass)?;
            counts[r * n + c] += 1;
        }
        Self::from_counts(class_names, &counts)
    }

    pub fn n_classes(&self) -> usize {
        self.class_names.len()
    }
    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }
    pub fn count(&self, row: usize, col: usize) -> u64 {
        let n = self.n_classes();
        if row < n && col < n {
            self.counts[row * n + col]
        } else {
            0
        }
    }
    pub fn row_total(&self, row: usize) -> u64 {
        self.row_totals.get(row).copied().unwrap_or(0)
    }
    pub fn col_total(&self, col: usize) -> u64 {
        self.col_totals.get(col).copied().unwrap_or(0)
    }
    pub fn grand_total(&self) -> u64 {
        self.grand_total
    }
    pub(crate) fn max_diag(&self) -> u64 {
        self.max_diag
    }
    pub(crate) fn max_off_diag(&self) -> u64 {
        self.max_off_diag
    }

    pub fn display_value(&self, row: usize, col: usize, norm: ConfusionNormalization) -> f64 {
        let c = self.count(row, col) as f64;
        match norm {
            ConfusionNormalization::Absolute => c,
            ConfusionNormalization::RowNormalized => {
                let d = self.row_total(row) as f64;
                if d > 0.0 { 100.0 * c / d } else { 0.0 }
            }
            ConfusionNormalization::ColumnNormalized => {
                let d = self.col_total(col) as f64;
                if d > 0.0 { 100.0 * c / d } else { 0.0 }
            }
            ConfusionNormalization::TotalNormalized => {
                let d = self.grand_total as f64;
                if d > 0.0 { 100.0 * c / d } else { 0.0 }
            }
        }
    }

    pub fn row_recall(&self, row: usize) -> Option<f64> {
        let total = self.row_total(row);
        (total > 0).then(|| self.count(row, row) as f64 / total as f64)
    }

    pub fn col_precision(&self, col: usize) -> Option<f64> {
        let total = self.col_total(col);
        (total > 0).then(|| self.count(col, col) as f64 / total as f64)
    }

    pub fn accuracy(&self) -> f64 {
        if self.grand_total == 0 {
            return 0.0;
        }
        let diag: u64 = (0..self.n_classes()).map(|i| self.count(i, i)).sum();
        diag as f64 / self.grand_total as f64
    }

    pub fn x_ticks(&self, summaries: bool) -> Vec<(f64, String)> {
        let mut ticks: Vec<(f64, String)> = self
            .class_names
            .iter()
            .enumerate()
            .map(|(i, s)| (i as f64 + 1.0, s.clone()))
            .collect();
        if summaries {
            let n = self.n_classes() as f64;
            ticks.push((n + 1.2, "TPR".into()));
            ticks.push((n + 2.1, "FNR".into()));
        }
        ticks
    }

    pub fn y_ticks(&self, summaries: bool) -> Vec<(f64, String)> {
        let mut ticks: Vec<(f64, String)> = self
            .class_names
            .iter()
            .enumerate()
            .map(|(i, s)| (i as f64 + 1.0, s.clone()))
            .collect();
        if summaries {
            let n = self.n_classes() as f64;
            ticks.push((n + 1.2, "PPV".into()));
            ticks.push((n + 2.1, "FDR".into()));
        }
        ticks
    }

    pub(crate) fn bounds(&self, scales: [AxisScale; 2], summaries: bool) -> Option<PlotBounds> {
        let lo = 0.5;
        let hi = self.n_classes() as f64 + if summaries { 2.65 } else { 0.5 };
        scales[0].forward(lo)?;
        scales[1].forward(lo)?;
        scales[0].forward(hi)?;
        scales[1].forward(hi)?;
        PlotBounds::new([lo, lo], [hi, hi]).ok()
    }
}
