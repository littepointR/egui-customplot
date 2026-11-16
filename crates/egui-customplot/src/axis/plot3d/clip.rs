use std::cmp::Ordering;

/// Liang-Barsky segment clipping against the `[0, 1]^3` unit cube.
pub(crate) fn clip_segment_unit_cube(a: [f64; 3], b: [f64; 3]) -> Option<([f64; 3], [f64; 3])> {
    if a.iter().chain(b.iter()).any(|v| !v.is_finite()) {
        return None;
    }
    let mut t0 = 0.0_f64;
    let mut t1 = 1.0_f64;
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    for i in 0..3 {
        let p = [-d[i], d[i]];
        let q = [a[i], 1.0 - a[i]];
        for k in 0..2 {
            if p[k].abs() <= 1e-15 {
                if q[k] < -1e-9 {
                    return None;
                }
            } else {
                let r = q[k] / p[k];
                if p[k] < 0.0 {
                    if r > t1 {
                        return None;
                    }
                    t0 = t0.max(r);
                } else {
                    if r < t0 {
                        return None;
                    }
                    t1 = t1.min(r);
                }
            }
        }
    }
    if t0 > t1 {
        return None;
    }
    let p0 = [
        (a[0] + t0 * d[0]).clamp(0.0, 1.0),
        (a[1] + t0 * d[1]).clamp(0.0, 1.0),
        (a[2] + t0 * d[2]).clamp(0.0, 1.0),
    ];
    let p1 = [
        (a[0] + t1 * d[0]).clamp(0.0, 1.0),
        (a[1] + t1 * d[1]).clamp(0.0, 1.0),
        (a[2] + t1 * d[2]).clamp(0.0, 1.0),
    ];
    Some((p0, p1))
}

