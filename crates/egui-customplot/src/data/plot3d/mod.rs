mod adaptive_surf;
mod alphashape;
mod analytic_surf;
mod cones3d;
mod contour3;
mod curve;
mod delaunay3d;
mod discrete;
mod flow3d;
mod graph3d;
mod implicit3;
mod isosurface;
mod particles3d;
mod patch3d;
mod pointcloud;
mod scalar_volume3d;
mod scatter;
mod stream_geom;
mod surface;
mod tetramesh;
mod trisurf;
mod vector_field3d;
mod vector_poly;
mod volshow;
mod volume;

use std::sync::Arc;

pub use alphashape::AlphaShape3DData;
pub use analytic_surf::Ribbon3DData;
pub use cones3d::{Cone3DItem, ConePlot3DData};
pub use contour3::{Contour3DData, Contour3DLine};
pub use curve::{AnimatedLine3D, Comet3DPlayback, ParametricCurve3DData};
pub use discrete::{
    Bar3DBox, Bar3DData, Bar3DMode, Bar3DOrientation, Pie3DData, Pie3DSlice, Stem3DData,
};
pub use flow3d::{Stream3DOptions, Streamline3DData, Streamline3DLine};
pub use graph3d::{Graph3DData, GraphLayout3D};
pub use implicit3::ImplicitSurface3DData;
pub use isosurface::{IsocapThreshold, Isocaps3DData, Isosurface3DData};
pub use particles3d::{StreamParticles3DData, StreamParticlesPlayback};
pub use patch3d::Patch3DData;
pub use pointcloud::PointCloud3DData;
pub use scalar_volume3d::ScalarVolume3D;
pub use scatter::{Bubble3DData, Bubble3DPoint, Swarm3DData, Swarm3DPoint};
pub use stream_geom::{StreamRibbon3DData, StreamTube3DData};
pub use surface::Surface3DData;
pub use tetramesh::TetraMesh3DData;
pub use trisurf::TriMesh3DData;
pub use vector_field3d::VectorField3D;
pub use vector_poly::{Polygon3D, Polygon3DData, Quiver3DArrow, Quiver3DData};
pub use volshow::{TransferStop, VolumeRenderMode, VolumeTransferFunction};
pub use volume::{SlicePlane3D, VolumeSliceData};

use crate::{AxisScale, Point3D};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plot3DError {
    EmptyInput,
    LengthMismatch,
    InvalidDimensions,
    InvalidThickness,
    DegeneratePolygon,
    NonPositiveTotal,
}

impl std::fmt::Display for Plot3DError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("3D input must not be empty"),
            Self::LengthMismatch => f.write_str("3D input slices must have equal length"),
            Self::InvalidDimensions => f.write_str("invalid 3D matrix or grid dimensions"),
            Self::InvalidThickness => f.write_str("pie3 thickness ratio must be finite and > 0"),
            Self::DegeneratePolygon => f.write_str("3D polygon requires >= 3 non-collinear points"),
            Self::NonPositiveTotal => {
                f.write_str("pie3 values must contain positive finite weights")
            }
        }
    }
}

impl std::error::Error for Plot3DError {}

/// Precomputed 8-domain extent cache (`(Linear | Log10)^3`) for O(1) bounds queries.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CachedExtents3D {
    extents: [Option<[Point3D; 2]>; 8],
    invalid: [usize; 8],
}

impl CachedExtents3D {
    pub fn push_point(&mut self, p: Point3D) {
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() {
            for inv in &mut self.invalid {
                *inv += 1;
            }
            return;
        }
        let pos = [p[0] > 0.0, p[1] > 0.0, p[2] > 0.0];
        for (domain, ext) in self.extents.iter_mut().enumerate() {
            let need_pos = [domain & 1 != 0, domain & 2 != 0, domain & 4 != 0];
            if (0..3).all(|a| !need_pos[a] || pos[a]) {
                let b = ext.get_or_insert([p, p]);
                for a in 0..3 {
                    b[0][a] = b[0][a].min(p[a]);
                    b[1][a] = b[1][a].max(p[a]);
                }
            } else {
                self.invalid[domain] += 1;
            }
        }
    }

    pub fn push_box(&mut self, lo: Point3D, hi: Point3D) {
        if !(0..3).all(|a| lo[a].is_finite() && hi[a].is_finite() && lo[a] <= hi[a]) {
            for inv in &mut self.invalid {
                *inv += 1;
            }
            return;
        }
        self.push_point(lo);
        self.push_point(hi);
    }

