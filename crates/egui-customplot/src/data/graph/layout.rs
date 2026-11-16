use std::{f64::consts::TAU, sync::Arc};

use super::GraphError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphLayoutKind {
    Circle,
    Force,
    Layered,
    Coordinates,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphLayout {
    Circle,
    Force { iterations: usize, seed: u64 },
    Layered,
    Coordinates(Arc<[[f64; 2]]>),
}

impl GraphLayout {
    pub fn force_default() -> Self {
        Self::Force {
            iterations: 80,
            seed: 0x517c_c1b7_2722_0a95,
        }
    }
}

pub(super) type ResolvedLayout = (Arc<[[f64; 2]]>, GraphLayoutKind);

pub(super) fn compute_layout(
    n: usize,
    edges: &[[usize; 2]],
    directed: bool,
    layout: GraphLayout,
) -> Result<ResolvedLayout, GraphError> {
    match layout {
        GraphLayout::Coordinates(coords) => {
            if coords.len() != n {
                return Err(GraphError::CoordinateCountMismatch);
            }
            Ok((coords, GraphLayoutKind::Coordinates))
        }
        GraphLayout::Circle => Ok((circle_coords(n, 0), GraphLayoutKind::Circle)),
        GraphLayout::Force { iterations, seed } => Ok((
            force_coords(n, edges, iterations.clamp(1, 250), seed),
            GraphLayoutKind::Force,
        )),
        GraphLayout::Layered => Ok((layered_coords(n, edges, directed), GraphLayoutKind::Layered)),
    }
}

fn circle_coords(n: usize, seed: u64) -> Arc<[[f64; 2]]> {
    if n == 1 {
        return Arc::from([[0.0, 0.0]]);
    }
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    let out: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let angle = (i as f64 / n as f64) * TAU;
            let jitter = if seed == 0 {
                0.0
            } else {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                ((state & 0xFFFF) as f64 / 65535.0 - 0.5) * 0.04
            };
            [(1.0 + jitter) * angle.cos(), (1.0 + jitter) * angle.sin()]
        })
        .collect();
    Arc::from(out)
}

fn force_coords(n: usize, edges: &[[usize; 2]], iterations: usize, seed: u64) -> Arc<[[f64; 2]]> {
    if n <= 1 {
        return Arc::from(vec![[0.0, 0.0]; n]);
    }
    let mut pos: Vec<[f64; 2]> = circle_coords(n, seed.max(1)).to_vec();
    let k = (4.0 / n as f64).sqrt().max(0.05);
    let k2 = k * k;
    let repulse_cap = 200;
    let mut disp = vec![[0.0_f64; 2]; n];

    for iter in 0..iterations {
        disp.fill([0.0, 0.0]);
        let temp = (0.55 * (1.0 - iter as f64 / iterations as f64)).max(0.02);
        // Bounded exact-or-strided repulsion preserves O(1) per-iter cap at large N.
        let step = (n / repulse_cap).max(1);
        let offset = iter % step;
        for i in 0..n {
            let mut j = offset;
            while j < n {
                if i != j {
                    let dx = pos[i][0] - pos[j][0];
                    let dy = pos[i][1] - pos[j][1];
                    let d2 = dx * dx + dy * dy + 1e-6;
                    let d = d2.sqrt();
                    let f = (k2 / d) * step as f64;
                    disp[i][0] += (dx / d) * f;
                    disp[i][1] += (dy / d) * f;
                }
                j += step;
            }
            // Gentle central gravity keeps disconnected components bounded.
            disp[i][0] -= pos[i][0] * 0.04;
            disp[i][1] -= pos[i][1] * 0.04;
        }
        for &[u, v] in edges {
            if u == v {
                continue;
            }
            let dx = pos[u][0] - pos[v][0];
            let dy = pos[u][1] - pos[v][1];
            let d = (dx * dx + dy * dy + 1e-6).sqrt();
            let f = (d * d) / k;
            let fx = (dx / d) * f;
            let fy = (dy / d) * f;
            disp[u][0] -= fx;
            disp[u][1] -= fy;
            disp[v][0] += fx;
            disp[v][1] += fy;
        }
        for i in 0..n {
            let len = disp[i][0].hypot(disp[i][1]);
            if len > 1e-12 {
                let step_len = len.min(temp);
                pos[i][0] += (disp[i][0] / len) * step_len;
                pos[i][1] += (disp[i][1] / len) * step_len;
            }
        }
    }
    normalize_box(&mut pos);
    Arc::from(pos)
}

