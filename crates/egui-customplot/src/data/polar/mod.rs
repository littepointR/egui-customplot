use std::{f64::consts::PI, sync::Arc};

use crate::{
    FunctionPlotError, FunctionPlotOptions, NegativeRadiusMode, layout::bubble_pixel_radius,
};

mod function;
mod histogram;

pub use function::PolarFunctionData;
pub use histogram::{PolarHistNormalization, PolarHistogramData, RoseData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolarError {
    EmptyInput,
    LengthMismatch,
    InvalidEdges,
    InvalidBins,
    InvalidRange,
}

impl std::fmt::Display for PolarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("polar input must not be empty"),
            Self::LengthMismatch => f.write_str("polar input slices must have equal length"),
            Self::InvalidEdges => {
                f.write_str("polar histogram bin edges must be finite and strictly increasing")
            }
            Self::InvalidBins => f.write_str("polar histogram bin count must be at least 1"),
            Self::InvalidRange => f.write_str("polar range must be finite with min <= max"),
        }
    }
}

impl std::error::Error for PolarError {}

/// Immutable host-owned `[theta_rad, r]` polar polyline/scatter samples (`ML-045` / `ML-047` / `LG-004`).
#[derive(Clone, Debug)]
pub struct PolarCurveData {
    points: Arc<[[f64; 2]]>,
    finite_points: usize,
    non_finite_points: usize,
    r_bounds_raw: Option<[f64; 2]>,
    r_bounds_reflected: Option<[f64; 2]>,
}

impl PolarCurveData {
    pub fn new(points: impl Into<Arc<[[f64; 2]]>>) -> Self {
        let points: Arc<[[f64; 2]]> = points.into();
        let mut finite = 0usize;
        let mut non_finite = 0usize;
        let mut raw = [f64::INFINITY, f64::NEG_INFINITY];
        let mut refl = [f64::INFINITY, f64::NEG_INFINITY];
        for &[theta, r] in points.iter() {
            if theta.is_finite() && r.is_finite() {
                finite += 1;
                raw[0] = raw[0].min(r);
                raw[1] = raw[1].max(r);
                let rr = r.abs();
                refl[0] = refl[0].min(rr);
                refl[1] = refl[1].max(rr);
            } else {
                non_finite += 1;
            }
        }
        Self {
            points,
            finite_points: finite,
            non_finite_points: non_finite,
            r_bounds_raw: (finite > 0).then_some(raw),
            r_bounds_reflected: (finite > 0).then_some(refl),
        }
    }

    pub fn from_theta_r(theta: &[f64], r: &[f64]) -> Result<Self, PolarError> {
        if theta.len() != r.len() {
            return Err(PolarError::LengthMismatch);
        }
        if theta.is_empty() {
            return Err(PolarError::EmptyInput);
        }
        let pts: Arc<[[f64; 2]]> = theta
            .iter()
            .zip(r.iter())
            .map(|(&t, &rv)| [t, rv])
            .collect();
        Ok(Self::new(pts))
    }

    pub fn from_degrees(theta_deg: &[f64], r: &[f64]) -> Result<Self, PolarError> {
        if theta_deg.len() != r.len() {
            return Err(PolarError::LengthMismatch);
        }
        if theta_deg.is_empty() {
            return Err(PolarError::EmptyInput);
        }
        let pts: Arc<[[f64; 2]]> = theta_deg
            .iter()
            .zip(r.iter())
            .map(|(&d, &rv)| [d.to_radians(), rv])
            .collect();
        Ok(Self::new(pts))
    }

    pub fn from_function(
        theta_range: [f64; 2],
        options: FunctionPlotOptions,
        mut f: impl FnMut(f64) -> f64,
    ) -> Result<PolarFunctionData, FunctionPlotError> {
        function::sample_polar(theta_range, options, &mut f)
    }

