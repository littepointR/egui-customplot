use std::{f64::consts::PI, sync::Arc};

use crate::{AxisScale, Point3D, layout::bubble_pixel_radius};

use super::{CachedExtents3D, Plot3DError};

/// Single bubble record in [`Bubble3DData`] (`ML-016`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bubble3DPoint {
    pub position: Point3D,
    pub size: f64,
    pub scalar: Option<f64>,
}

/// Immutable host-owned 3D bubble chart data (`bubblechart3`, `ML-016`).
#[derive(Clone, Debug)]
pub struct Bubble3DData {
    points: Arc<[Bubble3DPoint]>,
    size_extent: [f64; 2],
    radius_range: [f32; 2],
    extents: CachedExtents3D,
    non_finite_points: usize,
}

impl Bubble3DData {
    pub fn new(positions: &[Point3D], sizes: &[f64]) -> Result<Self, Plot3DError> {
        if positions.len() != sizes.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        if positions.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let vec: Vec<Bubble3DPoint> = positions
            .iter()
            .zip(sizes.iter())
            .map(|(&position, &size)| Bubble3DPoint {
                position,
                size,
                scalar: None,
            })
            .collect();
        Ok(Self::from_vec(vec, [4.0, 22.0]))
    }

    pub fn with_scalars(mut self, scalars: &[f64]) -> Result<Self, Plot3DError> {
        if scalars.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        let next: Vec<Bubble3DPoint> = self
            .points
            .iter()
            .zip(scalars.iter())
            .map(|(p, &s)| Bubble3DPoint {
                scalar: s.is_finite().then_some(s),
                ..*p
            })
            .collect();
        self.points = Arc::from(next);
        Ok(self)
    }

    pub fn size_range(mut self, radius_range: [f32; 2]) -> Self {
        let lo = radius_range[0].clamp(1.0, 64.0);
        let hi = radius_range[1].clamp(lo, 64.0);
        self.radius_range = [lo, hi];
        self
    }

    fn from_vec(pts: Vec<Bubble3DPoint>, radius_range: [f32; 2]) -> Self {
        let mut extents = CachedExtents3D::default();
        let mut s_lo = f64::INFINITY;
        let mut s_hi = f64::NEG_INFINITY;
        let mut non_finite = 0usize;
        for pt in &pts {
            if pt.position.iter().all(|v| v.is_finite()) && pt.size.is_finite() && pt.size >= 0.0 {
                extents.push_point(pt.position);
                s_lo = s_lo.min(pt.size);
                s_hi = s_hi.max(pt.size);
            } else {
                non_finite += 1;
            }
        }
        let size_extent = if s_lo <= s_hi {
            [s_lo, s_hi]
        } else {
            [0.0, 1.0]
        };
        Self {
            points: Arc::from(pts),
            size_extent,
            radius_range,
            extents,
            non_finite_points: non_finite,
        }
    }

    pub fn points(&self) -> &[Bubble3DPoint] {
        &self.points
    }
    pub fn size_extent(&self) -> [f64; 2] {
        self.size_extent
    }
    pub fn radius_range(&self) -> [f32; 2] {
        self.radius_range
    }
    pub fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }
    pub fn pixel_radius(&self, size: f64) -> f32 {
        bubble_pixel_radius(size, self.size_extent, self.radius_range)
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.points.as_ref())
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

/// One sample in [`Swarm3DData`] (`ML-018`) preserving original group/raw indices alongside jittered XY.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swarm3DPoint {
    pub raw_xy: [f64; 2],
    pub position: Point3D,
    pub group_index: usize,
    pub raw_index: usize,
}

/// Immutable host-owned 3D beeswarm data (`swarmchart3`, `ML-018`) with radial XY disk jitter by Z density.
#[derive(Clone, Debug)]
pub struct Swarm3DData {
    points: Arc<[Swarm3DPoint]>,
    extents: CachedExtents3D,
    non_finite_points: usize,
}

impl Swarm3DData {
    pub fn from_groups(
        xy_centers: &[[f64; 2]],
        z_groups: &[&[f64]],
        max_radius: f64,
    ) -> Result<Self, Plot3DError> {
        if xy_centers.len() != z_groups.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        if xy_centers.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let rad = if max_radius.is_finite() && max_radius > 0.0 {
            max_radius
        } else {
            0.35
        };
        let mut points = Vec::new();
        let mut extents = CachedExtents3D::default();
        let mut non_finite = 0usize;
        let bins = 24usize;
        let golden = PI * (3.0 - 5.0_f64.sqrt());

        for (gi, (&center, &zs)) in xy_centers.iter().zip(z_groups.iter()).enumerate() {
            if !center[0].is_finite() || !center[1].is_finite() {
                non_finite += zs.len();
                continue;
            }
            let mut z_min = f64::INFINITY;
            let mut z_max = f64::NEG_INFINITY;
            let mut valid = Vec::with_capacity(zs.len());
            for (ri, &z) in zs.iter().enumerate() {
                if z.is_finite() {
                    z_min = z_min.min(z);
                    z_max = z_max.max(z);
                    valid.push((ri, z));
                } else {
                    non_finite += 1;
                }
            }
            if valid.is_empty() {
                continue;
            }
            let z_span = (z_max - z_min).max(1e-12);
            let mut counts = vec![0usize; bins];
            for &(_, z) in &valid {
                let b = (((z - z_min) / z_span) * (bins - 1) as f64)
                    .floor()
                    .clamp(0.0, (bins - 1) as f64) as usize;
                counts[b] += 1;
            }
            let max_bin = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
            let mut bin_cursor = vec![0usize; bins];
            for (ri, z) in valid {
                let b = (((z - z_min) / z_span) * (bins - 1) as f64)
                    .floor()
                    .clamp(0.0, (bins - 1) as f64) as usize;
                let rank = bin_cursor[b];
                bin_cursor[b] += 1;
                let local_cnt = counts[b].max(1);
                let density_frac = (local_cnt as f64 / max_bin).sqrt();
                let r_frac = ((rank as f64 + 0.5) / local_cnt as f64).sqrt();
                let angle = ri as f64 * golden + gi as f64 * 0.7;
                let offset = rad * density_frac * r_frac;
                let pos = [
                    center[0] + offset * angle.cos(),
                    center[1] + offset * angle.sin(),
                    z,
                ];
                extents.push_point(pos);
                points.push(Swarm3DPoint {
                    raw_xy: center,
                    position: pos,
                    group_index: gi,
                    raw_index: ri,
                });
            }
        }
        Ok(Self {
            points: Arc::from(points),
            extents,
            non_finite_points: non_finite,
        })
    }

    pub fn points(&self) -> &[Swarm3DPoint] {
        &self.points
    }
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    pub fn non_finite_points(&self) -> usize {
        self.non_finite_points
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.points.as_ref())
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
