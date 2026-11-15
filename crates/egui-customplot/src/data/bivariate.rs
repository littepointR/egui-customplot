use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, GridData, GridError, PlotBounds, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BivariateNormalization {
    Count,
    Probability,
    Density,
    CumulativeCount,
    CumulativeProbability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinShape {
    Rectangle,
    Hexagon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BivariateError {
    EmptyData,
    LengthMismatch,
    ZeroBins,
    InvalidEdges,
}

impl std::fmt::Display for BivariateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bivariate data: {self:?}")
    }
}
impl std::error::Error for BivariateError {}

fn uniform_edges(lo: f64, hi: f64, bins: usize) -> Vec<f64> {
    let (a, b) = if (hi - lo).abs() <= f64::EPSILON {
        (lo - 0.5, hi + 0.5)
    } else {
        (lo, hi)
    };
    let step = (b - a) / bins as f64;
    (0..=bins).map(|i| a + step * i as f64).collect()
}

fn find_bin(edges: &[f64], value: f64) -> Option<usize> {
    let n = edges.len() - 1;
    if value < edges[0] || value > edges[n] {
        return None;
    }
    if value == edges[n] {
        return Some(n - 1);
    }
    let idx = edges.partition_point(|&edge| edge <= value);
    Some(idx.saturating_sub(1).min(n - 1))
}

/// Immutable 2D histogram binning `(x, y)` into half-open rectilinear bins (`ML-022` `histogram2`).
#[derive(Clone, Debug)]
pub struct Histogram2Data {
    x_edges: Arc<[f64]>,
    y_edges: Arc<[f64]>,
    counts: Arc<[u64]>,
    included_samples: usize,
    non_finite_samples: usize,
    outside_samples: usize,
    max_count: u64,
}

impl Histogram2Data {
    pub fn new(x: &[f64], y: &[f64], nx: usize, ny: usize) -> Result<Self, BivariateError> {
        if nx == 0 || ny == 0 {
            return Err(BivariateError::ZeroBins);
        }
        if x.is_empty() {
            return Err(BivariateError::EmptyData);
        }
        if x.len() != y.len() {
            return Err(BivariateError::LengthMismatch);
        }
        let (mut x_lo, mut x_hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut y_lo, mut y_hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let mut finite = 0usize;
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            if xi.is_finite() && yi.is_finite() {
                x_lo = x_lo.min(xi);
                x_hi = x_hi.max(xi);
                y_lo = y_lo.min(yi);
                y_hi = y_hi.max(yi);
                finite += 1;
            }
        }
        if finite == 0 {
            (x_lo, x_hi) = (0.0, 1.0);
            (y_lo, y_hi) = (0.0, 1.0);
        }
        Self::with_edges(
            x,
            y,
            uniform_edges(x_lo, x_hi, nx),
            uniform_edges(y_lo, y_hi, ny),
        )
    }

