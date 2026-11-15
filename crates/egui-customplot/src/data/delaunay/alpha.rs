use std::{collections::HashMap, sync::Arc};

use super::{
    TriMeshError,
    bowyer::{circumcenter, deduplicate_sites, delaunay_triangles},
};
use crate::{AxisScale, PatchData, PlotPoint, data::extents::CachedExtents};

/// Immutable host-owned 2D alpha-shape polygon + extracted boundary (`plot(alphaShape)` / `EX-017`).
#[derive(Clone, Debug)]
pub struct AlphaShapeData {
    patch: PatchData,
    boundary_edges: Arc<[[PlotPoint; 2]]>,
    alpha: f64,
    extents: CachedExtents,
}

impl AlphaShapeData {
    pub fn from_sites(sites: &[PlotPoint], alpha: f64) -> Result<Self, TriMeshError> {
        if !alpha.is_finite() || alpha < 0.0 {
            return Err(TriMeshError::InvalidAlpha);
        }
        let (unique, _) = deduplicate_sites(sites)?;
        if unique.len() < 3 {
            return Err(TriMeshError::InsufficientPoints);
        }
        let tris = delaunay_triangles(&unique)?;
        let mut kept: Vec<[u32; 3]> = Vec::new();
        let mut edge_counts: HashMap<[u32; 2], usize> = HashMap::new();
        let mut extents = CachedExtents::default();
        for t in tris {
            let [a, b, c] = t.map(|i| unique[i as usize]);
            if let Some((_, radius)) = circumcenter(a, b, c)
                && radius <= alpha
            {
                kept.push(t);
                for [u, v] in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
                    *edge_counts.entry([u.min(v), u.max(v)]).or_insert(0) += 1;
                }
                extents.push(Some((
                    [a[0].min(b[0]).min(c[0]), a[1].min(b[1]).min(c[1])],
                    [a[0].max(b[0]).max(c[0]), a[1].max(b[1]).max(c[1])],
                )));
            }
        }
        let mut offsets = Vec::with_capacity(kept.len() + 1);
        let mut indices = Vec::with_capacity(kept.len() * 3);
        offsets.push(0);
        for [i0, i1, i2] in &kept {
            indices.extend([*i0 as usize, *i1 as usize, *i2 as usize]);
            offsets.push(indices.len());
        }
        let patch = PatchData::from_indexed(unique.clone(), offsets, indices)
            .map_err(|_| TriMeshError::InvalidTriangleIndex)?;
        let boundary_edges: Vec<[PlotPoint; 2]> = edge_counts
            .into_iter()
            .filter_map(|([u, v], count)| {
                (count == 1).then_some([unique[u as usize], unique[v as usize]])
            })
            .collect();
        Ok(Self {
            patch,
            boundary_edges: Arc::from(boundary_edges),
            alpha,
            extents,
        })
    }

    pub fn critical_alpha(sites: &[PlotPoint]) -> Result<f64, TriMeshError> {
        let (unique, _) = deduplicate_sites(sites)?;
        if unique.len() < 3 {
            return Err(TriMeshError::InsufficientPoints);
        }
        let tris = delaunay_triangles(&unique)?;
        let max_r = tris
            .iter()
            .filter_map(|t| {
                circumcenter(
                    unique[t[0] as usize],
                    unique[t[1] as usize],
                    unique[t[2] as usize],
                )
                .map(|c| c.1)
            })
            .fold(0.0_f64, f64::max);
        Ok(max_r * (1.0 + 1e-6))
    }

    pub fn patch(&self) -> &PatchData {
        &self.patch
    }
    pub fn boundary_edges(&self) -> &[[PlotPoint; 2]] {
        &self.boundary_edges
    }
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    pub fn sample_bytes(&self) -> usize {
        self.patch.sample_bytes() + std::mem::size_of_val(self.boundary_edges.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.boundary_edges.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }
}
