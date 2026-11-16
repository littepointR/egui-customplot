use std::collections::HashMap;

use crate::{AxisScale, Bounds3D, FunctionContourError, Point3D};

use super::TriMesh3DData;

/// Immutable host-owned 3D implicit level-set surface `F(x, y, z) = 0` (`fimplicit3`, `ML-065`).
#[derive(Clone, Debug)]
pub struct ImplicitSurface3DData {
    mesh: Option<TriMesh3DData>,
    evaluations: usize,
    discontinuities: usize,
}

impl ImplicitSurface3DData {
    /// Evaluates `f(x, y, z)` across a regular `resolution` 3D grid inside `bounds` and extracts
    /// the zero level set via 6-tetrahedra Marching Tetrahedra with gradient normals and pole guard.
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
        let bmin = bounds.min();
        let bspan = bounds.span();
        let x: Vec<f64> = (0..nx)
            .map(|i| bmin[0] + bspan[0] * (i as f64 / (nx - 1) as f64))
            .collect();
        let y: Vec<f64> = (0..ny)
            .map(|i| bmin[1] + bspan[1] * (i as f64 / (ny - 1) as f64))
            .collect();
        let z: Vec<f64> = (0..nz)
            .map(|i| bmin[2] + bspan[2] * (i as f64 / (nz - 1) as f64))
            .collect();

        let mut vals = Vec::with_capacity(req);
        for &zi in &z {
            for &yi in &y {
                for &xi in &x {
                    vals.push(f(xi, yi, zi));
                }
            }
        }
        let (mesh, discontinuities) = extract_kuhn_mesh(&x, &y, &z, &vals, 0.0, true);
        Ok(Self {
            mesh,
            evaluations: req,
            discontinuities,
        })
    }

    pub fn mesh(&self) -> Option<&TriMesh3DData> {
        self.mesh.as_ref()
    }
    pub fn evaluations(&self) -> usize {
        self.evaluations
    }
    pub fn discontinuities(&self) -> usize {
        self.discontinuities
    }
    pub fn sample_bytes(&self) -> usize {
        self.mesh.as_ref().map_or(0, TriMesh3DData::sample_bytes)
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        self.mesh.as_ref().map_or((0, 0), TriMesh3DData::allocation)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.mesh.as_ref().and_then(|m| m.extent(scales))
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.mesh.as_ref().map_or(0, |m| m.domain_invalid(scales))
    }
}