    pub fn ezpolar(
        max_evaluations: usize,
        f: impl FnMut(f64) -> f64,
    ) -> Result<PolarFunctionData, FunctionPlotError> {
        Self::from_function(
            [0.0, std::f64::consts::TAU],
            FunctionPlotOptions {
                max_evaluations,
                ..Default::default()
            },
            f,
        )
    }

    pub fn points(&self) -> &[[f64; 2]] {
        &self.points
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
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.points.as_ptr() as usize, self.sample_bytes())
    }

    pub(crate) fn r_extent(&self, neg_mode: NegativeRadiusMode) -> Option<[f64; 2]> {
        match neg_mode {
            NegativeRadiusMode::ShiftMinRadius => self.r_bounds_raw,
            NegativeRadiusMode::ReflectOrigin => self.r_bounds_reflected,
            NegativeRadiusMode::ClipBelowMin => self
                .r_bounds_raw
                .map(|[lo, hi]| [lo.max(0.0), hi.max(0.001)]),
        }
    }
}

/// One sample in [`PolarBubbleData`] (`ML-048`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolarBubblePoint {
    pub theta: f64,
    pub r: f64,
    pub size: f64,
    pub scalar: Option<f64>,
}

/// Immutable host-owned polar bubble data with area-proportional pixel radius mapping.
#[derive(Clone, Debug)]
pub struct PolarBubbleData {
    points: Arc<[PolarBubblePoint]>,
    size_extent: [f64; 2],
    radius_range: [f32; 2],
    r_bounds_raw: Option<[f64; 2]>,
    r_bounds_reflected: Option<[f64; 2]>,
    non_finite_points: usize,
}

impl PolarBubbleData {
    pub fn new(theta: &[f64], r: &[f64], sizes: &[f64]) -> Result<Self, PolarError> {
        if theta.len() != r.len() || theta.len() != sizes.len() {
            return Err(PolarError::LengthMismatch);
        }
        if theta.is_empty() {
            return Err(PolarError::EmptyInput);
        }
        let pts: Vec<PolarBubblePoint> = theta
            .iter()
            .zip(r.iter())
            .zip(sizes.iter())
            .map(|((&t, &rv), &s)| PolarBubblePoint {
                theta: t,
                r: rv,
                size: s,
                scalar: None,
            })
            .collect();
        Ok(Self::from_vec(pts, [4.0, 22.0]))
    }

    pub fn with_scalars(mut self, scalars: &[f64]) -> Result<Self, PolarError> {
        if scalars.len() != self.points.len() {
            return Err(PolarError::LengthMismatch);
        }
        let updated: Vec<PolarBubblePoint> = self
            .points
            .iter()
            .zip(scalars.iter())
            .map(|(p, &c)| PolarBubblePoint {
                scalar: c.is_finite().then_some(c),
                ..*p
            })
            .collect();
        self.points = Arc::from(updated);
        Ok(self)
    }

    pub fn size_range(mut self, radius_range: [f32; 2]) -> Self {
        let lo = radius_range[0].clamp(1.0, 64.0);
        let hi = radius_range[1].clamp(lo, 64.0);
        self.radius_range = [lo, hi];
        self
    }

    fn from_vec(pts: Vec<PolarBubblePoint>, radius_range: [f32; 2]) -> Self {
        let mut raw = [f64::INFINITY, f64::NEG_INFINITY];
        let mut refl = [f64::INFINITY, f64::NEG_INFINITY];
        let mut s_ext = [f64::INFINITY, f64::NEG_INFINITY];
        let mut non_finite = 0usize;
        let mut finite = 0usize;
        for p in &pts {
            if p.theta.is_finite() && p.r.is_finite() && p.size.is_finite() && p.size >= 0.0 {
                finite += 1;
                raw[0] = raw[0].min(p.r);
                raw[1] = raw[1].max(p.r);
                refl[0] = refl[0].min(p.r.abs());
                refl[1] = refl[1].max(p.r.abs());
                s_ext[0] = s_ext[0].min(p.size);
                s_ext[1] = s_ext[1].max(p.size);
            } else {
                non_finite += 1;
            }
        }
        Self {
            points: Arc::from(pts),
            size_extent: if s_ext[0] <= s_ext[1] {
                s_ext
            } else {
                [0.0, 1.0]
            },
            radius_range,
            r_bounds_raw: (finite > 0).then_some(raw),
            r_bounds_reflected: (finite > 0).then_some(refl),
            non_finite_points: non_finite,
        }
    }

