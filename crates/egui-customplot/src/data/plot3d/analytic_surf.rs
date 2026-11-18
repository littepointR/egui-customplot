use std::{
    f64::consts::{PI, TAU},
    sync::Arc,
};

use crate::{AxisScale, FunctionContourError, GridError, Point3D};

use super::{CachedExtents3D, Plot3DError, Surface3DData};

/// Multi-strip 3D ribbon plot dataset (`ribbon`, `ML-062`).
#[derive(Clone, Debug)]
pub struct Ribbon3DData {
    pub(crate) version: u64,
    strips: Arc<[Surface3DData]>,
    extents: CachedExtents3D,
}

impl Ribbon3DData {
    pub fn new(
        y_columns: &[&[f64]],
        x_coords: Option<&[f64]>,
        width: f64,
    ) -> Result<Self, Plot3DError> {
        if y_columns.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let rows = y_columns[0].len();
        if rows < 2 {
            return Err(Plot3DError::InvalidDimensions);
        }
        if y_columns.iter().any(|c| c.len() != rows) || x_coords.is_some_and(|xs| xs.len() != rows)
        {
            return Err(Plot3DError::LengthMismatch);
        }
        let hw = if width.is_finite() && width > 0.0 {
            0.5 * width.clamp(0.05, 1.9)
        } else {
            0.35
        };
        let mut strips = Vec::with_capacity(y_columns.len());
        let mut extents = CachedExtents3D::default();
        for (j, &col) in y_columns.iter().enumerate() {
            let cx = (j + 1) as f64;
            let mut pts = Vec::with_capacity(rows * 2);
            let mut scal = Vec::with_capacity(rows * 2);
            for (i, &z) in col.iter().enumerate() {
                let yv = x_coords.map_or((i + 1) as f64, |xs| xs[i]);
                let p0 = [cx - hw, yv, z];
                let p1 = [cx + hw, yv, z];
                extents.push_point(p0);
                extents.push_point(p1);
                pts.push(p0);
                pts.push(p1);
                scal.push(cx);
                scal.push(cx);
            }
            let surf = Surface3DData::from_parametric_grid(rows, 2, pts)?.with_scalars(&scal)?;
            strips.push(surf);
        }
        Ok(Self {
            version: crate::data::points::next_version(),
            strips: Arc::from(strips),
            extents,
        })
    }

