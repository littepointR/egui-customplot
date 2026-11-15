use std::collections::HashMap;

use super::{ContourBand, ContourError, ContourLevels, ContourLine};
use crate::{AxisScale, PlotPoint, ScalarField2D};

const MAX_LEVELS: usize = 256;

#[derive(Clone, Copy)]
struct Node {
    p: PlotPoint,
    z: f64,
}

pub(super) fn resolve_line_levels(
    field: &ScalarField2D,
    levels: ContourLevels,
) -> Result<Vec<f64>, ContourError> {
    match levels {
        ContourLevels::Explicit(mut vals) => validate_explicit(&mut vals),
        ContourLevels::Count(count) => {
            if count == 0 || count > MAX_LEVELS {
                return Err(ContourError::InvalidLevels);
            }
            let Some([zmin, zmax]) = field.value_range(AxisScale::Linear) else {
                return Ok(Vec::new());
            };
            if zmin == zmax {
                return Ok(Vec::new());
            }
            let denom = (count + 1) as f64;
            Ok((1..=count)
                .map(|k| zmin + (zmax - zmin) * (k as f64 / denom))
                .collect())
        }
    }
}

pub(super) fn resolve_band_levels(
    field: &ScalarField2D,
    levels: ContourLevels,
) -> Result<(Vec<f64>, Vec<[f64; 2]>), ContourError> {
    let thresholds = match levels {
        ContourLevels::Explicit(mut vals) => validate_explicit(&mut vals)?,
        ContourLevels::Count(count) => {
            if count == 0 || count > MAX_LEVELS {
                return Err(ContourError::InvalidLevels);
            }
            let Some([zmin, zmax]) = field.value_range(AxisScale::Linear) else {
                return Ok((Vec::new(), Vec::new()));
            };
            if zmin == zmax {
                vec![zmin]
            } else {
                let denom = (count + 1) as f64;
                (0..=count)
                    .map(|k| zmin + (zmax - zmin) * (k as f64 / denom))
                    .collect()
            }
        }
    };
    let mut bands = Vec::with_capacity(thresholds.len());
    for (i, &lo) in thresholds.iter().enumerate() {
        let hi = thresholds.get(i + 1).copied().unwrap_or(f64::INFINITY);
        bands.push([lo, hi]);
    }
    Ok((thresholds, bands))
}

fn validate_explicit(vals: &mut Vec<f64>) -> Result<Vec<f64>, ContourError> {
    if vals.is_empty() || vals.len() > MAX_LEVELS || vals.iter().any(|v| !v.is_finite()) {
        return Err(ContourError::InvalidLevels);
    }
    vals.sort_by(f64::total_cmp);
    vals.dedup();
    Ok(std::mem::take(vals))
}

pub(super) fn extract_lines(field: &ScalarField2D, levels: &[f64]) -> Vec<ContourLine> {
    if levels.is_empty() {
        return Vec::new();
    }
    let mut per_level: Vec<Vec<[PlotPoint; 2]>> = vec![Vec::new(); levels.len()];
    visit_subtriangles(field, |tri| {
        let z_min = tri[0].z.min(tri[1].z).min(tri[2].z);
        let z_max = tri[0].z.max(tri[1].z).max(tri[2].z);
        let start = levels.partition_point(|&l| l < z_min);
        let end = levels.partition_point(|&l| l <= z_max);
        for li in start..end {
            if let Some(seg) = tri_isoline(tri, levels[li]) {
                per_level[li].push(seg);
            }
        }
    });
    let mut out = Vec::new();
    for (level, segs) in levels.iter().copied().zip(per_level) {
        for points in chain_segments(segs) {
            out.push(ContourLine {
                level,
                points: points.into_boxed_slice(),
            });
        }
    }
    out
}

pub(super) fn extract_bands(field: &ScalarField2D, band_bounds: &[[f64; 2]]) -> Vec<ContourBand> {
    if band_bounds.is_empty() {
        return Vec::new();
    }
    let mut band_verts: Vec<Vec<PlotPoint>> = vec![Vec::new(); band_bounds.len()];
    let mut band_tris: Vec<Vec<[u32; 3]>> = vec![Vec::new(); band_bounds.len()];
    visit_subtriangles(field, |tri| {
        let z_min = tri[0].z.min(tri[1].z).min(tri[2].z);
        let z_max = tri[0].z.max(tri[1].z).max(tri[2].z);
        let start = band_bounds
            .partition_point(|b| b[1] < z_min)
            .min(band_bounds.len());
        let end = band_bounds.partition_point(|b| b[0] <= z_max);
        for bi in start..end {
            let [lo, hi] = band_bounds[bi];
            let mut poly = clip_halfspace(&tri, lo, true);
            if poly.len() < 3 {
                continue;
            }
            if hi.is_finite() {
                poly = clip_halfspace(&poly, hi, false);
                if poly.len() < 3 {
                    continue;
                }
            }
            let base = band_verts[bi].len() as u32;
            band_verts[bi].extend(poly.iter().map(|n| n.p));
            for j in 1..poly.len() - 1 {
                band_tris[bi].push([base, base + j as u32, base + (j + 1) as u32]);
            }
        }
    });
    band_bounds
        .iter()
        .zip(band_verts)
        .zip(band_tris)
        .map(|((&[lower, upper], verts), tris)| ContourBand {
            lower,
            upper,
            vertices: verts.into_boxed_slice(),
            triangles: tris.into_boxed_slice(),
        })
        .collect()
}