    pub fn points(&self) -> &[PolarBubblePoint] {
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
    pub(crate) fn r_extent(&self, neg_mode: NegativeRadiusMode) -> Option<[f64; 2]> {
        match neg_mode {
            NegativeRadiusMode::ShiftMinRadius => self.r_bounds_raw,
            NegativeRadiusMode::ReflectOrigin => self.r_bounds_reflected,
            NegativeRadiusMode::ClipBelowMin => self
                .r_bounds_raw
                .map(|[lo, hi]| [lo.max(0.0), hi.max(0.001)]),
        }
    }
}

/// Immutable host-owned radial vector arrows from origin (`compassplot` / `compass`, `ML-049` / `LG-005`).
#[derive(Clone, Debug)]
pub struct CompassData {
    arrows: Arc<[[f64; 2]]>,
    max_rho: f64,
    non_finite: usize,
}

impl CompassData {
    pub fn from_polar(theta: &[f64], rho: &[f64]) -> Result<Self, PolarError> {
        if theta.len() != rho.len() {
            return Err(PolarError::LengthMismatch);
        }
        if theta.is_empty() {
            return Err(PolarError::EmptyInput);
        }
        let mut out = Vec::with_capacity(theta.len());
        let mut max_rho = 0.0_f64;
        let mut non_finite = 0usize;
        for (&t, &r) in theta.iter().zip(rho.iter()) {
            if t.is_finite() && r.is_finite() {
                let (ang, mag) = if r < 0.0 { (t + PI, -r) } else { (t, r) };
                max_rho = max_rho.max(mag);
                out.push([ang, mag]);
            } else {
                non_finite += 1;
                out.push([f64::NAN, f64::NAN]);
            }
        }
        Ok(Self {
            arrows: Arc::from(out),
            max_rho,
            non_finite,
        })
    }

    pub fn from_cartesian(u: &[f64], v: &[f64]) -> Result<Self, PolarError> {
        if u.len() != v.len() {
            return Err(PolarError::LengthMismatch);
        }
        if u.is_empty() {
            return Err(PolarError::EmptyInput);
        }
        let complex: Vec<[f64; 2]> = u.iter().zip(v.iter()).map(|(&x, &y)| [x, y]).collect();
        Ok(Self::from_complex(&complex))
    }

    pub fn from_complex(z: &[[f64; 2]]) -> Self {
        let mut out = Vec::with_capacity(z.len());
        let mut max_rho = 0.0_f64;
        let mut non_finite = 0usize;
        for &[re, im] in z {
            if re.is_finite() && im.is_finite() {
                let mag = re.hypot(im);
                let ang = if mag == 0.0 { 0.0 } else { im.atan2(re) };
                max_rho = max_rho.max(mag);
                out.push([ang, mag]);
            } else {
                non_finite += 1;
                out.push([f64::NAN, f64::NAN]);
            }
        }
        Self {
            arrows: Arc::from(out),
            max_rho,
            non_finite,
        }
    }

    pub fn arrows(&self) -> &[[f64; 2]] {
        &self.arrows
    }
    pub fn len(&self) -> usize {
        self.arrows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.arrows.is_empty()
    }
    pub fn max_rho(&self) -> f64 {
        self.max_rho
    }
    pub fn non_finite(&self) -> usize {
        self.non_finite
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.arrows.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.arrows.as_ptr() as usize, self.sample_bytes())
    }
}