    pub fn strips(&self) -> &[Surface3DData] {
        &self.strips
    }
    pub fn sample_bytes(&self) -> usize {
        self.strips.iter().map(Surface3DData::sample_bytes).sum()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.strips.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

impl Surface3DData {
    /// Explicit bivariate surface `z = f(x, y)` over a bounded grid (`fsurf` / `fmesh`).
    pub fn from_explicit_fn(
        x_range: [f64; 2],
        y_range: [f64; 2],
        resolution: [usize; 2],
        max_evaluations: usize,
        mut f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionContourError> {
        let [nx, ny] = validate_fn_grid(x_range, y_range, resolution, max_evaluations)?;
        let mut pts = Vec::with_capacity(nx * ny);
        for r in 0..ny {
            let y = sample_axis(y_range, r, ny);
            for c in 0..nx {
                let x = sample_axis(x_range, c, nx);
                let z = f(x, y);
                pts.push([x, y, if z.is_finite() { z } else { f64::NAN }]);
            }
        }
        pole_guard_z(ny, nx, &mut pts);
        Self::from_parametric_grid(ny, nx, pts).map_err(|_| FunctionContourError::InvalidResolution)
    }

    /// Parametric bivariate surface `[x(u, v), y(u, v), z(u, v)]` (`fsurf` / `fmesh`).
    pub fn from_parametric_fn(
        u_range: [f64; 2],
        v_range: [f64; 2],
        resolution: [usize; 2],
        max_evaluations: usize,
        mut f: impl FnMut(f64, f64) -> Point3D,
    ) -> Result<Self, FunctionContourError> {
        let [nu, nv] = validate_fn_grid(u_range, v_range, resolution, max_evaluations)?;
        let mut pts = Vec::with_capacity(nu * nv);
        for r in 0..nv {
            let v = sample_axis(v_range, r, nv);
            for c in 0..nu {
                let u = sample_axis(u_range, c, nu);
                let p = f(u, v);
                pts.push(if p.iter().all(|v| v.is_finite()) {
                    p
                } else {
                    [f64::NAN; 3]
                });
            }
        }
        Self::from_parametric_grid(nv, nu, pts).map_err(|_| FunctionContourError::InvalidResolution)
    }

    pub fn ez_explicit(
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionContourError> {
        let side = (max_evaluations as f64).sqrt().floor() as usize;
        let res = side.clamp(2, 49);
        if max_evaluations < 4 {
            return Err(FunctionContourError::BudgetExceeded {
                required: 4,
                max_evaluations,
            });
        }
        Self::from_explicit_fn([-TAU, TAU], [-TAU, TAU], [res, res], max_evaluations, f)
    }

    pub fn ez_parametric(
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> Point3D,
    ) -> Result<Self, FunctionContourError> {
        let side = (max_evaluations as f64).sqrt().floor() as usize;
        let res = side.clamp(2, 49);
        if max_evaluations < 4 {
            return Err(FunctionContourError::BudgetExceeded {
                required: 4,
                max_evaluations,
            });
        }
        Self::from_parametric_fn([-TAU, TAU], [-TAU, TAU], [res, res], max_evaluations, f)
    }

    /// Unit sphere parametric mesh (`sphere`, `EX-008`).
    pub fn sphere(subdivisions: usize) -> Result<Self, Plot3DError> {
        Self::ellipsoid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], subdivisions)
    }

    /// Analytic ellipsoid with exact gradient normals (`ellipsoid`, `EX-010`).
    pub fn ellipsoid(
        center: Point3D,
        radii: [f64; 3],
        subdivisions: usize,
    ) -> Result<Self, Plot3DError> {
        if subdivisions < 3
            || !center.iter().all(|v| v.is_finite())
            || !radii.iter().all(|v| v.is_finite() && *v > 0.0)
        {
            return Err(Plot3DError::InvalidDimensions);
        }
        let n = subdivisions.clamp(3, 512);
        let rows = n + 1;
        let cols = n + 1;
        let mut pts = Vec::with_capacity(rows * cols);
        let mut norms = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            let phi = -0.5 * PI + PI * (r as f64 / n as f64);
            let (sp, cp) = phi.sin_cos();
            for c in 0..cols {
                let theta = if c == n {
                    0.0
                } else {
                    TAU * (c as f64 / n as f64)
                };
                let (st, ct) = theta.sin_cos();
                let u = [cp * ct, cp * st, sp];
                pts.push([
                    center[0] + radii[0] * u[0],
                    center[1] + radii[1] * u[1],
                    center[2] + radii[2] * u[2],
                ]);
                let g = [u[0] / radii[0], u[1] / radii[1], u[2] / radii[2]];
                let gl = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt().max(1e-12);
                norms.push([g[0] / gl, g[1] / gl, g[2] / gl]);
            }
        }
        Self::from_parametric_grid(rows, cols, pts)?.with_normals(norms)
    }

    /// Surface of revolution cylinder from radial profile (`cylinder`, `EX-009`).
    pub fn cylinder(profile_radii: &[f64], subdivisions: usize) -> Result<Self, Plot3DError> {
        if profile_radii.len() < 2
            || subdivisions < 3
            || profile_radii.iter().any(|r| !r.is_finite() || *r < 0.0)
        {
            return Err(Plot3DError::InvalidDimensions);
        }
        let n = subdivisions.clamp(3, 512);
        let rows = profile_radii.len();
        let cols = n + 1;
        let mut pts = Vec::with_capacity(rows * cols);
        for (r, &rad) in profile_radii.iter().enumerate() {
            let z = r as f64 / (rows - 1) as f64;
            for c in 0..cols {
                let theta = if c == n {
                    0.0
                } else {
                    TAU * (c as f64 / n as f64)
                };
                pts.push([rad * theta.cos(), rad * theta.sin(), z]);
            }
        }
        Self::from_parametric_grid(rows, cols, pts)
    }
}

fn validate_fn_grid(
    r0: [f64; 2],
    r1: [f64; 2],
    res: [usize; 2],
    max_eval: usize,
) -> Result<[usize; 2], FunctionContourError> {
    if !r0[0].is_finite()
        || !r0[1].is_finite()
        || r0[1] <= r0[0]
        || !r1[0].is_finite()
        || !r1[1].is_finite()
        || r1[1] <= r1[0]
    {
        return Err(GridError::InvalidEdges.into());
    }
    if res[0] < 2 || res[1] < 2 {
        return Err(FunctionContourError::InvalidResolution);
    }
    let req = res[0]
        .checked_mul(res[1])
        .ok_or(FunctionContourError::BudgetExceeded {
            required: usize::MAX,
            max_evaluations: max_eval,
        })?;
    if req > max_eval {
        return Err(FunctionContourError::BudgetExceeded {
            required: req,
            max_evaluations: max_eval,
        });
    }
    Ok(res)
}

fn sample_axis(range: [f64; 2], i: usize, len: usize) -> f64 {
    if i + 1 == len {
        range[1]
    } else {
        range[0] + (range[1] - range[0]) * (i as f64 / (len - 1) as f64)
    }
}

fn pole_guard_z(rows: usize, cols: usize, pts: &mut [Point3D]) {
    let mut diffs = Vec::with_capacity(pts.len() * 2);
    for r in 0..rows {
        for c in 0..cols {
            let z = pts[r * cols + c][2];
            if !z.is_finite() {
                continue;
            }
            if c + 1 < cols {
                let zr = pts[r * cols + c + 1][2];
                if zr.is_finite() {
                    diffs.push((zr - z).abs());
                }
            }
            if r + 1 < rows {
                let zd = pts[(r + 1) * cols + c][2];
                if zd.is_finite() {
                    diffs.push((zd - z).abs());
                }
            }
        }
    }
    if diffs.is_empty() {
        return;
    }
    diffs.sort_unstable_by(f64::total_cmp);
    let med = diffs[diffs.len() / 2].max(1e-6);
    let mut flag = vec![false; pts.len()];
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            let z = pts[idx][2];
            if !z.is_finite() {
                continue;
            }
            for (nr, nc) in [(r, c + 1), (r + 1, c)] {
                if nr < rows && nc < cols {
                    let nidx = nr * cols + nc;
                    let zn = pts[nidx][2];
                    if zn.is_finite() && z * zn < 0.0 && (zn - z).abs() > 60.0 * med {
                        if z.abs() > zn.abs() {
                            flag[idx] = true;
                        } else {
                            flag[nidx] = true;
                        }
                    }
                }
            }
        }
    }
    for (i, &bad) in flag.iter().enumerate() {
        if bad {
            pts[i][2] = f64::NAN;
        }
    }
}
