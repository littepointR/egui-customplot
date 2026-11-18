use std::{f64::consts::TAU, sync::Arc};

use crate::{AxisScale, Point3D};

use super::{CachedExtents3D, Plot3DError, TriMesh3DData, VectorField3D, stream_geom::bishop_seed};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cone3DItem {
    pub origin: Point3D,
    pub apex: Point3D,
    pub vector: [f64; 3],
    pub speed: f64,
}

/// Host-owned 3D cone vector field (`coneplot`, `ML-076`).
#[derive(Clone, Debug)]
pub struct ConePlot3DData {
    pub(crate) version: u64,
    mesh: Option<TriMesh3DData>,
    cones: Arc<[Cone3DItem]>,
    extents: CachedExtents3D,
}

impl ConePlot3DData {
    pub fn from_field(
        field: &VectorField3D,
        points: &[Point3D],
        scale: f64,
        radial_subdivisions: usize,
    ) -> Result<Self, Plot3DError> {
        if points.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let mut orig = Vec::new();
        let mut vecs = Vec::new();
        for &p in points {
            if let Some(v) = field.sample(p) {
                orig.push(p);
                vecs.push(v);
            }
        }
        if orig.is_empty() {
            return Ok(Self {
                version: crate::data::points::next_version(),
                mesh: None,
                cones: Arc::from([]),
                extents: CachedExtents3D::default(),
            });
        }
        Self::from_vectors(&orig, &vecs, scale, radial_subdivisions)
    }

    pub fn from_vectors(
        origins: &[Point3D],
        vectors: &[[f64; 3]],
        scale: f64,
        radial_subdivisions: usize,
    ) -> Result<Self, Plot3DError> {
        if origins.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        if origins.len() != vectors.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        if !scale.is_finite() || scale <= 0.0 {
            return Err(Plot3DError::InvalidDimensions);
        }
        let n_div = radial_subdivisions.clamp(4, 36);
        let mut verts = Vec::<Point3D>::new();
        let mut scals = Vec::<f64>::new();
        let mut tris = Vec::<[u32; 3]>::new();
        let mut cones = Vec::new();
        let mut extents = CachedExtents3D::default();

        for (&o, &v) in origins.iter().zip(vectors.iter()) {
            if !o.iter().all(|x| x.is_finite()) || !v.iter().all(|x| x.is_finite()) {
                continue;
            }
            let spd = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if spd <= 1e-12 {
                continue;
            }
            let len = spd * scale;
            let rad = 0.28 * len;
            let dir = [v[0] / spd, v[1] / spd, v[2] / spd];
            let (fu, fv) = bishop_seed(dir);
            let apex = [
                o[0] + dir[0] * len,
                o[1] + dir[1] * len,
                o[2] + dir[2] * len,
            ];
            cones.push(Cone3DItem {
                origin: o,
                apex,
                vector: v,
                speed: spd,
            });
            extents.push_point(o);
            extents.push_point(apex);

            let center_i = verts.len() as u32;
            verts.push(o);
            scals.push(spd);
            let apex_i = verts.len() as u32;
            verts.push(apex);
            scals.push(spd);
            let rim_start = verts.len() as u32;
            for k in 0..n_div {
                let ang = TAU * (k as f64 / n_div as f64);
                let (sa, ca) = ang.sin_cos();
                let rp = [
                    o[0] + rad * (ca * fu[0] + sa * fv[0]),
                    o[1] + rad * (ca * fu[1] + sa * fv[1]),
                    o[2] + rad * (ca * fu[2] + sa * fv[2]),
                ];
                extents.push_point(rp);
                verts.push(rp);
                scals.push(spd);
            }
            for k in 0..n_div {
                let r0 = rim_start + k as u32;
                let r1 = rim_start + ((k + 1) % n_div) as u32;
                tris.push([r0, r1, apex_i]);
                tris.push([center_i, r1, r0]);
            }
        }
        let mesh = (!tris.is_empty())
            .then(|| {
                TriMesh3DData::new(verts, tris)
                    .ok()
                    .and_then(|m| m.with_vertex_scalars(&scals).ok())
            })
            .flatten();
        Ok(Self {
            version: crate::data::points::next_version(),
            mesh,
            cones: Arc::from(cones),
            extents,
        })
    }

    pub fn mesh(&self) -> Option<&TriMesh3DData> {
        self.mesh.as_ref()
    }
    pub fn cones(&self) -> &[Cone3DItem] {
        &self.cones
    }
    pub fn sample_bytes(&self) -> usize {
        self.mesh.as_ref().map_or(0, TriMesh3DData::sample_bytes)
            + std::mem::size_of_val(self.cones.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.cones.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}
