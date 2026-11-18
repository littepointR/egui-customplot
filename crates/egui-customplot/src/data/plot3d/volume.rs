use std::sync::Arc;

use crate::{AxisScale, ContourError, ContourLevels, Point3D, axis::intersect_plane_unit_cube};

use super::{
    CachedExtents3D, Contour3DData, Contour3DLine, Plot3DError, ScalarVolume3D, Surface3DData,
    contour3::{intersect_tri_scalar, resolve_levels, stitch_chains},
    domain_idx, union_bounds3,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SlicePlane3D {
    X(f64),
    Y(f64),
    Z(f64),
    Arbitrary {
        normal: [f64; 3],
        offset: f64,
        resolution: usize,
    },
}

/// Host-owned volumetric slice surfaces (`slice`, `ML-077`).
#[derive(Clone, Debug)]
pub struct VolumeSliceData {
    pub(crate) version: u64,
    slices: Arc<[Surface3DData]>,
    scalar_range: Option<[f64; 2]>,
}

impl VolumeSliceData {
    pub fn from_volume(vol: &ScalarVolume3D, planes: &[SlicePlane3D]) -> Result<Self, Plot3DError> {
        if planes.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let bmin = vol.bounds().min();
        let bmax = vol.bounds().max();
        let (x, y, z) = (vol.x(), vol.y(), vol.z());
        let mut slices = Vec::new();
        for &plane in planes {
            match plane {
                SlicePlane3D::X(x0) => {
                    if x0.is_finite() && x0 >= bmin[0] - 1e-9 && x0 <= bmax[0] + 1e-9 {
                        let x0 = x0.clamp(bmin[0], bmax[0]);
                        slices.push(axis_slice(
                            vol,
                            |iy, iz| [x0, y[iy], z[iz]],
                            z.len(),
                            y.len(),
                        )?);
                    }
                }
                SlicePlane3D::Y(y0) => {
                    if y0.is_finite() && y0 >= bmin[1] - 1e-9 && y0 <= bmax[1] + 1e-9 {
                        let y0 = y0.clamp(bmin[1], bmax[1]);
                        slices.push(axis_slice(
                            vol,
                            |ix, iz| [x[ix], y0, z[iz]],
                            z.len(),
                            x.len(),
                        )?);
                    }
                }
                SlicePlane3D::Z(z0) => {
                    if z0.is_finite() && z0 >= bmin[2] - 1e-9 && z0 <= bmax[2] + 1e-9 {
                        let z0 = z0.clamp(bmin[2], bmax[2]);
                        slices.push(axis_slice(
                            vol,
                            |ix, iy| [x[ix], y[iy], z0],
                            y.len(),
                            x.len(),
                        )?);
                    }
                }
                SlicePlane3D::Arbitrary {
                    normal,
                    offset,
                    resolution,
                } => {
                    if let Some(s) = arbitrary_slice(vol, normal, offset, resolution)? {
                        slices.push(s);
                    }
                }
            }
        }
        let scalar_range = slices.iter().filter_map(Surface3DData::scalar_range).fold(
            None,
            |acc: Option<[f64; 2]>, r| match acc {
                None => Some(r),
                Some([lo, hi]) => Some([lo.min(r[0]), hi.max(r[1])]),
            },
        );
        Ok(Self {
            version: crate::data::points::next_version(),
            slices: Arc::from(slices),
            scalar_range,
        })
    }

    pub fn slices(&self) -> &[Surface3DData] {
        &self.slices
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn sample_bytes(&self) -> usize {
        self.slices.iter().map(Surface3DData::sample_bytes).sum()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.slices.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.slices
            .iter()
            .fold(None, |acc, s| union_bounds3(acc, s.extent(scales)))
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        let mut inv = 0usize;
        for s in self.slices.iter() {
            inv += s.non_finite_points();
            if domain_idx(scales) != 0 {
                inv += s
                    .points()
                    .iter()
                    .filter(|p| p[0] <= 0.0 || p[1] <= 0.0 || p[2] <= 0.0)
                    .count();
            }
        }
        inv
    }
}

fn axis_slice(
    vol: &ScalarVolume3D,
    coord: impl Fn(usize, usize) -> Point3D,
    rows: usize,
    cols: usize,
) -> Result<Surface3DData, Plot3DError> {
    let mut pts = Vec::with_capacity(rows * cols);
    let mut scals = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            let p = coord(c, r);
            if let Some(s) = vol.sample(p) {
                pts.push(p);
                scals.push(s);
            } else {
                pts.push([f64::NAN; 3]);
                scals.push(f64::NAN);
            }
        }
    }
    Surface3DData::from_parametric_grid(rows, cols, pts)?.with_scalars(&scals)
}

