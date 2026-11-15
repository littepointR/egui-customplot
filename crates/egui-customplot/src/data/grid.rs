use std::sync::Arc;

use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridError {
    EmptyAxis,
    InvalidEdges,
    LengthMismatch,
    CapacityOverflow,
    AllocationFailed,
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid heatmap grid: {self:?}")
    }
}
impl std::error::Error for GridError {}

/// Immutable rectilinear cells, with explicit edges and row-major scalar values.
/// Row zero occupies the smallest Y interval; reverse the axis for image rows.
/// NaN/Inf values are missing cells. Clones share all three sample allocations.
#[derive(Clone, Debug)]
pub struct GridData {
    x: Arc<[f64]>,
    y: Arc<[f64]>,
    values: Arc<[f64]>,
    value_ranges: [Option<[f64; 2]>; 2],
    non_finite: usize,
    positive: usize,
}

impl GridData {
    pub fn new(
        x_edges: impl Into<Arc<[f64]>>,
        y_edges: impl Into<Arc<[f64]>>,
        values: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        let (x, y, values) = (x_edges.into(), y_edges.into(), values.into());
        for edges in [&x, &y] {
            if edges.len() < 2 {
                return Err(GridError::EmptyAxis);
            }
            if edges.iter().any(|v| !v.is_finite())
                || edges.windows(2).any(|e| e[0] >= e[1])
                || !(edges[edges.len() - 1] - edges[0]).is_finite()
            {
                return Err(GridError::InvalidEdges);
            }
        }
        let count = (x.len() - 1)
            .checked_mul(y.len() - 1)
            .ok_or(GridError::CapacityOverflow)?;
        if count != values.len() {
            return Err(GridError::LengthMismatch);
        }
        let mut value_ranges = [None; 2];
        let mut non_finite = 0;
        let mut positive = 0;
        for &value in values.iter() {
            if !value.is_finite() {
                non_finite += 1;
                continue;
            }
            positive += usize::from(value > 0.0);
            for (i, range) in value_ranges.iter_mut().enumerate() {
                if i == 1 && value <= 0.0 {
                    continue;
                }
                let [lo, hi] = range.get_or_insert([value; 2]);
                *lo = lo.min(value);
                *hi = hi.max(value);
            }
        }
        Ok(Self {
            x,
            y,
            values,
            value_ranges,
            non_finite,
            positive,
        })
    }

    /// Uniform edges in data space, not logarithmic space. Rejects rounded,
    /// collapsed cell edges rather than silently reusing their coordinates.
    pub fn uniform(
        columns: usize,
        rows: usize,
        bounds: PlotBounds,
        values: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        if columns == 0 || rows == 0 {
            return Err(GridError::EmptyAxis);
        }
        let count = columns
            .checked_mul(rows)
            .ok_or(GridError::CapacityOverflow)?;
        let values = values.into();
        if values.len() != count {
            return Err(GridError::LengthMismatch);
        }
        let edges = |axis: usize, cells: usize| -> Result<Vec<f64>, GridError> {
            let len = cells.checked_add(1).ok_or(GridError::CapacityOverflow)?;
            let mut result = Vec::new();
            result
                .try_reserve_exact(len)
                .map_err(|_| GridError::AllocationFailed)?;
            for i in 0..=cells {
                result.push(if i == cells {
                    bounds.max()[axis]
                } else {
                    bounds.min()[axis] + bounds.span()[axis] * (i as f64 / cells as f64)
                });
            }
            Ok(result)
        };
        Self::new(edges(0, columns)?, edges(1, rows)?, values)
    }

    pub fn x_edges(&self) -> &[f64] {
        &self.x
    }
    pub fn y_edges(&self) -> &[f64] {
        &self.y
    }
    pub fn values(&self) -> &[f64] {
        &self.values
    }
    pub fn columns(&self) -> usize {
        self.x.len() - 1
    }
    pub fn rows(&self) -> usize {
        self.y.len() - 1
    }
    pub fn non_finite_cells(&self) -> usize {
        self.non_finite
    }
    pub(crate) fn invalid_color_cells(&self, scale: AxisScale) -> usize {
        if scale == AxisScale::Log10 {
            self.values.len() - self.positive
        } else {
            self.non_finite
        }
    }
    pub fn value_range(&self, scale: AxisScale) -> Option<[f64; 2]> {
        self.value_ranges[usize::from(scale == AxisScale::Log10)]
    }
    pub fn sample_bytes(&self) -> usize {
        (self.x.len() + self.y.len() + self.values.len()) * 8
    }
    pub fn value(&self, column: usize, row: usize) -> Option<f64> {
        if column >= self.columns() || row >= self.rows() {
            return None;
        }
        Some(self.values[row * self.columns() + column])
    }
    pub fn cell_bounds(&self, column: usize, row: usize) -> Option<PlotBounds> {
        self.value(column, row)?;
        PlotBounds::new(
            [self.x[column], self.y[row]],
            [self.x[column + 1], self.y[row + 1]],
        )
        .ok()
    }

    pub(crate) fn allocations(&self) -> [(usize, usize); 3] {
        [&self.x, &self.y, &self.values].map(|v| (v.as_ptr() as usize, v.len() * 8))
    }
    pub(crate) fn domain_start(&self, axis: usize, scale: AxisScale) -> usize {
        if scale == AxisScale::Linear {
            return 0;
        }
        let edges = if axis == 0 { &self.x } else { &self.y };
        edges.partition_point(|v| *v <= 0.0).min(edges.len() - 1)
    }
    /// A cell crossing the nonpositive log domain is omitted as a whole.
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<([f64; 2], [f64; 2])> {
        let start = [
            self.domain_start(0, scales[0]),
            self.domain_start(1, scales[1]),
        ];
        if start[0] == self.columns() || start[1] == self.rows() {
            return None;
        }
        Some((
            [self.x[start[0]], self.y[start[1]]],
            [self.x[self.columns()], self.y[self.rows()]],
        ))
    }
    pub(crate) fn invalid_domain_cells(&self, scales: [AxisScale; 2]) -> usize {
        self.values.len()
            - (self.columns() - self.domain_start(0, scales[0]))
                * (self.rows() - self.domain_start(1, scales[1]))
    }
}
