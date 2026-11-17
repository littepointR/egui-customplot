use std::sync::Arc;

use egui::Color32;

use super::{CachedExtents3D, Plot3DError};
use crate::{AxisScale, PatchError, Point3D, data::triangulate_indexed_polygon};

/// Immutable host-owned 3D polygon patch mesh (`patch(X, Y, Z)` / Faces+Vertices, `EXT-09`).
/// Supports convex and concave arbitrary polygon faces with vertex/face colors & lighting normals.
#[derive(Clone, Debug)]
pub struct Patch3DData {
    vertices: Arc<[Point3D]>,
    face_offsets: Arc<[usize]>,
    face_indices: Arc<[u32]>,
    triangles: Arc<[[u32; 3]]>,
    tri_face: Arc<[u32]>,
    edges: Arc<[[u32; 2]]>,
    face_normals: Arc<[[f64; 3]]>,
    vertex_normals: Arc<[[f64; 3]]>,
    vertex_scalars: Option<Arc<[f64]>>,
    face_scalars: Option<Arc<[f64]>>,
    vertex_colors: Option<Arc<[Color32]>>,
    face_colors: Option<Arc<[Color32]>>,
    scalar_range: Option<[f64; 2]>,
    extents: CachedExtents3D,
}

impl Patch3DData {
    pub fn new(vertices: impl Into<Arc<[Point3D]>>, faces: &[&[u32]]) -> Result<Self, PatchError> {
        let total: usize = faces.iter().map(|f| f.len()).sum();
        let mut offsets = Vec::with_capacity(faces.len() + 1);
        let mut indices = Vec::with_capacity(total);
        offsets.push(0);
        for &face in faces {
            if face.len() < 3 {
                return Err(PatchError::FaceTooSmall);
            }
            indices.extend_from_slice(face);
            offsets.push(indices.len());
        }
        Self::from_indexed(vertices, offsets, indices)
    }

    pub fn from_indexed(
        vertices: impl Into<Arc<[Point3D]>>,
        face_offsets: impl Into<Arc<[usize]>>,
        face_indices: impl Into<Arc<[u32]>>,
    ) -> Result<Self, PatchError> {
        let vertices: Arc<[Point3D]> = vertices.into();
        let face_offsets: Arc<[usize]> = face_offsets.into();
        let face_indices: Arc<[u32]> = face_indices.into();
        let face_count = face_offsets.len().saturating_sub(1);
        if face_offsets.is_empty()
            || face_offsets[0] != 0
            || *face_offsets.last().unwrap() != face_indices.len()
        {
            return Err(PatchError::IndexOutOfBounds);
        }
        for w in face_offsets.windows(2) {
            if w[1] < w[0] || w[1] - w[0] < 3 {
                return Err(PatchError::FaceTooSmall);
            }
        }
        let vlen = vertices.len();
        if face_indices.iter().any(|&idx| (idx as usize) >= vlen) {
            return Err(PatchError::IndexOutOfBounds);
        }

        let mut extents = CachedExtents3D::default();
        for &p in vertices.iter() {
            extents.push_point(p);
        }

        let mut triangles = Vec::new();
        let mut tri_face = Vec::new();
        let mut edges = Vec::with_capacity(face_indices.len());
        let mut face_normals = Vec::with_capacity(face_count);
        let mut v_norm_acc = vec![[0.0_f64; 3]; vlen];

        for fi in 0..face_count {
            let face = &face_indices[face_offsets[fi]..face_offsets[fi + 1]];
            for i in 0..face.len() {
                let a = face[i];
                let b = face[(i + 1) % face.len()];
                edges.push([a.min(b), a.max(b)]);
            }
            let fn_raw = newell_normal(&vertices, face);
            let fn_len = norm_len(fn_raw);
            let fn_unit = if fn_len > 1e-14 && fn_len.is_finite() {
                [fn_raw[0] / fn_len, fn_raw[1] / fn_len, fn_raw[2] / fn_len]
            } else {
                [0.0, 0.0, 1.0]
            };
            face_normals.push(fn_unit);

            for mut tri in triangulate_face(&vertices, face, fn_unit) {
                let pa = vertices[tri[0] as usize];
                let pb = vertices[tri[1] as usize];
                let pc = vertices[tri[2] as usize];
                let c = cross(sub(pb, pa), sub(pc, pa));
                if dot(c, fn_unit) < 0.0 {
                    tri.swap(1, 2);
                }
                let pa = vertices[tri[0] as usize];
                let pb = vertices[tri[1] as usize];
                let pc = vertices[tri[2] as usize];
                let area_n = cross(sub(pb, pa), sub(pc, pa));
                for u in tri {
                    for k in 0..3 {
                        v_norm_acc[u as usize][k] += area_n[k];
                    }
                }
                triangles.push(tri);
                tri_face.push(fi as u32);
            }
        }

        edges.sort_unstable();
        edges.dedup();
        let vertex_normals: Vec<[f64; 3]> = v_norm_acc
            .into_iter()
            .map(|n| {
                let l = norm_len(n);
                if l > 1e-14 && l.is_finite() {
                    [n[0] / l, n[1] / l, n[2] / l]
                } else {
                    [0.0, 0.0, 1.0]
                }
            })
            .collect();

        Ok(Self {
            vertices,
            face_offsets,
            face_indices,
            triangles: Arc::from(triangles),
            tri_face: Arc::from(tri_face),
            edges: Arc::from(edges),
            face_normals: Arc::from(face_normals),
            vertex_normals: Arc::from(vertex_normals),
            vertex_scalars: None,
            face_scalars: None,
            vertex_colors: None,
            face_colors: None,
            scalar_range: None,
            extents,
        })
    }