fn layered_coords(n: usize, edges: &[[usize; 2]], _directed: bool) -> Arc<[[f64; 2]]> {
    if n <= 1 {
        return Arc::from(vec![[0.5, 0.5]; n]);
    }
    let mut adj = vec![Vec::new(); n];
    let mut in_deg = vec![0usize; n];
    for &[u, v] in edges {
        if u != v {
            adj[u].push(v);
            in_deg[v] += 1;
        }
    }
    // Break back-edges with deterministic DFS so cyclic graphs still layer smoothly.
    let mut state = vec![0u8; n];
    let mut dag_adj = vec![Vec::new(); n];
    for start in 0..n {
        if state[start] == 0 {
            dfs_acyclic(start, &adj, &mut state, &mut dag_adj);
        }
    }
    let mut dag_in = vec![0usize; n];
    for list in &dag_adj {
        for &v in list {
            dag_in[v] += 1;
        }
    }
    // Topological longest path defines rank (rank 0 at top).
    let mut rank = vec![0usize; n];
    let mut queue: std::collections::VecDeque<usize> = (0..n).filter(|&i| dag_in[i] == 0).collect();
    while let Some(u) = queue.pop_front() {
        for &v in &dag_adj[u] {
            rank[v] = rank[v].max(rank[u] + 1);
            dag_in[v] -= 1;
            if dag_in[v] == 0 {
                queue.push_back(v);
            }
        }
    }
    let max_rank = rank.iter().copied().max().unwrap_or(0);
    let mut layers = vec![Vec::new(); max_rank + 1];
    for (node, &r) in rank.iter().enumerate() {
        layers[r].push(node);
    }
    // Four passes of barycenter sorting reduce crossings on layered DAGs.
    let mut pos_in_layer = vec![0.0_f64; n];
    for layer in &layers {
        for (i, &node) in layer.iter().enumerate() {
            pos_in_layer[node] = i as f64;
        }
    }
    let mut pred = vec![Vec::new(); n];
    for (u, list) in dag_adj.iter().enumerate() {
        for &v in list {
            pred[v].push(u);
        }
    }
    for sweep in 0..4 {
        let range: Vec<usize> = if sweep % 2 == 0 {
            (1..=max_rank).collect()
        } else {
            (0..max_rank).rev().collect()
        };
        for r in range {
            let neighbors = if sweep % 2 == 0 { &pred } else { &dag_adj };
            layers[r].sort_by(|&a, &b| {
                let ba = barycenter(&neighbors[a], &pos_in_layer);
                let bb = barycenter(&neighbors[b], &pos_in_layer);
                ba.total_cmp(&bb).then(a.cmp(&b))
            });
            for (i, &node) in layers[r].iter().enumerate() {
                pos_in_layer[node] = i as f64;
            }
        }
    }
    let mut out = vec![[0.0, 0.0]; n];
    for (r, layer) in layers.iter().enumerate() {
        let y = if max_rank == 0 {
            0.5
        } else {
            1.0 - r as f64 / max_rank as f64
        };
        let len = layer.len();
        for (idx, &node) in layer.iter().enumerate() {
            let x = if len == 1 {
                0.5
            } else {
                (idx as f64 + 0.5) / len as f64
            };
            out[node] = [x, y];
        }
    }
    Arc::from(out)
}

fn dfs_acyclic(u: usize, adj: &[Vec<usize>], state: &mut [u8], dag_adj: &mut [Vec<usize>]) {
    state[u] = 1;
    for &v in &adj[u] {
        if state[v] == 0 {
            dag_adj[u].push(v);
            dfs_acyclic(v, adj, state, dag_adj);
        } else if state[v] == 2 {
            dag_adj[u].push(v);
        }
    }
    state[u] = 2;
}

fn barycenter(nbrs: &[usize], pos: &[f64]) -> f64 {
    if nbrs.is_empty() {
        0.0
    } else {
        nbrs.iter().map(|&u| pos[u]).sum::<f64>() / nbrs.len() as f64
    }
}

fn normalize_box(pos: &mut [[f64; 2]]) {
    let mut lo = [f64::INFINITY, f64::INFINITY];
    let mut hi = [f64::NEG_INFINITY, f64::NEG_INFINITY];
    for p in pos.iter() {
        lo[0] = lo[0].min(p[0]);
        lo[1] = lo[1].min(p[1]);
        hi[0] = hi[0].max(p[0]);
        hi[1] = hi[1].max(p[1]);
    }
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-6);
    let mid_x = 0.5 * (lo[0] + hi[0]);
    let mid_y = 0.5 * (lo[1] + hi[1]);
    for p in pos.iter_mut() {
        p[0] = ((p[0] - mid_x) / span) * 2.0;
        p[1] = ((p[1] - mid_y) / span) * 2.0;
    }
}
