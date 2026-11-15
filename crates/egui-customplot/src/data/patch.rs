use std::sync::Arc;

use super::{
    extents::CachedExtents,
    polygon::{domain_index, domain_scales},
    triangulate::triangulate_indexed_polygon,
};
use crate::{AxisScale, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchError {
    FaceTooSmall,
    IndexOutOfBounds,
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::FaceTooSmall => "every patch face must reference at least 3 vertices",
            Self::IndexOutOfBounds => "patch face index exceeds vertex slice length",
        })
    }
}

impl std::error::Error for PatchError {}

/// Immutable host-owned 2D faces sharing a vertex pool (MATLAB `patch` model).
#[derive(Clone, Debug)]
pub struct PatchData {
    vertices: Arc<[PlotPoint]>,
    face_offsets: Arc<[usize]>,
    face_indices: Arc<[usize]>,
    face_tris: [Arc<[[usize; 3]]>; 4],
    face_tri_offsets: [Arc<[usize]>; 4],
    extents: CachedExtents,
    invalid_linear: usize,
}

impl PatchData {
    pub fn new(
        vertices: impl Into<Arc<[PlotPoint]>>,
        faces: &[&[usize]],
    ) -> Result<Self, PatchError> {
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
        vertices: impl Into<Arc<[PlotPoint]>>,
        face_offsets: impl Into<Arc<[usize]>>,
        face_indices: impl Into<Arc<[usize]>>,
    ) -> Result<Self, PatchError> {
        let vertices: Arc<[PlotPoint]> = vertices.into();
        let face_offsets: Arc<[usize]> = face_offsets.into();
        let face_indices: Arc<[usize]> = face_indices.into();
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
        if face_indices.iter().any(|&idx| idx >= vertices.len()) {
            return Err(PatchError::IndexOutOfBounds);
        }

        let mut extents = CachedExtents::default();
        let mut invalid_linear = 0;
        let mut domain_tris = [const { Vec::new() }; 4];
        let mut domain_offsets = [const { Vec::new() }; 4];
        for offsets in &mut domain_offsets {
            offsets.reserve(face_count + 1);
            offsets.push(0);
        }

        for f in 0..face_count {
            let face = &face_indices[face_offsets[f]..face_offsets[f + 1]];
            let extent = face_extent(&vertices, face);
            extents.push(extent);
            if extent.is_none() {
                invalid_linear += 1;
            }
            for domain in 0..4 {
                let scales = domain_scales(domain);
                if let Some((lo, hi)) = extent
                    && scales[0].forward(lo[0]).is_some()
                    && scales[0].forward(hi[0]).is_some()
                    && scales[1].forward(lo[1]).is_some()
                    && scales[1].forward(hi[1]).is_some()
                {
                    if face.len() == 3 {
                        domain_tris[domain].push([face[0], face[1], face[2]]);
                    } else if let Some(tris) =
                        triangulate_indexed_polygon(&vertices, face, &[], scales)
                    {
                        domain_tris[domain].extend(tris);
                    }
                }
                domain_offsets[domain].push(domain_tris[domain].len());
            }
        }

        Ok(Self {
            vertices,
            face_offsets,
            face_indices,
            face_tris: domain_tris.map(Arc::from),
            face_tri_offsets: domain_offsets.map(Arc::from),
            extents,
            invalid_linear,
        })
    }

    pub fn from_polygons(faces: &[&[PlotPoint]]) -> Result<Self, PatchError> {
        let mut vertices = Vec::new();
        let mut offsets = Vec::with_capacity(faces.len() + 1);
        let mut indices = Vec::new();
        offsets.push(0);
        for &face in faces {
            if face.len() < 3 {
                return Err(PatchError::FaceTooSmall);
            }
            let base = vertices.len();
            vertices.extend_from_slice(face);
            indices.extend(base..base + face.len());
            offsets.push(indices.len());
        }
        Self::from_indexed(vertices, offsets, indices)
    }

    pub fn vertices(&self) -> &[PlotPoint] {
        &self.vertices
    }
    pub fn face_count(&self) -> usize {
        self.face_offsets.len().saturating_sub(1)
    }
    pub fn face(&self, index: usize) -> &[usize] {
        &self.face_indices[self.face_offsets[index]..self.face_offsets[index + 1]]
    }
    pub fn sample_bytes(&self) -> usize {
        self.vertices.len() * std::mem::size_of::<PlotPoint>()
            + (self.face_offsets.len() + self.face_indices.len()) * std::mem::size_of::<usize>()
    }
    pub(crate) fn allocations(&self) -> [(usize, usize); 2] {
        [
            (
                self.vertices.as_ptr() as usize,
                self.vertices.len() * std::mem::size_of::<PlotPoint>(),
            ),
            (
                self.face_indices.as_ptr() as usize,
                (self.face_offsets.len() + self.face_indices.len()) * std::mem::size_of::<usize>(),
            ),
        ]
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid_linear(&self) -> usize {
        self.invalid_linear
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents
            .invalid(scales)
            .saturating_sub(self.invalid_linear)
    }
    pub(crate) fn face_triangles(&self, face: usize, scales: [AxisScale; 2]) -> &[[usize; 3]] {
        let d = domain_index(scales);
        &self.face_tris[d][self.face_tri_offsets[d][face]..self.face_tri_offsets[d][face + 1]]
    }
}

fn face_extent(vertices: &[PlotPoint], face: &[usize]) -> Option<(PlotPoint, PlotPoint)> {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for &idx in face {
        let p = vertices[idx];
        if !p[0].is_finite() || !p[1].is_finite() {
            return None;
        }
        lo[0] = lo[0].min(p[0]);
        lo[1] = lo[1].min(p[1]);
        hi[0] = hi[0].max(p[0]);
        hi[1] = hi[1].max(p[1]);
    }
    ((hi[0] - lo[0]).is_finite() && (hi[1] - lo[1]).is_finite()).then_some((lo, hi))
}
