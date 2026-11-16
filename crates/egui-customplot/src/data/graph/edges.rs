use std::collections::{HashMap, HashSet};

use crate::PlotPoint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    InvalidNode { index: usize, node_count: usize },
    LengthMismatch,
    CoordinateCountMismatch,
    CycleDetected,
    EmptyGraph,
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNode { index, node_count } => write!(
                f,
                "graph node index {index} out of bounds for {node_count} nodes"
            ),
            Self::LengthMismatch => f.write_str("graph attribute slice length mismatch"),
            Self::CoordinateCountMismatch => {
                f.write_str("graph layout coordinates length must match node count")
            }
            Self::CycleDetected => f.write_str("cycle detected in tree parent relationships"),
            Self::EmptyGraph => f.write_str("graph must contain at least one node"),
        }
    }
}

impl std::error::Error for GraphError {}

/// Single preprocessed edge in [`super::GraphData`] with multi-edge curve disambiguation metadata.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphEdge {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
    pub parallel_index: usize,
    pub parallel_count: usize,
    pub bidirectional_pair: bool,
}

pub(super) struct AnnotatedEdges {
    pub edges: Vec<GraphEdge>,
    pub self_loops: usize,
    pub multi_edges: usize,
    pub in_degree: Vec<usize>,
    pub out_degree: Vec<usize>,
}

pub(super) fn annotate_edges(
    node_count: usize,
    directed: bool,
    raw_edges: &[[usize; 2]],
) -> AnnotatedEdges {
    let mut dir_set: HashSet<(usize, usize)> = HashSet::with_capacity(raw_edges.len());
    let mut group_counts: HashMap<(usize, usize), usize> = HashMap::new();
    let mut in_deg = vec![0usize; node_count];
    let mut out_deg = vec![0usize; node_count];
    let mut self_loops = 0usize;
    for &[u, v] in raw_edges {
        if u == v {
            self_loops += 1;
        }
        out_deg[u] += 1;
        in_deg[v] += 1;
        if !directed {
            if u != v {
                in_deg[u] += 1;
                out_deg[v] += 1;
            }
            let key = (u.min(v), u.max(v));
            *group_counts.entry(key).or_insert(0) += 1;
        } else {
            dir_set.insert((u, v));
            *group_counts.entry((u, v)).or_insert(0) += 1;
        }
    }
    let multi_edges = group_counts.values().filter(|&&c| c > 1).count();
    let mut seen: HashMap<(usize, usize), usize> = HashMap::new();
    let mut out = Vec::with_capacity(raw_edges.len());
    for &[u, v] in raw_edges {
        let key = if directed {
            (u, v)
        } else {
            (u.min(v), u.max(v))
        };
        let pidx = seen.entry(key).and_modify(|k| *k += 1).or_insert(0);
        let pcnt = group_counts[&key];
        let bidir = directed && u != v && dir_set.contains(&(v, u));
        out.push(GraphEdge {
            source: u,
            target: v,
            weight: 1.0,
            parallel_index: *pidx,
            parallel_count: pcnt,
            bidirectional_pair: bidir,
        });
    }
    AnnotatedEdges {
        edges: out,
        self_loops,
        multi_edges,
        in_degree: in_deg,
        out_degree: out_deg,
    }
}

pub(super) fn compute_bounds(coords: &[[f64; 2]]) -> (Option<[PlotPoint; 2]>, usize) {
    let mut lo = [f64::INFINITY, f64::INFINITY];
    let mut hi = [f64::NEG_INFINITY, f64::NEG_INFINITY];
    let mut non_finite = 0usize;
    for &[x, y] in coords {
        if x.is_finite() && y.is_finite() {
            lo[0] = lo[0].min(x);
            lo[1] = lo[1].min(y);
            hi[0] = hi[0].max(x);
            hi[1] = hi[1].max(y);
        } else {
            non_finite += 1;
        }
    }
    let valid = (lo[0] <= hi[0]).then_some([lo, hi]);
    (valid, non_finite)
}