    pub fn with_edges(
        x: &[f64],
        y: &[f64],
        x_edges: Vec<f64>,
        y_edges: Vec<f64>,
    ) -> Result<Self, BivariateError> {
        if x.len() != y.len() {
            return Err(BivariateError::LengthMismatch);
        }
        for edges in [&x_edges, &y_edges] {
            if edges.len() < 2
                || edges.iter().any(|e| !e.is_finite())
                || edges.windows(2).any(|w| w[0] >= w[1])
            {
                return Err(BivariateError::InvalidEdges);
            }
        }
        let nx = x_edges.len() - 1;
        let ny = y_edges.len() - 1;
        let total = nx.checked_mul(ny).ok_or(BivariateError::InvalidEdges)?;
        let mut counts = vec![0u64; total];
        let mut included = 0usize;
        let mut non_finite = 0usize;
        let mut outside = 0usize;
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            if !xi.is_finite() || !yi.is_finite() {
                non_finite += 1;
                continue;
            }
            let (Some(ix), Some(iy)) = (find_bin(&x_edges, xi), find_bin(&y_edges, yi)) else {
                outside += 1;
                continue;
            };
            counts[iy * nx + ix] += 1;
            included += 1;
        }
        let max_count = counts.iter().copied().max().unwrap_or(0);
        Ok(Self {
            x_edges: x_edges.into(),
            y_edges: y_edges.into(),
            counts: counts.into(),
            included_samples: included,
            non_finite_samples: non_finite,
            outside_samples: outside,
            max_count,
        })
    }

    pub fn nx(&self) -> usize {
        self.x_edges.len() - 1
    }

    pub fn ny(&self) -> usize {
        self.y_edges.len() - 1
    }

    pub fn x_edges(&self) -> &[f64] {
        &self.x_edges
    }

    pub fn y_edges(&self) -> &[f64] {
        &self.y_edges
    }

    pub fn counts(&self) -> &[u64] {
        &self.counts
    }

    pub fn included_samples(&self) -> usize {
        self.included_samples
    }

    pub fn non_finite_samples(&self) -> usize {
        self.non_finite_samples
    }

    pub fn outside_samples(&self) -> usize {
        self.outside_samples
    }

    pub fn max_count(&self) -> u64 {
        self.max_count
    }

    pub fn to_grid(&self, norm: BivariateNormalization) -> Result<GridData, GridError> {
        let nx = self.nx();
        let ny = self.ny();
        let total = self.included_samples.max(1) as f64;
        let mut values = vec![0.0_f64; nx * ny];
        match norm {
            BivariateNormalization::Count => {
                for (i, &c) in self.counts.iter().enumerate() {
                    values[i] = c as f64;
                }
            }
            BivariateNormalization::Probability => {
                for (i, &c) in self.counts.iter().enumerate() {
                    values[i] = c as f64 / total;
                }
            }
            BivariateNormalization::Density => {
                for iy in 0..ny {
                    let dy = self.y_edges[iy + 1] - self.y_edges[iy];
                    for ix in 0..nx {
                        let dx = self.x_edges[ix + 1] - self.x_edges[ix];
                        let area = dx * dy;
                        let idx = iy * nx + ix;
                        values[idx] = if area > 0.0 {
                            self.counts[idx] as f64 / (total * area)
                        } else {
                            0.0
                        };
                    }
                }
            }
            BivariateNormalization::CumulativeCount
            | BivariateNormalization::CumulativeProbability => {
                let scale = if norm == BivariateNormalization::CumulativeProbability {
                    1.0 / total
                } else {
                    1.0
                };
                for iy in 0..ny {
                    let mut row_sum = 0u64;
                    for ix in 0..nx {
                        row_sum += self.counts[iy * nx + ix];
                        let above = if iy > 0 {
                            values[(iy - 1) * nx + ix]
                        } else {
                            0.0
                        };
                        values[iy * nx + ix] = above + row_sum as f64 * scale;
                    }
                }
            }
        }
        GridData::new(self.x_edges.clone(), self.y_edges.clone(), values)
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let lo = [self.x_edges[0], self.y_edges[0]];
        let hi = [
            self.x_edges[self.x_edges.len() - 1],
            self.y_edges[self.y_edges.len() - 1],
        ];
        if scales[0].forward(lo[0]).is_none()
            || scales[1].forward(lo[1]).is_none()
            || scales[0].forward(hi[0]).is_none()
            || scales[1].forward(hi[1]).is_none()
        {
            return None;
        }
        PlotBounds::new(lo, hi).ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BinCell {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub cx: f64,
    pub cy: f64,
    pub count: u64,
}

/// Non-empty 2D density bins for large scatter clouds (`ML-019` `binscatter`).
#[derive(Clone, Debug)]
pub struct BinScatterData {
    cells: Arc<[BinCell]>,
    shape: BinShape,
    max_count: u64,
    total_samples: usize,
    non_finite_samples: usize,
    extents: CachedExtents,
}

impl BinScatterData {
    pub fn new(
        points: &[PlotPoint],
        nx: usize,
        ny: usize,
        shape: BinShape,
    ) -> Result<Self, BivariateError> {
        if nx == 0 || ny == 0 {
            return Err(BivariateError::ZeroBins);
        }
        if points.is_empty() {
            return Err(BivariateError::EmptyData);
        }
        let mut xs = Vec::with_capacity(points.len());
        let mut ys = Vec::with_capacity(points.len());
        let mut non_finite = 0usize;
        for &[px, py] in points {
            if px.is_finite() && py.is_finite() {
                xs.push(px);
                ys.push(py);
            } else {
                non_finite += 1;
            }
        }
        let total_samples = xs.len();
        if total_samples == 0 {
            return Ok(Self {
                cells: Arc::from([]),
                shape,
                max_count: 0,
                total_samples: 0,
                non_finite_samples: non_finite,
                extents: CachedExtents::default(),
            });
        }
        let hist = Histogram2Data::new(&xs, &ys, nx, ny)?;
        let dx = (hist.x_edges[nx] - hist.x_edges[0]) / nx as f64;
        let mut cells = Vec::new();
        let mut extents = CachedExtents::default();
        for iy in 0..ny {
            let y0 = hist.y_edges[iy];
            let y1 = hist.y_edges[iy + 1];
            let x_shift = if shape == BinShape::Hexagon && iy % 2 == 1 {
                dx * 0.5
            } else {
                0.0
            };
            for ix in 0..nx {
                let count = hist.counts[iy * nx + ix];
                if count == 0 {
                    continue;
                }
                let x0 = hist.x_edges[ix] + x_shift;
                let x1 = hist.x_edges[ix + 1] + x_shift;
                let cell = BinCell {
                    x0,
                    y0,
                    x1,
                    y1,
                    cx: 0.5 * (x0 + x1),
                    cy: 0.5 * (y0 + y1),
                    count,
                };
                extents.push(Some(([x0, y0], [x1, y1])));
                cells.push(cell);
            }
        }
        Ok(Self {
            cells: cells.into(),
            shape,
            max_count: hist.max_count,
            total_samples,
            non_finite_samples: non_finite,
            extents,
        })
    }

    pub fn cells(&self) -> &[BinCell] {
        &self.cells
    }

    pub fn shape(&self) -> BinShape {
        self.shape
    }

    pub fn max_count(&self) -> u64 {
        self.max_count
    }

    pub fn total_samples(&self) -> usize {
        self.total_samples
    }

    pub fn non_finite_samples(&self) -> usize {
        self.non_finite_samples
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }
}
