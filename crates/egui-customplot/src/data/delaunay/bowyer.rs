use std::collections::HashSet;

use super::TriMeshError;
use crate::PlotPoint;

pub(super) struct IsotropicNorm {
    pub center: PlotPoint,
    pub inv_span: f64,
}

impl IsotropicNorm {
    pub(super) fn from_sites(sites: &[PlotPoint]) -> Self {
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for &p in sites {
            lo[0] = lo[0].min(p[0]);
            lo[1] = lo[1].min(p[1]);
            hi[0] = hi[0].max(p[0]);
            hi[1] = hi[1].max(p[1]);
        }
        let span = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-12);
        Self {
            center: [0.5 * (lo[0] + hi[0]), 0.5 * (lo[1] + hi[1])],
            inv_span: 1.0 / span,
        }
    }

    pub(super) fn map(&self, p: PlotPoint) -> PlotPoint {
        [
            (p[0] - self.center[0]) * self.inv_span,
            (p[1] - self.center[1]) * self.inv_span,
        ]
    }
}

pub(super) fn deduplicate_sites(
    sites: &[PlotPoint],
) -> Result<(Vec<PlotPoint>, usize), TriMeshError> {
    let mut seen: HashSet<(u64, u64)> = HashSet::with_capacity(sites.len());
    let mut unique = Vec::with_capacity(sites.len());
    let mut dups = 0usize;
    for &p in sites {
        if !p[0].is_finite() || !p[1].is_finite() {
            return Err(TriMeshError::NonFinitePoint);
        }
        let key = (
            if p[0] == 0.0 { 0 } else { p[0].to_bits() },
            if p[1] == 0.0 { 0 } else { p[1].to_bits() },
        );
        if seen.insert(key) {
            unique.push(p);
        } else {
            dups += 1;
        }
    }
    Ok((unique, dups))
}

pub(super) fn is_collinear(normed: &[PlotPoint]) -> bool {
    if normed.len() < 3 {
        return true;
    }
    let a = normed[0];
    let mut b = normed[1];
    let mut max_d2 = 0.0;
    for &p in &normed[1..] {
        let d2 = (p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2);
        if d2 > max_d2 {
            max_d2 = d2;
            b = p;
        }
    }
    if max_d2 == 0.0 {
        return true;
    }
    let len = max_d2.sqrt();
    let max_cross = normed
        .iter()
        .map(|&p| ((b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])).abs() / len)
        .fold(0.0, f64::max);
    max_cross <= 1e-11
}

pub(super) fn delaunay_triangles(unique: &[PlotPoint]) -> Result<Vec<[u32; 3]>, TriMeshError> {
    let norm = IsotropicNorm::from_sites(unique);
    let mut pts: Vec<PlotPoint> = unique.iter().map(|&p| norm.map(p)).collect();
    if is_collinear(&pts) {
        return Err(TriMeshError::DegenerateSites);
    }
    let n = pts.len();
    pts.push([-100.0, -100.0]);
    pts.push([100.0, -100.0]);
    pts.push([0.0, 100.0]);
    let mut tris: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];

    // Deterministic sweep order reduces worst-case super-triangle cascades on sorted sites.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| {
        pts[i][0]
            .total_cmp(&pts[j][0])
            .then_with(|| pts[i][1].total_cmp(&pts[j][1]))
    });

    let mut bad = Vec::new();
    let mut edges: Vec<[usize; 2]> = Vec::new();
    for &p_idx in &order {
        let p = pts[p_idx];
        bad.clear();
        for (ti, &t) in tris.iter().enumerate() {
            if in_circumcircle(pts[t[0]], pts[t[1]], pts[t[2]], p) {
                bad.push(ti);
            }
        }
        edges.clear();
        for &ti in &bad {
            let t = tris[ti];
            for e in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
                if let Some(pos) = edges.iter().position(|other| *other == [e[1], e[0]]) {
                    edges.swap_remove(pos);
                } else {
                    edges.push(e);
                }
            }
        }
        for &ti in bad.iter().rev() {
            tris.swap_remove(ti);
        }
        for &[u, v] in &edges {
            let mut tri = [u, v, p_idx];
            if orient2d(pts[tri[0]], pts[tri[1]], pts[tri[2]]) < 0.0 {
                tri.swap(0, 1);
            }
            tris.push(tri);
        }
    }

    let mut out = Vec::with_capacity(tris.len());
    for t in tris {
        if t[0] < n
            && t[1] < n
            && t[2] < n
            && orient2d(pts[t[0]], pts[t[1]], pts[t[2]]).abs() > 1e-14
        {
            out.push([t[0] as u32, t[1] as u32, t[2] as u32]);
        }
    }
    if out.is_empty() {
        return Err(TriMeshError::DegenerateSites);
    }
    Ok(out)
}

pub(super) fn orient2d(a: PlotPoint, b: PlotPoint, c: PlotPoint) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

pub(super) fn in_circumcircle(a: PlotPoint, b: PlotPoint, c: PlotPoint, d: PlotPoint) -> bool {
    let ax = a[0] - d[0];
    let ay = a[1] - d[1];
    let bx = b[0] - d[0];
    let by = b[1] - d[1];
    let cx = c[0] - d[0];
    let cy = c[1] - d[1];
    let det = (ax * ax + ay * ay) * (bx * cy - by * cx) - (bx * bx + by * by) * (ax * cy - ay * cx)
        + (cx * cx + cy * cy) * (ax * by - ay * bx);
    if orient2d(a, b, c) > 0.0 {
        det > 0.0
    } else {
        det < 0.0
    }
}

pub(super) fn circumcenter(a: PlotPoint, b: PlotPoint, c: PlotPoint) -> Option<(PlotPoint, f64)> {
    let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
    if d.abs() <= 1e-24 || !d.is_finite() {
        return None;
    }
    let a2 = a[0] * a[0] + a[1] * a[1];
    let b2 = b[0] * b[0] + b[1] * b[1];
    let c2 = c[0] * c[0] + c[1] * c[1];
    let ux = (a2 * (b[1] - c[1]) + b2 * (c[1] - a[1]) + c2 * (a[1] - b[1])) / d;
    let uy = (a2 * (c[0] - b[0]) + b2 * (a[0] - c[0]) + c2 * (b[0] - a[0])) / d;
    let r = (a[0] - ux).hypot(a[1] - uy);
    (ux.is_finite() && uy.is_finite() && r.is_finite()).then_some(([ux, uy], r))
}
