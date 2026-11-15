use std::sync::Arc;

use crate::{GridError, PlotBounds, PlotPoint, QuiverArrow, QuiverData};

mod stream;

pub use stream::{StreamOptions, StreamlineData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowError {
    InvalidGrid(GridError),
    InvalidOptions,
    LengthMismatch,
}

impl std::fmt::Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGrid(err) => write!(f, "{err}"),
            Self::InvalidOptions => f.write_str("invalid streamline integration options"),
            Self::LengthMismatch => f.write_str("vector component slices must have equal length"),
        }
    }
}

impl std::error::Error for FlowError {}

impl From<GridError> for FlowError {
    fn from(value: GridError) -> Self {
        Self::InvalidGrid(value)
    }
}

/// Immutable host-owned node-centered 2D vector field (`streamline` / `streamslice`).
#[derive(Clone, Debug)]
pub struct VectorField2D {
    x: Arc<[f64]>,
    y: Arc<[f64]>,
    u: Arc<[f64]>,
    v: Arc<[f64]>,
    bounds: PlotBounds,
    min_cell_span: f64,
}

impl VectorField2D {
    pub fn new(
        x_nodes: impl Into<Arc<[f64]>>,
        y_nodes: impl Into<Arc<[f64]>>,
        u: impl Into<Arc<[f64]>>,
        v: impl Into<Arc<[f64]>>,
    ) -> Result<Self, FlowError> {
        let (x, y, u, v) = (x_nodes.into(), y_nodes.into(), u.into(), v.into());
        for nodes in [&x, &y] {
            if nodes.len() < 2 {
                return Err(GridError::EmptyAxis.into());
            }
            if nodes.iter().any(|val| !val.is_finite())
                || nodes.windows(2).any(|w| w[0] >= w[1])
                || !(nodes[nodes.len() - 1] - nodes[0]).is_finite()
            {
                return Err(GridError::InvalidEdges.into());
            }
        }
        let count = x
            .len()
            .checked_mul(y.len())
            .ok_or(GridError::CapacityOverflow)?;
        if u.len() != count || v.len() != count {
            return Err(GridError::LengthMismatch.into());
        }
        let min_dx = x
            .windows(2)
            .map(|w| w[1] - w[0])
            .fold(f64::INFINITY, f64::min);
        let min_dy = y
            .windows(2)
            .map(|w| w[1] - w[0])
            .fold(f64::INFINITY, f64::min);
        let bounds = PlotBounds::new([x[0], y[0]], [x[x.len() - 1], y[y.len() - 1]])
            .map_err(|_| FlowError::InvalidGrid(GridError::InvalidEdges))?;
        Ok(Self {
            x,
            y,
            u,
            v,
            bounds,
            min_cell_span: min_dx.min(min_dy),
        })
    }

    pub fn uniform(
        cols: usize,
        rows: usize,
        bounds: PlotBounds,
        u: impl Into<Arc<[f64]>>,
        v: impl Into<Arc<[f64]>>,
    ) -> Result<Self, FlowError> {
        if cols < 2 || rows < 2 {
            return Err(GridError::EmptyAxis.into());
        }
        let axis_nodes = |axis: usize, len: usize| -> Vec<f64> {
            let denom = (len - 1) as f64;
            (0..len)
                .map(|i| {
                    if i + 1 == len {
                        bounds.max()[axis]
                    } else {
                        bounds.min()[axis] + bounds.span()[axis] * (i as f64 / denom)
                    }
                })
                .collect()
        };
        Self::new(axis_nodes(0, cols), axis_nodes(1, rows), u, v)
    }

    pub fn bounds(&self) -> PlotBounds {
        self.bounds
    }

    pub(crate) fn min_cell_span(&self) -> f64 {
        self.min_cell_span
    }

