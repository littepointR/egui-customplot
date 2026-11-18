use std::sync::Arc;

use crate::{AxisScale, PlotPoint, Point3D, data::triangulate::triangulate_indexed_polygon};

use super::{CachedExtents3D, Plot3DError};

/// One 3D vector arrow originating at `origin` with displacement `vector` (`quiver3`, `ML-057`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quiver3DArrow {
    pub origin: Point3D,
    pub vector: Point3D,
}

impl Quiver3DArrow {
    pub fn new(origin: Point3D, vector: Point3D) -> Self {
        Self { origin, vector }
    }
    pub fn endpoint(self, scale: f64) -> Option<Point3D> {
        let end = [
            self.origin[0] + self.vector[0] * scale,
            self.origin[1] + self.vector[1] * scale,
            self.origin[2] + self.vector[2] * scale,
        ];
        (self.origin.iter().all(|v| v.is_finite()) && end.iter().all(|v| v.is_finite()))
            .then_some(end)
    }
}

/// Immutable host-owned 3D vector field (`quiver3`, `ML-057`).
#[derive(Clone, Debug)]
pub struct Quiver3DData {
    pub(crate) version: u64,
    arrows: Arc<[Quiver3DArrow]>,
    scale_factor: f64,
    extents: CachedExtents3D,
    non_finite_arrows: usize,
}

impl Quiver3DData {
    pub fn new(arrows: impl Into<Arc<[Quiver3DArrow]>>) -> Self {
        let arrows: Arc<[Quiver3DArrow]> = arrows.into();
        Self::build(arrows, 1.0)
    }

    pub fn from_components(
        x: &[f64],
        y: &[f64],
        z: &[f64],
        u: &[f64],
        v: &[f64],
        w: &[f64],
    ) -> Result<Self, Plot3DError> {
        let n = x.len();
        if y.len() != n || z.len() != n || u.len() != n || v.len() != n || w.len() != n {
            return Err(Plot3DError::LengthMismatch);
        }
        if n == 0 {
            return Err(Plot3DError::EmptyInput);
        }
        let arrows: Arc<[Quiver3DArrow]> = (0..n)
            .map(|i| Quiver3DArrow::new([x[i], y[i], z[i]], [u[i], v[i], w[i]]))
            .collect();
        Ok(Self::new(arrows))
    }

    pub fn autoscale(self, factor: f64) -> Self {
        let factor = if factor.is_finite() && factor > 0.0 {
            factor
        } else {
            1.0
        };
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let mut max_vec = 0.0_f64;
        for &a in self.arrows.iter() {
            if a.origin.iter().all(|v| v.is_finite()) && a.vector.iter().all(|v| v.is_finite()) {
                for i in 0..3 {
                    lo[i] = lo[i].min(a.origin[i]);
                    hi[i] = hi[i].max(a.origin[i]);
                }
                let mag = (a.vector[0] * a.vector[0]
                    + a.vector[1] * a.vector[1]
                    + a.vector[2] * a.vector[2])
                    .sqrt();
                max_vec = max_vec.max(mag);
            }
        }
        let diag = ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2) + (hi[2] - lo[2]).powi(2))
            .sqrt()
            .max(1e-6);
        let auto = if max_vec > 1e-12 && lo[0].is_finite() {
            (diag / (self.arrows.len() as f64).cbrt().max(1.0) * 0.5 / max_vec) * factor
        } else {
            1.0
        };
        Self::build(self.arrows, auto)
    }

    fn build(arrows: Arc<[Quiver3DArrow]>, scale_factor: f64) -> Self {
        let mut extents = CachedExtents3D::default();
        let mut non_finite = 0usize;
        for &a in arrows.iter() {
            if let Some(end) = a.endpoint(scale_factor) {
                extents.push_point(a.origin);
                extents.push_point(end);
            } else {
                non_finite += 1;
            }
        }
        Self {
            version: crate::data::points::next_version(),
            arrows,
            scale_factor,
            extents,
            non_finite_arrows: non_finite,
        }
    }

    pub fn arrows(&self) -> &[Quiver3DArrow] {
        &self.arrows
    }
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
    pub fn non_finite_arrows(&self) -> usize {
        self.non_finite_arrows
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.arrows.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.arrows.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

/// Triangulated planar 3D polygon (`fill3`, `EX-003`).
#[derive(Clone, Debug)]
pub struct Polygon3D {
    pub vertices: Arc<[Point3D]>,
    pub triangles: Arc<[[usize; 3]]>,
    pub normal: [f64; 3],
}

/// Immutable host-owned 3D filled polygon collection (`fill3`, `EX-003`).
#[derive(Clone, Debug)]
pub struct Polygon3DData {
    pub(crate) version: u64,
    polygons: Arc<[Polygon3D]>,
    extents: CachedExtents3D,
}

impl Polygon3DData {
    pub fn from_polygons(rings: &[&[Point3D]]) -> Result<Self, Plot3DError> {
        if rings.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let mut out = Vec::with_capacity(rings.len());
        let mut extents = CachedExtents3D::default();

        for &ring in rings {
            if ring.len() < 3 || ring.iter().any(|p| p.iter().any(|v| !v.is_finite())) {
                return Err(Plot3DError::DegeneratePolygon);
            }
            let n = newell_normal(ring);
            let nlen = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if nlen <= 1e-12 || !nlen.is_finite() {
                return Err(Plot3DError::DegeneratePolygon);
            }
            let nn = [n[0] / nlen, n[1] / nlen, n[2] / nlen];
            let seed = if nn[0].abs() < 0.8 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let u = normalize3(cross3(nn, seed));
            let v = cross3(nn, u);
            let origin = ring[0];
            let pts2d: Vec<PlotPoint> = ring
                .iter()
                .map(|p| {
                    let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
                    [dot3(d, u), dot3(d, v)]
                })
                .collect();
            let indices: Vec<usize> = (0..ring.len()).collect();
            let tris = triangulate_indexed_polygon(&pts2d, &indices, &[], [AxisScale::Linear; 2])
                .ok_or(Plot3DError::DegeneratePolygon)?;
            for &p in ring {
                extents.push_point(p);
            }
            out.push(Polygon3D {
                vertices: Arc::from(ring),
                triangles: Arc::from(tris),
                normal: nn,
            });
        }
        Ok(Self {
            version: crate::data::points::next_version(),
            polygons: Arc::from(out),
            extents,
        })
    }

    pub fn polygons(&self) -> &[Polygon3D] {
        &self.polygons
    }
    pub fn sample_bytes(&self) -> usize {
        self.polygons
            .iter()
            .map(|p| {
                std::mem::size_of_val(p.vertices.as_ref())
                    + std::mem::size_of_val(p.triangles.as_ref())
            })
            .sum()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.polygons.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

fn newell_normal(pts: &[Point3D]) -> [f64; 3] {
    let mut n = [0.0; 3];
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-12);
    [v[0] / l, v[1] / l, v[2] / l]
}
