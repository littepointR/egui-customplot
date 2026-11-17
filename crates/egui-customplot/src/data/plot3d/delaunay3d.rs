use std::collections::{HashMap, HashSet};

use crate::{Point3D, TriMeshError};

#[derive(Clone, Copy)]
struct Tet {
    v: [u32; 4],
    center: [f64; 3],
    r2: f64,
}

pub(super) type DelaunayTetrahedra = (Vec<Point3D>, Vec<[u32; 4]>, Vec<f64>);

/// Pure-Rust 3D Bowyer-Watson Delaunay tetrahedralization (`EXT-10`).
pub(super) fn delaunay_tetrahedra(sites: &[Point3D]) -> Result<DelaunayTetrahedra, TriMeshError> {
    let unique = deduplicate_sites_3d(sites)?;
    let n = unique.len();
    if n < 4 {
        return Err(TriMeshError::InsufficientPoints);
    }

    let (center, span) = bounds_3d(&unique);
    let inv = 1.0 / span;
    let mut work: Vec<[f64; 3]> = unique
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let q = [
                (p[0] - center[0]) * inv,
                (p[1] - center[1]) * inv,
                (p[2] - center[2]) * inv,
            ];
            let r2 = dot(q, q);
            let pillow = 1.0 + 1e-5 / (0.2 + r2);
            [
                q[0] * pillow + (halton(i + 1, 2) - 0.5) * 5e-7,
                q[1] * pillow + (halton(i + 1, 3) - 0.5) * 5e-7,
                q[2] * pillow + (halton(i + 1, 5) - 0.5) * 5e-7,
            ]
        })
        .collect();

    if is_coplanar(&unique, inv) {
        return Err(TriMeshError::DegenerateSites);
    }

    let m = 8.0_f64;
    let st_base = n as u32;
    work.push([-m, -m, -m]);
    work.push([3.0 * m, -m, -m]);
    work.push([-m, 3.0 * m, -m]);
    work.push([-m, -m, 3.0 * m]);

    let mut tets = Vec::new();
    if let Some(st) = make_tet(&work, [st_base, st_base + 1, st_base + 2, st_base + 3]) {
        tets.push(st);
    } else {
        return Err(TriMeshError::DegenerateSites);
    }

    for pi in 0..n {
        let p = work[pi];
        let mut bad = vec![false; tets.len()];
        let mut any_bad = false;
        for (ti, t) in tets.iter().enumerate() {
            let d2 = dist2(p, t.center);
            if d2 <= t.r2 * (1.0 + 1e-13) {
                bad[ti] = true;
                any_bad = true;
            }
        }
        if !any_bad {
            continue;
        }
        let mut face_counts: HashMap<[u32; 3], (usize, [u32; 3])> = HashMap::new();
        for (ti, is_bad) in bad.iter().enumerate() {
            if !*is_bad {
                continue;
            }
            let [a, b, c, d] = tets[ti].v;
            for f in [[a, b, c], [a, b, d], [a, c, d], [b, c, d]] {
                let mut key = f;
                key.sort_unstable();
                face_counts
                    .entry(key)
                    .and_modify(|e| e.0 += 1)
                    .or_insert((1, f));
            }
        }
        let mut idx = 0;
        while idx < tets.len() {
            if bad[idx] {
                tets.swap_remove(idx);
                bad.swap_remove(idx);
            } else {
                idx += 1;
            }
        }
        let pu = pi as u32;
        for (_, (count, f)) in face_counts {
            if count == 1
                && let Some(t) = make_tet(&work, [f[0], f[1], f[2], pu])
            {
                tets.push(t);
            }
        }
    }

    let mut out_tets = Vec::new();
    let mut out_radii = Vec::new();
    for t in tets {
        if t.v.iter().any(|&v| v >= st_base) {
            continue;
        }
        let [pa, pb, pc, pd] = t.v.map(|i| unique[i as usize]);
        let vol = orient_det(pa, pb, pc, pd);
        if vol.abs() * (inv * inv * inv) <= 1e-13 {
            // Keep non-collinear coplanar interior bridge tetrahedra using half-diameter
            // so internal grid diagonals never leave slit cavities; skip collinear slivers.
            let min_tri = min_face_area([pa, pb, pc, pd]) * inv * inv;
            if min_tri <= 1e-12 {
                continue;
            }
            let r_bridge = max_edge_len([pa, pb, pc, pd]) * 0.5;
            out_tets.push(t.v);
            out_radii.push(r_bridge);
            continue;
        }
        let Some((_, r2)) = circumsphere(pa, pb, pc, pd) else {
            continue;
        };
        let mut v = t.v;
        if vol < 0.0 {
            v.swap(1, 2);
        }
        out_tets.push(v);
        out_radii.push(r2.sqrt());
    }
    if out_tets.is_empty() {
        return Err(TriMeshError::DegenerateSites);
    }
    Ok((unique, out_tets, out_radii))
}

