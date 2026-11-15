use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct BoxSummary {
    pub position: f64,
    pub width: f64,
    pub count: usize,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub mean: f64,
    pub whisker_low: f64,
    pub whisker_high: f64,
    pub notch_low: f64,
    pub notch_high: f64,
    pub outliers: Arc<[f64]>,
}

pub(crate) fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let t = idx - lo as f64;
    sorted[lo] * (1.0 - t) + sorted[hi] * t
}

pub(crate) fn compute_box_stats(
    position: f64,
    values: &[f64],
    width: f64,
) -> Option<(BoxSummary, Vec<f64>)> {
    if !position.is_finite() || !width.is_finite() || width <= 0.0 {
        return None;
    }
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let q1 = percentile(&sorted, 0.25);
    let median = percentile(&sorted, 0.50);
    let q3 = percentile(&sorted, 0.75);
    let iqr = (q3 - q1).max(0.0);
    let low_fence = q1 - 1.5 * iqr;
    let high_fence = q3 + 1.5 * iqr;

    let mut whisker_low = sorted[sorted.len() - 1];
    let mut whisker_high = sorted[0];
    let mut outliers = Vec::new();
    let mut sum = 0.0;
    for &v in &sorted {
        sum += v;
        if v < low_fence || v > high_fence {
            outliers.push(v);
        } else {
            whisker_low = whisker_low.min(v);
            whisker_high = whisker_high.max(v);
        }
    }
    if whisker_low > whisker_high {
        whisker_low = q1;
        whisker_high = q3;
    }
    let half_notch = 1.57 * iqr / (sorted.len() as f64).sqrt();
    let summary = BoxSummary {
        position,
        width,
        count: sorted.len(),
        q1,
        median,
        q3,
        mean: sum / sorted.len() as f64,
        whisker_low,
        whisker_high,
        notch_low: median - half_notch,
        notch_high: median + half_notch,
        outliers: outliers.into(),
    };
    Some((summary, sorted))
}

pub(crate) fn gaussian_kde(
    sorted: &[f64],
    samples: usize,
    bandwidth: Option<f64>,
) -> (Arc<[f64]>, Arc<[f64]>) {
    let m = samples.clamp(16, 512);
    if sorted.is_empty() {
        return (Arc::from([]), Arc::from([]));
    }
    let n = sorted.len() as f64;
    let mean = sorted.iter().sum::<f64>() / n;
    let var = sorted.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n.max(1.0);
    let std = var.sqrt();
    let iqr = (percentile(sorted, 0.75) - percentile(sorted, 0.25)) / 1.34;
    let span = (sorted[sorted.len() - 1] - sorted[0]).max(0.0);

    let sigma = if std > 0.0 && iqr > 0.0 {
        std.min(iqr)
    } else if std > 0.0 {
        std
    } else if iqr > 0.0 {
        iqr
    } else {
        (span * 0.25).max(0.1)
    };
    let h = bandwidth
        .filter(|b| b.is_finite() && *b > 0.0)
        .unwrap_or_else(|| (0.9 * sigma * n.powf(-0.2)).max(1e-6));

    let lo = sorted[0] - 2.5 * h;
    let hi = sorted[sorted.len() - 1] + 2.5 * h;
    let step = (hi - lo) / (m - 1) as f64;
    let ys: Vec<f64> = (0..m).map(|i| lo + step * i as f64).collect();
    let mut density = vec![0.0_f64; m];
    let inv_norm = 1.0 / (h * (2.0 * std::f64::consts::PI).sqrt());

    if sorted.len() <= 1024 {
        for (k, &y) in ys.iter().enumerate() {
            let mut acc = 0.0;
            for &s in sorted {
                let z = (y - s) / h;
                acc += (-0.5 * z * z).exp();
            }
            density[k] = (acc / n) * inv_norm;
        }
    } else {
        let bins = 256usize;
        let b_lo = sorted[0];
        let b_span = (sorted[sorted.len() - 1] - b_lo).max(1e-12);
        let b_step = b_span / (bins - 1) as f64;
        let mut weights = vec![0.0_f64; bins];
        for &s in sorted {
            let idx = (((s - b_lo) / b_span) * (bins - 1) as f64)
                .round()
                .clamp(0.0, (bins - 1) as f64) as usize;
            weights[idx] += 1.0 / n;
        }
        for (k, &y) in ys.iter().enumerate() {
            let mut acc = 0.0;
            for (b, &w) in weights.iter().enumerate() {
                if w == 0.0 {
                    continue;
                }
                let center = b_lo + b as f64 * b_step;
                let z = (y - center) / h;
                acc += w * (-0.5 * z * z).exp();
            }
            density[k] = acc * inv_norm;
        }
    }
    (ys.into(), density.into())
}
