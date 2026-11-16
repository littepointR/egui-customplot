use crate::{AxisScale, Point3D};

use super::{Plot3DError, ScalarVolume3D, TriMesh3DData, implicit3::extract_kuhn_mesh};

/// Host-owned volumetric isosurface mesh (`isosurface`, `EX-023`).
#[derive(Clone, Debug)]
pub struct Isosurface3DData {
    mesh: Option<TriMesh3DData>,
    isovalue: f64,
}

impl Isosurface3DData {
    pub fn from_volume(vol: &ScalarVolume3D, isovalue: f64) -> Result<Self, Plot3DError> {
        if !isovalue.is_finite() {
            return Err(Plot3DError::InvalidDimensions);
        }
        let (mesh_opt, _) =
            extract_kuhn_mesh(vol.x(), vol.y(), vol.z(), vol.values(), isovalue, false);
        let mesh = mesh_opt.and_then(|m| {
            let const_s = vec![isovalue; m.vertices().len()];
            m.with_vertex_scalars(&const_s).ok()
        });
        Ok(Self { mesh, isovalue })
    }

    pub fn with_colors_from_volume(mut self, color_vol: &ScalarVolume3D) -> Self {
        if let Some(m) = self.mesh.take() {
            let scalars: Vec<f64> = m
                .vertices()
                .iter()
                .map(|&v| color_vol.sample(v).unwrap_or(self.isovalue))
                .collect();
            self.mesh = Some(
                m.with_vertex_scalars(&scalars)
                    .unwrap_or_else(|e| unreachable!("{e}")),
            );
        }
        self
    }

