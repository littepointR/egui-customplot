use std::sync::Arc;

use super::{
    ContourData, ContourError, ContourLevels, ContourLine, FilledContourData, GridError,
    contour::extract::{
        BandAccumulator, Node, chain_segments, resolve_band_levels_from_range,
        resolve_line_levels_from_range, tri_isoline,
    },
    extents::CachedExtents,
};
use crate::{AxisScale, PlotBounds, PlotPoint};

/// Host-owned curvilinear parametric 2D scalar field `pcolor(X, Y, Z)` / `contour(X, Y, Z)` (`EXT-08`).
/// Coordinates and scalar values are stored row-major (`row * cols + col`, length `rows * cols`).
#[derive(Clone, Debug)]
pub struct CurvilinearField2D {
    rows: usize,
    cols: usize,
    x_nodes: Arc<[f64]>,
    y_nodes: Arc<[f64]>,
    values: Arc<[f64]>,
    value_range: Option<[f64; 2]>,
    extents: CachedExtents,
    non_finite_nodes: usize,
}

impl CurvilinearField2D {
    pub fn from_nodes(
        rows: usize,
        cols: usize,
        x_nodes: impl Into<Arc<[f64]>>,
        y_nodes: impl Into<Arc<[f64]>>,
        values: impl Into<Arc<[f64]>>,
    ) -> Result<Self, GridError> {
        if rows < 2 || cols < 2 {
            return Err(GridError::EmptyAxis);
        }
        let len = rows.checked_mul(cols).ok_or(GridError::CapacityOverflow)?;
        let (x_nodes, y_nodes, values) = (x_nodes.into(), y_nodes.into(), values.into());
        if x_nodes.len() != len || y_nodes.len() != len || values.len() != len {
            return Err(GridError::LengthMismatch);
        }

        let mut extents = CachedExtents::default();
        let mut expected_sign: Option<i8> = None;
        for r in 0..(rows - 1) {
            for c in 0..(cols - 1) {
                let idx = [
                    r * cols + c,
                    r * cols + c + 1,
                    (r + 1) * cols + c + 1,
                    (r + 1) * cols + c,
                ];
                let pts = idx.map(|k| [x_nodes[k], y_nodes[k]]);
                if pts.iter().any(|p| !p[0].is_finite() || !p[1].is_finite()) {
                    extents.push(None);
                    continue;
                }
                if !check_quad_winding(&pts, &mut expected_sign) {
                    return Err(GridError::InvalidEdges);
                }
                if idx.iter().all(|&k| values[k].is_finite()) {
                    let lo = [
                        pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
                        pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
                    ];
                    let hi = [
                        pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max),
                        pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max),
                    ];
                    extents.push(Some((lo, hi)));
                } else {
                    extents.push(None);
                }
            }
        }

        let mut zmin = f64::INFINITY;
        let mut zmax = f64::NEG_INFINITY;
        let mut non_finite_nodes = 0;
        for i in 0..len {
            let (x, y, z) = (x_nodes[i], y_nodes[i], values[i]);
            if x.is_finite() && y.is_finite() && z.is_finite() {
                zmin = zmin.min(z);
                zmax = zmax.max(z);
            } else {
                non_finite_nodes += 1;
            }
        }
        let value_range = (zmin <= zmax).then_some([zmin, zmax]);

        Ok(Self {
            rows,
            cols,
            x_nodes,
            y_nodes,
            values,
            value_range,
            extents,
            non_finite_nodes,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn cell_rows(&self) -> usize {
        self.rows.saturating_sub(1)
    }
    pub fn cell_cols(&self) -> usize {
        self.cols.saturating_sub(1)
    }
    pub fn node_coords(&self, row: usize, col: usize) -> Option<PlotPoint> {
        let idx = row.checked_mul(self.cols)?.checked_add(col)?;
        let (&x, &y) = (self.x_nodes.get(idx)?, self.y_nodes.get(idx)?);
        (x.is_finite() && y.is_finite()).then_some([x, y])
    }
    pub fn node_value(&self, row: usize, col: usize) -> Option<f64> {
        let idx = row.checked_mul(self.cols)?.checked_add(col)?;
        let v = *self.values.get(idx)?;
        v.is_finite().then_some(v)
    }
    pub fn quad_nodes(&self, row: usize, col: usize) -> Option<[PlotPoint; 4]> {
        if row >= self.cell_rows() || col >= self.cell_cols() {
            return None;
        }
        Some([
            self.node_coords(row, col)?,
            self.node_coords(row, col + 1)?,
            self.node_coords(row + 1, col + 1)?,
            self.node_coords(row + 1, col)?,
        ])
    }
    pub fn value_range(&self) -> Option<[f64; 2]> {
        self.value_range
    }
    pub fn non_finite_nodes(&self) -> usize {
        self.non_finite_nodes
    }
    pub fn sample_bytes(&self) -> usize {
        (self.x_nodes.len() + self.y_nodes.len() + self.values.len()) * std::mem::size_of::<f64>()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.values.as_ptr() as usize, self.sample_bytes())
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (lo, hi) = self.extents.extent(scales)?;
        PlotBounds::new(lo, hi).ok()
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.extents.invalid(scales)
    }

    fn visit_subtriangles(&self, mut emit: impl FnMut([Node; 3])) {
        for r in 0..self.cell_rows() {
            for c in 0..self.cell_cols() {
                let (Some(pts), Some(z00), Some(z10), Some(z11), Some(z01)) = (
                    self.quad_nodes(r, c),
                    self.node_value(r, c),
                    self.node_value(r, c + 1),
                    self.node_value(r + 1, c + 1),
                    self.node_value(r + 1, c),
                ) else {
                    continue;
                };
                let n00 = Node { p: pts[0], z: z00 };
                let n10 = Node { p: pts[1], z: z10 };
                let n11 = Node { p: pts[2], z: z11 };
                let n01 = Node { p: pts[3], z: z01 };
                let nc = Node {
                    p: [
                        0.25 * (pts[0][0] + pts[1][0] + pts[2][0] + pts[3][0]),
                        0.25 * (pts[0][1] + pts[1][1] + pts[2][1] + pts[3][1]),
                    ],
                    z: 0.25 * (z00 + z10 + z11 + z01),
                };
                emit([n00, n10, nc]);
                emit([n10, n11, nc]);
                emit([n11, n01, nc]);
                emit([n01, n00, nc]);
            }
        }
    }
}

