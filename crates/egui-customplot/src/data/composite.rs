use std::{collections::BTreeMap, sync::Arc};

use super::{
    distribution::{compute_box_stats, gaussian_kde},
    hist_legacy::LegacyHistData,
};
use crate::{Bar, BarData, HistogramError, HistogramNormalization, PlotData, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeError {
    Empty,
    LengthMismatch,
    NoPositiveValues,
    InvalidBins,
}

impl std::fmt::Display for CompositeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid composite chart data: {self:?}")
    }
}
impl std::error::Error for CompositeError {}

/// Sorted bar + cumulative percentage curve data for `pareto` (`ML-038`).
#[derive(Clone, Debug)]
pub struct ParetoData {
    labels: Arc<[String]>,
    values: Arc<[f64]>,
    cumulative_percent: Arc<[f64]>,
    bars: BarData,
    cumulative_line: PlotData,
    total: f64,
    y_max: f64,
}

impl ParetoData {
    pub fn new(labels: &[&str], values: &[f64]) -> Result<Self, CompositeError> {
        Self::with_limit(labels, values, 1.0, usize::MAX)
    }

    pub fn with_limit(
        labels: &[&str],
        values: &[f64],
        threshold: f64,
        max_bars: usize,
    ) -> Result<Self, CompositeError> {
        if labels.is_empty() {
            return Err(CompositeError::Empty);
        }
        if labels.len() != values.len() {
            return Err(CompositeError::LengthMismatch);
        }
        let mut pairs: Vec<(usize, String, f64)> = labels
            .iter()
            .zip(values.iter())
            .enumerate()
            .filter(|&(_, (_, &v))| v.is_finite() && v > 0.0)
            .map(|(i, (&lbl, &v))| (i, lbl.to_owned(), v))
            .collect();
        if pairs.is_empty() {
            return Err(CompositeError::NoPositiveValues);
        }
        pairs.sort_by(|a, b| b.2.total_cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

        let total: f64 = pairs.iter().map(|p| p.2).sum();
        let cutoff = threshold.clamp(0.05, 1.0);
        let limit = max_bars.max(1);

        let mut out_labels = Vec::new();
        let mut out_vals = Vec::new();
        let mut out_pct = Vec::new();
        let mut cum = 0.0;
        for (_, lbl, val) in pairs {
            cum += val;
            let pct = (cum / total) * 100.0;
            out_labels.push(lbl);
            out_vals.push(val);
            out_pct.push(pct);
            if out_vals.len() >= limit || cum / total >= cutoff - 1e-12 {
                break;
            }
        }
        let y_max = out_vals[0];
        let bar_vec: Arc<[Bar]> = out_vals
            .iter()
            .enumerate()
            .map(|(i, &v)| Bar::new(i as f64 + 1.0, v).baseline(0.0).width(0.72))
            .collect();
        let line_vec: Arc<[PlotPoint]> = out_pct
            .iter()
            .enumerate()
            .map(|(i, &p)| [i as f64 + 1.0, p])
            .collect();
        Ok(Self {
            labels: out_labels.into(),
            values: out_vals.into(),
            cumulative_percent: out_pct.into(),
            bars: BarData::new(bar_vec),
            cumulative_line: PlotData::new(line_vec),
            total,
            y_max,
        })
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    pub fn cumulative_percent(&self) -> &[f64] {
        &self.cumulative_percent
    }

    pub fn bars(&self) -> &BarData {
        &self.bars
    }

    pub fn cumulative_line(&self) -> &PlotData {
        &self.cumulative_line
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn y_max(&self) -> f64 {
        self.y_max
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginalKind {
    Histogram,
    Kernel,
    None,
}

/// Precomputed central grouped scatter + X/Y marginal histograms or KDEs (`ML-023` `scatterhistogram`).
#[derive(Clone, Debug)]
pub struct ScatterHistogramData {
    pub groups: Arc<[(usize, PlotData)]>,
    pub x_bars: BarData,
    pub y_bars: BarData,
    pub x_kde: Option<PlotData>,
    pub y_kde: Option<PlotData>,
    pub x_range: [f64; 2],
    pub y_range: [f64; 2],
    pub marginal: MarginalKind,
}

impl ScatterHistogramData {
    pub fn new(
        x: &[f64],
        y: &[f64],
        groups: Option<&[usize]>,
        bins: usize,
        marginal: MarginalKind,
    ) -> Result<Self, CompositeError> {
        if x.is_empty() {
            return Err(CompositeError::Empty);
        }
        if x.len() != y.len() || groups.is_some_and(|g| g.len() != x.len()) {
            return Err(CompositeError::LengthMismatch);
        }
        if bins == 0 {
            return Err(CompositeError::InvalidBins);
        }
        let mut by_group: BTreeMap<usize, Vec<PlotPoint>> = BTreeMap::new();
        let mut xs = Vec::with_capacity(x.len());
        let mut ys = Vec::with_capacity(y.len());
        for i in 0..x.len() {
            if x[i].is_finite() && y[i].is_finite() {
                let g = groups.map_or(0, |gs| gs[i]);
                by_group.entry(g).or_default().push([x[i], y[i]]);
                xs.push(x[i]);
                ys.push(y[i]);
            }
        }
        if xs.is_empty() {
            return Err(CompositeError::Empty);
        }
        let x_lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let x_hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let y_lo = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let y_hi = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let x_pad = ((x_hi - x_lo) * 0.05).max(0.5);
        let y_pad = ((y_hi - y_lo) * 0.05).max(0.5);
        let x_range = [x_lo - x_pad, x_hi + x_pad];
        let y_range = [y_lo - y_pad, y_hi + y_pad];

        let x_bars = LegacyHistData::new(&xs, bins)
            .map_err(|_: HistogramError| CompositeError::InvalidBins)?
            .normalized_bars(HistogramNormalization::Density)
            .map_err(|_| CompositeError::InvalidBins)?;
        let y_bars = LegacyHistData::new(&ys, bins)
            .map_err(|_: HistogramError| CompositeError::InvalidBins)?
            .normalized_bars(HistogramNormalization::Density)
            .map_err(|_| CompositeError::InvalidBins)?;

        let (x_kde, y_kde) = if marginal == MarginalKind::Kernel {
            let x_pts = compute_box_stats(0.0, &xs, 1.0).map(|(_, s)| {
                let (gx, gd) = gaussian_kde(&s, 64, None);
                let pts: Arc<[PlotPoint]> =
                    gx.iter().zip(gd.iter()).map(|(&a, &b)| [a, b]).collect();
                PlotData::new(pts)
            });
            let y_pts = compute_box_stats(0.0, &ys, 1.0).map(|(_, s)| {
                let (gy, gd) = gaussian_kde(&s, 64, None);
                // X holds density so right marginal plots smoothly vertically against shared Y!
                let pts: Arc<[PlotPoint]> =
                    gd.iter().zip(gy.iter()).map(|(&d, &yv)| [d, yv]).collect();
                PlotData::new(pts)
            });
            (x_pts, y_pts)
        } else {
            (None, None)
        };

        let group_data: Arc<[(usize, PlotData)]> = by_group
            .into_iter()
            .map(|(g, pts)| (g, PlotData::new(pts)))
            .collect();
        Ok(Self {
            groups: group_data,
            x_bars,
            y_bars,
            x_kde,
            y_kde,
            x_range,
            y_range,
            marginal,
        })
    }
}

/// Pairwise scatter + diagonal histogram matrix dataset (`ML-033` `plotmatrix`).
#[derive(Clone, Debug)]
pub struct PlotMatrixData {
    pub names: Arc<[String]>,
    pub ranges: Arc<[[f64; 2]]>,
    pub diagonals: Arc<[BarData]>,
    /// Row-major `n_vars * n_vars`; off-diagonal `(r, c)` holds grouped `PlotData` (`x = col[c], y = col[r]`).
    pub off_diagonal: Arc<[Vec<(usize, PlotData)>]>,
    pub n_vars: usize,
}

impl PlotMatrixData {
    pub fn new(
        names: &[&str],
        columns: &[&[f64]],
        groups: Option<&[usize]>,
        bins: usize,
    ) -> Result<Self, CompositeError> {
        let m = names.len();
        if m == 0 || columns.len() != m {
            return Err(CompositeError::LengthMismatch);
        }
        let n_rows = columns[0].len();
        if n_rows == 0 {
            return Err(CompositeError::Empty);
        }
        if columns.iter().any(|c| c.len() != n_rows) || groups.is_some_and(|g| g.len() != n_rows) {
            return Err(CompositeError::LengthMismatch);
        }
        if bins == 0 {
            return Err(CompositeError::InvalidBins);
        }
        let mut ranges = Vec::with_capacity(m);
        let mut diagonals = Vec::with_capacity(m);
        for col in columns {
            let mut lo = f64::INFINITY;
            let mut hi = f64::NEG_INFINITY;
            for &v in *col {
                if v.is_finite() {
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
            }
            if !lo.is_finite() {
                lo = 0.0;
                hi = 1.0;
            }
            let pad = ((hi - lo) * 0.06).max(0.25);
            ranges.push([lo - pad, hi + pad]);
            let hist = LegacyHistData::new(col, bins)
                .map_err(|_| CompositeError::InvalidBins)?
                .normalized_bars(HistogramNormalization::Count)
                .map_err(|_| CompositeError::InvalidBins)?;
            diagonals.push(hist);
        }

        let mut cells = vec![Vec::new(); m * m];
        for r in 0..m {
            for c in 0..m {
                if r == c {
                    continue;
                }
                let mut by_group: BTreeMap<usize, Vec<PlotPoint>> = BTreeMap::new();
                for i in 0..n_rows {
                    let x = columns[c][i];
                    let y = columns[r][i];
                    if x.is_finite() && y.is_finite() {
                        let g = groups.map_or(0, |gs| gs[i]);
                        by_group.entry(g).or_default().push([x, y]);
                    }
                }
                cells[r * m + c] = by_group
                    .into_iter()
                    .map(|(g, pts)| (g, PlotData::new(pts)))
                    .collect();
            }
        }
        Ok(Self {
            names: names.iter().map(|s| (*s).to_owned()).collect(),
            ranges: ranges.into(),
            diagonals: diagonals.into(),
            off_diagonal: cells.into(),
            n_vars: m,
        })
    }
}