    pub fn mesh(&self) -> Option<&TriMesh3DData> {
        self.mesh.as_ref()
    }
    pub fn isovalue(&self) -> f64 {
        self.isovalue
    }
    pub fn sample_bytes(&self) -> usize {
        self.mesh.as_ref().map_or(0, TriMesh3DData::sample_bytes)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.mesh.as_ref().and_then(|m| m.extent(scales))
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.mesh.as_ref().map_or(0, |m| m.domain_invalid(scales))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsocapThreshold {
    Above,
    Below,
}

/// Host-owned boundary end-cap mesh for open isosurfaces (`isocaps`, `EX-024`).
#[derive(Clone, Debug)]
pub struct Isocaps3DData {
    mesh: Option<TriMesh3DData>,
}

impl Isocaps3DData {
    pub fn from_volume(
        vol: &ScalarVolume3D,
        isovalue: f64,
        threshold: IsocapThreshold,
    ) -> Result<Self, Plot3DError> {
        if !isovalue.is_finite() {
            return Err(Plot3DError::InvalidDimensions);
        }
        let [nx, ny, nz] = vol.dims();
        let (x, y, z, vals) = (vol.x(), vol.y(), vol.z(), vol.values());
        let at = |ix: usize, iy: usize, iz: usize| vals[(iz * ny + iy) * nx + ix];
        let mut verts = Vec::<Point3D>::new();
        let mut norms = Vec::<[f64; 3]>::new();
        let mut scals = Vec::<f64>::new();
        let mut tris = Vec::<[u32; 3]>::new();

        let mut push_quad = |p00: Point3D,
                             v00: f64,
                             p10: Point3D,
                             v10: f64,
                             p01: Point3D,
                             v01: f64,
                             p11: Point3D,
                             v11: f64,
                             face_n: [f64; 3]| {
            // Split along the same `(0,0)-(1,1)` main diagonal used by Kuhn tetrahedra.
            clip_cap_tri(
                [p00, p10, p11],
                [v00, v10, v11],
                face_n,
                isovalue,
                threshold,
                &mut verts,
                &mut norms,
                &mut scals,
                &mut tris,
            );
            clip_cap_tri(
                [p00, p11, p01],
                [v00, v11, v01],
                face_n,
                isovalue,
                threshold,
                &mut verts,
                &mut norms,
                &mut scals,
                &mut tris,
            );
        };

        for iy in 0..(ny - 1) {
            for ix in 0..(nx - 1) {
                push_quad(
                    [x[ix], y[iy], z[0]],
                    at(ix, iy, 0),
                    [x[ix + 1], y[iy], z[0]],
                    at(ix + 1, iy, 0),
                    [x[ix], y[iy + 1], z[0]],
                    at(ix, iy + 1, 0),
                    [x[ix + 1], y[iy + 1], z[0]],
                    at(ix + 1, iy + 1, 0),
                    [0.0, 0.0, -1.0],
                );
                let tz = nz - 1;
                push_quad(
                    [x[ix], y[iy], z[tz]],
                    at(ix, iy, tz),
                    [x[ix + 1], y[iy], z[tz]],
                    at(ix + 1, iy, tz),
                    [x[ix], y[iy + 1], z[tz]],
                    at(ix, iy + 1, tz),
                    [x[ix + 1], y[iy + 1], z[tz]],
                    at(ix + 1, iy + 1, tz),
                    [0.0, 0.0, 1.0],
                );
            }
        }
        for iz in 0..(nz - 1) {
            for ix in 0..(nx - 1) {
                push_quad(
                    [x[ix], y[0], z[iz]],
                    at(ix, 0, iz),
                    [x[ix + 1], y[0], z[iz]],
                    at(ix + 1, 0, iz),
                    [x[ix], y[0], z[iz + 1]],
                    at(ix, 0, iz + 1),
                    [x[ix + 1], y[0], z[iz + 1]],
                    at(ix + 1, 0, iz + 1),
                    [0.0, -1.0, 0.0],
                );
                let ty = ny - 1;
                push_quad(
                    [x[ix], y[ty], z[iz]],
                    at(ix, ty, iz),
                    [x[ix + 1], y[ty], z[iz]],
                    at(ix + 1, ty, iz),
                    [x[ix], y[ty], z[iz + 1]],
                    at(ix, ty, iz + 1),
                    [x[ix + 1], y[ty], z[iz + 1]],
                    at(ix + 1, ty, iz + 1),
                    [0.0, 1.0, 0.0],
                );
            }
        }
        for iz in 0..(nz - 1) {
            for iy in 0..(ny - 1) {
                push_quad(
                    [x[0], y[iy], z[iz]],
                    at(0, iy, iz),
                    [x[0], y[iy + 1], z[iz]],
                    at(0, iy + 1, iz),
                    [x[0], y[iy], z[iz + 1]],
                    at(0, iy, iz + 1),
                    [x[0], y[iy + 1], z[iz + 1]],
                    at(0, iy + 1, iz + 1),
                    [-1.0, 0.0, 0.0],
                );
                let tx = nx - 1;
                push_quad(
                    [x[tx], y[iy], z[iz]],
                    at(tx, iy, iz),
                    [x[tx], y[iy + 1], z[iz]],
                    at(tx, iy + 1, iz),
                    [x[tx], y[iy], z[iz + 1]],
                    at(tx, iy, iz + 1),
                    [x[tx], y[iy + 1], z[iz + 1]],
                    at(tx, iy + 1, iz + 1),
                    [1.0, 0.0, 0.0],
                );
            }
        }

        let mesh = if !tris.is_empty() {
            TriMesh3DData::new(verts, tris)
                .ok()
                .and_then(|m| m.with_normals(norms).ok())
                .and_then(|m| m.with_vertex_scalars(&scals).ok())
        } else {
            None
        };
        Ok(Self { mesh })
    }

    pub fn mesh(&self) -> Option<&TriMesh3DData> {
        self.mesh.as_ref()
    }
    pub fn sample_bytes(&self) -> usize {
        self.mesh.as_ref().map_or(0, TriMesh3DData::sample_bytes)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.mesh.as_ref().and_then(|m| m.extent(scales))
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.mesh.as_ref().map_or(0, |m| m.domain_invalid(scales))
    }
}

#[allow(clippy::too_many_arguments)]
fn clip_cap_tri(
    p: [Point3D; 3],
    v: [f64; 3],
    face_n: [f64; 3],
    iso: f64,
    threshold: IsocapThreshold,
    verts: &mut Vec<Point3D>,
    norms: &mut Vec<[f64; 3]>,
    scals: &mut Vec<f64>,
    tris: &mut Vec<[u32; 3]>,
) {
    if !v.iter().all(|s| s.is_finite()) {
        return;
    }
    let inside = |s: f64| match threshold {
        IsocapThreshold::Above => s >= iso,
        IsocapThreshold::Below => s <= iso,
    };
    let mut ins = [0usize; 3];
    let mut outs = [0usize; 3];
    let (mut ni, mut no) = (0usize, 0usize);
    for (i, &val) in v.iter().enumerate() {
        if inside(val) {
            ins[ni] = i;
            ni += 1;
        } else {
            outs[no] = i;
            no += 1;
        }
    }
    let lerp_cut = |a: usize, b: usize| -> (Point3D, f64) {
        let den = v[b] - v[a];
        let t = if den.abs() <= 1e-18 {
            0.5
        } else {
            ((iso - v[a]) / den).clamp(0.0, 1.0)
        };
        (
            [
                p[a][0] + t * (p[b][0] - p[a][0]),
                p[a][1] + t * (p[b][1] - p[a][1]),
                p[a][2] + t * (p[b][2] - p[a][2]),
            ],
            iso,
        )
    };
    let mut emit = |a: (Point3D, f64), b: (Point3D, f64), c: (Point3D, f64)| {
        let ab = [b.0[0] - a.0[0], b.0[1] - a.0[1], b.0[2] - a.0[2]];
        let ac = [c.0[0] - a.0[0], c.0[1] - a.0[1], c.0[2] - a.0[2]];
        let cr = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        if cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2] <= 1e-22 {
            return;
        }
        let base = verts.len() as u32;
        for item in [a, b, c] {
            verts.push(item.0);
            norms.push(face_n);
            scals.push(item.1);
        }
        tris.push([base, base + 1, base + 2]);
    };

    match ni {
        3 => emit((p[0], v[0]), (p[1], v[1]), (p[2], v[2])),
        1 => {
            let a = ins[0];
            let c1 = lerp_cut(a, outs[0]);
            let c2 = lerp_cut(a, outs[1]);
            emit((p[a], v[a]), c1, c2);
        }
        2 => {
            let (i0, i1, o) = (ins[0], ins[1], outs[0]);
            let c0 = lerp_cut(i0, o);
            let c1 = lerp_cut(i1, o);
            emit((p[i0], v[i0]), (p[i1], v[i1]), c1);
            emit((p[i0], v[i0]), c1, c0);
        }
        _ => {}
    }
}
