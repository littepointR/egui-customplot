use std::sync::Arc;

use super::{GraphData, GraphError, GraphLayout};
use crate::SpyData;

/// Immutable hierarchical tree / forest layout (`treeplot` / `EX-021`).
#[derive(Clone, Debug)]
pub struct TreeData {
    parents: Arc<[Option<usize>]>,
    coords: Arc<[[f64; 2]]>,
    roots: Arc<[usize]>,
    depths: Arc<[usize]>,
    height: usize,
}

impl TreeData {
    /// Accepts MATLAB 1-based parent indices (`0` = root, `1..=n` = 1-based parent).
    pub fn from_parents(matlab_parents_1based: &[usize]) -> Result<Self, GraphError> {
        let n = matlab_parents_1based.len();
        let mut converted = Vec::with_capacity(n);
        for &p in matlab_parents_1based {
            if p > n {
                return Err(GraphError::InvalidNode {
                    index: p - 1,
                    node_count: n,
                });
            }
            converted.push((p > 0).then(|| p - 1));
        }
        Self::from_optional_parents(&converted)
    }

    /// Accepts 0-based optional parent indices (`None` = root).
    pub fn from_optional_parents(parents_0based: &[Option<usize>]) -> Result<Self, GraphError> {
        let n = parents_0based.len();
        if n == 0 {
            return Err(GraphError::EmptyGraph);
        }
        let mut children = vec![Vec::new(); n];
        let mut roots = Vec::new();
        for (i, &parent) in parents_0based.iter().enumerate() {
            match parent {
                None => roots.push(i),
                Some(p) => {
                    if p >= n {
                        return Err(GraphError::InvalidNode {
                            index: p,
                            node_count: n,
                        });
                    }
                    if p == i {
                        return Err(GraphError::CycleDetected);
                    }
                    children[p].push(i);
                }
            }
        }
        if roots.is_empty() {
            return Err(GraphError::CycleDetected);
        }
        let mut state = vec![0u8; n];
        let mut depths = vec![0usize; n];
        verify_and_depth_iterative(&roots, &children, &mut state, &mut depths)?;
        if state.contains(&0) {
            return Err(GraphError::CycleDetected);
        }
        let max_depth = depths.iter().copied().max().unwrap_or(0);
        let mut total_leaves = 0usize;
        for list in &children {
            if list.is_empty() {
                total_leaves += 1;
            }
        }
        total_leaves = total_leaves.max(1);
        let mut x_raw = vec![0.0_f64; n];
        assign_leaf_x_iterative(&roots, &children, &mut x_raw);
        let mut coords = vec![[0.0, 0.0]; n];
        for i in 0..n {
            let x = if total_leaves == 1 {
                0.5
            } else {
                (x_raw[i] + 0.5) / total_leaves as f64
            };
            let y = 1.0 - (depths[i] + 1) as f64 / (max_depth + 2) as f64;
            coords[i] = [x, y];
        }
        Ok(Self {
            parents: Arc::from(parents_0based),
            coords: Arc::from(coords),
            roots: Arc::from(roots),
            depths: Arc::from(depths),
            height: max_depth,
        })
    }

    pub fn node_count(&self) -> usize {
        self.parents.len()
    }
    pub fn parents(&self) -> &[Option<usize>] {
        &self.parents
    }
    pub fn coords(&self) -> &[[f64; 2]] {
        &self.coords
    }
    pub fn roots(&self) -> &[usize] {
        &self.roots
    }
    pub fn depths(&self) -> &[usize] {
        &self.depths
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn to_graph(&self) -> GraphData {
        let mut edges = Vec::new();
        for (i, &p) in self.parents.iter().enumerate() {
            if let Some(parent) = p {
                edges.push([parent, i]);
            }
        }
        GraphData::undirected(
            self.parents.len(),
            &edges,
            GraphLayout::Coordinates(Arc::clone(&self.coords)),
        )
        .expect("validated tree coordinates and bounds")
    }
}

/// Liu's `O(nnz)` Cholesky elimination tree (`etree` / `etreeplot`, `EX-022`), separating computation and rendering.
#[derive(Clone, Debug)]
pub struct EliminationTree {
    parents: Arc<[Option<usize>]>,
    heights: Arc<[usize]>,
    postorder: Arc<[usize]>,
}

impl EliminationTree {
    pub fn from_symmetric_edges(n: usize, edges: &[[usize; 2]]) -> Result<Self, GraphError> {
        if n == 0 {
            return Err(GraphError::EmptyGraph);
        }
        let mut upper = vec![Vec::new(); n];
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
            if u != v {
                let lo = u.min(v);
                let hi = u.max(v);
                upper[hi].push(lo);
            }
        }
        for col in &mut upper {
            col.sort_unstable();
            col.dedup();
        }
        Ok(Self::from_upper_columns(n, &upper))
    }

