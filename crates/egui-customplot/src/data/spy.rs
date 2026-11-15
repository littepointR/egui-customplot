use std::sync::Arc;

use crate::{AxisScale, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpyError {
    ZeroDimensions,
    IndexOutOfBounds {
        row: usize,
        col: usize,
        rows: usize,
        cols: usize,
    },
    InvalidCsr,
    InvalidDenseLength,
    InvalidTolerance,
}

impl std::fmt::Display for SpyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimensions => f.write_str("matrix rows and cols must be non-zero"),
            Self::IndexOutOfBounds {
                row,
                col,
                rows,
                cols,
            } => write!(
                f,
                "entry ({row}, {col}) exceeds {rows}x{cols} matrix bounds"
            ),
            Self::InvalidCsr => f.write_str("invalid CSR row_offsets/col_indices arrays"),
            Self::InvalidDenseLength => f.write_str("dense matrix length must equal rows * cols"),
            Self::InvalidTolerance => f.write_str("spy zero tolerance must be finite and >= 0"),
        }
    }
}

impl std::error::Error for SpyError {}

/// Immutable host-owned CSR sparsity pattern (`spy` / `ML-020`), mapping 0-based
/// `(row, col)` to 1-based data coordinates `[col + 1, row + 1]`.
#[derive(Clone, Debug)]
pub struct SpyData {
    rows: usize,
    cols: usize,
    row_offsets: Arc<[u32]>,
    col_indices: Arc<[u32]>,
    duplicate_entries: usize,
}

impl SpyData {
    pub fn from_coords(
        rows: usize,
        cols: usize,
        entries: &[(usize, usize)],
    ) -> Result<Self, SpyError> {
        if rows == 0 || cols == 0 {
            return Err(SpyError::ZeroDimensions);
        }
        let mut counts = vec![0usize; rows];
        for &(r, c) in entries {
            if r >= rows || c >= cols {
                return Err(SpyError::IndexOutOfBounds {
                    row: r,
                    col: c,
                    rows,
                    cols,
                });
            }
            counts[r] += 1;
        }
        let mut offsets = Vec::with_capacity(rows + 1);
        offsets.push(0u32);
        let mut acc = 0usize;
        for c in counts {
            acc += c;
            offsets.push(u32::try_from(acc).map_err(|_| SpyError::InvalidCsr)?);
        }
        let mut write_pos = offsets[..rows].to_vec();
        let mut cols_buf = vec![0u32; acc];
        for &(r, c) in entries {
            let slot = write_pos[r] as usize;
            cols_buf[slot] = c as u32;
            write_pos[r] += 1;
        }
        Self::canonicalize(rows, cols, offsets, cols_buf)
    }

    pub fn from_csr(
        rows: usize,
        cols: usize,
        row_offsets: &[usize],
        col_indices: &[usize],
    ) -> Result<Self, SpyError> {
        if rows == 0 || cols == 0 {
            return Err(SpyError::ZeroDimensions);
        }
        if row_offsets.len() != rows + 1
            || row_offsets[0] != 0
            || row_offsets[rows] != col_indices.len()
            || row_offsets.windows(2).any(|w| w[1] < w[0])
        {
            return Err(SpyError::InvalidCsr);
        }
        for r in 0..rows {
            for &c in &col_indices[row_offsets[r]..row_offsets[r + 1]] {
                if c >= cols {
                    return Err(SpyError::IndexOutOfBounds {
                        row: r,
                        col: c,
                        rows,
                        cols,
                    });
                }
            }
        }
        let offsets: Vec<u32> = row_offsets
            .iter()
            .map(|&v| u32::try_from(v).map_err(|_| SpyError::InvalidCsr))
            .collect::<Result<_, _>>()?;
        let cols_buf: Vec<u32> = col_indices.iter().map(|&c| c as u32).collect();
        Self::canonicalize(rows, cols, offsets, cols_buf)
    }