    pub fn extent(self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents[domain_idx(scales)]
    }

    pub fn domain_invalid(self, scales: [AxisScale; 3]) -> usize {
        self.invalid[domain_idx(scales)].saturating_sub(self.invalid[0])
    }
}

pub(crate) fn domain_idx(scales: [AxisScale; 3]) -> usize {
    usize::from(scales[0] == AxisScale::Log10)
        + usize::from(scales[1] == AxisScale::Log10) * 2
        + usize::from(scales[2] == AxisScale::Log10) * 4
}

pub(crate) fn union_bounds3(
    a: Option<[Point3D; 2]>,
    b: Option<[Point3D; 2]>,
) -> Option<[Point3D; 2]> {
    match (a, b) {
        (None, other) | (other, None) => other,
        (Some([a0, a1]), Some([b0, b1])) => Some([
            [a0[0].min(b0[0]), a0[1].min(b0[1]), a0[2].min(b0[2])],
            [a1[0].max(b1[0]), a1[1].max(b1[1]), a1[2].max(b1[2])],
        ]),
    }
}

/// Immutable host-owned 3D polyline/scatter/stem points (`plot3` / `scatter3`).
#[derive(Clone, Debug)]
pub struct Point3DData {
    points: Arc<[Point3D]>,
    colors: Option<Arc<[egui::Color32]>>,
    radii: Option<Arc<[f32]>>,
    scalars: Option<Arc<[f64]>>,
    scalar_range: Option<[f64; 2]>,
    extents: CachedExtents3D,
    finite_points: usize,
    non_finite_points: usize,
}

impl Point3DData {
    pub fn new(points: impl Into<Arc<[Point3D]>>) -> Self {
        let points: Arc<[Point3D]> = points.into();
        let mut extents = CachedExtents3D::default();
        let mut finite = 0usize;
        let mut non_finite = 0usize;
        for &p in points.iter() {
            if p[0].is_finite() && p[1].is_finite() && p[2].is_finite() {
                finite += 1;
            } else {
                non_finite += 1;
            }
            extents.push_point(p);
        }
        Self {
            points,
            colors: None,
            radii: None,
            scalars: None,
            scalar_range: None,
            extents,
            finite_points: finite,
            non_finite_points: non_finite,
        }
    }

    pub fn from_xyz(x: &[f64], y: &[f64], z: &[f64]) -> Result<Self, Plot3DError> {
        if x.len() != y.len() || x.len() != z.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        if x.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let pts: Arc<[Point3D]> = x
            .iter()
            .zip(y.iter())
            .zip(z.iter())
            .map(|((&vx, &vy), &vz)| [vx, vy, vz])
            .collect();
        Ok(Self::new(pts))
    }

    pub fn with_colors(
        mut self,
        colors: impl Into<Arc<[egui::Color32]>>,
    ) -> Result<Self, Plot3DError> {
        let colors: Arc<[egui::Color32]> = colors.into();
        if colors.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.colors = Some(colors);
        Ok(self)
    }

    pub fn with_radii(mut self, radii: impl Into<Arc<[f32]>>) -> Result<Self, Plot3DError> {
        let radii: Arc<[f32]> = radii.into();
        if radii.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.radii = Some(radii);
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
        self.scalars = Some(Arc::from(scalars));
        self.scalar_range = (lo <= hi).then_some([lo, hi]);
        Ok(self)
    }

    pub fn points(&self) -> &[Point3D] {
        &self.points
    }
    pub fn colors(&self) -> Option<&[egui::Color32]> {
        self.colors.as_deref()
    }
    pub fn radii(&self) -> Option<&[f32]> {
        self.radii.as_deref()
    }
    pub fn scalars(&self) -> Option<&[f64]> {
        self.scalars.as_deref()
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    pub fn finite_points(&self) -> usize {
        self.finite_points
    }
    pub fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.points.as_ref())
            + self
                .colors
                .as_ref()
                .map_or(0, |c| std::mem::size_of_val(c.as_ref()))
            + self
                .radii
                .as_ref()
                .map_or(0, |r| std::mem::size_of_val(r.as_ref()))
            + self
                .scalars
                .as_ref()
                .map_or(0, |s| std::mem::size_of_val(s.as_ref()))
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
