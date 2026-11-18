use std::{f64::consts::PI, sync::Arc};

use super::CachedExtents3D;
use crate::{
    AxisScale, GraphData, GraphEdge, GraphError, Point3D, data::graph::edges::annotate_edges,
};

/// 3D graph node placement layout (`plot3(graph/digraph)`, `EXT-11`).
#[derive(Clone, Debug, PartialEq)]
pub enum GraphLayout3D {
    Force { iterations: usize },
    Spherical,
    Coordinates(Arc<[Point3D]>),
}

impl Default for GraphLayout3D {
    fn default() -> Self {
        Self::Force { iterations: 72 }
    }
}

/// Immutable host-owned 3D network / digraph dataset (`EXT-11`).
#[derive(Clone, Debug)]
pub struct Graph3DData {
    pub(crate) version: u64,
    node_count: usize,
    directed: bool,
    edges: Arc<[GraphEdge]>,
    coords: Arc<[Point3D]>,
    node_scalars: Option<Arc<[f64]>>,
    scalar_range: Option<[f64; 2]>,
    node_labels: Arc<[String]>,
    edge_labels: Arc<[String]>,
    extents: CachedExtents3D,
}

impl Graph3DData {
    pub fn undirected(
        node_count: usize,
        edges: &[[usize; 2]],
        layout: GraphLayout3D,
    ) -> Result<Self, GraphError> {
        Self::build(node_count, false, edges, layout)
    }

    pub fn directed(
        node_count: usize,
        edges: &[[usize; 2]],
        layout: GraphLayout3D,
    ) -> Result<Self, GraphError> {
        Self::build(node_count, true, edges, layout)
    }

    pub fn from_graph(graph: &GraphData, layout: GraphLayout3D) -> Result<Self, GraphError> {
        let raw_edges: Vec<[usize; 2]> =
            graph.edges().iter().map(|e| [e.source, e.target]).collect();
        let weights: Vec<f64> = graph.edges().iter().map(|e| e.weight).collect();
        let mut g3 = Self::build(graph.node_count(), graph.is_directed(), &raw_edges, layout)?;
        if !weights.is_empty() {
            g3 = g3.with_edge_weights(&weights)?;
        }
        if !graph.node_labels().is_empty() {
            g3 = g3.with_node_labels(graph.node_labels())?;
        }
        if !graph.edge_labels().is_empty() {
            g3 = g3.with_edge_labels(graph.edge_labels())?;
        }
        Ok(g3)
    }

    fn build(
        node_count: usize,
        directed: bool,
        raw_edges: &[[usize; 2]],
        layout: GraphLayout3D,
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
        let coords = compute_layout_3d(node_count, raw_edges, layout)?;
        let ann = annotate_edges(node_count, directed, raw_edges);
        let mut extents = CachedExtents3D::default();
        for &p in coords.iter() {
            extents.push_point(p);
        }
        Ok(Self {
            version: crate::data::points::next_version(),
            node_count,
            directed,
            edges: Arc::from(ann.edges),
            coords,
            node_scalars: None,
            scalar_range: None,
            node_labels: Arc::from([]),
            edge_labels: Arc::from([]),
            extents,
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
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn with_node_scalars(mut self, scalars: &[f64]) -> Result<Self, GraphError> {
        if scalars.len() != self.node_count {
            return Err(GraphError::LengthMismatch);
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &s in scalars {
            if s.is_finite() {
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        self.node_scalars = Some(Arc::from(scalars));
        self.scalar_range = (lo <= hi).then_some([lo, hi]);
        self.version = crate::data::points::next_version();
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
        self.version = crate::data::points::next_version();
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
        self.version = crate::data::points::next_version();
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
    pub fn coords(&self) -> &[Point3D] {
        &self.coords
    }
    pub fn node_scalars(&self) -> Option<&[f64]> {
        self.node_scalars.as_deref()
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn node_labels(&self) -> &[String] {
        &self.node_labels
    }
    pub fn edge_labels(&self) -> &[String] {
        &self.edge_labels
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.edges.as_ref()) + std::mem::size_of_val(self.coords.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.coords.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

fn compute_layout_3d(
    n: usize,
    edges: &[[usize; 2]],
    layout: GraphLayout3D,
) -> Result<Arc<[Point3D]>, GraphError> {
    if let GraphLayout3D::Coordinates(coords) = layout {
        if coords.len() != n {
            return Err(GraphError::CoordinateCountMismatch);
        }
        return Ok(coords);
    }
    let mut pos = fibonacci_sphere(n);
    let iterations = match layout {
        GraphLayout3D::Spherical => 0,
        GraphLayout3D::Force { iterations } => iterations.min(200),
        GraphLayout3D::Coordinates(_) => unreachable!(),
    };
    if iterations == 0 || n <= 1 {
        return Ok(Arc::from(pos));
    }
    let k = (8.0 / (n as f64).cbrt()).max(0.45);
    let mut disp = vec![[0.0_f64; 3]; n];
    for step in 0..iterations {
        disp.fill([0.0; 3]);
        for i in 0..n {
            for j in (i + 1)..n {
                let d = [
                    pos[i][0] - pos[j][0],
                    pos[i][1] - pos[j][1],
                    pos[i][2] - pos[j][2],
                ];
                let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
                let rep = (k * k) / (dist * dist);
                for a in 0..3 {
                    let f = (d[a] / dist) * rep;
                    disp[i][a] += f;
                    disp[j][a] -= f;
                }
            }
        }
        for &[u, v] in edges {
            if u == v {
                continue;
            }
            let d = [
                pos[v][0] - pos[u][0],
                pos[v][1] - pos[u][1],
                pos[v][2] - pos[u][2],
            ];
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
            let attr = (dist * dist) / k;
            for a in 0..3 {
                let f = (d[a] / dist) * attr;
                disp[u][a] += f;
                disp[v][a] -= f;
            }
        }
        let temp = 0.28 * (1.0 - step as f64 / iterations as f64).max(0.05);
        for i in 0..n {
            for a in 0..3 {
                disp[i][a] -= 0.08 * pos[i][a];
            }
            let len = (disp[i][0] * disp[i][0] + disp[i][1] * disp[i][1] + disp[i][2] * disp[i][2])
                .sqrt()
                .max(1e-9);
            let step_len = len.min(temp);
            for a in 0..3 {
                pos[i][a] += (disp[i][a] / len) * step_len;
            }
        }
    }
    Ok(Arc::from(pos))
}

fn fibonacci_sphere(n: usize) -> Vec<Point3D> {
    if n == 1 {
        return vec![[0.0, 0.0, 0.0]];
    }
    let golden = PI * (3.0 - 5.0_f64.sqrt());
    (0..n)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let theta = golden * i as f64;
            [r * theta.cos(), r * theta.sin(), z]
        })
        .collect()
}
