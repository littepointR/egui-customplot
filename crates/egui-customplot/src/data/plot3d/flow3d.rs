use std::sync::Arc;

use crate::{AxisScale, FlowError, Point3D, Quiver3DArrow, Quiver3DData};

use super::{CachedExtents3D, SlicePlane3D, VectorField3D};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stream3DOptions {
    pub step_size: f64,
    pub max_steps: usize,
    pub min_speed: f64,
}

impl Default for Stream3DOptions {
    fn default() -> Self {
        Self {
            step_size: 0.05,
            max_steps: 400,
            min_speed: 1e-6,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Streamline3DLine {
    pub points: Arc<[Point3D]>,
    pub speeds: Arc<[f64]>,
    pub times: Arc<[f64]>,
}

/// Host-owned 3D RK4 streamlines & planar stream slices (`streamline`, `streamslice` 3D).
#[derive(Clone, Debug)]
pub struct Streamline3DData {
    lines: Arc<[Streamline3DLine]>,
    speed_range: Option<[f64; 2]>,
    extents: CachedExtents3D,
}

impl Streamline3DData {
    pub fn from_field(
        field: &VectorField3D,
        seeds: &[Point3D],
        opts: Stream3DOptions,
    ) -> Result<Self, FlowError> {
        Self::integrate(field, seeds, opts, None)
    }

    pub fn streamslice(
        field: &VectorField3D,
        plane: SlicePlane3D,
        density: usize,
    ) -> Result<(Self, Quiver3DData), FlowError> {
        let d = density.clamp(2, 32);
        let bmin = field.bounds().min();
        let bmax = field.bounds().max();
        let (seeds, proj_n) = match plane {
            SlicePlane3D::X(x0) => (
                grid_seeds(d, |a, b| {
                    [
                        x0.clamp(bmin[0], bmax[0]),
                        lerp(bmin[1], bmax[1], a),
                        lerp(bmin[2], bmax[2], b),
                    ]
                }),
                [1.0, 0.0, 0.0],
            ),
            SlicePlane3D::Y(y0) => (
                grid_seeds(d, |a, b| {
                    [
                        lerp(bmin[0], bmax[0], a),
                        y0.clamp(bmin[1], bmax[1]),
                        lerp(bmin[2], bmax[2], b),
                    ]
                }),
                [0.0, 1.0, 0.0],
            ),
            SlicePlane3D::Z(z0) => (
                grid_seeds(d, |a, b| {
                    [
                        lerp(bmin[0], bmax[0], a),
                        lerp(bmin[1], bmax[1], b),
                        z0.clamp(bmin[2], bmax[2]),
                    ]
                }),
                [0.0, 0.0, 1.0],
            ),
            SlicePlane3D::Arbitrary { normal, offset, .. } => {
                let nl =
                    (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
                if !nl.is_finite() || nl <= 1e-12 || !offset.is_finite() {
                    return Err(FlowError::InvalidOptions);
                }
                let nn = [normal[0] / nl, normal[1] / nl, normal[2] / nl];
                return arbitrary_slice_streamlines(field, offset / nl, nn, d);
            }
        };
        let opts = Stream3DOptions {
            max_steps: 120,
            ..Default::default()
        };
        let sl = Self::integrate(field, &seeds, opts, Some(proj_n))?;
        let quiver = sample_slice_quiver(field, &sl, proj_n);
        Ok((sl, quiver))
    }

    fn integrate(
        field: &VectorField3D,
        seeds: &[Point3D],
        opts: Stream3DOptions,
        proj_normal: Option<[f64; 3]>,
    ) -> Result<Self, FlowError> {
        if !opts.step_size.is_finite()
            || opts.step_size <= 0.0
            || opts.max_steps == 0
            || !opts.min_speed.is_finite()
        {
            return Err(FlowError::InvalidOptions);
        }
        let eval = |p: Point3D| -> Option<([f64; 3], f64)> {
            let mut v = field.sample(p)?;
            if let Some(n) = proj_normal {
                let dn = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
                v = [v[0] - dn * n[0], v[1] - dn * n[1], v[2] - dn * n[2]];
            }
            let spd = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            (spd >= opts.min_speed && spd.is_finite()).then_some((v, spd))
        };

        let mut lines = Vec::with_capacity(seeds.len());
        let mut extents = CachedExtents3D::default();
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);

        for &seed in seeds {
            let Some((_, mut spd)) = eval(seed) else {
                continue;
            };
            let mut pts = vec![seed];
            let mut spds = vec![spd];
            let mut times = vec![0.0];
            let mut p = seed;
            let mut t = 0.0;
            for _ in 0..opts.max_steps {
                let Some((k1, s1)) = eval(p) else { break };
                let d1 = [k1[0] / s1, k1[1] / s1, k1[2] / s1];
                let h = opts.step_size;
                let p2 = [
                    p[0] + 0.5 * h * d1[0],
                    p[1] + 0.5 * h * d1[1],
                    p[2] + 0.5 * h * d1[2],
                ];
                let Some((k2, s2)) = eval(p2) else { break };
                let d2 = [k2[0] / s2, k2[1] / s2, k2[2] / s2];
                let p3 = [
                    p[0] + 0.5 * h * d2[0],
                    p[1] + 0.5 * h * d2[1],
                    p[2] + 0.5 * h * d2[2],
                ];
                let Some((k3, s3)) = eval(p3) else { break };
                let d3 = [k3[0] / s3, k3[1] / s3, k3[2] / s3];
                let p4 = [p[0] + h * d3[0], p[1] + h * d3[1], p[2] + h * d3[2]];
                let Some((k4, s4)) = eval(p4) else { break };
                let d4 = [k4[0] / s4, k4[1] / s4, k4[2] / s4];
                p = [
                    p[0] + (h / 6.0) * (d1[0] + 2.0 * d2[0] + 2.0 * d3[0] + d4[0]),
                    p[1] + (h / 6.0) * (d1[1] + 2.0 * d2[1] + 2.0 * d3[1] + d4[1]),
                    p[2] + (h / 6.0) * (d1[2] + 2.0 * d2[2] + 2.0 * d3[2] + d4[2]),
                ];
                let Some((_, s_next)) = eval(p) else { break };
                spd = s_next;
                t += h / spd.max(1e-12);
                pts.push(p);
                spds.push(spd);
                times.push(t);
            }
            if pts.len() >= 2 {
                for &pt in &pts {
                    extents.push_point(pt);
                }
                for &s in &spds {
                    lo = lo.min(s);
                    hi = hi.max(s);
                }
                lines.push(Streamline3DLine {
                    points: Arc::from(pts),
                    speeds: Arc::from(spds),
                    times: Arc::from(times),
                });
            }
        }
        Ok(Self {
            lines: Arc::from(lines),
            speed_range: (lo <= hi).then_some([lo, hi]),
            extents,
        })
    }

    pub fn lines(&self) -> &[Streamline3DLine] {
        &self.lines
    }
    pub fn speed_range(&self) -> Option<[f64; 2]> {
        self.speed_range
    }
    pub fn sample_bytes(&self) -> usize {
        self.lines
            .iter()
            .map(|l| {
                std::mem::size_of_val(l.points.as_ref())
                    + std::mem::size_of_val(l.speeds.as_ref()) * 2
            })
            .sum()
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

fn sample_slice_quiver(
    field: &VectorField3D,
    sl: &Streamline3DData,
    proj_n: [f64; 3],
) -> Quiver3DData {
    let mut arrows = Vec::new();
    for line in sl.lines() {
        if let Some(&p) = line.points.get(line.points.len() / 2)
            && let Some(mut v) = field.sample(p)
        {
            let dn = v[0] * proj_n[0] + v[1] * proj_n[1] + v[2] * proj_n[2];
            for k in 0..3 {
                v[k] -= dn * proj_n[k];
            }
            arrows.push(Quiver3DArrow::new(p, v));
        }
    }
    Quiver3DData::new(arrows).autoscale(0.35)
}

fn arbitrary_slice_streamlines(
    field: &VectorField3D,
    d_norm: f64,
    nn: [f64; 3],
    density: usize,
) -> Result<(Streamline3DData, Quiver3DData), FlowError> {
    let (bmin, bmax) = (field.bounds().min(), field.bounds().max());
    let box_mid = [
        0.5 * (bmin[0] + bmax[0]),
        0.5 * (bmin[1] + bmax[1]),
        0.5 * (bmin[2] + bmax[2]),
    ];
    let shift = d_norm - (box_mid[0] * nn[0] + box_mid[1] * nn[1] + box_mid[2] * nn[2]);
    let plane_origin = [
        box_mid[0] + shift * nn[0],
        box_mid[1] + shift * nn[1],
        box_mid[2] + shift * nn[2],
    ];
    let seed = if nn[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = [
        nn[1] * seed[2] - nn[2] * seed[1],
        nn[2] * seed[0] - nn[0] * seed[2],
        nn[0] * seed[1] - nn[1] * seed[0],
    ];
    let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt().max(1e-12);
    let tu = [u[0] / ul, u[1] / ul, u[2] / ul];
    let tv = [
        nn[1] * tu[2] - nn[2] * tu[1],
        nn[2] * tu[0] - nn[0] * tu[2],
        nn[0] * tu[1] - nn[1] * tu[0],
    ];
    let span = field.bounds().span();
    let rad = 0.6 * (span[0] * span[0] + span[1] * span[1] + span[2] * span[2]).sqrt();
    let seeds: Vec<Point3D> = grid_seeds(density, |a, b| {
        let (su, sv) = ((a - 0.5) * 2.0 * rad, (b - 0.5) * 2.0 * rad);
        [
            plane_origin[0] + su * tu[0] + sv * tv[0],
            plane_origin[1] + su * tu[1] + sv * tv[1],
            plane_origin[2] + su * tu[2] + sv * tv[2],
        ]
    })
    .into_iter()
    .filter(|&p| field.bounds().contains(p))
    .collect();
    let sl = Streamline3DData::integrate(
        field,
        &seeds,
        Stream3DOptions {
            max_steps: 120,
            ..Default::default()
        },
        Some(nn),
    )?;
    let quiver = sample_slice_quiver(field, &sl, nn);
    Ok((sl, quiver))
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn grid_seeds(d: usize, mut f: impl FnMut(f64, f64) -> Point3D) -> Vec<Point3D> {
    let mut out = Vec::with_capacity(d * d);
    for j in 0..d {
        let b = (j as f64 + 0.5) / d as f64;
        for i in 0..d {
            out.push(f((i as f64 + 0.5) / d as f64, b));
        }
    }
    out
}
