use std::{fmt, sync::Arc};

use super::AxisScale;

#[derive(Clone, Debug)]
pub struct AxisTick {
    pub value: f64,
    pub label: String,
    pub major: bool,
}

/// Axis-local options. Custom formatters run only for visible major ticks and
/// outside Context locks. Keep expensive configuration in the application model.
#[derive(Clone)]
pub struct Axis {
    pub(crate) scale: AxisScale,
    pub(crate) reversed: bool,
    pub(crate) label: String,
    pub(crate) minor_grid: bool,
    max_ticks: usize,
    formatter: Option<Arc<dyn Fn(f64) -> String + Send + Sync>>,
}

impl fmt::Debug for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Axis")
            .field("scale", &self.scale)
            .field("reversed", &self.reversed)
            .field("label", &self.label)
            .field("max_ticks", &self.max_ticks)
            .finish_non_exhaustive()
    }
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            scale: AxisScale::Linear,
            reversed: false,
            label: String::new(),
            minor_grid: false,
            max_ticks: 8,
            formatter: None,
        }
    }
}

impl Axis {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn scale(mut self, scale: AxisScale) -> Self {
        self.scale = scale;
        self
    }
    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    pub fn minor_grid(mut self, show: bool) -> Self {
        self.minor_grid = show;
        self
    }
    pub fn max_ticks(mut self, count: usize) -> Self {
        self.max_ticks = count.clamp(2, 64);
        self
    }
    pub fn formatter(mut self, formatter: impl Fn(f64) -> String + Send + Sync + 'static) -> Self {
        self.formatter = Some(Arc::new(formatter));
        self
    }

    /// Ascending data values, independent of axis direction. Bounded work even
    /// for extreme ranges; the caller positions these using PlotTransform.
    pub fn ticks(&self, range: [f64; 2], pixels: f32) -> Vec<AxisTick> {
        let [min, max] = range;
        if self.scale.forward(min).is_none() || self.scale.forward(max).is_none() || max <= min {
            return Vec::new();
        }
        let count = ((pixels / 80.0) as usize).clamp(2, self.max_ticks);
        let log_span = if self.scale == AxisScale::Log10 {
            max.log10() - min.log10()
        } else {
            0.0
        };
        let (mut ticks, step) = if log_span >= 1.0 {
            let step = (log_span / (count - 1) as f64).ceil().max(1.0) as i32;
            let start = min.log10().floor() as i32;
            let end = max.log10().ceil() as i32;
            let mut ticks = Vec::new();
            for exponent in start..=end {
                let base = 10.0_f64.powi(exponent);
                let major = exponent.rem_euclid(step) == 0;
                if major && base >= min && base <= max && base.is_finite() && base > 0.0 {
                    ticks.push(AxisTick {
                        value: base,
                        label: String::new(),
                        major: true,
                    });
                }
                if self.minor_grid && step == 1 {
                    for multiplier in 2..10 {
                        let value = base * f64::from(multiplier);
                        if value >= min && value <= max && value.is_finite() && value > 0.0 {
                            ticks.push(AxisTick {
                                value,
                                label: String::new(),
                                major: false,
                            });
                        }
                    }
                }
            }
            (ticks, 0.0)
        } else {
            let span = max - min;
            if !span.is_finite() {
                return Vec::new();
            }
            let raw = (span / (count - 1) as f64).max(f64::from_bits(1));
            let magnitude = 10.0_f64.powf(raw.log10().floor());
            let fraction = raw / magnitude;
            let nice = [1.0, 2.0, 2.5, 5.0, 10.0]
                .into_iter()
                .find(|n| *n >= fraction)
                .unwrap_or(10.0);
            let step = (nice * magnitude).max(f64::from_bits(1));
            if !step.is_finite() {
                return Vec::new();
            }
            let start = (min / step).ceil() * step;
            let mut ticks = Vec::new();
            let mut previous = f64::NEG_INFINITY;
            for index in 0..=128 {
                let value = (index as f64).mul_add(step, start);
                if value > max {
                    break;
                }
                if value < min || !value.is_finite() || value <= previous {
                    continue;
                }
                previous = value;
                ticks.push(AxisTick {
                    value,
                    label: String::new(),
                    major: true,
                });
            }
            (ticks, step)
        };
        ticks.sort_by(|a, b| a.value.total_cmp(&b.value));
        for tick in &mut ticks {
            if tick.major {
                tick.label = self
                    .formatter
                    .as_ref()
                    .map_or_else(|| format_tick(tick.value, step), |f| f(tick.value));
            }
        }
        ticks
    }
}

pub(crate) fn format_tick(value: f64, step: f64) -> String {
    let value = if step > 0.0 && value.abs() < step * 1e-8 {
        0.0
    } else {
        value
    };
    let magnitude = value.abs();
    if magnitude != 0.0 && !(0.0001..10_000_000.0).contains(&magnitude) {
        let precision = if step > 0.0 {
            (magnitude.log10().floor() - step.log10().floor() + 1.0).clamp(0.0, 16.0) as usize
        } else {
            0
        };
        format!("{value:.precision$e}")
    } else {
        let precision = if step > 0.0 {
            (-step.log10().floor() + 1.0).clamp(0.0, 16.0) as usize
        } else {
            6
        };
        let s = format!("{value:.precision$}");
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nice_ticks_are_bounded_ordered_and_distinct() {
        for range in [
            [-1.1, 1.1],
            [1e12, 1e12 + 1.0],
            [0.0, 1e-300],
            [-1e300, 1e300],
            [1.0, 1.0 + 1e-14],
        ] {
            let ticks = Axis::default().ticks(range, 800.0);
            assert!(!ticks.is_empty(), "{range:?}");
            assert!(ticks.len() <= 129);
            for pair in ticks.windows(2) {
                assert!(pair[0].value < pair[1].value);
                assert_ne!(pair[0].label, pair[1].label);
            }
            assert!(
                ticks
                    .iter()
                    .all(|t| t.value >= range[0] && t.value <= range[1])
            );
        }
    }
    #[test]
    fn log_ticks_and_minor_grid_use_real_positive_values() {
        let ticks = Axis::new()
            .scale(AxisScale::Log10)
            .minor_grid(true)
            .ticks([0.1, 1000.0], 800.0);
        assert_eq!(
            ticks
                .iter()
                .filter(|t| t.major)
                .map(|t| t.value)
                .collect::<Vec<_>>(),
            vec![0.1, 1.0, 10.0, 100.0, 1000.0]
        );
        assert!(ticks.iter().any(|t| !t.major && t.value == 2.0));
        assert!(
            Axis::new()
                .scale(AxisScale::Log10)
                .ticks([-1.0, 1.0], 800.0)
                .is_empty()
        );
    }
    #[test]
    fn formatter_only_runs_for_major_ticks() {
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls = count.clone();
        let axis = Axis::new()
            .scale(AxisScale::Log10)
            .minor_grid(true)
            .formatter(move |v| {
                calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                format!("{v} Hz")
            });
        let ticks = axis.ticks([1.0, 100.0], 800.0);
        assert_eq!(
            count.load(std::sync::atomic::Ordering::Relaxed),
            ticks.iter().filter(|t| t.major).count()
        );
        assert!(
            ticks
                .iter()
                .filter(|t| t.major)
                .all(|t| t.label.ends_with(" Hz"))
        );
    }
}