impl ContourData {
    pub fn from_curvilinear(
        field: &CurvilinearField2D,
        levels: ContourLevels,
    ) -> Result<Self, ContourError> {
        let levels = resolve_line_levels_from_range(field.value_range(), levels)?;
        if levels.is_empty() {
            return Ok(Self::from_raw_lines(levels, Vec::new()));
        }
        let mut per_level = vec![Vec::new(); levels.len()];
        field.visit_subtriangles(|tri| {
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
        let mut lines = Vec::new();
        for (level, segs) in levels.iter().copied().zip(per_level) {
            for points in chain_segments(segs) {
                lines.push(ContourLine {
                    level,
                    points: points.into_boxed_slice(),
                });
            }
        }
        Ok(Self::from_raw_lines(levels, lines))
    }
}

impl FilledContourData {
    pub fn from_curvilinear(
        field: &CurvilinearField2D,
        levels: ContourLevels,
    ) -> Result<Self, ContourError> {
        let (levels, band_bounds) = resolve_band_levels_from_range(field.value_range(), levels)?;
        let mut acc = BandAccumulator::new(&band_bounds);
        field.visit_subtriangles(|tri| acc.push_triangle(tri));
        Ok(Self::from_raw_bands(levels, acc.finish()))
    }
}

fn check_quad_winding(pts: &[PlotPoint; 4], expected_sign: &mut Option<i8>) -> bool {
    let pc = [
        0.25 * (pts[0][0] + pts[1][0] + pts[2][0] + pts[3][0]),
        0.25 * (pts[0][1] + pts[1][1] + pts[2][1] + pts[3][1]),
    ];
    let scale = (pts[2][0] - pts[0][0])
        .hypot(pts[2][1] - pts[0][1])
        .max((pts[3][0] - pts[1][0]).hypot(pts[3][1] - pts[1][1]))
        .max(1e-12);
    let tol = 1e-12 * scale * scale;
    for i in 0..4 {
        let a = pts[i];
        let b = pts[(i + 1) % 4];
        let area = (b[0] - a[0]) * (pc[1] - a[1]) - (b[1] - a[1]) * (pc[0] - a[0]);
        if area.abs() <= tol {
            continue;
        }
        let sgn = if area > 0.0 { 1 } else { -1 };
        match expected_sign {
            None => *expected_sign = Some(sgn),
            Some(prev) if *prev != sgn => return false,
            _ => {}
        }
    }
    true
}
