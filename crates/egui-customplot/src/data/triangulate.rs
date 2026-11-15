use crate::{AxisScale, PlotPoint};

#[cfg(test)]
#[path = "triangulate_tests.rs"]
mod tests;

/// Project vertices into the requested axis domain and triangulate one outer
/// ring minus zero or more interior holes. Indices reference `vertices`.
pub(crate) fn triangulate_indexed_polygon(
    vertices: &[PlotPoint],
    outer: &[usize],
    holes: &[&[usize]],
    scales: [AxisScale; 2],
) -> Option<Vec<[usize; 3]>> {
    let mut proj = vec![[0.0; 2]; vertices.len()];
    for &idx in outer.iter().chain(holes.iter().flat_map(|h| h.iter())) {
        let p = *vertices.get(idx)?;
        proj[idx] = [scales[0].forward(p[0])?, scales[1].forward(p[1])?];
    }
    let mut outer_loop = clean_ring(&proj, outer);
    if outer_loop.len() < 3 {
        return None;
    }
    let outer_area = signed_area(&proj, &outer_loop);
    if outer_area == 0.0 {
        return None;
    }
    if outer_area < 0.0 {
        outer_loop.reverse();
    }

    let mut clean_holes = Vec::new();
    for &hole in holes {
        let mut loop_idx = clean_ring(&proj, hole);
        if loop_idx.len() < 3 {
            continue;
        }
        let area = signed_area(&proj, &loop_idx);
        if area == 0.0 {
            continue;
        }
        if area > 0.0 {
            loop_idx.reverse();
        }
        let (max_pos, max_pt) = rightmost_vertex(&proj, &loop_idx);
        clean_holes.push((max_pt[0], max_pt[1], max_pos, loop_idx));
    }
    clean_holes.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| b.1.total_cmp(&a.1)));

    for (_, _, max_pos, hole) in clean_holes {
        bridge_hole(&proj, &mut outer_loop, &hole, max_pos);
    }

    Some(ear_clip(&proj, outer_loop))
}

fn clean_ring(proj: &[PlotPoint], ring: &[usize]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(ring.len());
    for &idx in ring {
        if out.last().is_some_and(|&prev| proj[prev] == proj[idx]) {
            continue;
        }
        out.push(idx);
    }
    while out.len() > 1 && proj[out[0]] == proj[*out.last().unwrap()] {
        out.pop();
    }
    let mut changed = true;
    while changed && out.len() >= 3 {
        changed = false;
        let n = out.len();
        for i in 0..n {
            let a = proj[out[(i + n - 1) % n]];
            let b = proj[out[i]];
            let c = proj[out[(i + 1) % n]];
            if cross(a, b, c) == 0.0
                && ((b[0] - a[0]) * (c[0] - b[0]) + (b[1] - a[1]) * (c[1] - b[1])) >= 0.0
            {
                out.remove(i);
                changed = true;
                break;
            }
        }
    }
    out
}

fn signed_area(proj: &[PlotPoint], ring: &[usize]) -> f64 {
    let n = ring.len();
    let mut sum = 0.0;
    for i in 0..n {
        let a = proj[ring[i]];
        let b = proj[ring[(i + 1) % n]];
        sum += a[0] * b[1] - b[0] * a[1];
    }
    sum * 0.5
}

fn rightmost_vertex(proj: &[PlotPoint], ring: &[usize]) -> (usize, PlotPoint) {
    let mut best_pos = 0;
    let mut best_pt = proj[ring[0]];
    for (pos, &idx) in ring.iter().enumerate().skip(1) {
        let p = proj[idx];
        if p[0] > best_pt[0] || (p[0] == best_pt[0] && p[1] > best_pt[1]) {
            best_pos = pos;
            best_pt = p;
        }
    }
    (best_pos, best_pt)
}

