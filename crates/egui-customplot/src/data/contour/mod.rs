use std::sync::Arc;

use super::{FunctionContourError, ScalarField2D, polygon::domain_index};
use crate::{AxisScale, PlotBounds, PlotPoint};

mod extract;

#[derive(Clone, Debug, PartialEq)]
pub enum ContourLevels {
    Count(usize),
    Explicit(Vec<f64>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContourError {
    InvalidLevels,
    FunctionError(FunctionContourError),
}

impl std::fmt::Display for ContourError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLevels => f.write_str("contour levels must contain 1..=256 finite values"),
            Self::FunctionError(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ContourError {}

impl From<FunctionContourError> for ContourError {
    fn from(value: FunctionContourError) -> Self {
        Self::FunctionError(value)
    }
}

#[derive(Clone, Debug)]
pub struct ContourLine {
    pub level: f64,
    pub points: Box<[PlotPoint]>,
}

/// Immutable host-owned extracted isoline polylines from a 2D scalar field.
#[derive(Clone, Debug)]
pub struct ContourData {
    levels: Arc<[f64]>,
    lines: Arc<[ContourLine]>,
    extents: [Option<(PlotPoint, PlotPoint)>; 4],
    domain_invalid: [usize; 4],
    total_bytes: usize,
}

impl ContourData {
    pub fn from_field(field: &ScalarField2D, levels: ContourLevels) -> Result<Self, ContourError> {
        let levels = extract::resolve_line_levels(field, levels)?;
        let lines = extract::extract_lines(field, &levels);
        let mut extents = [None; 4];
        let mut domain_invalid = [0; 4];
        let mut total_bytes = levels.len() * 8;
        for line in &lines {
            total_bytes += line.points.len() * std::mem::size_of::<PlotPoint>();
            for domain in 0..4 {
                let scales = super::polygon::domain_scales(domain);
                let mut valid_any = false;
                for &p in &line.points {
                    if scales[0].forward(p[0]).is_some() && scales[1].forward(p[1]).is_some() {
                        merge_point(&mut extents[domain], p);
                        valid_any = true;
                    }
                }
                if !valid_any {
                    domain_invalid[domain] += 1;
                }
            }
        }
        Ok(Self {
            levels: Arc::from(levels),
            lines: Arc::from(lines),
            extents,
            domain_invalid,
            total_bytes,
        })
    }

    pub fn from_function(
        bounds: PlotBounds,
        resolution: [usize; 2],
        levels: ContourLevels,
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, ContourError> {
        let field = ScalarField2D::sample_function(bounds, resolution, max_evaluations, f)?;
        Self::from_field(&field, levels)
    }

    pub fn ezcontour(
        levels: ContourLevels,
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, ContourError> {
        let field = ScalarField2D::ez_sample([64, 64], max_evaluations, f)?;
        Self::from_field(&field, levels)
    }

    pub fn levels(&self) -> &[f64] {
        &self.levels
    }
    pub fn lines(&self) -> &[ContourLine] {
        &self.lines
    }
    pub fn sample_bytes(&self) -> usize {
        self.total_bytes
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.lines.as_ptr() as usize, self.total_bytes)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents[domain_index(scales)]
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.domain_invalid[domain_index(scales)]
    }
}

#[derive(Clone, Debug)]
pub struct ContourBand {
    pub lower: f64,
    pub upper: f64,
    pub vertices: Box<[PlotPoint]>,
    pub triangles: Box<[[u32; 3]]>,
}

/// Immutable host-owned triangulated filled contour bands (`contourf` / `ezcontourf`).
#[derive(Clone, Debug)]
pub struct FilledContourData {
    levels: Arc<[f64]>,
    bands: Arc<[ContourBand]>,
    extents: [Option<(PlotPoint, PlotPoint)>; 4],
    domain_invalid: [usize; 4],
    total_bytes: usize,
}

impl FilledContourData {
    pub fn from_field(field: &ScalarField2D, levels: ContourLevels) -> Result<Self, ContourError> {
        let (levels, band_bounds) = extract::resolve_band_levels(field, levels)?;
        let bands = extract::extract_bands(field, &band_bounds);
        let mut extents = [None; 4];
        let mut domain_invalid = [0; 4];
        let mut total_bytes = levels.len() * 8;
        for band in &bands {
            total_bytes += band.vertices.len() * std::mem::size_of::<PlotPoint>()
                + band.triangles.len() * std::mem::size_of::<[u32; 3]>();
            for domain in 0..4 {
                let scales = super::polygon::domain_scales(domain);
                let mut valid_any = false;
                for &p in &band.vertices {
                    if scales[0].forward(p[0]).is_some() && scales[1].forward(p[1]).is_some() {
                        merge_point(&mut extents[domain], p);
                        valid_any = true;
                    }
                }
                if !valid_any && !band.triangles.is_empty() {
                    domain_invalid[domain] += 1;
                }
            }
        }
        Ok(Self {
            levels: Arc::from(levels),
            bands: Arc::from(bands),
            extents,
            domain_invalid,
            total_bytes,
        })
    }

    pub fn from_function(
        bounds: PlotBounds,
        resolution: [usize; 2],
        levels: ContourLevels,
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, ContourError> {
        let field = ScalarField2D::sample_function(bounds, resolution, max_evaluations, f)?;
        Self::from_field(&field, levels)
    }

    pub fn ezcontourf(
        levels: ContourLevels,
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, ContourError> {
        let field = ScalarField2D::ez_sample([64, 64], max_evaluations, f)?;
        Self::from_field(&field, levels)
    }

    pub fn levels(&self) -> &[f64] {
        &self.levels
    }
    pub fn bands(&self) -> &[ContourBand] {
        &self.bands
    }
    pub fn sample_bytes(&self) -> usize {
        self.total_bytes
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.bands.as_ptr() as usize, self.total_bytes)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        self.extents[domain_index(scales)]
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.domain_invalid[domain_index(scales)]
    }
}

fn merge_point(extent: &mut Option<(PlotPoint, PlotPoint)>, p: PlotPoint) {
    let (lo, hi) = extent.get_or_insert((p, p));
    lo[0] = lo[0].min(p[0]);
    lo[1] = lo[1].min(p[1]);
    hi[0] = hi[0].max(p[0]);
    hi[1] = hi[1].max(p[1]);
}
