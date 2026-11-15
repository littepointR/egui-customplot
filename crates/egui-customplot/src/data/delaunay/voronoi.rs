use std::{collections::HashMap, sync::Arc};

use super::{
    TriMeshError,
    bowyer::{IsotropicNorm, circumcenter, deduplicate_sites, delaunay_triangles, is_collinear},
};
use crate::{
    AxisScale, PlotBounds, PlotPoint, axis::clip_normalized_segment, data::extents::CachedExtents,
};

/// Immutable host-owned 2D Voronoi diagram clipped to finite bounds (`voronoi` / `EX-015`).
#[derive(Clone, Debug)]
pub struct VoronoiData {
    sites: Arc<[PlotPoint]>,
    edges: Arc<[[PlotPoint; 2]]>,
    clip_bounds: PlotBounds,
    duplicate_sites: usize,
    extents: CachedExtents,
}

impl VoronoiData {
    pub fn from_sites(sites: &[PlotPoint], clip_bounds: PlotBounds) -> Result<Self, TriMeshError> {
        let span = clip_bounds.span();
        if !span[0].is_finite() || !span[1].is_finite() || span[0] <= 0.0 || span[1] <= 0.0 {
            return Err(TriMeshError::InvalidClipBounds);
        }
        let (unique, duplicate_sites) = deduplicate_sites(sites)?;
        let mut extents = CachedExtents::default();
        for &s in &unique {
            extents.push(Some((s, s)));
        }
        let edges = if unique.len() < 2 {
            Vec::new()
        } else {
            let norm = IsotropicNorm::from_sites(&unique);
            let pts: Vec<PlotPoint> = unique.iter().map(|&p| norm.map(p)).collect();
            if is_collinear(&pts) {
                collinear_edges(&unique, clip_bounds)
            } else {
                let tris = delaunay_triangles(&unique)?;
                dual_edges(&unique, &tris, clip_bounds)
            }
        };
        Ok(Self {
            sites: Arc::from(unique),
            edges: Arc::from(edges),
            clip_bounds,
            duplicate_sites,
            extents,
        })
    }

    pub fn from_sites_auto(sites: &[PlotPoint]) -> Result<Self, TriMeshError> {
        let (unique, _) = deduplicate_sites(sites)?;
        let (lo, hi) = if unique.is_empty() {
            ([0.0, 0.0], [1.0, 1.0])
        } else {
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for p in &unique {
                lo[0] = lo[0].min(p[0]);
                lo[1] = lo[1].min(p[1]);
                hi[0] = hi[0].max(p[0]);
                hi[1] = hi[1].max(p[1]);
            }
            let pad_x = (hi[0] - lo[0]).max(1.0) * 0.2;
            let pad_y = (hi[1] - lo[1]).max(1.0) * 0.2;
            (
                [lo[0] - pad_x, lo[1] - pad_y],
                [hi[0] + pad_x, hi[1] + pad_y],
            )
        };
        let bounds = PlotBounds::new(lo, hi).map_err(|_| TriMeshError::InvalidClipBounds)?;
        Self::from_sites(sites, bounds)
    }

    pub fn sites(&self) -> &[PlotPoint] {
        &self.sites
    }
    pub fn edges(&self) -> &[[PlotPoint; 2]] {
        &self.edges
    }
    pub fn clip_bounds(&self) -> PlotBounds {
        self.clip_bounds
    }
    pub fn duplicate_sites(&self) -> usize {
        self.duplicate_sites
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.sites.as_ref()) + std::mem::size_of_val(self.edges.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.sites.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents.extent(scales)
    }
    pub(crate) fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }
}