    pub fn sample(&self, p: PlotPoint) -> Option<[f64; 2]> {
        let [px, py] = p;
        let lo = self.bounds.min();
        let hi = self.bounds.max();
        if !px.is_finite()
            || !py.is_finite()
            || px < lo[0]
            || px > hi[0]
            || py < lo[1]
            || py > hi[1]
        {
            return None;
        }
        let cols = self.x.len();
        let rows = self.y.len();
        let ix = self
            .x
            .partition_point(|&vx| vx <= px)
            .saturating_sub(1)
            .min(cols - 2);
        let iy = self
            .y
            .partition_point(|&vy| vy <= py)
            .saturating_sub(1)
            .min(rows - 2);
        let tx = (px - self.x[ix]) / (self.x[ix + 1] - self.x[ix]);
        let ty = (py - self.y[iy]) / (self.y[iy + 1] - self.y[iy]);
        let idx = |r: usize, c: usize| r * cols + c;
        let i00 = idx(iy, ix);
        let i10 = idx(iy, ix + 1);
        let i01 = idx(iy + 1, ix);
        let i11 = idx(iy + 1, ix + 1);
        let interp = |vals: &[f64]| -> Option<f64> {
            let [v00, v10, v01, v11] = [vals[i00], vals[i10], vals[i01], vals[i11]];
            if !v00.is_finite() || !v10.is_finite() || !v01.is_finite() || !v11.is_finite() {
                return None;
            }
            let b0 = v00 + tx * (v10 - v00);
            let b1 = v01 + tx * (v11 - v01);
            Some(b0 + ty * (b1 - b0))
        };
        Some([interp(&self.u)?, interp(&self.v)?])
    }
}

/// Immutable host-owned vectors arranged along a baseline (`feather` / `ML-058`).
#[derive(Clone, Debug)]
pub struct FeatherData {
    vectors: Arc<[PlotPoint]>,
    quiver: QuiverData,
    baseline_y: f64,
    x_span: Option<[f64; 2]>,
}

impl FeatherData {
    pub fn from_components(u: &[f64], v: &[f64]) -> Result<Self, FlowError> {
        if u.len() != v.len() {
            return Err(FlowError::LengthMismatch);
        }
        let vectors: Vec<PlotPoint> = u.iter().zip(v).map(|(&ux, &vy)| [ux, vy]).collect();
        Ok(Self::from_vectors(vectors, 1.0, 1.0, 0.0))
    }

    pub fn from_complex(re_im: &[[f64; 2]]) -> Self {
        Self::from_vectors(re_im.to_vec(), 1.0, 1.0, 0.0)
    }

    pub fn from_polar(theta: &[f64], rho: &[f64]) -> Result<Self, FlowError> {
        if theta.len() != rho.len() {
            return Err(FlowError::LengthMismatch);
        }
        let vectors = theta
            .iter()
            .zip(rho)
            .map(|(&t, &r)| [r * t.cos(), r * t.sin()])
            .collect();
        Ok(Self::from_vectors(vectors, 1.0, 1.0, 0.0))
    }

    pub fn with_baseline(self, x0: f64, dx: f64, baseline_y: f64) -> Self {
        Self::from_vectors(self.vectors.to_vec(), x0, dx, baseline_y)
    }

    fn from_vectors(vectors: Vec<PlotPoint>, x0: f64, dx: f64, baseline_y: f64) -> Self {
        let mut x_span: Option<[f64; 2]> = None;
        let arrows: Vec<QuiverArrow> = vectors
            .iter()
            .enumerate()
            .map(|(i, &vec)| {
                let x = x0 + (i as f64) * dx;
                if x.is_finite() {
                    let [lo, hi] = x_span.get_or_insert([x, x]);
                    *lo = lo.min(x);
                    *hi = hi.max(x);
                }
                QuiverArrow::new([x, baseline_y], vec)
            })
            .collect();
        Self {
            vectors: Arc::from(vectors),
            quiver: QuiverData::new(arrows),
            baseline_y,
            x_span,
        }
    }

    pub fn quiver(&self) -> &QuiverData {
        &self.quiver
    }
    pub fn baseline_y(&self) -> f64 {
        self.baseline_y
    }
    pub fn x_span(&self) -> Option<[f64; 2]> {
        self.x_span
    }
    pub fn len(&self) -> usize {
        self.quiver.len()
    }
    pub fn is_empty(&self) -> bool {
        self.quiver.is_empty()
    }
    pub fn sample_bytes(&self) -> usize {
        self.quiver.sample_bytes() + std::mem::size_of_val(self.vectors.as_ref())
    }
}
