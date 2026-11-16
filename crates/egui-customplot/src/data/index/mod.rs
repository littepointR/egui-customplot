//! Explicit or automatic immutable monotonic-X acceleration. No UI state or render geometry.

mod build;
mod pick;
mod query;
#[cfg(test)]
mod tests;
mod tree;

use std::ops::Range;

use crate::{AxisScale, PlotPoint};

pub(crate) const BLOCK_SIZE: usize = 64;
pub(crate) use pick::{ScreenAabb, project_axis_raw, screen_aabb};
pub(crate) use query::gallop_partition_point;
pub(crate) use tree::Extrema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineIndexError {
    UnsortedX {
        point_index: usize,
    },
    BudgetExceeded {
        required_bytes: usize,
        budget_bytes: usize,
    },
    SizeOverflow,
    AllocationFailed,
}

impl std::fmt::Display for LineIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsortedX { point_index } => {
                write!(f, "X decreases within a finite run at point {point_index}")
            }
            Self::BudgetExceeded {
                required_bytes,
                budget_bytes,
            } => write!(
                f,
                "line index requires {required_bytes} bytes; budget is {budget_bytes}"
            ),
            Self::SizeOverflow => f.write_str("line index size is not representable"),
            Self::AllocationFailed => f.write_str("line index allocation failed"),
        }
    }
}

impl std::error::Error for LineIndexError {}

#[derive(Debug)]
pub(crate) struct LineIndex {
    runs: [Box<[Range<usize>]>; 4],
    tree: Box<[Extrema]>,
    leaf_count: usize,
    bytes: usize,
}

#[derive(Default)]
pub(crate) struct QueryStats {
    pub comparisons: usize,
    pub visited_nodes: usize,
    pub scanned_points: usize,
    pub selected_points: usize,
    pub visited_runs: usize,
    pub reduced: bool,
}

pub(crate) struct LineQuery {
    pub paths: Vec<Vec<usize>>,
    pub stats: QueryStats,
    pub capacity_bytes: usize,
}

impl LineIndex {
    pub(super) fn bytes(&self) -> usize {
        self.bytes
    }

    pub(crate) fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    pub(crate) fn node_extrema(&self, node_idx: usize) -> Extrema {
        self.tree[node_idx]
    }

    pub(crate) fn runs_for_scales(&self, scales: [AxisScale; 2]) -> &[Range<usize>] {
        self.runs(scales)
    }

    fn runs(&self, scales: [AxisScale; 2]) -> &[Range<usize>] {
        let mask = usize::from(scales[0] == AxisScale::Log10)
            | (usize::from(scales[1] == AxisScale::Log10) << 1);
        &self.runs[mask]
    }
}

fn valid(point: PlotPoint, mask: usize) -> bool {
    point[0].is_finite()
        && point[1].is_finite()
        && (mask & 1 == 0 || point[0] > 0.0)
        && (mask & 2 == 0 || point[1] > 0.0)
}