fn deduplicate_sites_3d(sites: &[Point3D]) -> Result<Vec<Point3D>, TriMeshError> {
    let mut seen: HashSet<(u64, u64, u64)> = HashSet::with_capacity(sites.len());
    let mut out = Vec::with_capacity(sites.len());
    for &p in sites {
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() {
            return Err(TriMeshError::NonFinitePoint);
        }
        let key = (
            if p[0] == 0.0 { 0 } else { p[0].to_bits() },
            if p[1] == 0.0 { 0 } else { p[1].to_bits() },
            if p[2] == 0.0 { 0 } else { p[2].to_bits() },
        );
        if seen.insert(key) {
            out.push(p);
        }
    }
    Ok(out)
}

fn bounds_3d(pts: &[Point3D]) -> (Point3D, f64) {
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for &p in pts {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let span = (hi[0] - lo[0])
        .max(hi[1] - lo[1])
        .max(hi[2] - lo[2])
        .max(1e-12);
    let c = [
        0.5 * (lo[0] + hi[0]),
        0.5 * (lo[1] + hi[1]),
        0.5 * (lo[2] + hi[2]),
    ];
    (c, span)
}

fn is_coplanar(pts: &[Point3D], inv: f64) -> bool {
    let p0 = pts[0];
    let mut p1 = None;
    for &p in &pts[1..] {
        if dist2(p, p0).sqrt() * inv > 1e-11 {
            p1 = Some(p);
            break;
        }
    }
    let Some(p1) = p1 else { return true };
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let mut p2 = None;
    for &p in pts {
        let e2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let c = cross(e1, e2);
        if dot(c, c).sqrt() * inv * inv > 1e-11 {
            p2 = Some(p);
            break;
        }
    }
    let Some(p2) = p2 else { return true };
    let inv3 = inv * inv * inv;
    pts.iter()
        .all(|&p3| orient_det(p0, p1, p2, p3).abs() * inv3 <= 1e-12)
}

fn make_tet(work: &[Point3D], v: [u32; 4]) -> Option<Tet> {
    let [a, b, c, d] = v.map(|i| work[i as usize]);
    let (center, r2) = circumsphere(a, b, c, d)?;
    Some(Tet { v, center, r2 })
}

fn circumsphere(a: Point3D, b: Point3D, c: Point3D, d: Point3D) -> Option<(Point3D, f64)> {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let w = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    let rhs = [dot(u, u), dot(v, v), dot(w, w)];
    let uxv = cross(u, v);
    let vxw = cross(v, w);
    let wxu = cross(w, u);
    let denom = 2.0 * dot(u, vxw);
    if denom.abs() <= 1e-24 || !denom.is_finite() {
        return None;
    }
    let rel = [
        (rhs[0] * vxw[0] + rhs[1] * wxu[0] + rhs[2] * uxv[0]) / denom,
        (rhs[0] * vxw[1] + rhs[1] * wxu[1] + rhs[2] * uxv[1]) / denom,
        (rhs[0] * vxw[2] + rhs[1] * wxu[2] + rhs[2] * uxv[2]) / denom,
    ];
    let center = [a[0] + rel[0], a[1] + rel[1], a[2] + rel[2]];
    let r2 = dot(rel, rel);
    r2.is_finite().then_some((center, r2))
}

fn halton(mut idx: usize, base: usize) -> f64 {
    let mut f = 1.0;
    let mut r = 0.0;
    let bf = base as f64;
    while idx > 0 {
        f /= bf;
        r += f * (idx % base) as f64;
        idx /= base;
    }
    r
}

#[inline]
fn orient_det(a: Point3D, b: Point3D, c: Point3D, d: Point3D) -> f64 {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let w = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    dot(u, cross(v, w))
}

#[inline]
fn dist2(a: Point3D, b: Point3D) -> f64 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    dx * dx + dy * dy + dz * dz
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

fn min_face_area(pts: [Point3D; 4]) -> f64 {
    let mut m = f64::INFINITY;
    for (i, j, k) in [(0, 1, 2), (0, 1, 3), (0, 2, 3), (1, 2, 3)] {
        let ab = [
            pts[j][0] - pts[i][0],
            pts[j][1] - pts[i][1],
            pts[j][2] - pts[i][2],
        ];
        let ac = [
            pts[k][0] - pts[i][0],
            pts[k][1] - pts[i][1],
            pts[k][2] - pts[i][2],
        ];
        let c = cross(ab, ac);
        m = m.min(dot(c, c).sqrt());
    }
    m
}

fn max_edge_len(pts: [Point3D; 4]) -> f64 {
    let mut m = 0.0_f64;
    for i in 0..4 {
        for j in (i + 1)..4 {
            m = m.max(dist2(pts[i], pts[j]).sqrt());
        }
    }
    m
}
