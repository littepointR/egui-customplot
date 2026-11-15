use std::sync::Arc;

use super::{
    extents::CachedExtents,
    polygon::{domain_index, domain_scales},
    triangulate::triangulate_indexed_polygon,
};
use crate::{AxisScale, PlotPoint};

/// A single outer ring with zero or more interior hole rings (`polyshape` region).
#[derive(Clone, Debug, PartialEq)]
pub struct PolyRegion {
    pub outer: Vec<PlotPoint>,
    pub holes: Vec<Vec<PlotPoint>>,
}

impl PolyRegion {
    pub fn new(outer: impl Into<Vec<PlotPoint>>) -> Self {
        Self {
            outer: outer.into(),
            holes: Vec::new(),
        }
    }

    pub fn hole(mut self, hole: impl Into<Vec<PlotPoint>>) -> Self {
        self.holes.push(hole.into());
        self
    }
}

#[derive(Clone, Debug)]
struct RegionRecord {
    vertices: Arc<[PlotPoint]>,
    outer_len: usize,
    hole_offsets: Arc<[usize]>,
    tris: [Option<Arc<[[usize; 3]]>>; 4],
}

/// Immutable host-owned multi-region / holed polygon set (MATLAB `polyshape`).
#[derive(Clone, Debug)]
pub struct PolyShapeData {
    regions: Arc<[RegionRecord]>,
    extents: CachedExtents,
    invalid_linear: usize,
    total_vertex_bytes: usize,
}

impl PolyShapeData {
    pub fn new(input: Vec<PolyRegion>) -> Self {
        let mut regions = Vec::with_capacity(input.len());
        let mut extents = CachedExtents::default();
        let mut invalid_linear = 0;
        let mut total_vertex_bytes = 0;

        for region in input {
            let mut vertices = Vec::with_capacity(
                region.outer.len() + region.holes.iter().map(Vec::len).sum::<usize>(),
            );
            let outer_len = region.outer.len();
            vertices.extend(region.outer);
            let mut hole_offsets = Vec::with_capacity(region.holes.len() + 1);
            hole_offsets.push(outer_len);
            for hole in region.holes {
                vertices.extend(hole);
                hole_offsets.push(vertices.len());
            }
            let extent = region_extent(&vertices, outer_len);
            extents.push(extent);
            if extent.is_none() {
                invalid_linear += 1;
            }
            let outer_idx: Vec<usize> = (0..outer_len).collect();
            let hole_idx_vecs: Vec<Vec<usize>> = hole_offsets
                .windows(2)
                .map(|w| (w[0]..w[1]).collect())
                .collect();
            let hole_slices: Vec<&[usize]> = hole_idx_vecs.iter().map(Vec::as_slice).collect();
            let tris = std::array::from_fn(|domain| {
                let scales = domain_scales(domain);
                extent.filter(|(lo, hi)| {
                    scales[0].forward(lo[0]).is_some()
                        && scales[0].forward(hi[0]).is_some()
                        && scales[1].forward(lo[1]).is_some()
                        && scales[1].forward(hi[1]).is_some()
                })?;
                triangulate_indexed_polygon(&vertices, &outer_idx, &hole_slices, scales)
                    .map(Arc::from)
            });
            total_vertex_bytes += vertices.len() * std::mem::size_of::<PlotPoint>();
            regions.push(RegionRecord {
                vertices: Arc::from(vertices),
                outer_len,
                hole_offsets: Arc::from(hole_offsets),
                tris,
            });
        }

        Self {
            regions: Arc::from(regions),
            extents,
            invalid_linear,
            total_vertex_bytes,
        }
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
    pub fn sample_bytes(&self) -> usize {
        self.total_vertex_bytes
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.regions.as_ptr() as usize, self.total_vertex_bytes)
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
    pub(crate) fn region_vertices(&self, index: usize) -> &[PlotPoint] {
        &self.regions[index].vertices
    }
    pub(crate) fn region_outer(&self, index: usize) -> &[PlotPoint] {
        let r = &self.regions[index];
        &r.vertices[..r.outer_len]
    }
    pub(crate) fn region_hole_count(&self, index: usize) -> usize {
        self.regions[index].hole_offsets.len().saturating_sub(1)
    }
    pub(crate) fn region_hole(&self, index: usize, hole: usize) -> &[PlotPoint] {
        let r = &self.regions[index];
        &r.vertices[r.hole_offsets[hole]..r.hole_offsets[hole + 1]]
    }
    pub(crate) fn region_triangles(
        &self,
        index: usize,
        scales: [AxisScale; 2],
    ) -> Option<&[[usize; 3]]> {
        self.regions[index].tris[domain_index(scales)].as_deref()
    }
}

fn region_extent(vertices: &[PlotPoint], outer_len: usize) -> Option<(PlotPoint, PlotPoint)> {
    if outer_len < 3 {
        return None;
    }
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for p in vertices {
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
