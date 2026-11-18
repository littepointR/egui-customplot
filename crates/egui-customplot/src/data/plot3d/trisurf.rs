use std::sync::Arc;

use crate::{AxisScale, PlotPoint, Point3D, TriMesh2D, TriMeshError};

use super::{CachedExtents3D, Plot3DError};

/// Immutable host-owned 3D indexed triangle mesh (`trisurf` / 3D `trimesh`, `EX-013` / `EX-012`).
#[derive(Clone, Debug)]
pub struct TriMesh3DData {
    pub(crate) version: u64,
    vertices: Arc<[Point3D]>,
    triangles: Arc<[[u32; 3]]>,
    edges: Arc<[[u32; 2]]>,
    vertex_normals: Arc<[[f64; 3]]>,
    vertex_scalars: Arc<[f64]>,
    face_scalars: Option<Arc<[f64]>>,
    scalar_range: Option<[f64; 2]>,
    extents: CachedExtents3D,
}

impl TriMesh3DData {
    pub fn new(
        vertices: impl Into<Arc<[Point3D]>>,
        triangles: impl Into<Arc<[[u32; 3]]>>,
    ) -> Result<Self, TriMeshError> {
        let vertices: Arc<[Point3D]> = vertices.into();
        let triangles: Arc<[[u32; 3]]> = triangles.into();
        if vertices.len() < 3 || triangles.is_empty() {
            return Err(TriMeshError::InsufficientPoints);
        }
        if vertices
            .iter()
            .any(|p| !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite())
        {
            return Err(TriMeshError::NonFinitePoint);
        }
        let vlen = vertices.len();
        let mut raw_edges = Vec::with_capacity(triangles.len() * 3);
        let mut accum_normals = vec![[0.0_f64; 3]; vlen];
        let mut extents = CachedExtents3D::default();

        for &[i0, i1, i2] in triangles.iter() {
            let [u0, u1, u2] = [i0 as usize, i1 as usize, i2 as usize];
            if u0 >= vlen || u1 >= vlen || u2 >= vlen || i0 == i1 || i1 == i2 || i0 == i2 {
                return Err(TriMeshError::InvalidTriangleIndex);
            }
            let a = vertices[u0];
            let b = vertices[u1];
            let c = vertices[u2];
            extents.push_point(a);
            extents.push_point(b);
            extents.push_point(c);

            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let fn_raw = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            for u in [u0, u1, u2] {
                for k in 0..3 {
                    accum_normals[u][k] += fn_raw[k];
                }
            }
            for [p, q] in [[i0, i1], [i1, i2], [i2, i0]] {
                raw_edges.push([p.min(q), p.max(q)]);
            }
        }
        raw_edges.sort_unstable();
        raw_edges.dedup();
        let vertex_normals: Vec<[f64; 3]> = accum_normals
            .into_iter()
            .map(|n| {
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if len > 1e-14 && len.is_finite() {
                    [n[0] / len, n[1] / len, n[2] / len]
                } else {
                    [0.0, 0.0, 1.0]
                }
            })
            .collect();

        let mut scalars = Vec::with_capacity(vlen);
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &p in vertices.iter() {
            scalars.push(p[2]);
            lo = lo.min(p[2]);
            hi = hi.max(p[2]);
        }

        Ok(Self {
            version: crate::data::points::next_version(),
            vertices,
            triangles,
            edges: Arc::from(raw_edges),
            vertex_normals: Arc::from(vertex_normals),
            vertex_scalars: Arc::from(scalars),
            face_scalars: None,
            scalar_range: (lo <= hi).then_some([lo, hi]),
            extents,
        })
    }

    /// Triangulates `points` in XY (`TriMesh2D::delaunay`) and preserves full 3D vertex `[x, y, z]`.
    pub fn from_delaunay_xy(points: &[Point3D]) -> Result<Self, TriMeshError> {
        if points.len() < 3 {
            return Err(TriMeshError::InsufficientPoints);
        }
        if points
            .iter()
            .any(|p| !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite())
        {
            return Err(TriMeshError::NonFinitePoint);
        }
        let xy: Vec<PlotPoint> = points.iter().map(|p| [p[0], p[1]]).collect();
        let mesh2d = TriMesh2D::delaunay(&xy)?;
        let mut verts3 = Vec::with_capacity(mesh2d.vertices().len());
        for &[ux, uy] in mesh2d.vertices() {
            let matched = points
                .iter()
                .find(|p| (p[0] - ux).hypot(p[1] - uy) <= 1e-9)
                .copied()
                .unwrap_or([ux, uy, 0.0]);
            verts3.push(matched);
        }
        Self::new(verts3, mesh2d.triangles().to_vec())
    }

    pub fn with_vertex_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.vertices.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &s in scalars {
            if s.is_finite() {
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        self.vertex_scalars = Arc::from(scalars);
        self.face_scalars = None;
        self.scalar_range = (lo <= hi).then_some([lo, hi]);
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn with_face_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.triangles.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &s in scalars {
            if s.is_finite() {
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        self.face_scalars = Some(Arc::from(scalars));
        self.scalar_range = (lo <= hi).then_some([lo, hi]);
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn with_normals(
        mut self,
        normals: impl Into<Arc<[[f64; 3]]>>,
    ) -> Result<Self, Plot3DError> {
        let normals: Arc<[[f64; 3]]> = normals.into();
        if normals.len() != self.vertices.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.vertex_normals = normals;
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
    pub fn edges(&self) -> &[[u32; 2]] {
        &self.edges
    }
    pub fn vertex_normals(&self) -> &[[f64; 3]] {
        &self.vertex_normals
    }
    pub fn vertex_scalars(&self) -> &[f64] {
        &self.vertex_scalars
    }
    pub fn face_scalars(&self) -> Option<&[f64]> {
        self.face_scalars.as_deref()
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.vertices.as_ref())
            + std::mem::size_of_val(self.triangles.as_ref())
            + std::mem::size_of_val(self.edges.as_ref())
            + std::mem::size_of_val(self.vertex_normals.as_ref())
            + std::mem::size_of_val(self.vertex_scalars.as_ref())
            + self
                .face_scalars
                .as_ref()
                .map_or(0, |f| std::mem::size_of_val(f.as_ref()))
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