fn bridge_hole(proj: &[PlotPoint], outer: &mut Vec<usize>, hole: &[usize], hole_start: usize) {
    let m_idx = hole[hole_start];
    let m = proj[m_idx];
    let n = outer.len();
    let mut best_edge: Option<(f64, usize, usize)> = None;

    for i in 0..n {
        let a_pos = i;
        let b_pos = (i + 1) % n;
        let a = proj[outer[a_pos]];
        let b = proj[outer[b_pos]];
        let straddles = (a[1] <= m[1] && b[1] > m[1]) || (b[1] <= m[1] && a[1] > m[1]);
        if straddles {
            let x_hit = a[0] + (m[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1]);
            if x_hit >= m[0] && best_edge.is_none_or(|(best_x, _, _)| x_hit < best_x) {
                best_edge = Some((x_hit, a_pos, b_pos));
            }
        } else if a[1] == m[1] && a[0] >= m[0] && best_edge.is_none_or(|(bx, _, _)| a[0] < bx) {
            best_edge = Some((a[0], a_pos, a_pos));
        }
    }

    let p_pos = match best_edge {
        Some((hit_x, a_pos, b_pos)) => {
            let a = proj[outer[a_pos]];
            let b = proj[outer[b_pos]];
            let mut candidate_pos = if a[0] >= b[0] { a_pos } else { b_pos };
            let candidate_pt = proj[outer[candidate_pos]];
            if candidate_pt != [hit_x, m[1]] {
                let mut best_score = f64::NEG_INFINITY;
                let mut best_dist = f64::INFINITY;
                for (pos, &idx) in outer.iter().enumerate() {
                    if pos == candidate_pos {
                        continue;
                    }
                    let r = proj[idx];
                    if r[0] < m[0] {
                        continue;
                    }
                    let prev = proj[outer[(pos + n - 1) % n]];
                    let next = proj[outer[(pos + 1) % n]];
                    if cross(prev, r, next) < 0.0 && point_in_tri(r, m, [hit_x, m[1]], candidate_pt)
                    {
                        let dx = r[0] - m[0];
                        let dy = r[1] - m[1];
                        let dist = dx.hypot(dy);
                        let cos_angle = if dist > 0.0 { dx / dist } else { 1.0 };
                        if cos_angle > best_score || (cos_angle == best_score && dist < best_dist) {
                            best_score = cos_angle;
                            best_dist = dist;
                            candidate_pos = pos;
                        }
                    }
                }
            }
            candidate_pos
        }
        None => (0..n)
            .min_by(|&i, &j| {
                let di = (proj[outer[i]][0] - m[0]).hypot(proj[outer[i]][1] - m[1]);
                let dj = (proj[outer[j]][0] - m[0]).hypot(proj[outer[j]][1] - m[1]);
                di.total_cmp(&dj)
            })
            .unwrap_or(0),
    };

    let p_idx = outer[p_pos];
    let mut spliced = Vec::with_capacity(outer.len() + hole.len() + 2);
    spliced.extend_from_slice(&outer[..=p_pos]);
    for offset in 0..hole.len() {
        spliced.push(hole[(hole_start + offset) % hole.len()]);
    }
    spliced.push(m_idx);
    spliced.push(p_idx);
    spliced.extend_from_slice(&outer[p_pos + 1..]);
    *outer = spliced;
}

fn ear_clip(proj: &[PlotPoint], mut ring: Vec<usize>) -> Vec<[usize; 3]> {
    let mut triangles = Vec::with_capacity(ring.len().saturating_sub(2));
    prune_adjacent(&mut ring);
    while ring.len() >= 3 {
        let n = ring.len();
        let mut clipped = None;
        for pass in 0..2 {
            for i in 0..n {
                let prev = ring[(i + n - 1) % n];
                let curr = ring[i];
                let next = ring[(i + 1) % n];
                if prev == curr || curr == next || prev == next {
                    clipped = Some(i);
                    break;
                }
                let (a, b, c) = (proj[prev], proj[curr], proj[next]);
                let area2 = cross(a, b, c);
                let valid_orient = if pass == 0 { area2 > 0.0 } else { area2 >= 0.0 };
                if !valid_orient {
                    continue;
                }
                let empty = (0..n).all(|j| {
                    let v = ring[j];
                    v == prev
                        || v == curr
                        || v == next
                        || proj[v] == a
                        || proj[v] == b
                        || proj[v] == c
                        || !point_in_tri(proj[v], a, b, c)
                });
                if empty {
                    clipped = Some(i);
                    break;
                }
            }
            if clipped.is_some() {
                break;
            }
        }
        let pos = clipped.unwrap_or(0);
        let prev = ring[(pos + n - 1) % n];
        let curr = ring[pos];
        let next = ring[(pos + 1) % n];
        if prev != curr
            && curr != next
            && prev != next
            && cross(proj[prev], proj[curr], proj[next]) != 0.0
        {
            triangles.push([prev, curr, next]);
        }
        ring.remove(pos);
        prune_adjacent(&mut ring);
    }
    triangles
}

fn prune_adjacent(ring: &mut Vec<usize>) {
    let mut changed = true;
    while changed && ring.len() >= 3 {
        changed = false;
        let n = ring.len();
        for i in 0..n {
            if ring[i] == ring[(i + 1) % n] {
                ring.remove(i);
                changed = true;
                break;
            }
            if ring[(i + n - 1) % n] == ring[(i + 1) % n] {
                ring.remove(i);
                changed = true;
                break;
            }
        }
    }
}

fn cross(a: PlotPoint, b: PlotPoint, c: PlotPoint) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn point_in_tri(p: PlotPoint, a: PlotPoint, b: PlotPoint, c: PlotPoint) -> bool {
    let c1 = cross(a, b, p);
    let c2 = cross(b, c, p);
    let c3 = cross(c, a, p);
    (c1 >= 0.0 && c2 >= 0.0 && c3 >= 0.0) || (c1 <= 0.0 && c2 <= 0.0 && c3 <= 0.0)
}
