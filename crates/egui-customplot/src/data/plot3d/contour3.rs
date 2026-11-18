use std::sync::Arc;

use crate::{AxisScale, ContourData, ContourError, ContourLevels, Point3D, ScalarField2D};

use super::{CachedExtents3D, Surface3DData};

/// Single iso-level polyline in 3D (`contour3` / `surfc` / `meshc`).
#[derive(Clone, Debug)]
pub struct Contour3DLine {
    pub level: f64,
    pub points: Arc<[Point3D]>,
}

/// Immutable host-owned 3D contour collection (`contour3`, `ML-053`).
#[derive(Clone, Debug)]
pub struct Contour3DData {
    pub(crate) version: u64,
    levels: Arc<[f64]>,
    lines: Arc<[Contour3DLine]>,
    extents: CachedExtents3D,
}

impl Contour3DData {
    pub fn from_contour2d(c2d: &ContourData) -> Self {
        let mut lines = Vec::with_capacity(c2d.lines().len());
        let mut extents = CachedExtents3D::default();
        for line in c2d.lines() {
            let pts: Arc<[Point3D]> = line
                .points
                .iter()
                .map(|p| [p[0], p[1], line.level])
                .collect();
            for &p in pts.iter() {
                extents.push_point(p);
            }
            lines.push(Contour3DLine {
                level: line.level,
                points: pts,
            });
        }
        Self {
            version: crate::data::points::next_version(),
            levels: Arc::from(c2d.levels()),
            lines: Arc::from(lines),
            extents,
        }
    }

    pub fn from_field(field: &ScalarField2D, levels: ContourLevels) -> Result<Self, ContourError> {
        let c2d = ContourData::from_field(field, levels)?;
        Ok(Self::from_contour2d(&c2d))
    }

    /// Extracts 3D contour lines directly at their true `z = level` height across quad triangles.
    pub fn from_surface(surf: &Surface3DData, levels: ContourLevels) -> Result<Self, ContourError> {
        let levels = resolve_levels(surf.z_range(), levels)?;
        if levels.is_empty() {
            return Ok(Self {
                version: crate::data::points::next_version(),
                levels: Arc::from([]),
                lines: Arc::from([]),
                extents: CachedExtents3D::default(),
            });
        }
        let rows = surf.rows();
        let cols = surf.cols();
        let pts = surf.points();
        let mut segs_per_level: Vec<Vec<[Point3D; 2]>> = vec![Vec::new(); levels.len()];

        for r in 0..(rows - 1) {
            for c in 0..(cols - 1) {
                let p00 = pts[r * cols + c];
                let p10 = pts[r * cols + c + 1];
                let p01 = pts[(r + 1) * cols + c];
                let p11 = pts[(r + 1) * cols + c + 1];
                if ![p00, p10, p01, p11]
                    .iter()
                    .all(|p| p[0].is_finite() && p[1].is_finite() && p[2].is_finite())
                {
                    continue;
                }
                for tri in [[p00, p10, p11], [p00, p11, p01]] {
                    let zmin = tri[0][2].min(tri[1][2]).min(tri[2][2]);
                    let zmax = tri[0][2].max(tri[1][2]).max(tri[2][2]);
                    let i0 = levels.partition_point(|&l| l < zmin - 1e-12);
                    for (li, &lvl) in levels.iter().enumerate().skip(i0) {
                        if lvl > zmax + 1e-12 {
                            break;
                        }
                        if let Some(seg) = intersect_tri_z(tri, lvl) {
                            segs_per_level[li].push(seg);
                        }
                    }
                }
            }
        }

        let mut lines = Vec::new();
        let mut extents = CachedExtents3D::default();
        for (&lvl, segs) in levels.iter().zip(segs_per_level) {
            for chain in stitch_chains(segs) {
                for &p in &chain {
                    extents.push_point(p);
                }
                lines.push(Contour3DLine {
                    level: lvl,
                    points: Arc::from(chain),
                });
            }
        }
        Ok(Self::from_lines(levels, lines, extents))
    }

    pub(super) fn from_lines(
        levels: Vec<f64>,
        lines: Vec<Contour3DLine>,
        extents: CachedExtents3D,
    ) -> Self {
        Self {
            version: crate::data::points::next_version(),
            levels: Arc::from(levels),
            lines: Arc::from(lines),
            extents,
        }
    }

