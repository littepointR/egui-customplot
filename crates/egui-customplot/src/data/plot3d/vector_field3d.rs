use std::sync::Arc;

use crate::{Bounds3D, FunctionContourError, GridError, Point3D};

use super::scalar_volume3d::cell_pos;

/// Host-owned rectilinear 3D vector field `[u, v, w]` (`streamline`, `streamslice`, `streamribbon`, `streamtube`, `coneplot`).
#[derive(Clone, Debug)]
pub struct VectorField3D {
    x: Arc<[f64]>,
    y: Arc<[f64]>,
    z: Arc<[f64]>,
    u: Arc<[f64]>,
    v: Arc<[f64]>,
    w: Arc<[f64]>,
    bounds: Bounds3D,
    speed_range: Option<[f64; 2]>,
}

impl VectorField3D {
    pub fn new(
        x: impl Into<Arc<[f64]>>,
        y: impl Into<Arc<[f64]>>,
        z: impl Into<Arc<[f64]>>,
        u: impl Into<Arc<[f64]>>,
        v: impl Into<Arc<[f64]>>,
        w: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        let (x, y, z) = (x.into(), y.into(), z.into());
        let (u, v, w) = (u.into(), v.into(), w.into());
        for ax in [&x, &y, &z] {
            if ax.len() < 2 {
                return Err(GridError::EmptyAxis);
            }
            if ax.iter().any(|k| !k.is_finite()) || ax.windows(2).any(|p| p[0] >= p[1]) {
                return Err(GridError::InvalidEdges);
            }
        }
        let n = x
            .len()
            .checked_mul(y.len())
            .and_then(|s| s.checked_mul(z.len()))
            .ok_or(GridError::CapacityOverflow)?;
        if u.len() != n || v.len() != n || w.len() != n {
            return Err(GridError::LengthMismatch);
        }
        let bounds = Bounds3D::new(
            [x[0], y[0], z[0]],
            [x[x.len() - 1], y[y.len() - 1], z[z.len() - 1]],
        )
        .map_err(|_| GridError::InvalidEdges)?;
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for ((&ui, &vi), &wi) in u.iter().zip(v.iter()).zip(w.iter()) {
            if ui.is_finite() && vi.is_finite() && wi.is_finite() {
                let s = (ui * ui + vi * vi + wi * wi).sqrt();
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        Ok(Self {
            x,
            y,
            z,
            u,
            v,
            w,
            bounds,
            speed_range: (lo <= hi).then_some([lo, hi]),
        })
    }

    pub fn from_function(
        bounds: Bounds3D,
        resolution: [usize; 3],
        max_evaluations: usize,
        mut f: impl FnMut(f64, f64, f64) -> [f64; 3],
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
        let (mut u, mut v, mut w) = (
            Vec::with_capacity(req),
            Vec::with_capacity(req),
            Vec::with_capacity(req),
        );
        for &zi in &z {
            for &yi in &y {
                for &xi in &x {
                    let vec = f(xi, yi, zi);
                    u.push(vec[0]);
                    v.push(vec[1]);
                    w.push(vec[2]);
                }
            }
        }
        Ok(Self::new(x, y, z, u, v, w)?)
    }

    pub fn bounds(&self) -> Bounds3D {
        self.bounds
    }
    pub fn speed_range(&self) -> Option<[f64; 2]> {
        self.speed_range
    }

    pub fn sample(&self, p: Point3D) -> Option<[f64; 3]> {
        let (ix, tx) = cell_pos(&self.x, p[0])?;
        let (iy, ty) = cell_pos(&self.y, p[1])?;
        let (iz, tz) = cell_pos(&self.z, p[2])?;
        let (nx, ny) = (self.x.len(), self.y.len());
        let lerp_comp = |arr: &[f64]| -> Option<f64> {
            let at = |dx: usize, dy: usize, dz: usize| {
                arr[((iz + dz) * ny + (iy + dy)) * nx + (ix + dx)]
            };
            let corners = [
                at(0, 0, 0),
                at(1, 0, 0),
                at(0, 1, 0),
                at(1, 1, 0),
                at(0, 0, 1),
                at(1, 0, 1),
                at(0, 1, 1),
                at(1, 1, 1),
            ];
            if !corners.iter().all(|c| c.is_finite()) {
                return None;
            }
            let c00 = corners[0] + tx * (corners[1] - corners[0]);
            let c10 = corners[2] + tx * (corners[3] - corners[2]);
            let c01 = corners[4] + tx * (corners[5] - corners[4]);
            let c11 = corners[6] + tx * (corners[7] - corners[6]);
            let c0 = c00 + ty * (c10 - c00);
            let c1 = c01 + ty * (c11 - c01);
            Some(c0 + tz * (c1 - c0))
        };
        Some([
            lerp_comp(&self.u)?,
            lerp_comp(&self.v)?,
            lerp_comp(&self.w)?,
        ])
    }

    fn jacobian(&self, p: Point3D) -> Option<[[f64; 3]; 3]> {
        let c = self.sample(p)?;
        let span = self.bounds.span();
        let eps = [
            0.25 * span[0] / (self.x.len() - 1) as f64,
            0.25 * span[1] / (self.y.len() - 1) as f64,
            0.25 * span[2] / (self.z.len() - 1) as f64,
        ];
        let mut cols = [[0.0; 3]; 3];
        for a in 0..3 {
            let mut pl = p;
            let mut ph = p;
            pl[a] -= eps[a];
            ph[a] += eps[a];
            cols[a] = match (self.sample(pl), self.sample(ph)) {
                (Some(vl), Some(vh)) => std::array::from_fn(|k| (vh[k] - vl[k]) / (2.0 * eps[a])),
                (Some(vl), None) => std::array::from_fn(|k| (c[k] - vl[k]) / eps[a]),
                (None, Some(vh)) => std::array::from_fn(|k| (vh[k] - c[k]) / eps[a]),
                (None, None) => [0.0; 3],
            };
        }
        Some(cols)
    }

    pub fn curl(&self, p: Point3D) -> Option<[f64; 3]> {
        let j = self.jacobian(p)?;
        Some([j[1][2] - j[2][1], j[2][0] - j[0][2], j[0][1] - j[1][0]])
    }

    pub fn divergence(&self, p: Point3D) -> Option<f64> {
        let j = self.jacobian(p)?;
        Some(j[0][0] + j[1][1] + j[2][2])
    }
}
