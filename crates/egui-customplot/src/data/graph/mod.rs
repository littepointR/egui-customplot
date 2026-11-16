use std::{collections::HashSet, sync::Arc};

use crate::{AxisScale, PlotPoint, SpyData};

mod edges;
mod layout;
mod tree;

use edges::{AnnotatedEdges, annotate_edges, compute_bounds};
pub use edges::{GraphEdge, GraphError};
pub use layout::{GraphLayout, GraphLayoutKind};
pub use tree::{EliminationTree, TreeData};

/// Immutable host-owned 2D graph / digraph / `gplot` network data (`EX-018` / `EX-019` / `EX-020`).
#[derive(Clone, Debug)]
pub struct GraphData {
    node_count: usize,
    directed: bool,
    edges: Arc<[GraphEdge]>,
    coords: Arc<[[f64; 2]]>,
    layout_kind: GraphLayoutKind,
    self_loops: usize,
    multi_edges: usize,
    in_degree: Arc<[usize]>,
    out_degree: Arc<[usize]>,
    node_labels: Arc<[String]>,
    edge_labels: Arc<[String]>,
    finite_bounds: Option<[PlotPoint; 2]>,
    non_finite_nodes: usize,
}

impl GraphData {
    pub fn undirected(
        node_count: usize,
        edges: &[[usize; 2]],
        layout: GraphLayout,
    ) -> Result<Self, GraphError> {
        Self::build(node_count, false, edges, layout)
    }

    pub fn directed(
        node_count: usize,
        edges: &[[usize; 2]],
        layout: GraphLayout,
    ) -> Result<Self, GraphError> {
        Self::build(node_count, true, edges, layout)
    }

    /// `gplot` (`EX-020`): preserves host node coordinates and deduplicates undirected adjacency pairs.
    pub fn from_adjacency_coords(
        edges: &[[usize; 2]],
        coords: &[[f64; 2]],
    ) -> Result<Self, GraphError> {
        if coords.is_empty() {
            return Err(GraphError::EmptyGraph);
        }
        let n = coords.len();
        let mut seen = HashSet::new();
        let mut clean = Vec::new();
        for &[u, v] in edges {
            if u >= n {
                return Err(GraphError::InvalidNode {
                    index: u,
                    node_count: n,
                });
            }
            if v >= n {
                return Err(GraphError::InvalidNode {
                    index: v,
                    node_count: n,
                });
            }
            let key = (u.min(v), u.max(v));
            if seen.insert(key) {
                clean.push([u, v]);
            }
        }
        Self::build(
            n,
            false,
            &clean,
            GraphLayout::Coordinates(Arc::from(coords)),
        )
    }

    pub fn from_dense_adjacency(
        matrix_row_major: &[f64],
        n: usize,
        coords: &[[f64; 2]],
    ) -> Result<Self, GraphError> {
        if n == 0 {
            return Err(GraphError::EmptyGraph);
        }
        if matrix_row_major.len() != n * n {
            return Err(GraphError::LengthMismatch);
        }
        if coords.len() != n {
            return Err(GraphError::CoordinateCountMismatch);
        }
        let mut edges = Vec::new();
        for r in 0..n {
            for c in r..n {
                let a = matrix_row_major[r * n + c];
                let b = matrix_row_major[c * n + r];
                if (a.is_finite() && a != 0.0) || (b.is_finite() && b != 0.0) {
                    edges.push([r, c]);
                }
            }
        }
        Self::from_adjacency_coords(&edges, coords)
    }

    /// Build `gplot` graph from a square [`SpyData`] sparsity pattern and node coordinates (`EX-020`).
    pub fn from_spy(spy: &SpyData, coords: &[[f64; 2]]) -> Result<Self, GraphError> {
        let n = spy.rows().max(spy.cols());
        if coords.len() != n || spy.rows() != spy.cols() {
            return Err(GraphError::CoordinateCountMismatch);
        }
        let mut edges = Vec::new();
        let offsets = spy.row_offsets();
        let cols = spy.col_indices();
        for r in 0..spy.rows() {
            let start = offsets[r] as usize;
            let end = offsets[r + 1] as usize;
            for &c in &cols[start..end] {
                let c = c as usize;
                if r <= c {
                    edges.push([r, c]);
                }
            }
        }
        Self::from_adjacency_coords(&edges, coords)
    }

