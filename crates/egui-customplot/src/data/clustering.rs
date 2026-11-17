use std::sync::Arc;

use crate::{AxisScale, BarOrientation, MultivariateError, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkageMethod {
    Single,
    Complete,
    Average,
    Ward,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkageStep {
    pub left: usize,
    pub right: usize,
    pub distance: f64,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DendrogramBranch {
    pub index: usize,
    pub x_left: f64,
    pub x_right: f64,
    pub x_mid: f64,
    pub y_left: f64,
    pub y_right: f64,
    pub y_top: f64,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub struct DendrogramData {
    n_leaves: usize,
    steps: Arc<[LinkageStep]>,
    branches: Arc<[DendrogramBranch]>,
    leaf_order: Arc<[usize]>,
    labels: Vec<String>,
    max_distance: f64,
}

impl DendrogramData {
    pub fn from_observations(
        rows: &[&[f64]],
        method: LinkageMethod,
    ) -> Result<Self, MultivariateError> {
        let n = rows.len();
        if n < 2 {
            return Err(MultivariateError::Empty);
        }
        let cols = rows[0].len();
        if cols == 0 || rows.iter().any(|r| r.len() != cols) {
            return Err(MultivariateError::ShapeMismatch);
        }
        if rows.iter().flat_map(|r| r.iter()).any(|v| !v.is_finite()) {
            return Err(MultivariateError::NoFiniteValues);
        }
        let mut condensed = Vec::with_capacity(n * (n - 1) / 2);
        for i in 0..n {
            for j in (i + 1)..n {
                let d2: f64 = rows[i]
                    .iter()
                    .zip(rows[j].iter())
                    .map(|(&a, &b)| {
                        let d = a - b;
                        d * d
                    })
                    .sum();
                condensed.push(d2.sqrt());
            }
        }
        Self::from_condensed_distances(n, &condensed, method)
    }

    pub fn from_condensed_distances(
        n_leaves: usize,
        distances: &[f64],
        method: LinkageMethod,
    ) -> Result<Self, MultivariateError> {
        if n_leaves < 2 {
            return Err(MultivariateError::Empty);
        }
        let expected = n_leaves * (n_leaves - 1) / 2;
        if distances.len() != expected {
            return Err(MultivariateError::ShapeMismatch);
        }
        if distances.iter().any(|d| !d.is_finite() || *d < 0.0) {
            return Err(MultivariateError::NoFiniteValues);
        }

        let total_nodes = 2 * n_leaves - 1;
        let mut active = vec![false; total_nodes];
        let mut sizes = vec![1usize; total_nodes];
        let mut heights = vec![0.0_f64; total_nodes];
        for flag in active.iter_mut().take(n_leaves) {
            *flag = true;
        }
        let mut mat = vec![0.0_f64; total_nodes * total_nodes];
        let mut k = 0usize;
        for i in 0..n_leaves {
            for j in (i + 1)..n_leaves {
                mat[i * total_nodes + j] = distances[k];
                mat[j * total_nodes + i] = distances[k];
                k += 1;
            }
        }

        let mut steps = Vec::with_capacity(n_leaves - 1);
        for step_idx in 0..(n_leaves - 1) {
            let new_node = n_leaves + step_idx;
            let active_ids: Vec<usize> = (0..new_node).filter(|&id| active[id]).collect();
            let mut best = (0usize, 0usize, f64::INFINITY);
            for a_idx in 0..active_ids.len() {
                let u = active_ids[a_idx];
                for &v in &active_ids[(a_idx + 1)..] {
                    let d = mat[u * total_nodes + v];
                    if d < best.2 {
                        best = (u, v, d);
                    }
                }
            }
            let (u, v, mut d_uv) = best;
            d_uv = d_uv.max(heights[u].max(heights[v]));
            let su = sizes[u] as f64;
            let sv = sizes[v] as f64;
            sizes[new_node] = sizes[u] + sizes[v];
            heights[new_node] = d_uv;
            active[u] = false;
            active[v] = false;
            active[new_node] = true;

            for &w in &active_ids {
                if w == u || w == v {
                    continue;
                }
                let duw = mat[u * total_nodes + w];
                let dvw = mat[v * total_nodes + w];
                let sw = sizes[w] as f64;
                let d_new = match method {
                    LinkageMethod::Single => duw.min(dvw),
                    LinkageMethod::Complete => duw.max(dvw),
                    LinkageMethod::Average => (su * duw + sv * dvw) / (su + sv),
                    LinkageMethod::Ward => {
                        let denom = su + sv + sw;
                        let val = ((su + sw) * duw * duw + (sv + sw) * dvw * dvw
                            - sw * d_uv * d_uv)
                            / denom;
                        val.max(0.0).sqrt()
                    }
                };
                mat[new_node * total_nodes + w] = d_new;
                mat[w * total_nodes + new_node] = d_new;
            }
            steps.push(LinkageStep {
                left: u,
                right: v,
                distance: d_uv,
                count: sizes[new_node],
            });
        }

        let mut leaf_order = Vec::with_capacity(n_leaves);
        collect_leaf_order(total_nodes - 1, n_leaves, &steps, &mut leaf_order);
        let mut x_pos = vec![0.0_f64; total_nodes];
        for (pos, &leaf) in leaf_order.iter().enumerate() {
            x_pos[leaf] = pos as f64 + 1.0;
        }

        let mut branches = Vec::with_capacity(steps.len());
        let mut max_distance = 0.0_f64;
        for (idx, step) in steps.iter().enumerate() {
            let node = n_leaves + idx;
            let xl = x_pos[step.left];
            let xr = x_pos[step.right];
            let mid = 0.5 * (xl + xr);
            x_pos[node] = mid;
            max_distance = max_distance.max(step.distance);
            branches.push(DendrogramBranch {
                index: idx,
                x_left: xl,
                x_right: xr,
                x_mid: mid,
                y_left: heights[step.left],
                y_right: heights[step.right],
                y_top: step.distance,
                count: step.count,
            });
        }

        let labels = (0..n_leaves).map(|i| format!("{}", i + 1)).collect();
        Ok(Self {
            n_leaves,
            steps: steps.into(),
            branches: branches.into(),
            leaf_order: leaf_order.into(),
            labels,
            max_distance,
        })
    }

    pub fn with_labels(mut self, labels: impl Into<Vec<String>>) -> Self {
        let labels = labels.into();
        if labels.len() == self.n_leaves {
            self.labels = labels;
        }
        self
    }

    pub fn n_leaves(&self) -> usize {
        self.n_leaves
    }
    pub fn steps(&self) -> &[LinkageStep] {
        &self.steps
    }
    pub fn branches(&self) -> &[DendrogramBranch] {
        &self.branches
    }
    pub fn leaf_order(&self) -> &[usize] {
        &self.leaf_order
    }
    pub fn labels(&self) -> &[String] {
        &self.labels
    }
    pub fn max_distance(&self) -> f64 {
        self.max_distance
    }
    /// Tick `(position, label)` ordered from leaf 1..=n along the leaf axis.
    pub fn leaf_ticks(&self) -> Vec<(f64, String)> {
        self.leaf_order
            .iter()
            .enumerate()
            .map(|(i, &leaf)| (i as f64 + 1.0, self.labels[leaf].clone()))
            .collect()
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.steps) + std::mem::size_of_val(&*self.branches)
    }

    pub fn bounds(
        &self,
        orientation: BarOrientation,
        scales: [AxisScale; 2],
    ) -> Option<PlotBounds> {
        let leaf_lo = 0.5;
        let leaf_hi = self.n_leaves as f64 + 0.5;
        let dist_hi = if self.max_distance > 0.0 {
            self.max_distance * 1.05
        } else {
            1.0
        };
        let (min, max) = match orientation {
            BarOrientation::Vertical => ([leaf_lo, 0.0], [leaf_hi, dist_hi]),
            BarOrientation::Horizontal => ([0.0, leaf_lo], [dist_hi, leaf_hi]),
        };
        if scales[0].forward(min[0]).is_none()
            || scales[0].forward(max[0]).is_none()
            || scales[1].forward(min[1]).is_none()
            || scales[1].forward(max[1]).is_none()
        {
            return None;
        }
        PlotBounds::new(min, max).ok()
    }
}

fn collect_leaf_order(node: usize, n_leaves: usize, steps: &[LinkageStep], out: &mut Vec<usize>) {
    if node < n_leaves {
        out.push(node);
        return;
    }
    let step = steps[node - n_leaves];
    collect_leaf_order(step.left, n_leaves, steps, out);
    collect_leaf_order(step.right, n_leaves, steps, out);
}
