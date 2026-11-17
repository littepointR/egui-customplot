use std::sync::Arc;

use crate::{
    AxisScale, PlotBounds, PlotPoint, StatError,
    axis::{norm_cdf_clamped, norm_inv, weibull_forward, weibull_inverse},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbabilityDist {
    Normal,
    Weibull,
    QqNormal,
    QqTwoSample,
}

#[derive(Clone, Debug)]
pub struct ProbabilityPlotData {
    dist: ProbabilityDist,
    points: Arc<[PlotPoint]>,
    ref_line: [PlotPoint; 2],
    param_a: f64,
    param_b: f64,
}

impl ProbabilityPlotData {
    /// Normal probability plot (`normplot` / `probplot('normal', x)`):
    /// X is sorted data value, Y is empirical plotting position `(i + 0.5) / n` on `AxisScale::Probit`.
    pub fn normal(samples: &[f64]) -> Result<Self, StatError> {
        let sorted = validated_sorted(samples)?;
        let n = sorted.len();
        let points: Vec<PlotPoint> = sorted
            .iter()
            .enumerate()
            .map(|(i, &x)| [x, (i as f64 + 0.5) / n as f64])
            .collect();
        let (q1_x, q3_x) = quartiles(&sorted);
        let q1_p = 0.25_f64;
        let q3_p = 0.75_f64;
        let z1 = norm_inv(q1_p).unwrap_or(-0.674_489_75);
        let z3 = norm_inv(q3_p).unwrap_or(0.674_489_75);
        let mean = sorted.iter().sum::<f64>() / n as f64;
        let slope_z = (z3 - z1) / (q3_x - q1_x).max(1e-12);
        let sigma = if (q3_x - q1_x) > 1e-12 {
            (q3_x - q1_x) / (z3 - z1)
        } else {
            sample_std(&sorted, mean)
        };
        let x_lo = sorted[0];
        let x_hi = sorted[n - 1];
        let z_lo = (z1 + slope_z * (x_lo - q1_x)).clamp(-5.5, 5.5);
        let z_hi = (z1 + slope_z * (x_hi - q1_x)).clamp(-5.5, 5.5);
        let ref_line = [
            [x_lo, norm_cdf_clamped(z_lo).unwrap_or(0.001)],
            [x_hi, norm_cdf_clamped(z_hi).unwrap_or(0.999)],
        ];
        Ok(Self {
            dist: ProbabilityDist::Normal,
            points: points.into(),
            ref_line,
            param_a: mean,
            param_b: sigma,
        })
    }

    /// Weibull probability plot (`wblplot` / `probplot('weibull', x)`):
    /// X is positive sample value (`AxisScale::Log10`), Y is probability (`AxisScale::Weibull`).
    pub fn weibull(samples: &[f64]) -> Result<Self, StatError> {
        let sorted = validated_sorted(samples)?;
        if sorted[0] <= 0.0 {
            return Err(StatError::NoFiniteSamples);
        }
        let n = sorted.len();
        let points: Vec<PlotPoint> = sorted
            .iter()
            .enumerate()
            .map(|(i, &x)| [x, (i as f64 + 0.5) / n as f64])
            .collect();
        let (q1_x, q3_x) = quartiles(&sorted);
        let w1 = weibull_forward(0.25).unwrap_or(-1.245_899);
        let w3 = weibull_forward(0.75).unwrap_or(0.326_634);
        let lq1 = q1_x.ln();
        let lq3 = q3_x.ln();
        let shape = if (lq3 - lq1).abs() > 1e-12 {
            (w3 - w1) / (lq3 - lq1)
        } else {
            1.0
        };
        let scale = (lq1 - w1 / shape.max(1e-6)).exp();
        let lx_lo = sorted[0].ln();
        let lx_hi = sorted[n - 1].ln();
        let w_lo = (w1 + shape * (lx_lo - lq1)).clamp(-9.0, 2.5);
        let w_hi = (w1 + shape * (lx_hi - lq1)).clamp(-9.0, 2.5);
        let ref_line = [
            [sorted[0], weibull_inverse(w_lo).unwrap_or(0.001)],
            [sorted[n - 1], weibull_inverse(w_hi).unwrap_or(0.999)],
        ];
        Ok(Self {
            dist: ProbabilityDist::Weibull,
            points: points.into(),
            ref_line,
            param_a: scale,
            param_b: shape,
        })
    }

    /// Standard Normal QQ plot (`qqplot(x)`): X is theoretical `Phi^{-1}((i+0.5)/n)`, Y is `x_(i)`.
    pub fn qq_normal(samples: &[f64]) -> Result<Self, StatError> {
        let sorted = validated_sorted(samples)?;
        let n = sorted.len();
        let mut points = Vec::with_capacity(n);
        for (i, &val) in sorted.iter().enumerate() {
            let p = (i as f64 + 0.5) / n as f64;
            let z = norm_inv(p).unwrap_or(0.0);
            points.push([z, val]);
        }
        let ref_line = linear_qq_ref(&points);
        let mean = sorted.iter().sum::<f64>() / n as f64;
        let sigma = sample_std(&sorted, mean);
        Ok(Self {
            dist: ProbabilityDist::QqNormal,
            points: points.into(),
            ref_line,
            param_a: mean,
            param_b: sigma,
        })
    }

    /// Two-sample QQ plot (`qqplot(x, y)`): interpolates larger sample onto smaller sample quantiles.
    pub fn qq_two_sample(x: &[f64], y: &[f64]) -> Result<Self, StatError> {
        let sx = validated_sorted(x)?;
        let sy = validated_sorted(y)?;
        let m = sx.len().min(sy.len());
        let mut points = Vec::with_capacity(m);
        for i in 0..m {
            let q = if m == 1 {
                0.5
            } else {
                (i as f64 + 0.5) / m as f64
            };
            points.push([quantile_sorted(&sx, q), quantile_sorted(&sy, q)]);
        }
        let ref_line = linear_qq_ref(&points);
        let slope = if (ref_line[1][0] - ref_line[0][0]).abs() > 1e-12 {
            (ref_line[1][1] - ref_line[0][1]) / (ref_line[1][0] - ref_line[0][0])
        } else {
            1.0
        };
        let intercept = ref_line[0][1] - slope * ref_line[0][0];
        Ok(Self {
            dist: ProbabilityDist::QqTwoSample,
            points: points.into(),
            ref_line,
            param_a: intercept,
            param_b: slope,
        })
    }

    pub fn dist(&self) -> ProbabilityDist {
        self.dist
    }
    pub fn points(&self) -> &[PlotPoint] {
        &self.points
    }
    pub fn ref_line(&self) -> [PlotPoint; 2] {
        self.ref_line
    }
    /// Mean (Normal / QqNormal), scale `lambda` (Weibull), or intercept (QqTwoSample).
    pub fn location_or_scale(&self) -> f64 {
        self.param_a
    }
    /// Stddev (Normal / QqNormal), shape `k` (Weibull), or slope (QqTwoSample).
    pub fn scale_or_shape(&self) -> f64 {
        self.param_b
    }
    pub fn recommended_scales(&self) -> [AxisScale; 2] {
        match self.dist {
            ProbabilityDist::Normal => [AxisScale::Linear, AxisScale::Probit],
            ProbabilityDist::Weibull => [AxisScale::Log10, AxisScale::Weibull],
            ProbabilityDist::QqNormal | ProbabilityDist::QqTwoSample => {
                [AxisScale::Linear, AxisScale::Linear]
            }
        }
    }
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.points)
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for pt in self.points.iter().chain(self.ref_line.iter()) {
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
        self.points
            .iter()
            .filter(|pt| scales[0].forward(pt[0]).is_none() || scales[1].forward(pt[1]).is_none())
            .count()
    }
}