    pub fn levels(&self) -> &[f64] {
        &self.levels
    }
    pub fn lines(&self) -> &[Contour3DLine] {
        &self.lines
    }
    pub fn sample_bytes(&self) -> usize {
        self.levels.len() * 8
            + self
                .lines
                .iter()
                .map(|l| std::mem::size_of_val(l.points.as_ref()))
                .sum::<usize>()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.lines.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

pub(super) fn resolve_levels(
    z_range: Option<[f64; 2]>,
    levels: ContourLevels,
) -> Result<Vec<f64>, ContourError> {
    match levels {
        ContourLevels::Explicit(mut vals) => {
            if vals.is_empty() || vals.len() > 256 || vals.iter().any(|v| !v.is_finite()) {
                return Err(ContourError::InvalidLevels);
            }
            vals.sort_by(f64::total_cmp);
            vals.dedup();
            Ok(vals)
        }
        ContourLevels::Count(n) => {
            if n == 0 || n > 256 {
                return Err(ContourError::InvalidLevels);
            }
            let Some([zmin, zmax]) = z_range else {
                return Ok(Vec::new());
            };
            if (zmax - zmin).abs() <= 1e-12 {
                return Ok(Vec::new());
            }
            Ok((1..=n)
                .map(|k| zmin + (zmax - zmin) * (k as f64 / (n + 1) as f64))
                .collect())
        }
    }
}

fn intersect_tri_z(tri: [Point3D; 3], lvl: f64) -> Option<[Point3D; 2]> {
    intersect_tri_scalar(tri, [tri[0][2], tri[1][2], tri[2][2]], lvl)
}

pub(super) fn intersect_tri_scalar(
    tri: [Point3D; 3],
    scalars: [f64; 3],
    lvl: f64,
) -> Option<[Point3D; 2]> {
    let s = [0, 1, 2].map(|i| {
        if (scalars[i] - lvl).abs() < 1e-12 {
            lvl + 1e-12
        } else {
            scalars[i]
        }
    });
    let mut hits = [None, None];
    let mut count = 0usize;
    for (i, j) in [(0, 1), (1, 2), (2, 0)] {
        if (s[i] < lvl && s[j] > lvl) || (s[i] > lvl && s[j] < lvl) {
            let t = (lvl - s[i]) / (s[j] - s[i]);
            let p = [
                tri[i][0] + t * (tri[j][0] - tri[i][0]),
                tri[i][1] + t * (tri[j][1] - tri[i][1]),
                tri[i][2] + t * (tri[j][2] - tri[i][2]),
            ];
            if count < 2 {
                hits[count] = Some(p);
                count += 1;
            }
        }
    }
    match (hits[0], hits[1]) {
        (Some(a), Some(b))
            if ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)) > 1e-24 =>
        {
            Some([a, b])
        }
        _ => None,
    }
}

pub(super) fn stitch_chains(mut segs: Vec<[Point3D; 2]>) -> Vec<Vec<Point3D>> {
    let mut chains = Vec::new();
    let near = |a: Point3D, b: Point3D| {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        dx * dx + dy * dy + dz * dz <= 1e-16
    };
    while let Some([s0, s1]) = segs.pop() {
        let mut chain = vec![s0, s1];
        loop {
            let head = chain[0];
            let tail = chain[chain.len() - 1];
            let mut found = false;
            for idx in (0..segs.len()).rev() {
                let [a, b] = segs[idx];
                if near(tail, a) {
                    chain.push(b);
                    segs.swap_remove(idx);
                    found = true;
                    break;
                } else if near(tail, b) {
                    chain.push(a);
                    segs.swap_remove(idx);
                    found = true;
                    break;
                } else if near(head, b) {
                    chain.insert(0, a);
                    segs.swap_remove(idx);
                    found = true;
                    break;
                } else if near(head, a) {
                    chain.insert(0, b);
                    segs.swap_remove(idx);
                    found = true;
                    break;
                }
            }
            if !found {
                break;
            }
        }
        chains.push(chain);
    }
    chains
}