/// Shared 6-tetrahedra Kuhn Marching Tetrahedra iso-level extractor for rectilinear 3D grids.
pub(super) fn extract_kuhn_mesh(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    vals: &[f64],
    isovalue: f64,
    pole_guard: bool,
) -> (Option<TriMesh3DData>, usize) {
    let (nx, ny, nz) = (x.len(), y.len(), z.len());
    let med_df = if pole_guard {
        median_edge_diff(nx, ny, nz, vals)
    } else {
        f64::INFINITY
    };
    let idx = |ix: usize, iy: usize, iz: usize| (iz * ny + iy) * nx + ix;
    let pos = |ix: usize, iy: usize, iz: usize| [x[ix], y[iy], z[iz]];
    let grad_at = |ix: usize, iy: usize, iz: usize| -> [f64; 3] {
        let x0 = ix.saturating_sub(1);
        let x1 = (ix + 1).min(nx - 1);
        let y0 = iy.saturating_sub(1);
        let y1 = (iy + 1).min(ny - 1);
        let z0 = iz.saturating_sub(1);
        let z1 = (iz + 1).min(nz - 1);
        let dx = (x[x1] - x[x0]).max(1e-12);
        let dy = (y[y1] - y[y0]).max(1e-12);
        let dz = (z[z1] - z[z0]).max(1e-12);
        let gx = (vals[idx(x1, iy, iz)] - vals[idx(x0, iy, iz)]) / dx;
        let gy = (vals[idx(ix, y1, iz)] - vals[idx(ix, y0, iz)]) / dy;
        let gz = (vals[idx(ix, iy, z1)] - vals[idx(ix, iy, z0)]) / dz;
        if gx.is_finite() && gy.is_finite() && gz.is_finite() {
            [gx, gy, gz]
        } else {
            [0.0, 0.0, 1.0]
        }
    };

    let mut vertices = Vec::<Point3D>::new();
    let mut normals = Vec::<[f64; 3]>::new();
    let mut triangles = Vec::<[u32; 3]>::new();
    let mut edge_cache: HashMap<(usize, usize), u32> = HashMap::new();
    let mut discontinuities = 0usize;

    let tets: [[usize; 4]; 6] = [
        [0, 1, 3, 7],
        [0, 3, 2, 7],
        [0, 2, 6, 7],
        [0, 6, 4, 7],
        [0, 4, 5, 7],
        [0, 5, 1, 7],
    ];

    for iz in 0..(nz - 1) {
        for iy in 0..(ny - 1) {
            for ix in 0..(nx - 1) {
                let c_idx = [
                    idx(ix, iy, iz),
                    idx(ix + 1, iy, iz),
                    idx(ix, iy + 1, iz),
                    idx(ix + 1, iy + 1, iz),
                    idx(ix, iy, iz + 1),
                    idx(ix + 1, iy, iz + 1),
                    idx(ix, iy + 1, iz + 1),
                    idx(ix + 1, iy + 1, iz + 1),
                ];
                let c_vals: [f64; 8] = std::array::from_fn(|k| vals[c_idx[k]] - isovalue);
                if c_vals.iter().any(|v| !v.is_finite()) {
                    discontinuities += 1;
                    continue;
                }
                let c_pos: [Point3D; 8] = [
                    pos(ix, iy, iz),
                    pos(ix + 1, iy, iz),
                    pos(ix, iy + 1, iz),
                    pos(ix + 1, iy + 1, iz),
                    pos(ix, iy, iz + 1),
                    pos(ix + 1, iy, iz + 1),
                    pos(ix, iy + 1, iz + 1),
                    pos(ix + 1, iy + 1, iz + 1),
                ];
                let c_grd: [[f64; 3]; 8] = [
                    grad_at(ix, iy, iz),
                    grad_at(ix + 1, iy, iz),
                    grad_at(ix, iy + 1, iz),
                    grad_at(ix + 1, iy + 1, iz),
                    grad_at(ix, iy, iz + 1),
                    grad_at(ix + 1, iy, iz + 1),
                    grad_at(ix, iy + 1, iz + 1),
                    grad_at(ix + 1, iy + 1, iz + 1),
                ];

                for tet in tets {
                    if let Some((tri_tris, pole)) = extract_tet(
                        tet,
                        &c_idx,
                        &c_vals,
                        &c_pos,
                        &c_grd,
                        med_df,
                        pole_guard,
                        &mut edge_cache,
                        &mut vertices,
                        &mut normals,
                    ) {
                        if pole {
                            discontinuities += 1;
                        } else {
                            triangles.extend_from_slice(&tri_tris);
                        }
                    }
                }
            }
        }
    }

    let mesh = if vertices.len() >= 3 && !triangles.is_empty() {
        TriMesh3DData::new(vertices, triangles)
            .ok()
            .and_then(|m| m.with_normals(normals).ok())
    } else {
        None
    };
    (mesh, discontinuities)
}