    pub fn from_spy(spy: &SpyData) -> Result<Self, GraphError> {
        if spy.rows() != spy.cols() {
            return Err(GraphError::CoordinateCountMismatch);
        }
        let n = spy.rows();
        let mut upper = vec![Vec::new(); n];
        let offsets = spy.row_offsets();
        let cols = spy.col_indices();
        for r in 0..n {
            let start = offsets[r] as usize;
            let end = offsets[r + 1] as usize;
            for &c in &cols[start..end] {
                let c = c as usize;
                if r != c {
                    upper[r.max(c)].push(r.min(c));
                }
            }
        }
        for col in &mut upper {
            col.sort_unstable();
            col.dedup();
        }
        Ok(Self::from_upper_columns(n, &upper))
    }

    fn from_upper_columns(n: usize, upper_rows_by_col: &[Vec<usize>]) -> Self {
        // Liu (1990) path-compressed elimination tree computation over columns k = 0..n.
        let mut parent: Vec<Option<usize>> = vec![None; n];
        let mut ancestor: Vec<usize> = (0..n).collect();
        for (k, rows) in upper_rows_by_col.iter().enumerate() {
            for &i in rows {
                let mut r = i;
                while r != k {
                    let next = ancestor[r];
                    ancestor[r] = k;
                    if next == r {
                        parent[r] = Some(k);
                        break;
                    }
                    r = next;
                }
            }
        }
        let mut children = vec![Vec::new(); n];
        let mut roots = Vec::new();
        for (i, &p) in parent.iter().enumerate() {
            if let Some(par) = p {
                children[par].push(i);
            } else {
                roots.push(i);
            }
        }
        let mut heights = vec![0usize; n];
        let mut postorder = Vec::with_capacity(n);
        for &r in &roots {
            compute_postorder(r, &children, &mut heights, &mut postorder);
        }
        Self {
            parents: Arc::from(parent),
            heights: Arc::from(heights),
            postorder: Arc::from(postorder),
        }
    }

    pub fn node_count(&self) -> usize {
        self.parents.len()
    }
    pub fn parents_zero_based(&self) -> &[Option<usize>] {
        &self.parents
    }
    pub fn parents_matlab_one_based(&self) -> Vec<usize> {
        self.parents
            .iter()
            .map(|p| p.map_or(0, |v| v + 1))
            .collect()
    }
    pub fn heights(&self) -> &[usize] {
        &self.heights
    }
    pub fn postorder(&self) -> &[usize] {
        &self.postorder
    }
    pub fn to_tree_data(&self) -> TreeData {
        TreeData::from_optional_parents(&self.parents).expect("elimination tree is always acyclic")
    }
}

fn verify_and_depth_iterative(
    roots: &[usize],
    children: &[Vec<usize>],
    state: &mut [u8],
    depths: &mut [usize],
) -> Result<(), GraphError> {
    let mut stack: Vec<(usize, usize)> = roots.iter().rev().map(|&r| (r, 0)).collect();
    while let Some((node, depth)) = stack.pop() {
        if state[node] != 0 {
            return Err(GraphError::CycleDetected);
        }
        state[node] = 2;
        depths[node] = depth;
        for &c in children[node].iter().rev() {
            stack.push((c, depth + 1));
        }
    }
    Ok(())
}

fn assign_leaf_x_iterative(roots: &[usize], children: &[Vec<usize>], x_raw: &mut [f64]) {
    let mut next_leaf = 0usize;
    let mut stack: Vec<(usize, bool)> = roots.iter().rev().map(|&r| (r, false)).collect();
    while let Some((node, done)) = stack.pop() {
        if done {
            let first = children[node][0];
            let last = children[node][children[node].len() - 1];
            x_raw[node] = 0.5 * (x_raw[first] + x_raw[last]);
        } else if children[node].is_empty() {
            x_raw[node] = next_leaf as f64;
            next_leaf += 1;
        } else {
            stack.push((node, true));
            for &c in children[node].iter().rev() {
                stack.push((c, false));
            }
        }
    }
}

fn compute_postorder(
    node: usize,
    children: &[Vec<usize>],
    heights: &mut [usize],
    postorder: &mut Vec<usize>,
) {
    let mut stack = vec![(node, false)];
    while let Some((cur, visited)) = stack.pop() {
        if visited {
            let h = children[cur]
                .iter()
                .map(|&c| heights[c] + 1)
                .max()
                .unwrap_or(0);
            heights[cur] = h;
            postorder.push(cur);
        } else {
            stack.push((cur, true));
            for &c in children[cur].iter().rev() {
                stack.push((c, false));
            }
        }
    }
}