    fn build(
        node_count: usize,
        directed: bool,
        raw_edges: &[[usize; 2]],
        layout: GraphLayout,
    ) -> Result<Self, GraphError> {
        if node_count == 0 {
            return Err(GraphError::EmptyGraph);
        }
        for &[u, v] in raw_edges {
            if u >= node_count {
                return Err(GraphError::InvalidNode {
                    index: u,
                    node_count,
                });
            }
            if v >= node_count {
                return Err(GraphError::InvalidNode {
                    index: v,
                    node_count,
                });
            }
        }
        let (coords, layout_kind) =
            layout::compute_layout(node_count, raw_edges, directed, layout)?;
        let AnnotatedEdges {
            edges,
            self_loops,
            multi_edges,
            in_degree,
            out_degree,
        } = annotate_edges(node_count, directed, raw_edges);
        let (finite_bounds, non_finite_nodes) = compute_bounds(&coords);
        Ok(Self {
            node_count,
            directed,
            edges: Arc::from(edges),
            coords,
            layout_kind,
            self_loops,
            multi_edges,
            in_degree: Arc::from(in_degree),
            out_degree: Arc::from(out_degree),
            node_labels: Arc::from([]),
            edge_labels: Arc::from([]),
            finite_bounds,
            non_finite_nodes,
        })
    }

    pub fn with_edge_weights(mut self, weights: &[f64]) -> Result<Self, GraphError> {
        if weights.len() != self.edges.len() {
            return Err(GraphError::LengthMismatch);
        }
        let next: Vec<GraphEdge> = self
            .edges
            .iter()
            .zip(weights.iter())
            .map(|(e, &w)| GraphEdge { weight: w, ..*e })
            .collect();
        self.edges = Arc::from(next);
        Ok(self)
    }

    pub fn with_node_labels<S: Into<String> + Clone>(
        mut self,
        labels: &[S],
    ) -> Result<Self, GraphError> {
        if labels.len() != self.node_count {
            return Err(GraphError::LengthMismatch);
        }
        self.node_labels = labels.iter().cloned().map(Into::into).collect();
        Ok(self)
    }

    pub fn with_edge_labels<S: Into<String> + Clone>(
        mut self,
        labels: &[S],
    ) -> Result<Self, GraphError> {
        if labels.len() != self.edges.len() {
            return Err(GraphError::LengthMismatch);
        }
        self.edge_labels = labels.iter().cloned().map(Into::into).collect();
        Ok(self)
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }
    pub fn is_directed(&self) -> bool {
        self.directed
    }
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    pub fn coords(&self) -> &[[f64; 2]] {
        &self.coords
    }
    pub fn layout_kind(&self) -> GraphLayoutKind {
        self.layout_kind
    }
    pub fn self_loops(&self) -> usize {
        self.self_loops
    }
    pub fn multi_edges(&self) -> usize {
        self.multi_edges
    }
    pub fn in_degree(&self, node: usize) -> usize {
        self.in_degree.get(node).copied().unwrap_or(0)
    }
    pub fn out_degree(&self, node: usize) -> usize {
        self.out_degree.get(node).copied().unwrap_or(0)
    }
    pub fn node_labels(&self) -> &[String] {
        &self.node_labels
    }
    pub fn edge_labels(&self) -> &[String] {
        &self.edge_labels
    }
    pub fn non_finite_nodes(&self) -> usize {
        self.non_finite_nodes
    }

    pub fn centroid(&self) -> PlotPoint {
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut count = 0usize;
        for &[x, y] in self.coords.iter() {
            if x.is_finite() && y.is_finite() {
                sx += x;
                sy += y;
                count += 1;
            }
        }
        if count == 0 {
            [0.0, 0.0]
        } else {
            [sx / count as f64, sy / count as f64]
        }
    }

    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.edges.as_ref()) + std::mem::size_of_val(self.coords.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.coords.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        if scales == [AxisScale::Linear, AxisScale::Linear] {
            let [lo, hi] = self.finite_bounds?;
            let pad_x = ((hi[0] - lo[0]) * 0.12).max(0.15);
            let pad_y = ((hi[1] - lo[1]) * 0.12).max(0.15);
            return Some((
                [lo[0] - pad_x, lo[1] - pad_y],
                [hi[0] + pad_x, hi[1] + pad_y],
            ));
        }
        let mut lo = [f64::INFINITY, f64::INFINITY];
        let mut hi = [f64::NEG_INFINITY, f64::NEG_INFINITY];
        for &[x, y] in self.coords.iter() {
            if scales[0].forward(x).is_some() && scales[1].forward(y).is_some() {
                lo[0] = lo[0].min(x);
                lo[1] = lo[1].min(y);
                hi[0] = hi[0].max(x);
                hi[1] = hi[1].max(y);
            }
        }
        (lo[0] <= hi[0]).then_some((lo, hi))
    }
}