/// Sutherland-Hodgman polygon clipping against all 6 faces of `[0, 1]^3` with `1e-9` tolerance.
pub(crate) fn clip_polygon_unit_cube(poly: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let mut cur = poly.to_vec();
    for axis in 0..3 {
        for (is_min, bound) in [(true, 0.0_f64), (false, 1.0_f64)] {
            if cur.len() < 3 {
                return Vec::new();
            }
            let mut next = Vec::with_capacity(cur.len() + 2);
            let eps = 1e-9;
            let inside = |p: [f64; 3]| -> bool {
                if is_min {
                    p[axis] >= bound - eps
                } else {
                    p[axis] <= bound + eps
                }
            };
            let intersect = |a: [f64; 3], b: [f64; 3]| -> [f64; 3] {
                let denom = b[axis] - a[axis];
                let t = if denom.abs() <= 1e-15 {
                    0.5
                } else {
                    ((bound - a[axis]) / denom).clamp(0.0, 1.0)
                };
                [
                    a[0] + t * (b[0] - a[0]),
                    a[1] + t * (b[1] - a[1]),
                    a[2] + t * (b[2] - a[2]),
                ]
            };
            let mut prev = cur[cur.len() - 1];
            let mut prev_in = inside(prev);
            for &p in &cur {
                let p_in = inside(p);
                if p_in {
                    if !prev_in {
                        next.push(intersect(prev, p));
                    }
                    next.push(p);
                } else if prev_in {
                    next.push(intersect(prev, p));
                }
                prev = p;
                prev_in = p_in;
            }
            cur = next;
        }
    }
    cur
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ClipVertex3D {
    pub u: [f64; 3],
    pub rgba: [f32; 4],
}

pub(crate) fn clip_gouraud_tri_unit_cube(tri: [ClipVertex3D; 3]) -> Vec<ClipVertex3D> {
    let mut cur = tri.to_vec();
    for axis in 0..3 {
        for (is_min, bound) in [(true, 0.0_f64), (false, 1.0_f64)] {
            if cur.len() < 3 {
                return Vec::new();
            }
            let mut next = Vec::with_capacity(cur.len() + 2);
            let eps = 1e-9;
            let inside = |p: [f64; 3]| {
                if is_min {
                    p[axis] >= bound - eps
                } else {
                    p[axis] <= bound + eps
                }
            };
            let mut prev = cur[cur.len() - 1];
            let mut prev_in = inside(prev.u);
            for &v in &cur {
                let v_in = inside(v.u);
                if v_in != prev_in {
                    let denom = v.u[axis] - prev.u[axis];
                    let t = if denom.abs() <= 1e-15 {
                        0.5
                    } else {
                        ((bound - prev.u[axis]) / denom).clamp(0.0, 1.0)
                    };
                    let tf = t as f32;
                    next.push(ClipVertex3D {
                        u: std::array::from_fn(|k| prev.u[k] + t * (v.u[k] - prev.u[k])),
                        rgba: std::array::from_fn(|k| {
                            prev.rgba[k] + tf * (v.rgba[k] - prev.rgba[k])
                        }),
                    });
                }
                if v_in {
                    next.push(v);
                }
                prev = v;
                prev_in = v_in;
            }
            cur = next;
        }
    }
    cur
}

pub(crate) fn clip_triangle_unit_cube(tri: [[f64; 3]; 3]) -> Vec<[f64; 3]> {
    clip_polygon_unit_cube(&tri)
}

/// Computes the ordered convex intersection polygon `[3..=6]` of plane `normal · u = offset` with `[0, 1]^3`.
pub(crate) fn intersect_plane_unit_cube(normal: [f64; 3], offset: f64) -> Vec<[f64; 3]> {
    let nlen = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if !nlen.is_finite() || nlen <= 1e-12 || !offset.is_finite() {
        return Vec::new();
    }
    let edges = [
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ([0.0, 1.0, 0.0], [1.0, 1.0, 0.0]),
        ([0.0, 0.0, 1.0], [1.0, 0.0, 1.0]),
        ([0.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
        ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([1.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
        ([0.0, 0.0, 1.0], [0.0, 1.0, 1.0]),
        ([1.0, 0.0, 1.0], [1.0, 1.0, 1.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([1.0, 0.0, 0.0], [1.0, 0.0, 1.0]),
        ([0.0, 1.0, 0.0], [0.0, 1.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 1.0, 1.0]),
    ];
    let eps = 1e-9;
    let mut hits: Vec<[f64; 3]> = Vec::new();
    for (a, b) in edges {
        let fa = normal[0] * a[0] + normal[1] * a[1] + normal[2] * a[2] - offset;
        let fb = normal[0] * b[0] + normal[1] * b[1] + normal[2] * b[2] - offset;
        if fa.abs() <= eps && fb.abs() <= eps {
            push_unique(&mut hits, a);
            push_unique(&mut hits, b);
        } else if (fa <= eps && fb >= -eps) || (fa >= -eps && fb <= eps) {
            let denom = fb - fa;
            if denom.abs() > 1e-14 {
                let t = (-fa / denom).clamp(0.0, 1.0);
                let p = [
                    (a[0] + t * (b[0] - a[0])).clamp(0.0, 1.0),
                    (a[1] + t * (b[1] - a[1])).clamp(0.0, 1.0),
                    (a[2] + t * (b[2] - a[2])).clamp(0.0, 1.0),
                ];
                push_unique(&mut hits, p);
            }
        }
    }
    if hits.len() < 3 {
        return Vec::new();
    }
    let inv = 1.0 / hits.len() as f64;
    let centroid = [
        hits.iter().map(|p| p[0]).sum::<f64>() * inv,
        hits.iter().map(|p| p[1]).sum::<f64>() * inv,
        hits.iter().map(|p| p[2]).sum::<f64>() * inv,
    ];
    let nn = [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen];
    let seed = if nn[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = normalize(cross(nn, seed));
    let v = cross(nn, u);
    hits.sort_by(|a, b| {
        let da = [a[0] - centroid[0], a[1] - centroid[1], a[2] - centroid[2]];
        let db = [b[0] - centroid[0], b[1] - centroid[1], b[2] - centroid[2]];
        let aa = (da[0] * v[0] + da[1] * v[1] + da[2] * v[2])
            .atan2(da[0] * u[0] + da[1] * u[1] + da[2] * u[2]);
        let ab = (db[0] * v[0] + db[1] * v[1] + db[2] * v[2])
            .atan2(db[0] * u[0] + db[1] * u[1] + db[2] * u[2]);
        aa.partial_cmp(&ab).unwrap_or(Ordering::Equal)
    });
    hits
}

fn push_unique(out: &mut Vec<[f64; 3]>, p: [f64; 3]) {
    if !out.iter().any(|q| {
        (p[0] - q[0]).abs() < 1e-7 && (p[1] - q[1]).abs() < 1e-7 && (p[2] - q[2]).abs() < 1e-7
    }) {
        out.push(p);
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-12);
    [v[0] / l, v[1] / l, v[2] / l]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_helpers_produce_hexagon_for_diagonal_plane_and_clip_segments() {
        let hex = intersect_plane_unit_cube([1.0, 1.0, 1.0], 1.5);
        assert_eq!(hex.len(), 6);
        let seg = clip_segment_unit_cube([-0.5, 0.5, 0.5], [1.5, 0.5, 0.5]).unwrap();
        assert!((seg.0[0] - 0.0).abs() < 1e-9 && (seg.1[0] - 1.0).abs() < 1e-9);
        assert!(clip_segment_unit_cube([2.0, 0.5, 0.5], [3.0, 0.5, 0.5]).is_none());
    }
}