    pub fn from_dense(
        rows: usize,
        cols: usize,
        values: &[f64],
        eps: f64,
    ) -> Result<Self, SpyError> {
        if rows == 0 || cols == 0 {
            return Err(SpyError::ZeroDimensions);
        }
        if !eps.is_finite() || eps < 0.0 {
            return Err(SpyError::InvalidTolerance);
        }
        let expected = rows.checked_mul(cols).ok_or(SpyError::InvalidDenseLength)?;
        if values.len() != expected {
            return Err(SpyError::InvalidDenseLength);
        }
        let mut offsets = Vec::with_capacity(rows + 1);
        let mut cols_buf = Vec::new();
        offsets.push(0);
        for r in 0..rows {
            for c in 0..cols {
                let v = values[r * cols + c];
                if !v.is_finite() || v.abs() > eps {
                    cols_buf.push(c as u32);
                }
            }
            offsets.push(cols_buf.len() as u32);
        }
        Ok(Self {
            rows,
            cols,
            row_offsets: Arc::from(offsets),
            col_indices: Arc::from(cols_buf),
            duplicate_entries: 0,
        })
    }

    fn canonicalize(
        rows: usize,
        cols: usize,
        raw_offsets: Vec<u32>,
        mut raw_cols: Vec<u32>,
    ) -> Result<Self, SpyError> {
        let mut out_offsets = Vec::with_capacity(rows + 1);
        let mut out_cols = Vec::with_capacity(raw_cols.len());
        let mut duplicates = 0usize;
        out_offsets.push(0);
        for r in 0..rows {
            let slice = &mut raw_cols[raw_offsets[r] as usize..raw_offsets[r + 1] as usize];
            slice.sort_unstable();
            let mut prev = None;
            for &c in slice.iter() {
                if prev == Some(c) {
                    duplicates += 1;
                } else {
                    out_cols.push(c);
                    prev = Some(c);
                }
            }
            out_offsets.push(out_cols.len() as u32);
        }
        Ok(Self {
            rows,
            cols,
            row_offsets: Arc::from(out_offsets),
            col_indices: Arc::from(out_cols),
            duplicate_entries: duplicates,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn nnz(&self) -> usize {
        self.col_indices.len()
    }
    pub fn duplicate_entries(&self) -> usize {
        self.duplicate_entries
    }
    pub fn density(&self) -> f64 {
        self.nnz() as f64 / (self.rows as f64 * self.cols as f64)
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.row_offsets.as_ref())
            + std::mem::size_of_val(self.col_indices.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.col_indices.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        let lo = [0.5, 0.5];
        let hi = [self.cols as f64 + 0.5, self.rows as f64 + 0.5];
        (scales[0].forward(lo[0]).is_some()
            && scales[0].forward(hi[0]).is_some()
            && scales[1].forward(lo[1]).is_some()
            && scales[1].forward(hi[1]).is_some())
        .then_some((lo, hi))
    }

    /// Iterate visible `(flat_index, [col + 1.0, row + 1.0])` within data-coordinate bounds.
    pub(crate) fn visit_window(
        &self,
        x_range: [f64; 2],
        y_range: [f64; 2],
        mut f: impl FnMut(usize, PlotPoint),
    ) {
        if x_range[1] < 1.0 || y_range[1] < 1.0 {
            return;
        }
        let r0 = ((y_range[0].ceil() as isize) - 1).clamp(0, self.rows as isize) as usize;
        let r1 = (y_range[1].floor() as isize).clamp(0, self.rows as isize) as usize;
        if r0 >= r1 {
            return;
        }
        let c0 = ((x_range[0].ceil() as isize) - 1).clamp(0, self.cols as isize) as u32;
        let c1 = (x_range[1].floor() as isize).clamp(0, self.cols as isize) as u32;
        if c0 >= c1 {
            return;
        }
        for row in r0..r1 {
            let row_start = self.row_offsets[row] as usize;
            let row_end = self.row_offsets[row + 1] as usize;
            let row_cols = &self.col_indices[row_start..row_end];
            let s = row_cols.partition_point(|&c| c < c0);
            let e = row_cols.partition_point(|&c| c < c1);
            let y = row as f64 + 1.0;
            for (rel, &col) in row_cols[s..e].iter().enumerate() {
                f(row_start + s + rel, [col as f64 + 1.0, y]);
            }
        }
    }
}
