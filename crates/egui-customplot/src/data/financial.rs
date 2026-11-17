use std::sync::Arc;

use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinancialError {
    Empty,
    LengthMismatch,
    NonFinite,
    InvalidBar { index: usize },
}

impl std::fmt::Display for FinancialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("financial dataset must not be empty"),
            Self::LengthMismatch => f.write_str("input series lengths do not match"),
            Self::NonFinite => f.write_str("financial inputs must be finite"),
            Self::InvalidBar { index } => {
                write!(
                    f,
                    "invalid OHLC bar at index {index}: require width > 0 and low <= min(open, close) <= max(open, close) <= high"
                )
            }
        }
    }
}

impl std::error::Error for FinancialError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OhlcBar {
    pub x: f64,
    pub width: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl OhlcBar {
    pub fn is_up(self) -> bool {
        self.close >= self.open
    }
}

#[derive(Clone, Debug)]
pub struct OhlcData {
    bars: Arc<[OhlcBar]>,
}

impl OhlcData {
    pub fn new(bars: impl Into<Arc<[OhlcBar]>>) -> Result<Self, FinancialError> {
        let bars = bars.into();
        if bars.is_empty() {
            return Err(FinancialError::Empty);
        }
        for (index, b) in bars.iter().enumerate() {
            if ![b.x, b.width, b.open, b.high, b.low, b.close]
                .into_iter()
                .all(f64::is_finite)
                || b.width <= 0.0
                || b.low > b.open.min(b.close)
                || b.high < b.open.max(b.close)
            {
                return Err(FinancialError::InvalidBar { index });
            }
        }
        Ok(Self { bars })
    }

    pub fn from_slices(
        dates: &[f64],
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Result<Self, FinancialError> {
        let n = dates.len();
        if n == 0 {
            return Err(FinancialError::Empty);
        }
        if open.len() != n || high.len() != n || low.len() != n || close.len() != n {
            return Err(FinancialError::LengthMismatch);
        }
        let mut min_dx = f64::INFINITY;
        for w in dates.windows(2) {
            let dx = (w[1] - w[0]).abs();
            if dx.is_finite() && dx > 0.0 {
                min_dx = min_dx.min(dx);
            }
        }
        let width = if min_dx.is_finite() {
            min_dx * 0.62
        } else {
            0.62
        };
        let bars: Vec<OhlcBar> = (0..n)
            .map(|i| OhlcBar {
                x: dates[i],
                width,
                open: open[i],
                high: high[i],
                low: low[i],
                close: close[i],
            })
            .collect();
        Self::new(bars)
    }

    pub fn bars(&self) -> &[OhlcBar] {
        &self.bars
    }
    pub fn len(&self) -> usize {
        self.bars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.bars)
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for &b in self.bars.iter() {
            let x0 = b.x - 0.5 * b.width;
            let x1 = b.x + 0.5 * b.width;
            if scales[0].forward(x0).is_none()
                || scales[0].forward(x1).is_none()
                || scales[1].forward(b.low).is_none()
                || scales[1].forward(b.high).is_none()
            {
                continue;
            }
            min[0] = min[0].min(x0);
            max[0] = max[0].max(x1);
            min[1] = min[1].min(b.low);
            max[1] = max[1].max(b.high);
            count += 1;
        }
        (count > 0)
            .then(|| PlotBounds::new(min, max).ok())
            .flatten()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterfallKind {
    Positive,
    Negative,
    Total,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaterfallColumn {
    pub x: f64,
    pub width: f64,
    pub delta: f64,
    pub start: f64,
    pub end: f64,
    pub kind: WaterfallKind,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct Waterfall2DData {
    columns: Arc<[WaterfallColumn]>,
}

impl Waterfall2DData {
    /// Bridge/cumulative waterfall chart (`start` -> `end`); `is_total[i]` draws from `0` to running sum.
    pub fn from_deltas(
        deltas: &[f64],
        is_total: Option<&[bool]>,
        labels: Option<&[&str]>,
    ) -> Result<Self, FinancialError> {
        if deltas.is_empty() {
            return Err(FinancialError::Empty);
        }
        if is_total.is_some_and(|t| t.len() != deltas.len())
            || labels.is_some_and(|l| l.len() != deltas.len())
        {
            return Err(FinancialError::LengthMismatch);
        }
        if deltas.iter().any(|v| !v.is_finite()) {
            return Err(FinancialError::NonFinite);
        }
        let mut running = 0.0_f64;
        let mut columns = Vec::with_capacity(deltas.len());
        for (i, &delta) in deltas.iter().enumerate() {
            let total_flag = is_total.is_some_and(|t| t[i]);
            let (start, end, kind) = if total_flag {
                (0.0, running, WaterfallKind::Total)
            } else {
                let prev = running;
                running += delta;
                let k = if delta >= 0.0 {
                    WaterfallKind::Positive
                } else {
                    WaterfallKind::Negative
                };
                (prev, running, k)
            };
            let label = labels.map_or_else(|| format!("{}", i + 1), |l| l[i].to_owned());
            columns.push(WaterfallColumn {
                x: (i + 1) as f64,
                width: 0.64,
                delta: if total_flag { running } else { delta },
                start,
                end,
                kind,
                label,
            });
        }
        Ok(Self {
            columns: columns.into(),
        })
    }

    pub fn columns(&self) -> &[WaterfallColumn] {
        &self.columns
    }
    pub fn len(&self) -> usize {
        self.columns.len()
    }
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
    pub fn ticks(&self) -> Vec<(f64, String)> {
        self.columns
            .iter()
            .map(|c| (c.x, c.label.clone()))
            .collect()
    }
    pub fn sample_bytes(&self) -> usize {
        self.columns.len() * std::mem::size_of::<WaterfallColumn>()
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for c in self.columns.iter() {
            let x0 = c.x - 0.5 * c.width;
            let x1 = c.x + 0.5 * c.width;
            let y0 = c.start.min(c.end).min(0.0);
            let y1 = c.start.max(c.end).max(0.0);
            if scales[0].forward(x0).is_none()
                || scales[0].forward(x1).is_none()
                || scales[1].forward(y0).is_none()
                || scales[1].forward(y1).is_none()
            {
                continue;
            }
            min[0] = min[0].min(x0);
            max[0] = max[0].max(x1);
            min[1] = min[1].min(y0);
            max[1] = max[1].max(y1);
            count += 1;
        }
        (count > 0)
            .then(|| PlotBounds::new(min, max).ok())
            .flatten()
    }
}
