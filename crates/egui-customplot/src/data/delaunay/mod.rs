use std::sync::Arc;

use super::extents::CachedExtents;
use crate::{AxisScale, PlotPoint};

mod alpha;
mod bowyer;
#[cfg(test)]
mod tests;
mod voronoi;

pub use alpha::AlphaShapeData;
pub use voronoi::VoronoiData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriMeshError {
    InsufficientPoints,
    NonFinitePoint,
    InvalidTriangleIndex,
    DegenerateSites,
    InvalidAlpha,
    InvalidClipBounds,
}

impl std::fmt::Display for TriMeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InsufficientPoints => "at least 3 vertices and 1 triangle are required",
            Self::NonFinitePoint => "mesh vertex coordinates must be finite",
            Self::InvalidTriangleIndex => "triangle index exceeds vertex pool or degenerates",
            Self::DegenerateSites => "input sites are collinear or too degenerate to triangulate",
            Self::InvalidAlpha => "alpha radius must be finite and non-negative",
            Self::InvalidClipBounds => "Voronoi clip bounds must have positive finite span",
        })
    }
}

impl std::error::Error for TriMeshError {}

/// Immutable host-owned 2D triangular mesh (`triplot` / 2D `trimesh`), storing deduplicated edges.
#[derive(Clone, Debug)]
pub struct TriMesh2D {
    vertices: Arc<[PlotPoint]>,
    triangles: Arc<[[u32; 3]]>,
    edges: Arc<[[u32; 2]]>,
    extents: CachedExtents,
}

impl TriMesh2D {
    pub fn new(
        vertices: impl Into<Arc<[PlotPoint]>>,
        triangles: impl Into<Arc<[[u32; 3]]>>,
    ) -> Result<Self, TriMeshError> {
        let vertices: Arc<[PlotPoint]> = vertices.into();
        let triangles: Arc<[[u32; 3]]> = triangles.into();
        if vertices.len() < 3 || triangles.is_empty() {
            return Err(TriMeshError::InsufficientPoints);
        }
        if vertices
            .iter()
            .any(|p| !p[0].is_finite() || !p[1].is_finite())
        {
            return Err(TriMeshError::NonFinitePoint);
        }
        let vlen = vertices.len();
        let mut raw_edges = Vec::with_capacity(triangles.len() * 3);
        let mut extents = CachedExtents::default();
        for &[i0, i1, i2] in triangles.iter() {
            let [u0, u1, u2] = [i0 as usize, i1 as usize, i2 as usize];
            if u0 >= vlen || u1 >= vlen || u2 >= vlen || i0 == i1 || i1 == i2 || i0 == i2 {
                return Err(TriMeshError::InvalidTriangleIndex);
            }
            for [a, b] in [[i0, i1], [i1, i2], [i2, i0]] {
                let edge = [a.min(b), a.max(b)];
                raw_edges.push(edge);
                let pa = vertices[edge[0] as usize];
                let pb = vertices[edge[1] as usize];
                extents.push(Some((
                    [pa[0].min(pb[0]), pa[1].min(pb[1])],
                    [pa[0].max(pb[0]), pa[1].max(pb[1])],
                )));
            }
        }
        raw_edges.sort_unstable();
        raw_edges.dedup();
        Ok(Self {
            vertices,
            triangles,
            edges: Arc::from(raw_edges),
            extents,
        })
    }

    pub fn delaunay(sites: &[PlotPoint]) -> Result<Self, TriMeshError> {
        let (unique, _dups) = bowyer::deduplicate_sites(sites)?;
        if unique.len() < 3 {
            return Err(TriMeshError::InsufficientPoints);
        }
        let tris = bowyer::delaunay_triangles(&unique)?;
        Self::new(unique, tris)
    }

    pub fn vertices(&self) -> &[PlotPoint] {
        &self.vertices
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
    pub fn edges(&self) -> &[[u32; 2]] {
        &self.edges
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.vertices.as_ref())
            + std::mem::size_of_val(self.triangles.as_ref())
            + std::mem::size_of_val(self.edges.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.vertices.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }
}
