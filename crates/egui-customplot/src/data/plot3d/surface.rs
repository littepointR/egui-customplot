use std::sync::Arc;

use crate::{
    AxisScale, ContourError, ContourLevels, Point3D, Quiver3DArrow, Quiver3DData, ScalarField2D,
};

use super::{CachedExtents3D, Contour3DData, Plot3DError};

/// Immutable host-owned structured `(rows x cols)` 3D surface grid (`surf` / `mesh` / `surfc`).
#[derive(Clone, Debug)]
pub struct Surface3DData {
    pub(crate) version: u64,
    rows: usize,
    cols: usize,
    points: Arc<[Point3D]>,
    normals: Arc<[[f64; 3]]>,
    scalars: Arc<[f64]>,
    scalar_range: Option<[f64; 2]>,
    z_range: Option<[f64; 2]>,
    contours: Option<Contour3DData>,
    extents: CachedExtents3D,
    non_finite_points: usize,
}

impl Surface3DData {
    /// Builds a rectangular `(y.len() x x.len())` height surface `Z(y, x)` in row-major order.
    pub fn from_grid(x: &[f64], y: &[f64], z_row_major: &[f64]) -> Result<Self, Plot3DError> {
        let cols = x.len();
        let rows = y.len();
        if rows < 2 || cols < 2 {
            return Err(Plot3DError::InvalidDimensions);
        }
        let expected = rows
            .checked_mul(cols)
            .ok_or(Plot3DError::InvalidDimensions)?;
        if z_row_major.len() != expected {
            return Err(Plot3DError::LengthMismatch);
        }
        let mut pts = Vec::with_capacity(expected);
        for (r, &yv) in y.iter().enumerate() {
            for (c, &xv) in x.iter().enumerate() {
                pts.push([xv, yv, z_row_major[r * cols + c]]);
            }
        }
        Self::from_parametric_grid(rows, cols, pts)
    }

    pub fn from_field(field: &ScalarField2D) -> Self {
        Self::from_grid(field.x_nodes(), field.y_nodes(), field.values())
            .expect("ScalarField2D always has >= 2x2 dimensions")
    }

    /// Builds a general parametric 3D surface grid of size `(rows x cols)` in row-major order.
    pub fn from_parametric_grid(
        rows: usize,
        cols: usize,
        points: impl Into<Arc<[Point3D]>>,
    ) -> Result<Self, Plot3DError> {
        if rows < 2 || cols < 2 {
            return Err(Plot3DError::InvalidDimensions);
        }
        let points: Arc<[Point3D]> = points.into();
        let expected = rows
            .checked_mul(cols)
            .ok_or(Plot3DError::InvalidDimensions)?;
        if points.len() != expected {
            return Err(Plot3DError::LengthMismatch);
        }
        let mut extents = CachedExtents3D::default();
        let mut scalars = Vec::with_capacity(expected);
        let mut non_finite = 0usize;
        let mut z_lo = f64::INFINITY;
        let mut z_hi = f64::NEG_INFINITY;
        for &p in points.iter() {
            extents.push_point(p);
            if p[0].is_finite() && p[1].is_finite() && p[2].is_finite() {
                z_lo = z_lo.min(p[2]);
                z_hi = z_hi.max(p[2]);
                scalars.push(p[2]);
            } else {
                non_finite += 1;
                scalars.push(f64::NAN);
            }
        }
        let z_range = (z_lo <= z_hi).then_some([z_lo, z_hi]);
        let normals = compute_grid_normals(rows, cols, &points);
        Ok(Self {
            version: crate::data::points::next_version(),
            rows,
            cols,
            points,
            normals: Arc::from(normals),
            scalars: Arc::from(scalars),
            scalar_range: z_range,
            z_range,
            contours: None,
            extents,
            non_finite_points: non_finite,
        })
    }