fn collinear_edges(unique: &[PlotPoint], bounds: PlotBounds) -> Vec<[PlotPoint; 2]> {
    let dir = [unique[1][0] - unique[0][0], unique[1][1] - unique[0][1]];
    let mut order: Vec<usize> = (0..unique.len()).collect();
    order.sort_by(|&i, &j| {
        let pi = (unique[i][0] - unique[0][0]) * dir[0] + (unique[i][1] - unique[0][1]) * dir[1];
        let pj = (unique[j][0] - unique[0][0]) * dir[0] + (unique[j][1] - unique[0][1]) * dir[1];
        pi.total_cmp(&pj)
    });
    let diag = bounds.span()[0].hypot(bounds.span()[1]) * 2.0;
    let mut out = Vec::with_capacity(unique.len() - 1);
    for w in order.windows(2) {
        let a = unique[w[0]];
        let b = unique[w[1]];
        let mid = [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let len = dx.hypot(dy);
        if len == 0.0 {
            continue;
        }
        let n = [-dy / len, dx / len];
        let p0 = [mid[0] - diag * n[0], mid[1] - diag * n[1]];
        let p1 = [mid[0] + diag * n[0], mid[1] + diag * n[1]];
        if let Some(edge) = clip_segment_to_bounds(p0, p1, bounds) {
            out.push(edge);
        }
    }
    out
}

fn dual_edges(unique: &[PlotPoint], tris: &[[u32; 3]], bounds: PlotBounds) -> Vec<[PlotPoint; 2]> {
    let centers: Vec<Option<PlotPoint>> = tris
        .iter()
        .map(|&[i, j, k]| {
            circumcenter(unique[i as usize], unique[j as usize], unique[k as usize]).map(|c| c.0)
        })
        .collect();
    let mut adj: HashMap<[u32; 2], Vec<(usize, u32)>> = HashMap::with_capacity(tris.len() * 2);
    for (ti, &[a, b, c]) in tris.iter().enumerate() {
        for (u, v, opp) in [(a, b, c), (b, c, a), (c, a, b)] {
            adj.entry([u.min(v), u.max(v)]).or_default().push((ti, opp));
        }
    }
    let bounds_center = [
        0.5 * (bounds.min()[0] + bounds.max()[0]),
        0.5 * (bounds.min()[1] + bounds.max()[1]),
    ];
    let bounds_diag = bounds.span()[0].hypot(bounds.span()[1]);
    let mut out = Vec::with_capacity(adj.len());
    for ([u, v], incident) in adj {
        match incident.as_slice() {
            [(t1, _), (t2, _)] => {
                if let (Some(c1), Some(c2)) = (centers[*t1], centers[*t2])
                    && c1 != c2
                    && let Some(seg) = clip_segment_to_bounds(c1, c2, bounds)
                {
                    out.push(seg);
                }
            }
            [(t1, opp)] => {
                let Some(c) = centers[*t1] else { continue };
                let pu = unique[u as usize];
                let pv = unique[v as usize];
                let pw = unique[*opp as usize];
                let dx = pv[0] - pu[0];
                let dy = pv[1] - pu[1];
                let len = dx.hypot(dy);
                if len == 0.0 {
                    continue;
                }
                let mut n = [-dy / len, dx / len];
                let mid = [0.5 * (pu[0] + pv[0]), 0.5 * (pu[1] + pv[1])];
                // Orient ray strictly away from opposite vertex pw (outside convex hull).
                if n[0] * (pw[0] - mid[0]) + n[1] * (pw[1] - mid[1]) > 0.0 {
                    n = [-n[0], -n[1]];
                }
                let ray_len = 2.0
                    * ((c[0] - bounds_center[0]).hypot(c[1] - bounds_center[1]) + bounds_diag)
                        .max(1.0);
                let far = [c[0] + ray_len * n[0], c[1] + ray_len * n[1]];
                if let Some(seg) = clip_segment_to_bounds(c, far, bounds) {
                    out.push(seg);
                }
            }
            _ => {}
        }
    }
    out
}

fn clip_segment_to_bounds(
    a: PlotPoint,
    b: PlotPoint,
    bounds: PlotBounds,
) -> Option<[PlotPoint; 2]> {
    let lo = bounds.min();
    let span = bounds.span();
    let na = [(a[0] - lo[0]) / span[0], (a[1] - lo[1]) / span[1]];
    let nb = [(b[0] - lo[0]) / span[0], (b[1] - lo[1]) / span[1]];
    let (ca, cb) = clip_normalized_segment(na, nb)?;
    let pa = [lo[0] + ca[0] * span[0], lo[1] + ca[1] * span[1]];
    let pb = [lo[0] + cb[0] * span[0], lo[1] + cb[1] * span[1]];
    (pa != pb).then_some([pa, pb])
}