fn validated_sorted(samples: &[f64]) -> Result<Vec<f64>, StatError> {
    if samples.is_empty() {
        return Err(StatError::EmptyInput);
    }
    let mut vals: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
    if vals.is_empty() {
        return Err(StatError::NoFiniteSamples);
    }
    vals.sort_by(f64::total_cmp);
    Ok(vals)
}

fn quantile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let pos = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let t = pos - lo as f64;
    sorted[lo] * (1.0 - t) + sorted[hi] * t
}

fn quartiles(sorted: &[f64]) -> (f64, f64) {
    (quantile_sorted(sorted, 0.25), quantile_sorted(sorted, 0.75))
}

fn sample_std(sorted: &[f64], mean: f64) -> f64 {
    if sorted.len() <= 1 {
        return 0.0;
    }
    let var = sorted
        .iter()
        .map(|&v| {
            let d = v - mean;
            d * d
        })
        .sum::<f64>()
        / (sorted.len() - 1) as f64;
    var.sqrt()
}

fn linear_qq_ref(points: &[PlotPoint]) -> [PlotPoint; 2] {
    let n = points.len();
    if n < 2 {
        let p = points[0];
        return [p, p];
    }
    let i1 = ((n - 1) as f64 * 0.25).round() as usize;
    let i3 = ((n - 1) as f64 * 0.75).round() as usize;
    let p1 = points[i1];
    let p3 = points[i3.max(i1 + 1).min(n - 1)];
    let dx = p3[0] - p1[0];
    let slope = if dx.abs() > 1e-12 {
        (p3[1] - p1[1]) / dx
    } else {
        1.0
    };
    let x0 = points[0][0];
    let x1 = points[n - 1][0];
    [
        [x0, p1[1] + slope * (x0 - p1[0])],
        [x1, p1[1] + slope * (x1 - p1[0])],
    ]
}