#[allow(clippy::too_many_arguments)]
fn extract_tet(
    tet: [usize; 4],
    c_idx: &[usize; 8],
    c_vals: &[f64; 8],
    c_pos: &[Point3D; 8],
    c_grd: &[[f64; 3]; 8],
    med_df: f64,
    pole_guard: bool,
    edge_cache: &mut HashMap<(usize, usize), u32>,
    vertices: &mut Vec<Point3D>,
    normals: &mut Vec<[f64; 3]>,
) -> Option<(Vec<[u32; 3]>, bool)> {
    let mut neg = [0usize; 4];
    let mut pos = [0usize; 4];
    let mut n_neg = 0usize;
    let mut n_pos = 0usize;
    for &c in &tet {
        if c_vals[c] < 0.0 {
            neg[n_neg] = c;
            n_neg += 1;
        } else {
            pos[n_pos] = c;
            n_pos += 1;
        }
    }
    if n_neg == 0 || n_pos == 0 {
        return None;
    }

    if pole_guard {
        for &a in &neg[..n_neg] {
            for &b in &pos[..n_pos] {
                let fa = c_vals[a];
                let fb = c_vals[b];
                if (fb - fa).abs() > 40.0 * med_df && fa.abs().min(fb.abs()) > 10.0 * med_df {
                    return Some((Vec::new(), true));
                }
            }
        }
    }

    let mut interp = |a: usize, b: usize| -> u32 {
        let ga = c_idx[a];
        let gb = c_idx[b];
        let key = (ga.min(gb), ga.max(gb));
        if let Some(&vid) = edge_cache.get(&key) {
            return vid;
        }
        let fa = c_vals[a];
        let fb = c_vals[b];
        let t = if (fb - fa).abs() <= 1e-18 {
            0.5
        } else {
            (-fa / (fb - fa)).clamp(0.0, 1.0)
        };
        let pa = c_pos[a];
        let pb = c_pos[b];
        let p = [
            pa[0] + t * (pb[0] - pa[0]),
            pa[1] + t * (pb[1] - pa[1]),
            pa[2] + t * (pb[2] - pa[2]),
        ];
        let na = c_grd[a];
        let nb = c_grd[b];
        let gx = na[0] + t * (nb[0] - na[0]);
        let gy = na[1] + t * (nb[1] - na[1]);
        let gz = na[2] + t * (nb[2] - na[2]);
        let gl = (gx * gx + gy * gy + gz * gz).sqrt();
        let n = if gl > 1e-14 && gl.is_finite() {
            [gx / gl, gy / gl, gz / gl]
        } else {
            [0.0, 0.0, 1.0]
        };
        let vid = vertices.len() as u32;
        vertices.push(p);
        normals.push(n);
        edge_cache.insert(key, vid);
        vid
    };

    let mut tris = Vec::with_capacity(2);
    if n_neg == 1 {
        let a = neg[0];
        tris.push([interp(a, pos[0]), interp(a, pos[1]), interp(a, pos[2])]);
    } else if n_pos == 1 {
        let a = pos[0];
        tris.push([interp(a, neg[0]), interp(a, neg[2]), interp(a, neg[1])]);
    } else {
        let p00 = interp(neg[0], pos[0]);
        let p01 = interp(neg[0], pos[1]);
        let p10 = interp(neg[1], pos[0]);
        let p11 = interp(neg[1], pos[1]);
        tris.push([p00, p01, p11]);
        tris.push([p00, p11, p10]);
    }
    Some((tris, false))
}

fn median_edge_diff(nx: usize, ny: usize, nz: usize, vals: &[f64]) -> f64 {
    let mut diffs = Vec::new();
    let idx = |ix: usize, iy: usize, iz: usize| (iz * ny + iy) * nx + ix;
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let v = vals[idx(ix, iy, iz)];
                if !v.is_finite() {
                    continue;
                }
                if ix + 1 < nx {
                    let vr = vals[idx(ix + 1, iy, iz)];
                    if vr.is_finite() {
                        diffs.push((vr - v).abs());
                    }
                }
                if iy + 1 < ny {
                    let vu = vals[idx(ix, iy + 1, iz)];
                    if vu.is_finite() {
                        diffs.push((vu - v).abs());
                    }
                }
                if iz + 1 < nz {
                    let vf = vals[idx(ix, iy, iz + 1)];
                    if vf.is_finite() {
                        diffs.push((vf - v).abs());
                    }
                }
            }
        }
    }
    if diffs.is_empty() {
        return 1.0;
    }
    diffs.sort_unstable_by(f64::total_cmp);
    diffs[diffs.len() / 2].max(1e-6)
}