fn visit_subtriangles(field: &ScalarField2D, mut emit: impl FnMut([Node; 3])) {
    let x = field.x_nodes();
    let y = field.y_nodes();
    let cols = field.cols();
    let vals = field.values();
    for row in 0..field.cell_rows() {
        for col in 0..field.cell_cols() {
            let i00 = row * cols + col;
            let z00 = vals[i00];
            let z10 = vals[i00 + 1];
            let z01 = vals[i00 + cols];
            let z11 = vals[i00 + cols + 1];
            if !z00.is_finite() || !z10.is_finite() || !z01.is_finite() || !z11.is_finite() {
                continue;
            }
            let p00 = Node {
                p: [x[col], y[row]],
                z: z00,
            };
            let p10 = Node {
                p: [x[col + 1], y[row]],
                z: z10,
            };
            let p11 = Node {
                p: [x[col + 1], y[row + 1]],
                z: z11,
            };
            let p01 = Node {
                p: [x[col], y[row + 1]],
                z: z01,
            };
            let pc = Node {
                p: [0.5 * (x[col] + x[col + 1]), 0.5 * (y[row] + y[row + 1])],
                z: 0.25 * (z00 + z10 + z11 + z01),
            };
            emit([p00, p10, pc]);
            emit([p10, p11, pc]);
            emit([p11, p01, pc]);
            emit([p01, p00, pc]);
        }
    }
}

fn tri_isoline(tri: [Node; 3], level: f64) -> Option<[PlotPoint; 2]> {
    let above = tri.map(|n| n.z >= level);
    let count = usize::from(above[0]) + usize::from(above[1]) + usize::from(above[2]);
    if count == 0 || count == 3 {
        return None;
    }
    let mut hits = [[0.0; 2]; 2];
    let mut n = 0;
    for i in 0..3 {
        let j = (i + 1) % 3;
        if above[i] != above[j] {
            hits[n] = interp_iso(tri[i], tri[j], level);
            n += 1;
        }
    }
    (n == 2 && hits[0] != hits[1]).then_some(hits)
}

fn interp_iso(mut a: Node, mut b: Node, level: f64) -> PlotPoint {
    // Canonicalize edge orientation so neighboring cells compute identical points.
    if (a.p[0], a.p[1]) > (b.p[0], b.p[1]) {
        std::mem::swap(&mut a, &mut b);
    }
    let dz = b.z - a.z;
    let t = if dz == 0.0 {
        0.0
    } else {
        ((level - a.z) / dz).clamp(0.0, 1.0)
    };
    [
        a.p[0] + t * (b.p[0] - a.p[0]),
        a.p[1] + t * (b.p[1] - a.p[1]),
    ]
}

fn clip_halfspace(poly: &[Node], threshold: f64, keep_ge: bool) -> Vec<Node> {
    let mut out = Vec::with_capacity(poly.len() + 1);
    let inside = |z: f64| {
        if keep_ge {
            z >= threshold
        } else {
            z <= threshold
        }
    };
    let mut prev = *poly.last().expect("non-empty polygon");
    let mut prev_in = inside(prev.z);
    for &curr in poly {
        let curr_in = inside(curr.z);
        if curr_in != prev_in {
            let p = interp_iso(prev, curr, threshold);
            out.push(Node { p, z: threshold });
        }
        if curr_in {
            out.push(curr);
        }
        prev = curr;
        prev_in = curr_in;
    }
    out
}

type Key = (u64, u64);

fn pt_key(p: PlotPoint) -> Key {
    (
        if p[0] == 0.0 { 0 } else { p[0].to_bits() },
        if p[1] == 0.0 { 0 } else { p[1].to_bits() },
    )
}

fn chain_segments(segs: Vec<[PlotPoint; 2]>) -> Vec<Vec<PlotPoint>> {
    if segs.is_empty() {
        return Vec::new();
    }
    let mut adj: HashMap<Key, Vec<usize>> = HashMap::with_capacity(segs.len() * 2);
    for (i, &[a, b]) in segs.iter().enumerate() {
        adj.entry(pt_key(a)).or_default().push(i);
        adj.entry(pt_key(b)).or_default().push(i);
    }
    let mut used = vec![false; segs.len()];
    let mut polylines = Vec::new();

    // Trace open chains first (degree == 1), then remaining closed loops.
    for pass in 0..2 {
        for start_idx in 0..segs.len() {
            if used[start_idx] {
                continue;
            }
            let [a, b] = segs[start_idx];
            if pass == 0 {
                let deg_a = adj[&pt_key(a)].len();
                let deg_b = adj[&pt_key(b)].len();
                if deg_a != 1 && deg_b != 1 {
                    continue;
                }
            }
            used[start_idx] = true;
            let mut line = if pass == 0 && adj[&pt_key(b)].len() == 1 {
                vec![b, a]
            } else {
                vec![a, b]
            };
            extend_chain(&segs, &adj, &mut used, &mut line, true);
            if pass == 1 {
                extend_chain(&segs, &adj, &mut used, &mut line, false);
            }
            polylines.push(line);
        }
    }
    polylines
}

fn extend_chain(
    segs: &[[PlotPoint; 2]],
    adj: &HashMap<Key, Vec<usize>>,
    used: &mut [bool],
    line: &mut Vec<PlotPoint>,
    at_end: bool,
) {
    loop {
        let tip = if at_end {
            *line.last().unwrap()
        } else {
            line[0]
        };
        let key = pt_key(tip);
        let Some(candidates) = adj.get(&key) else {
            break;
        };
        let mut next = None;
        for &seg_idx in candidates {
            if used[seg_idx] {
                continue;
            }
            used[seg_idx] = true;
            let [a, b] = segs[seg_idx];
            next = Some(if pt_key(a) == key { b } else { a });
            break;
        }
        let Some(pt) = next else { break };
        if at_end {
            line.push(pt);
        } else {
            line.insert(0, pt);
        }
    }
}
