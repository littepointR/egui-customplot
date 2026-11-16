use std::sync::Arc;

use crate::{Bounds3D, FunctionContourError, GridError, Point3D};

/// Rectilinear 3D scalar volume `(iz * ny + iy) * nx + ix` (`slice`, `contourslice`, `isosurface`, `isocaps`).
#[derive(Clone, Debug)]
pub struct ScalarVolume3D {
    x: Arc<[f64]>,
    y: Arc<[f64]>,
    z: Arc<[f64]>,
    values: Arc<[f64]>,
    bounds: Bounds3D,
    value_range: Option<[f64; 2]>,
    non_finite: usize,
}

impl ScalarVolume3D {
    pub fn new(
        x: impl Into<Arc<[f64]>>,
        y: impl Into<Arc<[f64]>>,
        z: impl Into<Arc<[f64]>>,
        values: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        let (x, y, z, values) = (x.into(), y.into(), z.into(), values.into());
        for ax in [&x, &y, &z] {
            if ax.len() < 2 {
                return Err(GridError::EmptyAxis);
            }
            if ax.iter().any(|v| !v.is_finite())
                || ax.windows(2).any(|w| w[0] >= w[1])
                || !(ax[ax.len() - 1] - ax[0]).is_finite()
            {
                return Err(GridError::InvalidEdges);
            }
        }
        let expected = x
            .len()
            .checked_mul(y.len())
            .and_then(|v| v.checked_mul(z.len()))
            .ok_or(GridError::CapacityOverflow)?;
        if values.len() != expected {
            return Err(GridError::LengthMismatch);
        }
        let bounds = Bounds3D::new(
            [x[0], y[0], z[0]],
            [x[x.len() - 1], y[y.len() - 1], z[z.len() - 1]],
        )
        .map_err(|_| GridError::InvalidEdges)?;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let mut non_finite = 0usize;
        for &v in values.iter() {
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
            } else {
                non_finite += 1;
            }
        }
        Ok(Self {
            x,
            y,
            z,
            values,
            bounds,
            value_range: (lo <= hi).then_some([lo, hi]),
            non_finite,
        })
    }

    pub fn from_function(
        bounds: Bounds3D,
        resolution: [usize; 3],
        max_evaluations: usize,
        mut f: impl FnMut(f64, f64, f64) -> f64,
    ) -> Result<Self, FunctionContourError> {
        let [nx, ny, nz] = resolution;
        if nx < 2 || ny < 2 || nz < 2 {
            return Err(FunctionContourError::InvalidResolution);
        }
        let req = nx.checked_mul(ny).and_then(|v| v.checked_mul(nz)).ok_or(
            FunctionContourError::BudgetExceeded {
                required: usize::MAX,
                max_evaluations,
            },
        )?;
        if req > max_evaluations {
            return Err(FunctionContourError::BudgetExceeded {
                required: req,
                max_evaluations,
            });
        }
        let (bmin, span) = (bounds.min(), bounds.span());
        let axis = |len: usize, a: usize| -> Vec<f64> {
            (0..len)
                .map(|i| bmin[a] + span[a] * (i as f64 / (len - 1) as f64))
                .collect()
        };
        let (x, y, z) = (axis(nx, 0), axis(ny, 1), axis(nz, 2));
        let mut vals = Vec::with_capacity(req);
        for &zi in &z {
            for &yi in &y {
                for &xi in &x {
                    vals.push(f(xi, yi, zi));
                }
            }
        }
        Ok(Self::new(x, y, z, vals)?)
    }

    pub fn dims(&self) -> [usize; 3] {
        [self.x.len(), self.y.len(), self.z.len()]
    }
    pub fn x(&self) -> &[f64] {
        &self.x
    }
    pub fn y(&self) -> &[f64] {
        &self.y
    }
    pub fn z(&self) -> &[f64] {
        &self.z
    }
    pub fn values(&self) -> &[f64] {
        &self.values
    }
    pub fn bounds(&self) -> Bounds3D {
        self.bounds
    }
    pub fn value_range(&self) -> Option<[f64; 2]> {
        self.value_range
    }
    pub fn non_finite(&self) -> usize {
        self.non_finite
    }

    /// Trilinear interpolation at `p`; returns `None` outside bounds or near non-finite corners.
    pub fn sample(&self, p: Point3D) -> Option<f64> {
        let (ix, tx) = cell_pos(&self.x, p[0])?;
        let (iy, ty) = cell_pos(&self.y, p[1])?;
        let (iz, tz) = cell_pos(&self.z, p[2])?;
        let (nx, ny) = (self.x.len(), self.y.len());
        let at = |dx: usize, dy: usize, dz: usize| {
            self.values[((iz + dz) * ny + (iy + dy)) * nx + (ix + dx)]
        };
        let c000 = at(0, 0, 0);
        let c100 = at(1, 0, 0);
        let c010 = at(0, 1, 0);
        let c110 = at(1, 1, 0);
        let c001 = at(0, 0, 1);
        let c101 = at(1, 0, 1);
        let c011 = at(0, 1, 1);
        let c111 = at(1, 1, 1);
        if ![c000, c100, c010, c110, c001, c101, c011, c111]
            .iter()
            .all(|v| v.is_finite())
        {
            return None;
        }
        let c00 = c000 + tx * (c100 - c000);
        let c10 = c010 + tx * (c110 - c010);
        let c01 = c001 + tx * (c101 - c001);
        let c11 = c011 + tx * (c111 - c011);
        let c0 = c00 + ty * (c10 - c00);
        let c1 = c01 + ty * (c11 - c01);
        Some(c0 + tz * (c1 - c0))
    }

    pub fn gradient(&self, p: Point3D) -> Option<[f64; 3]> {
        self.sample(p)?;
        let span = self.bounds.span();
        let eps = [
            0.25 * span[0] / (self.x.len() - 1) as f64,
            0.25 * span[1] / (self.y.len() - 1) as f64,
            0.25 * span[2] / (self.z.len() - 1) as f64,
        ];
        let mut g = [0.0; 3];
        for a in 0..3 {
            let mut p_lo = p;
            let mut p_hi = p;
            p_lo[a] -= eps[a];
            p_hi[a] += eps[a];
            g[a] = match (self.sample(p_lo), self.sample(p_hi)) {
                (Some(a0), Some(a1)) => (a1 - a0) / (2.0 * eps[a]),
                (Some(a0), None) => (self.sample(p)? - a0) / eps[a],
                (None, Some(a1)) => (a1 - self.sample(p)?) / eps[a],
                (None, None) => 0.0,
            };
        }
        Some(g)
    }

    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.x.as_ref())
            + std::mem::size_of_val(self.y.as_ref())
            + std::mem::size_of_val(self.z.as_ref())
            + std::mem::size_of_val(self.values.as_ref())
    }
}

pub(super) fn cell_pos(nodes: &[f64], val: f64) -> Option<(usize, f64)> {
    let last = nodes.len() - 1;
    if !val.is_finite() || val < nodes[0] - 1e-12 || val > nodes[last] + 1e-12 {
        return None;
    }
    let v = val.clamp(nodes[0], nodes[last]);
    let idx = nodes
        .partition_point(|&n| n <= v)
        .saturating_sub(1)
        .min(last - 1);
    let den = (nodes[idx + 1] - nodes[idx]).max(1e-18);
    Some((idx, ((v - nodes[idx]) / den).clamp(0.0, 1.0)))
}