    pub fn with_normals(
        mut self,
        normals: impl Into<Arc<[[f64; 3]]>>,
    ) -> Result<Self, Plot3DError> {
        let normals: Arc<[[f64; 3]]> = normals.into();
        if normals.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.normals = normals;
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn with_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &s in scalars {
            if s.is_finite() {
                lo = lo.min(s);
                hi = hi.max(s);
            }
        }
        self.scalars = Arc::from(scalars);
        self.scalar_range = (lo <= hi).then_some([lo, hi]);
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    pub fn with_contours(mut self, levels: ContourLevels) -> Result<Self, ContourError> {
        let c = Contour3DData::from_surface(&self, levels)?;
        self.contours = Some(c);
        self.version = crate::data::points::next_version();
        Ok(self)
    }

    /// Extracts surface normal vectors on a regular `(stride x stride)` subgrid (`surfnorm`, `EX-007`).
    pub fn normals_quiver(&self, stride: usize, scale: f64) -> Quiver3DData {
        let step = stride.max(1);
        let len = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            0.25
        };
        let mut arrows = Vec::new();
        for r in (0..self.rows).step_by(step) {
            for c in (0..self.cols).step_by(step) {
                let idx = r * self.cols + c;
                let p = self.points[idx];
                let n = self.normals[idx];
                if p.iter().all(|v| v.is_finite()) && n.iter().all(|v| v.is_finite()) {
                    arrows.push(Quiver3DArrow::new(p, [n[0] * len, n[1] * len, n[2] * len]));
                }
            }
        }
        Quiver3DData::new(arrows)
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn points(&self) -> &[Point3D] {
        &self.points
    }
    pub fn normals(&self) -> &[[f64; 3]] {
        &self.normals
    }
    pub fn scalars(&self) -> &[f64] {
        &self.scalars
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn z_range(&self) -> Option<[f64; 2]> {
        self.z_range
    }
    pub fn contours(&self) -> Option<&Contour3DData> {
        self.contours.as_ref()
    }
    pub fn point(&self, row: usize, col: usize) -> Option<Point3D> {
        (row < self.rows && col < self.cols).then(|| self.points[row * self.cols + col])
    }
    pub fn normal(&self, row: usize, col: usize) -> Option<[f64; 3]> {
        (row < self.rows && col < self.cols).then(|| self.normals[row * self.cols + col])
    }
    pub fn scalar(&self, row: usize, col: usize) -> Option<f64> {
        (row < self.rows && col < self.cols).then(|| self.scalars[row * self.cols + col])
    }
    pub fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.points.as_ref())
            + std::mem::size_of_val(self.normals.as_ref())
            + std::mem::size_of_val(self.scalars.as_ref())
            + self
                .contours
                .as_ref()
                .map_or(0, Contour3DData::sample_bytes)
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.points.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

fn compute_grid_normals(rows: usize, cols: usize, pts: &[Point3D]) -> Vec<[f64; 3]> {
    let mut normals = Vec::with_capacity(pts.len());
    let fin = |p: Point3D| p[0].is_finite() && p[1].is_finite() && p[2].is_finite();
    for r in 0..rows {
        for c in 0..cols {
            let p = pts[r * cols + c];
            if !fin(p) {
                normals.push([0.0, 0.0, 1.0]);
                continue;
            }
            let du = diff_vec(
                pts,
                cols,
                r,
                c.saturating_sub(1),
                r,
                (c + 1).min(cols - 1),
                p,
            );
            let dv = diff_vec(
                pts,
                cols,
                r.saturating_sub(1),
                c,
                (r + 1).min(rows - 1),
                c,
                p,
            );
            let n = [
                du[1] * dv[2] - du[2] * dv[1],
                du[2] * dv[0] - du[0] * dv[2],
                du[0] * dv[1] - du[1] * dv[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-14 && len.is_finite() {
                normals.push([n[0] / len, n[1] / len, n[2] / len]);
            } else {
                normals.push([0.0, 0.0, 1.0]);
            }
        }
    }
    normals
}

fn diff_vec(
    pts: &[Point3D],
    cols: usize,
    r0: usize,
    c0: usize,
    r1: usize,
    c1: usize,
    center: Point3D,
) -> [f64; 3] {
    let a = pts[r0 * cols + c0];
    let b = pts[r1 * cols + c1];
    let fin = |p: Point3D| p[0].is_finite() && p[1].is_finite() && p[2].is_finite();
    match (fin(a), fin(b)) {
        (true, true) => [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        (false, true) => [b[0] - center[0], b[1] - center[1], b[2] - center[2]],
        (true, false) => [center[0] - a[0], center[1] - a[1], center[2] - a[2]],
        (false, false) => [0.0, 0.0, 0.0],
    }
}
