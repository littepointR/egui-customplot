use std::sync::Arc;

use crate::{Bar, BarData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarSeriesError {
    EmptySeries,
    LengthMismatch,
    NonFiniteCenter,
    InvalidWidth,
}

impl std::fmt::Display for BarSeriesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bar series: {self:?}")
    }
}
impl std::error::Error for BarSeriesError {}

fn validate_centers_and_step(centers: &[f64], width_ratio: f64) -> Result<f64, BarSeriesError> {
    if centers.is_empty() {
        return Err(BarSeriesError::EmptySeries);
    }
    if centers.iter().any(|c| !c.is_finite()) {
        return Err(BarSeriesError::NonFiniteCenter);
    }
    if !width_ratio.is_finite() || width_ratio <= 0.0 || width_ratio > 1.0 {
        return Err(BarSeriesError::InvalidWidth);
    }
    if centers.len() == 1 {
        return Ok(1.0);
    }
    let mut sorted = centers.to_vec();
    sorted.sort_by(f64::total_cmp);
    let min_gap = sorted
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0.0 && d.is_finite())
        .fold(f64::INFINITY, f64::min);
    Ok(if min_gap.is_finite() { min_gap } else { 1.0 })
}

impl BarData {
    /// Builds side-by-side grouped bars sharing `centers` across multiple series.
    pub fn grouped(
        centers: &[f64],
        series: &[&[f64]],
        group_width: f64,
    ) -> Result<Vec<Self>, BarSeriesError> {
        if series.is_empty() {
            return Err(BarSeriesError::EmptySeries);
        }
        let step = validate_centers_and_step(centers, group_width)?;
        for values in series {
            if values.len() != centers.len() {
                return Err(BarSeriesError::LengthMismatch);
            }
        }
        let m = series.len() as f64;
        let bar_step = (step * group_width) / m;
        let bar_w = bar_step * 0.9;
        let half_span = (m - 1.0) * 0.5;
        let out = series
            .iter()
            .enumerate()
            .map(|(s_idx, values)| {
                let offset = (s_idx as f64 - half_span) * bar_step;
                let bars: Arc<[Bar]> = centers
                    .iter()
                    .zip(values.iter())
                    .map(|(&c, &v)| Bar::new(c + offset, v).baseline(0.0).width(bar_w))
                    .collect();
                Self::new(bars)
            })
            .collect();
        Ok(out)
    }

    /// Builds vertically/horizontally stacked bars accumulating positive and negative stacks separately.
    pub fn stacked(
        centers: &[f64],
        series: &[&[f64]],
        bar_width: f64,
    ) -> Result<Vec<Self>, BarSeriesError> {
        if series.is_empty() {
            return Err(BarSeriesError::EmptySeries);
        }
        let step = validate_centers_and_step(centers, bar_width)?;
        for values in series {
            if values.len() != centers.len() {
                return Err(BarSeriesError::LengthMismatch);
            }
        }
        let bar_w = step * bar_width;
        let mut pos_acc = vec![0.0_f64; centers.len()];
        let mut neg_acc = vec![0.0_f64; centers.len()];
        let out = series
            .iter()
            .map(|values| {
                let bars: Arc<[Bar]> = centers
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        let v = values[i];
                        if !v.is_finite() {
                            return Bar::new(c, f64::NAN).width(bar_w);
                        }
                        if v >= 0.0 {
                            let base = pos_acc[i];
                            let top = base + v;
                            pos_acc[i] = top;
                            Bar::new(c, top).baseline(base).width(bar_w)
                        } else {
                            let base = neg_acc[i];
                            let top = base + v;
                            neg_acc[i] = top;
                            Bar::new(c, top).baseline(base).width(bar_w)
                        }
                    })
                    .collect();
                Self::new(bars)
            })
            .collect();
        Ok(out)
    }
}
