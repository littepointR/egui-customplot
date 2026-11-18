use std::{collections::HashMap, sync::Arc};

use crate::{AxisScale, Point3D, TriMeshError};

use super::{CachedExtents3D, Plot3DError};

/// Host-owned indexed 3D tetrahedral mesh with shared-face boundary culling (`tetramesh`, `EX-014`).
#[derive(Clone, Debug)]
pub struct TetraMesh3DData {
    pub(crate) version: u64,
    vertices: Arc<[Point3D]>,
    tetrahedra: Arc<[[u32; 4]]>,
    edges: Arc<[[u32; 2]]>,
    all_faces: Arc<[[u32; 3]]>,
    all_face_tet: Arc<[u32]>,
    boundary_faces: Arc<[[u32; 3]]>,
    boundary_face_tet: Arc<[u32]>,
    cell_scalars: Arc<[f64]>,
    scalar_range: Option<[f64; 2]>,
    extents: CachedExtents3D,
}

impl TetraMesh3DData {
    pub fn new(
        vertices: impl Into<Arc<[Point3D]>>,
        tetrahedra: impl Into<Arc<[[u32; 4]]>>,
    ) -> Result<Self, TriMeshError> {
        let vertices: Arc<[Point3D]> = vertices.into();
        let tetrahedra: Arc<[[u32; 4]]> = tetrahedra.into();
        if vertices.len() < 4 || tetrahedra.is_empty() {
            return Err(TriMeshError::InsufficientPoints);
        }
        if vertices
            .iter()
            .any(|p| !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite())
        {
            return Err(TriMeshError::NonFinitePoint);
        }
        let vlen = vertices.len();
        let mut extents = CachedExtents3D::default();
        for &p in vertices.iter() {
            extents.push_point(p);
        }

        let mut raw_edges = Vec::with_capacity(tetrahedra.len() * 6);
        let mut all_faces = Vec::with_capacity(tetrahedra.len() * 4);
        let mut all_face_tet = Vec::with_capacity(tetrahedra.len() * 4);
        let mut counts: HashMap<[u32; 3], usize> = HashMap::new();

        for (ti, &[a, b, c, d]) in tetrahedra.iter().enumerate() {
            let idx = [a, b, c, d];
            if idx.iter().any(|&i| (i as usize) >= vlen)
                || a == b
                || a == c
                || a == d
                || b == c
                || b == d
                || c == d
            {
                return Err(TriMeshError::InvalidTriangleIndex);
            }
            for i in 0..4 {
                for j in (i + 1)..4 {
                    raw_edges.push([idx[i].min(idx[j]), idx[i].max(idx[j])]);
                }
            }
            let faces = [
                ([a, b, c], d),
                ([a, b, d], c),
                ([a, c, d], b),
                ([b, c, d], a),
            ];
            for (mut f, opp) in faces {
                let pa = vertices[f[0] as usize];
                let pb = vertices[f[1] as usize];
                let pc = vertices[f[2] as usize];
                let pd = vertices[opp as usize];
                let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
                let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
                let ad = [pd[0] - pa[0], pd[1] - pa[1], pd[2] - pa[2]];
                let n = [
                    ab[1] * ac[2] - ab[2] * ac[1],
                    ab[2] * ac[0] - ab[0] * ac[2],
                    ab[0] * ac[1] - ab[1] * ac[0],
                ];
                if n[0] * ad[0] + n[1] * ad[1] + n[2] * ad[2] > 0.0 {
                    f.swap(1, 2);
                }
                let mut key = f;
                key.sort_unstable();
                *counts.entry(key).or_insert(0) += 1;
                all_faces.push(f);
                all_face_tet.push(ti as u32);
            }
        }

        raw_edges.sort_unstable();
        raw_edges.dedup();

        let mut boundary_faces = Vec::new();
        let mut boundary_face_tet = Vec::new();
        for (&f, &tid) in all_faces.iter().zip(all_face_tet.iter()) {
            let mut key = f;
            key.sort_unstable();
            if counts.get(&key).copied() == Some(1) {
                boundary_faces.push(f);
                boundary_face_tet.push(tid);
            }
        }

        let scalars: Vec<f64> = (0..tetrahedra.len()).map(|i| i as f64).collect();
        let scalar_range = (!scalars.is_empty()).then(|| [0.0, (tetrahedra.len() - 1) as f64]);
        Ok(Self {
            version: crate::data::points::next_version(),
            vertices,
            tetrahedra,
            edges: Arc::from(raw_edges),
            all_faces: Arc::from(all_faces),
            all_face_tet: Arc::from(all_face_tet),
            boundary_faces: Arc::from(boundary_faces),
            boundary_face_tet: Arc::from(boundary_face_tet),
            cell_scalars: Arc::from(scalars),
            scalar_range,
            extents,
        })
    }

    pub fn with_cell_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.tetrahedra.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &s in scalars {
            if s.is_finite() {
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        self.cell_scalars = Arc::from(scalars);
        self.scalar_range = (lo <= hi).then_some([lo, hi]);
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }
    pub fn tetrahedra(&self) -> &[[u32; 4]] {
        &self.tetrahedra
    }
    pub fn edges(&self) -> &[[u32; 2]] {
        &self.edges
    }
    pub fn all_faces(&self) -> &[[u32; 3]] {
        &self.all_faces
    }
    pub fn all_face_tet(&self) -> &[u32] {
        &self.all_face_tet
    }
    pub fn boundary_faces(&self) -> &[[u32; 3]] {
        &self.boundary_faces
    }
    pub fn boundary_face_tet(&self) -> &[u32] {
        &self.boundary_face_tet
    }
    pub fn cell_scalars(&self) -> &[f64] {
        &self.cell_scalars
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.vertices.as_ref())
            + std::mem::size_of_val(self.tetrahedra.as_ref())
            + std::mem::size_of_val(self.edges.as_ref())
            + std::mem::size_of_val(self.all_faces.as_ref())
            + std::mem::size_of_val(self.boundary_faces.as_ref())
            + std::mem::size_of_val(self.cell_scalars.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.vertices.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}
