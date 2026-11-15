use std::{f32::consts::FRAC_PI_2, f32::consts::TAU, sync::Arc};

use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieError {
    Empty,
    LengthMismatch,
    NoPositiveValues,
}

impl std::fmt::Display for PieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid pie data: {self:?}")
    }
}
impl std::error::Error for PieError {}

#[derive(Clone, Debug, PartialEq)]
pub struct PieSlice {
    pub index: usize,
    pub label: String,
    pub value: f64,
    pub fraction: f64,
    pub start_angle: f32,
    pub end_angle: f32,
    pub explode: bool,
}

/// Immutable normalized slice geometry for `piechart` / `donutchart` / legacy `pie` (`ML-027`, `ML-028`, `LG-001`).
#[derive(Clone, Debug)]
pub struct PieData {
    slices: Arc<[PieSlice]>,
    total: f64,
    skipped: usize,
}

impl PieData {
    pub fn new(values: &[f64]) -> Result<Self, PieError> {
        Self::build(values, None, None, -FRAC_PI_2)
    }

    pub fn with_labels(values: &[f64], labels: &[&str]) -> Result<Self, PieError> {
        if values.len() != labels.len() {
            return Err(PieError::LengthMismatch);
        }
        Self::build(values, Some(labels), None, -FRAC_PI_2)
    }

    pub fn with_options(
        values: &[f64],
        labels: Option<&[&str]>,
        explode: Option<&[bool]>,
        start_angle: f32,
    ) -> Result<Self, PieError> {
        if let Some(l) = labels
            && l.len() != values.len()
        {
            return Err(PieError::LengthMismatch);
        }
        if let Some(e) = explode
            && e.len() != values.len()
        {
            return Err(PieError::LengthMismatch);
        }
        Self::build(values, labels, explode, start_angle)
    }

    fn build(
        values: &[f64],
        labels: Option<&[&str]>,
        explode: Option<&[bool]>,
        start_angle: f32,
    ) -> Result<Self, PieError> {
        if values.is_empty() {
            return Err(PieError::Empty);
        }
        let mut total = 0.0;
        let mut skipped = 0usize;
        for &v in values {
            if v.is_finite() && v > 0.0 {
                total += v;
            } else {
                skipped += 1;
            }
        }
        if !total.is_finite() || total <= 0.0 {
            return Err(PieError::NoPositiveValues);
        }
        let mut angle = if start_angle.is_finite() {
            start_angle
        } else {
            -FRAC_PI_2
        };
        let mut slices = Vec::with_capacity(values.len());
        for (i, &v) in values.iter().enumerate() {
            if !v.is_finite() || v <= 0.0 {
                continue;
            }
            let frac = v / total;
            let span = (frac as f32) * TAU;
            let label = labels
                .map(|l| l[i].to_owned())
                .unwrap_or_else(|| format!("{:.1}%", frac * 100.0));
            let exploded = explode.is_some_and(|e| e[i]);
            slices.push(PieSlice {
                index: i,
                label,
                value: v,
                fraction: frac,
                start_angle: angle,
                end_angle: angle + span,
                explode: exploded,
            });
            angle += span;
        }
        Ok(Self {
            slices: slices.into(),
            total,
            skipped,
        })
    }

    pub fn slices(&self) -> &[PieSlice] {
        &self.slices
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn skipped_slices(&self) -> usize {
        self.skipped
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales != [AxisScale::Linear, AxisScale::Linear] {
            return None;
        }
        PlotBounds::new([-1.35, -1.35], [1.35, 1.35]).ok()
    }
}
