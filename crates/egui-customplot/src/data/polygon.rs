use std::sync::Arc;

use super::{AreaError, extents::CachedExtents, triangulate::triangulate_indexed_polygon};
use crate::{AxisScale, PlotPoint};

#[derive(Clone, Debug)]
struct RingEntry {
    start: usize,
    len: usize,
    tris: [Option<Arc<[[usize; 3]]>>; 4],
}

/// Immutable host-owned 2D polygons with precomputed per-domain triangulations.
#[derive(Clone, Debug)]
pub struct PolygonData {
    vertices: Arc<[PlotPoint]>,
    rings: Arc<[RingEntry]>,
    extents: CachedExtents,
    invalid_linear: usize,
}

impl PolygonData {
    pub fn new(ring: impl Into<Vec<PlotPoint>>) -> Self {
        Self::from_rings(vec![ring.into()])
    }

    pub fn from_rings(input: Vec<Vec<PlotPoint>>) -> Self {
        let total: usize = input.iter().map(Vec::len).sum();
        let mut vertices = Vec::with_capacity(total);
        let mut rings = Vec::with_capacity(input.len());
        let mut extents = CachedExtents::default();
        let mut invalid_linear = 0;

        for ring in input {
            let start = vertices.len();
            let len = ring.len();
            vertices.extend(ring);
            let ring_pts = &vertices[start..start + len];
            let indices: Vec<usize> = (start..start + len).collect();
            let ring_extent = ring_extent(ring_pts);
            extents.push(ring_extent);
            if ring_extent.is_none() || len < 3 {
                invalid_linear += 1;
            }
            let tris = std::array::from_fn(|domain| {
                let scales = domain_scales(domain);
                ring_extent.filter(|(lo, hi)| {
                    scales[0].forward(lo[0]).is_some()
                        && scales[0].forward(hi[0]).is_some()
                        && scales[1].forward(lo[1]).is_some()
                        && scales[1].forward(hi[1]).is_some()
                })?;
                triangulate_indexed_polygon(&vertices, &indices, &[], scales).map(Arc::from)
            });
            rings.push(RingEntry { start, len, tris });
        }

        Self {
            vertices: Arc::from(vertices),
            rings: Arc::from(rings),
            extents,
            invalid_linear,
        }
    }

    /// Build one or more closed confidence-band polygons split across NaN/Inf breaks.
    pub fn confidence_band(x: &[f64], lower: &[f64], upper: &[f64]) -> Result<Self, AreaError> {
        if x.len() != lower.len() || x.len() != upper.len() {
            return Err(AreaError::LengthMismatch);
        }
        let mut rings = Vec::new();
        let mut i = 0;
        while i < x.len() {
            while i < x.len()
                && (!x[i].is_finite() || !lower[i].is_finite() || !upper[i].is_finite())
            {
                i += 1;
            }
            let start = i;
            while i < x.len() && x[i].is_finite() && lower[i].is_finite() && upper[i].is_finite() {
                i += 1;
            }
            if i - start >= 2 {
                let mut ring = Vec::with_capacity((i - start) * 2);
                for k in start..i {
                    ring.push([x[k], upper[k]]);
                }
                for k in (start..i).rev() {
                    ring.push([x[k], lower[k]]);
                }
                rings.push(ring);
            }
        }
        Ok(Self::from_rings(rings))
    }

    pub fn ring_count(&self) -> usize {
        self.rings.len()
    }
    pub fn vertices(&self) -> &[PlotPoint] {
        &self.vertices
    }
    pub fn ring(&self, index: usize) -> &[PlotPoint] {
        let entry = &self.rings[index];
        &self.vertices[entry.start..entry.start + entry.len]
    }
    pub fn sample_bytes(&self) -> usize {
        self.vertices.len() * std::mem::size_of::<PlotPoint>()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.vertices.as_ptr() as usize, self.sample_bytes())
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
    pub(crate) fn ring_triangles(
        &self,
        index: usize,
        scales: [AxisScale; 2],
    ) -> Option<&[[usize; 3]]> {
        self.rings[index].tris[domain_index(scales)].as_deref()
    }
}

fn ring_extent(points: &[PlotPoint]) -> Option<(PlotPoint, PlotPoint)> {
    if points.len() < 3 {
        return None;
    }
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for p in points {
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

pub(crate) fn domain_scales(domain: usize) -> [AxisScale; 2] {
    [
        if domain & 1 != 0 {
            AxisScale::Log10
        } else {
            AxisScale::Linear
        },
        if domain & 2 != 0 {
            AxisScale::Log10
        } else {
            AxisScale::Linear
        },
    ]
}

pub(crate) fn domain_index(scales: [AxisScale; 2]) -> usize {
    usize::from(scales[0] == AxisScale::Log10) + usize::from(scales[1] == AxisScale::Log10) * 2
}
