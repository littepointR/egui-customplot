use std::{f64::consts::TAU, sync::Arc};

use crate::{AxisScale, FlowError, Point3D};

use super::{Stream3DOptions, Streamline3DData, Surface3DData, VectorField3D, union_bounds3};

/// Host-owned curl-twisted stream ribbons (`streamribbon`, `ML-074`).
#[derive(Clone, Debug)]
pub struct StreamRibbon3DData {
    strips: Arc<[Surface3DData]>,
    twists_rad: Arc<[f64]>,
    scalar_range: Option<[f64; 2]>,
}

impl StreamRibbon3DData {
    pub fn from_field(
        field: &VectorField3D,
        seeds: &[Point3D],
        width: f64,
        twist_scale: f64,
        opts: Stream3DOptions,
    ) -> Result<Self, FlowError> {
        if !width.is_finite() || width <= 0.0 || !twist_scale.is_finite() {
            return Err(FlowError::InvalidOptions);
        }
        let sl = Streamline3DData::from_field(field, seeds, opts)?;
        let hw = 0.5 * width;
        let mut strips = Vec::with_capacity(sl.lines().len());
        let mut twists = Vec::with_capacity(sl.lines().len());
        for line in sl.lines() {
            let n = line.points.len();
            if n < 2 {
                continue;
            }
            let t0 = norm3(sub3(line.points[1], line.points[0]));
            let (mut frame_u, _) = bishop_seed(t0);
            let mut prev_t = t0;
            let mut theta = 0.0_f64;
            let mut pts = Vec::with_capacity(n * 2);
            let mut scals = Vec::with_capacity(n * 2);

            for i in 0..n {
                let cur_t = if i + 1 < n {
                    norm3(sub3(line.points[i + 1], line.points[i]))
                } else {
                    prev_t
                };
                if i > 0 {
                    let ds = dist3(line.points[i], line.points[i - 1]);
                    frame_u = transport_normal(frame_u, prev_t, cur_t);
                    if let Some(curl) = field.curl(line.points[i]) {
                        let axial = curl[0] * cur_t[0] + curl[1] * cur_t[1] + curl[2] * cur_t[2];
                        theta += twist_scale * 0.5 * axial * ds / line.speeds[i].max(1e-9);
                    }
                    prev_t = cur_t;
                }
                let frame_v = cross3(cur_t, frame_u);
                let (st, ct) = theta.sin_cos();
                let side = [
                    frame_u[0] * ct + frame_v[0] * st,
                    frame_u[1] * ct + frame_v[1] * st,
                    frame_u[2] * ct + frame_v[2] * st,
                ];
                let p = line.points[i];
                pts.push([
                    p[0] - hw * side[0],
                    p[1] - hw * side[1],
                    p[2] - hw * side[2],
                ]);
                pts.push([
                    p[0] + hw * side[0],
                    p[1] + hw * side[1],
                    p[2] + hw * side[2],
                ]);
                scals.push(line.speeds[i]);
                scals.push(line.speeds[i]);
            }
            if let Ok(surf) =
                Surface3DData::from_parametric_grid(n, 2, pts).and_then(|s| s.with_scalars(&scals))
            {
                strips.push(surf);
                twists.push(theta);
            }
        }
        Ok(Self {
            strips: Arc::from(strips),
            twists_rad: Arc::from(twists),
            scalar_range: sl.speed_range(),
        })
    }