fn arbitrary_slice(
    vol: &ScalarVolume3D,
    normal: [f64; 3],
    offset: f64,
    resolution: usize,
) -> Result<Option<Surface3DData>, Plot3DError> {
    let nlen = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if !nlen.is_finite() || nlen <= 1e-12 || !offset.is_finite() {
        return Err(Plot3DError::InvalidDimensions);
    }
    let bmin = vol.bounds().min();
    let span = vol.bounds().span();
    let unit_off = offset - (normal[0] * bmin[0] + normal[1] * bmin[1] + normal[2] * bmin[2]);
    let poly_u = intersect_plane_unit_cube(
        [
            normal[0] * span[0],
            normal[1] * span[1],
            normal[2] * span[2],
        ],
        unit_off,
    );
    if poly_u.len() < 3 {
        return Ok(None);
    }
    let poly: Vec<Point3D> = poly_u
        .into_iter()
        .map(|u| {
            [
                bmin[0] + span[0] * u[0],
                bmin[1] + span[1] * u[1],
                bmin[2] + span[2] * u[2],
            ]
        })
        .collect();
    let nn = [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen];
    let seed = if nn[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let tu = cross(nn, seed);
    let tu_l = (tu[0] * tu[0] + tu[1] * tu[1] + tu[2] * tu[2])
        .sqrt()
        .max(1e-12);
    let tu = [tu[0] / tu_l, tu[1] / tu_l, tu[2] / tu_l];
    let tv = cross(nn, tu);
    let c = [
        poly.iter().map(|p| p[0]).sum::<f64>() / poly.len() as f64,
        poly.iter().map(|p| p[1]).sum::<f64>() / poly.len() as f64,
        poly.iter().map(|p| p[2]).sum::<f64>() / poly.len() as f64,
    ];
    let mut uv: Vec<[f64; 2]> = Vec::with_capacity(poly.len());
    let (mut u0, mut u1, mut v0, mut v1) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for p in &poly {
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        let pu = d[0] * tu[0] + d[1] * tu[1] + d[2] * tu[2];
        let pv = d[0] * tv[0] + d[1] * tv[1] + d[2] * tv[2];
        u0 = u0.min(pu);
        u1 = u1.max(pu);
        v0 = v0.min(pv);
        v1 = v1.max(pv);
        uv.push([pu, pv]);
    }
    let res = resolution.clamp(4, 256);
    let (x, y, z) = (vol.x(), vol.y(), vol.z());
    let mut pts = Vec::with_capacity(res * res);
    let mut scals = Vec::with_capacity(res * res);
    for r in 0..res {
        let fv = v0 + (v1 - v0) * (r as f64 / (res - 1) as f64);
        for k in 0..res {
            let fu = u0 + (u1 - u0) * (k as f64 / (res - 1) as f64);
            let p = [
                c[0] + fu * tu[0] + fv * tv[0],
                c[1] + fu * tu[1] + fv * tv[1],
                c[2] + fu * tu[2] + fv * tv[2],
            ];
            let p_clamped = [
                p[0].clamp(x[0], x[x.len() - 1]),
                p[1].clamp(y[0], y[y.len() - 1]),
                p[2].clamp(z[0], z[z.len() - 1]),
            ];
            if in_convex_2d(fu, fv, &uv)
                && let Some(val) = vol.sample(p_clamped)
            {
                pts.push(p);
                scals.push(val);
            } else {
                pts.push([f64::NAN; 3]);
                scals.push(f64::NAN);
            }
        }
    }
    Ok(Some(
        Surface3DData::from_parametric_grid(res, res, pts)?.with_scalars(&scals)?,
    ))
}

fn in_convex_2d(u: f64, v: f64, poly: &[[f64; 2]]) -> bool {
    let mut pos = 0usize;
    let mut neg = 0usize;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let cr = (b[0] - a[0]) * (v - a[1]) - (b[1] - a[1]) * (u - a[0]);
        if cr > 1e-9 {
            pos += 1;
        } else if cr < -1e-9 {
            neg += 1;
        }
    }
    pos == 0 || neg == 0
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

impl Contour3DData {
    /// Extracts 3D contours directly across volume slice planes (`contourslice`, `ML-054`).
    pub fn from_volume_slices(
        vol: &ScalarVolume3D,
        planes: &[SlicePlane3D],
        levels: ContourLevels,
    ) -> Result<Self, ContourError> {
        let slices =
            VolumeSliceData::from_volume(vol, planes).map_err(|_| ContourError::InvalidLevels)?;
        let lvls = resolve_levels(slices.scalar_range(), levels)?;
        if lvls.is_empty() {
            return Ok(Self::from_lines(
                Vec::new(),
                Vec::new(),
                CachedExtents3D::default(),
            ));
        }
        let mut segs_per_lvl: Vec<Vec<[Point3D; 2]>> = vec![Vec::new(); lvls.len()];
        for slice in slices.slices() {
            let (rows, cols) = (slice.rows(), slice.cols());
            let (pts, scals) = (slice.points(), slice.scalars());
            for r in 0..(rows - 1) {
                for c in 0..(cols - 1) {
                    let idx = [
                        r * cols + c,
                        r * cols + c + 1,
                        (r + 1) * cols + c,
                        (r + 1) * cols + c + 1,
                    ];
                    if !idx
                        .iter()
                        .all(|&i| scals[i].is_finite() && pts[i].iter().all(|x| x.is_finite()))
                    {
                        continue;
                    }
                    for tri_i in [[idx[0], idx[1], idx[3]], [idx[0], idx[3], idx[2]]] {
                        let tri = tri_i.map(|i| pts[i]);
                        let s = tri_i.map(|i| scals[i]);
                        let smin = s[0].min(s[1]).min(s[2]);
                        let smax = s[0].max(s[1]).max(s[2]);
                        let i0 = lvls.partition_point(|&l| l < smin - 1e-12);
                        for (li, &lvl) in lvls.iter().enumerate().skip(i0) {
                            if lvl > smax + 1e-12 {
                                break;
                            }
                            if let Some(seg) = intersect_tri_scalar(tri, s, lvl) {
                                segs_per_lvl[li].push(seg);
                            }
                        }
                    }
                }
            }
        }
        let mut lines = Vec::new();
        let mut extents = CachedExtents3D::default();
        for (&lvl, segs) in lvls.iter().zip(segs_per_lvl) {
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
        Ok(Self::from_lines(lvls, lines, extents))
    }
}
