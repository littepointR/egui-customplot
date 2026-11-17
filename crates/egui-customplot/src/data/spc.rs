use std::sync::Arc;

use crate::{AxisScale, MultivariateError, PlotBounds, StatError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpcChartKind {
    IMR,
    XbarR,
    PChart,
    UChart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpcViolationKind {
    BeyondThreeSigma,
    NineOneSide,
    SixTrend,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpcPoint {
    pub index: usize,
    pub x: f64,
    pub value: f64,
    pub cl: f64,
    pub ucl: f64,
    pub lcl: f64,
    pub violation: Option<SpcViolationKind>,
}

#[derive(Clone, Debug)]
pub struct ControlChartData {
    kind: SpcChartKind,
    center_line: f64,
    points: Arc<[SpcPoint]>,
    nelson_rules: bool,
}

impl ControlChartData {
    /// Individuals (I-MR) control chart using average moving range (`d2 = 1.128`).
    pub fn imr(values: &[f64]) -> Result<Self, StatError> {
        if values.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(StatError::NoFiniteSamples);
        }
        let n = values.len();
        let cl = values.iter().sum::<f64>() / n as f64;
        let mr_bar = if n > 1 {
            values.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (n - 1) as f64
        } else {
            0.0
        };
        const D2_N2: f64 = 1.128;
        let half = 3.0 * mr_bar / D2_N2;
        let mut pts: Vec<SpcPoint> = values
            .iter()
            .enumerate()
            .map(|(i, &value)| SpcPoint {
                index: i,
                x: (i + 1) as f64,
                value,
                cl,
                ucl: cl + half,
                lcl: cl - half,
                violation: None,
            })
            .collect();
        apply_rules(&mut pts, true);
        Ok(Self {
            kind: SpcChartKind::IMR,
            center_line: cl,
            points: pts.into(),
            nelson_rules: true,
        })
    }

    /// Subgroup mean (Xbar-R) control chart using standard `A2` subgroup constant.
    pub fn xbar_r(subgroups: &[&[f64]]) -> Result<Self, MultivariateError> {
        if subgroups.is_empty() {
            return Err(MultivariateError::Empty);
        }
        let m = subgroups[0].len();
        if m < 2 || subgroups.iter().any(|g| g.len() != m) {
            return Err(MultivariateError::ShapeMismatch);
        }
        if subgroups
            .iter()
            .flat_map(|g| g.iter())
            .any(|v| !v.is_finite())
        {
            return Err(MultivariateError::NoFiniteValues);
        }
        let mut means = Vec::with_capacity(subgroups.len());
        let mut ranges = Vec::with_capacity(subgroups.len());
        for g in subgroups {
            let mean = g.iter().sum::<f64>() / m as f64;
            let lo = g.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = g.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            means.push(mean);
            ranges.push(hi - lo);
        }
        let cl = means.iter().sum::<f64>() / means.len() as f64;
        let r_bar = ranges.iter().sum::<f64>() / ranges.len() as f64;
        let a2 = xbar_a2(m);
        let half = a2 * r_bar;
        let mut pts: Vec<SpcPoint> = means
            .into_iter()
            .enumerate()
            .map(|(i, value)| SpcPoint {
                index: i,
                x: (i + 1) as f64,
                value,
                cl,
                ucl: cl + half,
                lcl: cl - half,
                violation: None,
            })
            .collect();
        apply_rules(&mut pts, true);
        Ok(Self {
            kind: SpcChartKind::XbarR,
            center_line: cl,
            points: pts.into(),
            nelson_rules: true,
        })
    }

    /// Proportion nonconforming (`p`) control chart with variable subgroup size limits.
    pub fn p_chart(defects: &[f64], sizes: &[f64]) -> Result<Self, StatError> {
        if defects.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if defects.len() != sizes.len() {
            return Err(StatError::LengthMismatch);
        }
        let mut sum_d = 0.0_f64;
        let mut sum_n = 0.0_f64;
        for (&d, &n) in defects.iter().zip(sizes.iter()) {
            if !d.is_finite() || !n.is_finite() || n <= 0.0 || d < 0.0 || d > n {
                return Err(StatError::NoFiniteSamples);
            }
            sum_d += d;
            sum_n += n;
        }
        let p_bar = sum_d / sum_n;
        let mut pts: Vec<SpcPoint> = defects
            .iter()
            .zip(sizes.iter())
            .enumerate()
            .map(|(i, (&d, &n))| {
                let sigma = (p_bar * (1.0 - p_bar) / n).max(0.0).sqrt();
                SpcPoint {
                    index: i,
                    x: (i + 1) as f64,
                    value: d / n,
                    cl: p_bar,
                    ucl: (p_bar + 3.0 * sigma).min(1.0),
                    lcl: (p_bar - 3.0 * sigma).max(0.0),
                    violation: None,
                }
            })
            .collect();
        apply_rules(&mut pts, true);
        Ok(Self {
            kind: SpcChartKind::PChart,
            center_line: p_bar,
            points: pts.into(),
            nelson_rules: true,
        })
    }

    /// Defects per unit (`u`) control chart with variable inspection unit limits.
    pub fn u_chart(defects: &[f64], units: &[f64]) -> Result<Self, StatError> {
        if defects.is_empty() {
            return Err(StatError::EmptyInput);
        }
        if defects.len() != units.len() {
            return Err(StatError::LengthMismatch);
        }
        let mut sum_c = 0.0_f64;
        let mut sum_n = 0.0_f64;
        for (&c, &n) in defects.iter().zip(units.iter()) {
            if !c.is_finite() || !n.is_finite() || n <= 0.0 || c < 0.0 {
                return Err(StatError::NoFiniteSamples);
            }
            sum_c += c;
            sum_n += n;
        }
        let u_bar = sum_c / sum_n;
        let mut pts: Vec<SpcPoint> = defects
            .iter()
            .zip(units.iter())
            .enumerate()
            .map(|(i, (&c, &n))| {
                let sigma = (u_bar / n).max(0.0).sqrt();
                SpcPoint {
                    index: i,
                    x: (i + 1) as f64,
                    value: c / n,
                    cl: u_bar,
                    ucl: u_bar + 3.0 * sigma,
                    lcl: (u_bar - 3.0 * sigma).max(0.0),
                    violation: None,
                }
            })
            .collect();
        apply_rules(&mut pts, true);
        Ok(Self {
            kind: SpcChartKind::UChart,
            center_line: u_bar,
            points: pts.into(),
            nelson_rules: true,
        })
    }

    pub fn with_nelson_rules(mut self, enabled: bool) -> Self {
        self.nelson_rules = enabled;
        let mut pts = self.points.to_vec();
        apply_rules(&mut pts, enabled);
        self.points = pts.into();
        self
    }

    pub fn kind(&self) -> SpcChartKind {
        self.kind
    }
    pub fn center_line(&self) -> f64 {
        self.center_line
    }
    pub fn points(&self) -> &[SpcPoint] {
        &self.points
    }
    pub fn nelson_rules(&self) -> bool {
        self.nelson_rules
    }
    pub fn violation_count(&self) -> usize {
        self.points.iter().filter(|p| p.violation.is_some()).count()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.points)
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for pt in self.points.iter() {
            if scales[0].forward(pt.x).is_none() {
                continue;
            }
            for y in [pt.value, pt.ucl, pt.lcl, pt.cl] {
                if scales[1].forward(y).is_some() {
                    min[0] = min[0].min(pt.x);
                    max[0] = max[0].max(pt.x);
                    min[1] = min[1].min(y);
                    max[1] = max[1].max(y);
                    count += 1;
                }
            }
        }
        (count > 0)
            .then(|| PlotBounds::new(min, max).ok())
            .flatten()
    }
}

fn xbar_a2(m: usize) -> f64 {
    match m {
        2 => 1.880,
        3 => 1.023,
        4 => 0.729,
        5 => 0.577,
        6 => 0.483,
        7 => 0.419,
        8 => 0.373,
        9 => 0.337,
        10 => 0.308,
        _ => 3.0 / ((m as f64).sqrt() * 2.0 * (m as f64).ln().max(1.0)),
    }
}

fn apply_rules(pts: &mut [SpcPoint], nelson: bool) {
    for pt in pts.iter_mut() {
        pt.violation = if pt.value > pt.ucl || pt.value < pt.lcl {
            Some(SpcViolationKind::BeyondThreeSigma)
        } else {
            None
        };
    }
    if !nelson || pts.is_empty() {
        return;
    }
    // Nelson Rule 2: 9 consecutive points strictly on the same side of CL.
    let mut above = 0usize;
    let mut below = 0usize;
    for pt in pts.iter_mut() {
        let d = pt.value - pt.cl;
        if d > 0.0 {
            above += 1;
            below = 0;
        } else if d < 0.0 {
            below += 1;
            above = 0;
        } else {
            above = 0;
            below = 0;
        }
        if (above >= 9 || below >= 9) && pt.violation.is_none() {
            pt.violation = Some(SpcViolationKind::NineOneSide);
        }
    }
    // Nelson Rule 3: 6 consecutive strictly increasing or decreasing points.
    let mut inc = 1usize;
    let mut dec = 1usize;
    for i in 1..pts.len() {
        if pts[i].value > pts[i - 1].value {
            inc += 1;
            dec = 1;
        } else if pts[i].value < pts[i - 1].value {
            dec += 1;
            inc = 1;
        } else {
            inc = 1;
            dec = 1;
        }
        if (inc >= 6 || dec >= 6) && pts[i].violation.is_none() {
            pts[i].violation = Some(SpcViolationKind::SixTrend);
        }
    }
}