    pub fn strips(&self) -> &[Surface3DData] {
        &self.strips
    }
    pub fn total_twist_rad(&self) -> &[f64] {
        &self.twists_rad
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn sample_bytes(&self) -> usize {
        self.strips.iter().map(Surface3DData::sample_bytes).sum()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.strips.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.strips
            .iter()
            .fold(None, |acc, s| union_bounds3(acc, s.extent(scales)))
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        let _ = scales;
        self.strips
            .iter()
            .map(Surface3DData::non_finite_points)
            .sum()
    }
}

/// Host-owned divergence-scaled cylindrical stream tubes (`streamtube`, `ML-075`).
#[derive(Clone, Debug)]
pub struct StreamTube3DData {
    tubes: Arc<[Surface3DData]>,
    scalar_range: Option<[f64; 2]>,
}

impl StreamTube3DData {
    pub fn from_field(
        field: &VectorField3D,
        seeds: &[Point3D],
        base_radius: f64,
        radial_subdivisions: usize,
        opts: Stream3DOptions,
    ) -> Result<Self, FlowError> {
        if !base_radius.is_finite() || base_radius <= 0.0 {
            return Err(FlowError::InvalidOptions);
        }
        let n_div = radial_subdivisions.clamp(4, 48);
        let sl = Streamline3DData::from_field(field, seeds, opts)?;
        let mut tubes = Vec::with_capacity(sl.lines().len());

        for line in sl.lines() {
            let n = line.points.len();
            if n < 2 {
                continue;
            }
            let t0 = norm3(sub3(line.points[1], line.points[0]));
            let (mut fu, _) = bishop_seed(t0);
            let mut prev_t = t0;
            let mut log_r = 0.0_f64;
            let mut pts = Vec::with_capacity(n * (n_div + 1));
            let mut norms = Vec::with_capacity(n * (n_div + 1));
            let mut scals = Vec::with_capacity(n * (n_div + 1));

            for i in 0..n {
                let cur_t = if i + 1 < n {
                    norm3(sub3(line.points[i + 1], line.points[i]))
                } else {
                    prev_t
                };
                if i > 0 {
                    let ds = dist3(line.points[i], line.points[i - 1]);
                    fu = transport_normal(fu, prev_t, cur_t);
                    if let Some(div) = field.divergence(line.points[i]) {
                        log_r =
                            (log_r + 0.5 * div * ds / line.speeds[i].max(1e-9)).clamp(-1.05, 0.79);
                    }
                    prev_t = cur_t;
                }
                let fv = cross3(cur_t, fu);
                let r_loc =
                    (base_radius * log_r.exp()).clamp(0.35 * base_radius, 2.2 * base_radius);
                let p = line.points[i];
                for k in 0..=n_div {
                    let ang = if k == n_div {
                        0.0
                    } else {
                        TAU * (k as f64 / n_div as f64)
                    };
                    let (sa, ca) = ang.sin_cos();
                    let rad_dir = [
                        ca * fu[0] + sa * fv[0],
                        ca * fu[1] + sa * fv[1],
                        ca * fu[2] + sa * fv[2],
                    ];
                    pts.push([
                        p[0] + r_loc * rad_dir[0],
                        p[1] + r_loc * rad_dir[1],
                        p[2] + r_loc * rad_dir[2],
                    ]);
                    norms.push(rad_dir);
                    scals.push(line.speeds[i]);
                }
            }
            if let Ok(surf) = Surface3DData::from_parametric_grid(n, n_div + 1, pts)
                .and_then(|s| s.with_normals(norms))
                .and_then(|s| s.with_scalars(&scals))
            {
                tubes.push(surf);
            }
        }
        Ok(Self {
            tubes: Arc::from(tubes),
            scalar_range: sl.speed_range(),
        })
    }

    pub fn tubes(&self) -> &[Surface3DData] {
        &self.tubes
    }
    pub fn scalar_range(&self) -> Option<[f64; 2]> {
        self.scalar_range
    }
    pub fn sample_bytes(&self) -> usize {
        self.tubes.iter().map(Surface3DData::sample_bytes).sum()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.tubes.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.tubes
            .iter()
            .fold(None, |acc, s| union_bounds3(acc, s.extent(scales)))
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        let _ = scales;
        self.tubes
            .iter()
            .map(Surface3DData::non_finite_points)
            .sum()
    }
}

fn sub3(a: Point3D, b: Point3D) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dist3(a: Point3D, b: Point3D) -> f64 {
    let d = sub3(a, b);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm3(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 1e-12 && l.is_finite() {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 0.0, 1.0]
    }
}

pub(super) fn bishop_seed(t: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let seed = if t[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = norm3(cross3(t, seed));
    let v = norm3(cross3(t, u));
    (u, v)
}

fn transport_normal(prev_u: [f64; 3], prev_t: [f64; 3], cur_t: [f64; 3]) -> [f64; 3] {
    let b = cross3(prev_t, cur_t);
    let bl = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    if bl < 1e-10 {
        let dot = prev_u[0] * cur_t[0] + prev_u[1] * cur_t[1] + prev_u[2] * cur_t[2];
        return norm3([
            prev_u[0] - dot * cur_t[0],
            prev_u[1] - dot * cur_t[1],
            prev_u[2] - dot * cur_t[2],
        ]);
    }
    let ax = [b[0] / bl, b[1] / bl, b[2] / bl];
    let cos_a =
        (prev_t[0] * cur_t[0] + prev_t[1] * cur_t[1] + prev_t[2] * cur_t[2]).clamp(-1.0, 1.0);
    let sin_a = (1.0 - cos_a * cos_a).max(0.0).sqrt();
    let k_cross_u = cross3(ax, prev_u);
    let k_dot_u = ax[0] * prev_u[0] + ax[1] * prev_u[1] + ax[2] * prev_u[2];
    norm3([
        prev_u[0] * cos_a + k_cross_u[0] * sin_a + ax[0] * k_dot_u * (1.0 - cos_a),
        prev_u[1] * cos_a + k_cross_u[1] * sin_a + ax[1] * k_dot_u * (1.0 - cos_a),
        prev_u[2] * cos_a + k_cross_u[2] * sin_a + ax[2] * k_dot_u * (1.0 - cos_a),
    ])
}
