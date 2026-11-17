use std::sync::Arc;

use crate::{AxisScale, GridData, GridError, PlotBounds, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionContourError {
    InvalidResolution,
    BudgetExceeded {
        required: usize,
        max_evaluations: usize,
    },
    InvalidGrid(GridError),
}

impl std::fmt::Display for FunctionContourError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidResolution => {
                f.write_str("function sampling resolution must be at least 2x2")
            }
            Self::BudgetExceeded {
                required,
                max_evaluations,
            } => write!(
                f,
                "function sampling requires {required} evaluations, exceeding budget {max_evaluations}"
            ),
            Self::InvalidGrid(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for FunctionContourError {}

impl From<GridError> for FunctionContourError {
    fn from(value: GridError) -> Self {
        Self::InvalidGrid(value)
    }
}

/// Immutable host-owned node-centered rectilinear 2D scalar field (`contour` / `contourf` / `pcolor`).
/// `values` has length `x_nodes.len() * y_nodes.len()` in row-major order (`row * cols + col`).
#[derive(Clone, Debug)]
pub struct ScalarField2D {
    x: Arc<[f64]>,
    y: Arc<[f64]>,
    values: Arc<[f64]>,
    value_ranges: [Option<[f64; 2]>; 2],
    non_finite: usize,
    pcolor_non_finite: usize,
    pcolor_positive: usize,
}

impl ScalarField2D {
    pub fn new(
        x_nodes: impl Into<Arc<[f64]>>,
        y_nodes: impl Into<Arc<[f64]>>,
        values: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        let (x, y, values) = (x_nodes.into(), y_nodes.into(), values.into());
        for nodes in [&x, &y] {
            if nodes.len() < 2 {
                return Err(GridError::EmptyAxis);
            }
            if nodes.iter().any(|v| !v.is_finite())
                || nodes.windows(2).any(|w| w[0] >= w[1])
                || !(nodes[nodes.len() - 1] - nodes[0]).is_finite()
            {
                return Err(GridError::InvalidEdges);
            }
        }
        let count = x
            .len()
            .checked_mul(y.len())
            .ok_or(GridError::CapacityOverflow)?;
        if values.len() != count {
            return Err(GridError::LengthMismatch);
        }
        let mut value_ranges = [None; 2];
        let mut non_finite = 0;
        let mut pcolor_non_finite = 0;
        let mut pcolor_positive = 0;
        let (cols, rows) = (x.len(), y.len());
        for (idx, &v) in values.iter().enumerate() {
            let row = idx / cols;
            let col = idx % cols;
            let in_flat_cell = col + 1 < cols && row + 1 < rows;
            if !v.is_finite() {
                non_finite += 1;
                pcolor_non_finite += usize::from(in_flat_cell);
                continue;
            }
            pcolor_positive += usize::from(in_flat_cell && v > 0.0);
            for (i, range) in value_ranges.iter_mut().enumerate() {
                if i == 1 && v <= 0.0 {
                    continue;
                }
                let [lo, hi] = range.get_or_insert([v; 2]);
                *lo = lo.min(v);
                *hi = hi.max(v);
            }
        }
        Ok(Self {
            x,
            y,
            values,
            value_ranges,
            non_finite,
            pcolor_non_finite,
            pcolor_positive,
        })
    }

    pub fn uniform(
        cols: usize,
        rows: usize,
        bounds: PlotBounds,
        values: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        if cols < 2 || rows < 2 {
            return Err(GridError::EmptyAxis);
        }
        let count = cols.checked_mul(rows).ok_or(GridError::CapacityOverflow)?;
        let values = values.into();
        if values.len() != count {
            return Err(GridError::LengthMismatch);
        }
        let axis_nodes = |axis: usize, len: usize| -> Result<Vec<f64>, GridError> {
            let mut out = Vec::new();
            out.try_reserve_exact(len)
                .map_err(|_| GridError::AllocationFailed)?;
            let denom = (len - 1) as f64;
            for i in 0..len {
                out.push(if i + 1 == len {
                    bounds.max()[axis]
                } else {
                    bounds.min()[axis] + bounds.span()[axis] * (i as f64 / denom)
                });
            }
            Ok(out)
        };
        Self::new(axis_nodes(0, cols)?, axis_nodes(1, rows)?, values)
    }

    pub fn from_grid_centers(grid: &GridData) -> Result<Self, GridError> {
        if grid.columns() < 2 || grid.rows() < 2 {
            return Err(GridError::EmptyAxis);
        }
        let x: Vec<f64> = grid
            .x_edges()
            .windows(2)
            .map(|w| 0.5 * (w[0] + w[1]))
            .collect();
        let y: Vec<f64> = grid
            .y_edges()
            .windows(2)
            .map(|w| 0.5 * (w[0] + w[1]))
            .collect();
        Self::new(x, y, Arc::from(grid.values()))
    }

    /// Sample a bivariate callback over a bounded uniform grid outside UI passes.
    pub fn sample_function(
        bounds: PlotBounds,
        resolution: [usize; 2],
        max_evaluations: usize,
        mut f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionContourError> {
        if resolution[0] < 2 || resolution[1] < 2 {
            return Err(FunctionContourError::InvalidResolution);
        }
        let required = resolution[0].checked_mul(resolution[1]).ok_or(
            FunctionContourError::BudgetExceeded {
                required: usize::MAX,
                max_evaluations,
            },
        )?;
        if required > max_evaluations {
            return Err(FunctionContourError::BudgetExceeded {
                required,
                max_evaluations,
            });
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(required)
            .map_err(|_| GridError::AllocationFailed)?;
        let [nx, ny] = resolution;
        let dx = bounds.span()[0] / (nx - 1) as f64;
        let dy = bounds.span()[1] / (ny - 1) as f64;
        for row in 0..ny {
            let y = if row + 1 == ny {
                bounds.max()[1]
            } else {
                bounds.min()[1] + dy * row as f64
            };
            for col in 0..nx {
                let x = if col + 1 == nx {
                    bounds.max()[0]
                } else {
                    bounds.min()[0] + dx * col as f64
                };
                let z = f(x, y);
                values.push(if z.is_finite() { z } else { f64::NAN });
            }
        }
        Ok(Self::uniform(nx, ny, bounds, values)?)
    }

    /// Legacy `[-2π, 2π]^2` default domain helper (`ezcontour` / `ezcontourf`).
    pub fn ez_sample(
        resolution: [usize; 2],
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionContourError> {
        let tau = std::f64::consts::TAU;
        let bounds = PlotBounds::new([-tau, -tau], [tau, tau]).expect("finite [-2pi, 2pi]");
        Self::sample_function(bounds, resolution, max_evaluations, f)
    }

    pub fn x_nodes(&self) -> &[f64] {
        &self.x
    }
    pub fn y_nodes(&self) -> &[f64] {
        &self.y
    }
    pub fn values(&self) -> &[f64] {
        &self.values
    }
    pub fn cols(&self) -> usize {
        self.x.len()
    }
    pub fn rows(&self) -> usize {
        self.y.len()
    }
    pub fn cell_cols(&self) -> usize {
        self.x.len() - 1
    }
    pub fn cell_rows(&self) -> usize {
        self.y.len() - 1
    }
    pub fn node_value(&self, col: usize, row: usize) -> Option<f64> {
        (col < self.cols() && row < self.rows()).then(|| self.values[row * self.cols() + col])
    }
    pub fn non_finite_nodes(&self) -> usize {
        self.non_finite
    }
    pub fn value_range(&self, scale: AxisScale) -> Option<[f64; 2]> {
        self.value_ranges[usize::from(scale == AxisScale::Log10)]
    }
    pub fn sample_bytes(&self) -> usize {
        (self.x.len() + self.y.len() + self.values.len()) * 8
    }
    pub(crate) fn allocations(&self) -> [(usize, usize); 3] {
        [&self.x, &self.y, &self.values].map(|v| (v.as_ptr() as usize, v.len() * 8))
    }
    pub(crate) fn domain_start(&self, axis: usize, scale: AxisScale) -> usize {
        if scale == AxisScale::Linear {
            return 0;
        }
        let nodes = if axis == 0 { &self.x } else { &self.y };
        nodes.partition_point(|v| *v <= 0.0).min(nodes.len() - 1)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        let sx = self.domain_start(0, scales[0]);
        let sy = self.domain_start(1, scales[1]);
        if sx + 1 >= self.cols() || sy + 1 >= self.rows() {
            return None;
        }
        Some((
            [self.x[sx], self.y[sy]],
            [self.x[self.cols() - 1], self.y[self.rows() - 1]],
        ))
    }
    pub(crate) fn invalid_pcolor_cells(&self, scales: [AxisScale; 2]) -> usize {
        let total = self.cell_cols() * self.cell_rows();
        let vx = self.cols() - 1 - self.domain_start(0, scales[0]);
        let vy = self.rows() - 1 - self.domain_start(1, scales[1]);
        total.saturating_sub(vx * vy)
    }
    pub(crate) fn pcolor_color_invalid_cells(&self, color_scale: AxisScale) -> usize {
        match color_scale {
            AxisScale::Linear => self.pcolor_non_finite,
            AxisScale::Log10 => self.cell_cols() * self.cell_rows() - self.pcolor_positive,
            AxisScale::Probit | AxisScale::Weibull => {
                let mut invalid = 0usize;
                for r in 0..self.cell_rows() {
                    for c in 0..self.cell_cols() {
                        if self
                            .node_value(c, r)
                            .and_then(|v| color_scale.forward(v))
                            .is_none()
                        {
                            invalid += 1;
                        }
                    }
                }
                invalid
            }
        }
    }
    pub(crate) fn pcolor_non_finite_cells(&self) -> usize {
        self.pcolor_non_finite
    }
}