    pub fn with_vertex_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.vertices.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.scalar_range = finite_range(scalars);
        self.vertex_scalars = Some(Arc::from(scalars));
        self.face_scalars = None;
        Ok(self)
    }

    pub fn with_face_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.face_count() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.scalar_range = finite_range(scalars);
        self.face_scalars = Some(Arc::from(scalars));
        Ok(self)
    }

    pub fn with_vertex_colors(mut self, colors: &[Color32]) -> Result<Self, Plot3DError> {
        if colors.len() != self.vertices.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.vertex_colors = Some(Arc::from(colors));
        Ok(self)
    }

    pub fn with_face_colors(mut self, colors: &[Color32]) -> Result<Self, Plot3DError> {
        if colors.len() != self.face_count() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.face_colors = Some(Arc::from(colors));
        Ok(self)
    }

    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }
    pub fn face_count(&self) -> usize {
        self.face_offsets.len().saturating_sub(1)
    }
    pub fn face(&self, index: usize) -> &[u32] {
        &self.face_indices[self.face_offsets[index]..self.face_offsets[index + 1]]
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
    pub fn triangle_face_indices(&self) -> &[u32] {
        &self.tri_face
    }
    pub fn edges(&self) -> &[[u32; 2]] {
        &self.edges
    }
    pub fn face_normals(&self) -> &[[f64; 3]] {
        &self.face_normals
    }
    pub fn vertex_normals(&self) -> &[[f64; 3]] {
        &self.vertex_normals
    }
    pub fn vertex_scalars(&self) -> Option<&[f64]> {
        self.vertex_scalars.as_deref()
    }
    pub fn face_scalars(&self) -> Option<&[f64]> {
        self.face_scalars.as_deref()
    }
    pub fn vertex_colors(&self) -> Option<&[Color32]> {
        self.vertex_colors.as_deref()
    }
    pub fn face_colors(&self) -> Option<&[Color32]> {
        self.face_colors.as_deref()
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.vertices.as_ref())
            + std::mem::size_of_val(self.face_offsets.as_ref())
            + std::mem::size_of_val(self.face_indices.as_ref())
            + std::mem::size_of_val(self.triangles.as_ref())
            + std::mem::size_of_val(self.edges.as_ref())
            + std::mem::size_of_val(self.vertex_normals.as_ref())
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

fn triangulate_face(vertices: &[Point3D], face: &[u32], normal: [f64; 3]) -> Vec<[u32; 3]> {
    if face.len() == 3 {
        return vec![[face[0], face[1], face[2]]];
    }
    let origin = vertices[face[0] as usize];
    let ref_ax = if normal[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u_raw = cross(normal, ref_ax);
    let ul = norm_len(u_raw).max(1e-14);
    let u = [u_raw[0] / ul, u_raw[1] / ul, u_raw[2] / ul];
    let v = cross(normal, u);
    let pts2d: Vec<[f64; 2]> = face
        .iter()
        .map(|&idx| {
            let d = sub(vertices[idx as usize], origin);
            [dot(d, u), dot(d, v)]
        })
        .collect();
    let ring: Vec<usize> = (0..face.len()).collect();
    if let Some(tris) = triangulate_indexed_polygon(&pts2d, &ring, &[], [AxisScale::Linear; 2]) {
        return tris
            .into_iter()
            .map(|[a, b, c]| [face[a], face[b], face[c]])
            .collect();
    }
    (1..(face.len() - 1))
        .map(|i| [face[0], face[i], face[i + 1]])
        .collect()
}

fn newell_normal(vertices: &[Point3D], face: &[u32]) -> [f64; 3] {
    let mut n = [0.0; 3];
    for i in 0..face.len() {
        let a = vertices[face[i] as usize];
        let b = vertices[face[(i + 1) % face.len()] as usize];
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

fn finite_range(s: &[f64]) -> Option<[f64; 2]> {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &v in s {
        if v.is_finite() {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    (lo <= hi).then_some([lo, hi])
}

#[inline]
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn norm_len(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}
