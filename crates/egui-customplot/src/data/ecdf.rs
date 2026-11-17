use std::sync::Arc;

use crate::{AxisScale, PlotBounds, PlotPoint, StatError, axis::norm_inv};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EcdfKind {
    Cdf,
    Survival,
    CumulativeHazard,
}

#[derive(Clone, Debug)]
pub struct EcdfData {
    kind: EcdfKind,
    conf_level: f64,
    step_points: Arc<[PlotPoint]>,
    ci_lower: Arc<[PlotPoint]>,
    ci_upper: Arc<[PlotPoint]>,
    event_points: Arc<[PlotPoint]>,
    censored_marks: Arc<[PlotPoint]>,
}

impl EcdfData {
    /// Uncensored empirical cumulative distribution function with Greenwood 95% bounds.
    pub fn from_samples(samples: &[f64]) -> Result<Self, StatError> {
        let censored = vec![false; samples.len()];
        Self::kaplan_meier(samples, &censored, EcdfKind::Cdf, 0.95)
    }

    /// Kaplan-Meier ECDF / Survival / Cumulative Hazard with tied-time handling and Greenwood CI.
    pub fn kaplan_meier(
        times: &[f64],
        censored: &[bool],
        kind: EcdfKind,
        conf_level: f64,
    ) -> Result<Self, StatError> {
        if times.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if times.len() != censored.len() {
            return Err(StatError::LengthMismatch);
        }
        if !conf_level.is_finite() || conf_level <= 0.0 || conf_level >= 1.0 {
            return Err(StatError::InvalidWidth);
        }
        let mut records: Vec<(f64, bool)> = times
            .iter()
            .copied()
            .zip(censored.iter().copied())
            .filter(|(t, _)| t.is_finite())
            .collect();
        if records.is_empty() {
            return Err(StatError::NoFiniteSamples);
        }
        // Sort by time; tie-break uncensored events before right-censorings.
        records.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let z_alpha = norm_inv(0.5 + 0.5 * conf_level).unwrap_or(1.959_963_984_540_054);
        let init_val = match kind {
            EcdfKind::Cdf => 0.0,
            EcdfKind::Survival => 1.0,
            EcdfKind::CumulativeHazard => 0.0,
        };
        let first_t = records[0].0;
        let mut step_points = vec![[first_t, init_val]];
        let mut ci_lower = vec![[first_t, init_val]];
        let mut ci_upper = vec![[first_t, init_val]];
        let mut event_points = Vec::new();
        let mut censored_marks = Vec::new();

        let mut at_risk = records.len();
        let mut survival = 1.0_f64;
        let mut greenwood_sum = 0.0_f64;
        let mut cum_hazard = 0.0_f64;
        let mut prev_val = init_val;
        let mut prev_lo = init_val;
        let mut prev_hi = init_val;

        let mut i = 0;
        while i < records.len() {
            let t = records[i].0;
            let mut events = 0usize;
            let mut cens = 0usize;
            while i < records.len() && records[i].0 == t {
                if records[i].1 {
                    cens += 1;
                } else {
                    events += 1;
                }
                i += 1;
            }
            if events > 0 && at_risk > 0 {
                let d = events as f64;
                let n = at_risk as f64;
                survival *= 1.0 - d / n;
                cum_hazard += d / n;
                if at_risk > events {
                    greenwood_sum += d / (n * (n - d));
                }
                let val = match kind {
                    EcdfKind::Cdf => (1.0 - survival).clamp(0.0, 1.0),
                    EcdfKind::Survival => survival.clamp(0.0, 1.0),
                    EcdfKind::CumulativeHazard => cum_hazard,
                };
                let se_s = survival * greenwood_sum.sqrt();
                let se = match kind {
                    EcdfKind::Cdf | EcdfKind::Survival => se_s,
                    EcdfKind::CumulativeHazard => {
                        if survival > 1e-9 {
                            se_s / survival
                        } else {
                            greenwood_sum.sqrt()
                        }
                    }
                };
                let (lo, hi) = match kind {
                    EcdfKind::Cdf | EcdfKind::Survival => (
                        (val - z_alpha * se).clamp(0.0, 1.0),
                        (val + z_alpha * se).clamp(0.0, 1.0),
                    ),
                    EcdfKind::CumulativeHazard => {
                        ((val - z_alpha * se).max(0.0), val + z_alpha * se)
                    }
                };
                step_points.push([t, prev_val]);
                step_points.push([t, val]);
                ci_lower.push([t, prev_lo]);
                ci_lower.push([t, lo]);
                ci_upper.push([t, prev_hi]);
                ci_upper.push([t, hi]);
                event_points.push([t, val]);
                prev_val = val;
                prev_lo = lo;
                prev_hi = hi;
            } else if cens > 0 {
                // Extend horizontal plateau up to censoring time.
                if step_points.last().is_some_and(|p| p[0] != t) {
                    step_points.push([t, prev_val]);
                    ci_lower.push([t, prev_lo]);
                    ci_upper.push([t, prev_hi]);
                }
            }
            if cens > 0 {
                censored_marks.push([t, prev_val]);
            }
            at_risk -= events + cens;
        }

        Ok(Self {
            kind,
            conf_level,
            step_points: step_points.into(),
            ci_lower: ci_lower.into(),
            ci_upper: ci_upper.into(),
            event_points: event_points.into(),
            censored_marks: censored_marks.into(),
        })
    }

    pub fn kind(&self) -> EcdfKind {
        self.kind
    }
    pub fn conf_level(&self) -> f64 {
        self.conf_level
    }
    pub fn step_points(&self) -> &[PlotPoint] {
        &self.step_points
    }
    pub fn ci_lower(&self) -> &[PlotPoint] {
        &self.ci_lower
    }
    pub fn ci_upper(&self) -> &[PlotPoint] {
        &self.ci_upper
    }
    pub fn event_points(&self) -> &[PlotPoint] {
        &self.event_points
    }
    pub fn censored_marks(&self) -> &[PlotPoint] {
        &self.censored_marks
    }
    pub fn len(&self) -> usize {
        self.step_points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.step_points.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.step_points)
            + std::mem::size_of_val(&*self.ci_lower)
            + std::mem::size_of_val(&*self.ci_upper)
            + std::mem::size_of_val(&*self.censored_marks)
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for pt in self
            .step_points
            .iter()
            .chain(self.ci_lower.iter())
            .chain(self.ci_upper.iter())
            .chain(self.censored_marks.iter())
        {
            if scales[0].forward(pt[0]).is_none() || scales[1].forward(pt[1]).is_none() {
                continue;
            }
            min[0] = min[0].min(pt[0]);
            min[1] = min[1].min(pt[1]);
            max[0] = max[0].max(pt[0]);
            max[1] = max[1].max(pt[1]);
            count += 1;
        }
        (count > 0)
            .then(|| PlotBounds::new(min, max).ok())
            .flatten()
    }

    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.step_points
            .iter()
            .filter(|pt| scales[0].forward(pt[0]).is_none() || scales[1].forward(pt[1]).is_none())
            .count()
    }
}
