use std::{f64::consts::TAU, sync::Arc};

use crate::{AxisScale, Point3D};

use super::{CachedExtents3D, Plot3DError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Bar3DMode {
    #[default]
    Detached,
    Grouped,
    Stacked,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Bar3DOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// One extruded rectangular 3D bar prism in `[x0, x1] x [y0, y1] x [z0, z1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar3DBox {
    pub row: usize,
    pub col: usize,
    pub value: f64,
    pub min: Point3D,
    pub max: Point3D,
}

/// Immutable host-owned 3D bar prisms (`bar3` / `bar3h`, `ML-036` / `ML-037`).
#[derive(Clone, Debug)]
pub struct Bar3DData {
    boxes: Arc<[Bar3DBox]>,
    cols: usize,
    mode: Bar3DMode,
    orientation: Bar3DOrientation,
    extents: CachedExtents3D,
    non_finite_items: usize,
}

impl Bar3DData {
    pub fn from_matrix(
        values_row_major: &[f64],
        rows: usize,
        cols: usize,
        mode: Bar3DMode,
        orientation: Bar3DOrientation,
        width_fraction: f64,
    ) -> Result<Self, Plot3DError> {
        if rows == 0 || cols == 0 || values_row_major.len() != rows * cols {
            return Err(Plot3DError::InvalidDimensions);
        }
        let w = if width_fraction.is_finite() && width_fraction > 0.0 {
            width_fraction.clamp(0.1, 0.98)
        } else {
            0.75
        };
        let mut boxes = Vec::with_capacity(rows * cols);
        let mut extents = CachedExtents3D::default();
        let mut non_finite = 0usize;

        let mut pos_acc = vec![0.0_f64; rows];
        let mut neg_acc = vec![0.0_f64; rows];

        for r in 0..rows {
            let y_center = r as f64 + 1.0;
            for c in 0..cols {
                let val = values_row_major[r * cols + c];
                if !val.is_finite() {
                    non_finite += 1;
                    continue;
                }
                let (x0, x1, y0, y1) = match mode {
                    Bar3DMode::Detached => {
                        let hw = 0.5 * w;
                        let cx = c as f64 + 1.0;
                        (cx - hw, cx + hw, y_center - hw, y_center + hw)
                    }
                    Bar3DMode::Grouped => {
                        let sub = w / cols as f64;
                        let x_start = (c as f64 - 0.5 * (cols as f64 - 1.0)) * sub + 1.0;
                        (
                            x_start - 0.45 * sub,
                            x_start + 0.45 * sub,
                            y_center - 0.4 * w,
                            y_center + 0.4 * w,
                        )
                    }
                    Bar3DMode::Stacked => (
                        1.0 - 0.45 * w,
                        1.0 + 0.45 * w,
                        y_center - 0.45 * w,
                        y_center + 0.45 * w,
                    ),
                };
                let (z0, z1) = match mode {
                    Bar3DMode::Stacked => {
                        if val >= 0.0 {
                            let b = pos_acc[r];
                            pos_acc[r] += val;
                            (b, pos_acc[r])
                        } else {
                            let t = neg_acc[r];
                            neg_acc[r] += val;
                            (neg_acc[r], t)
                        }
                    }
                    _ => (val.min(0.0), val.max(0.0)),
                };
                let (min, max) = match orientation {
                    Bar3DOrientation::Vertical => ([x0, y0, z0], [x1, y1, z1]),
                    Bar3DOrientation::Horizontal => ([z0, y0, x0], [z1, y1, x1]),
                };
                extents.push_box(min, max);
                boxes.push(Bar3DBox {
                    row: r,
                    col: c,
                    value: val,
                    min,
                    max,
                });
            }
        }
        Ok(Self {
            boxes: Arc::from(boxes),
            cols,
            mode,
            orientation,
            extents,
            non_finite_items: non_finite,
        })
    }

    /// Builds 3D histogram bars from binned bivariate XY counts (`histogram2` 3D mode, `ML-022`).
    pub fn from_histogram2(
        hist: &crate::Histogram2Data,
        norm: crate::BivariateNormalization,
    ) -> Result<Self, Plot3DError> {
        let grid = hist
            .to_grid(norm)
            .map_err(|_| Plot3DError::InvalidDimensions)?;
        let nx = hist.nx();
        let ny = hist.ny();
        let xe = grid.x_edges();
        let ye = grid.y_edges();
        let vals = grid.values();
        let mut boxes = Vec::with_capacity(nx * ny);
        let mut extents = CachedExtents3D::default();
        let mut non_finite = 0usize;
        for iy in 0..ny {
            for ix in 0..nx {
                let val = vals[iy * nx + ix];
                if !val.is_finite() {
                    non_finite += 1;
                    continue;
                }
                let min = [xe[ix], ye[iy], val.min(0.0)];
                let max = [xe[ix + 1], ye[iy + 1], val.max(0.0)];
                extents.push_box(min, max);
                boxes.push(Bar3DBox {
                    row: iy,
                    col: ix,
                    value: val,
                    min,
                    max,
                });
            }
        }
        Ok(Self {
            boxes: Arc::from(boxes),
            cols: nx,
            mode: Bar3DMode::Detached,
            orientation: Bar3DOrientation::Vertical,
            extents,
            non_finite_items: non_finite,
        })
    }

    pub fn boxes(&self) -> &[Bar3DBox] {
        &self.boxes
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn mode(&self) -> Bar3DMode {
        self.mode
    }
    pub fn orientation(&self) -> Bar3DOrientation {
        self.orientation
    }
    pub fn non_finite_items(&self) -> usize {
        self.non_finite_items
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.boxes.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.boxes.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

/// Immutable 3D discrete stems rooted at `z = base_z` (`stem3`, `ML-040`).
#[derive(Clone, Debug)]
pub struct Stem3DData {
    points: Arc<[Point3D]>,
    base_z: f64,
    extents: CachedExtents3D,
    non_finite_points: usize,
}

impl Stem3DData {
    pub fn new(points: impl Into<Arc<[Point3D]>>, base_z: f64) -> Result<Self, Plot3DError> {
        if !base_z.is_finite() {
            return Err(Plot3DError::InvalidDimensions);
        }
        let points: Arc<[Point3D]> = points.into();
        let mut extents = CachedExtents3D::default();
        let mut non_finite = 0usize;
        for &p in points.iter() {
            if p.iter().all(|v| v.is_finite()) {
                extents.push_point(p);
                extents.push_point([p[0], p[1], base_z]);
            } else {
                non_finite += 1;
            }
        }
        Ok(Self {
            points,
            base_z,
            extents,
            non_finite_points: non_finite,
        })
    }

    pub fn points(&self) -> &[Point3D] {
        &self.points
    }
    pub fn base_z(&self) -> f64 {
        self.base_z
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

/// One wedge slice inside [`Pie3DData`] (`LG-002` `pie3`).
#[derive(Clone, Debug)]
pub struct Pie3DSlice {
    pub index: usize,
    pub value: f64,
    pub fraction: f64,
    pub start_angle_rad: f64,
    pub end_angle_rad: f64,
    pub exploded: bool,
    pub label: String,
}

/// True extruded 3D cylinder-sector pie dataset (`pie3`, `LG-002`).
#[derive(Clone, Debug)]
pub struct Pie3DData {
    slices: Arc<[Pie3DSlice]>,
    thickness: f64,
    explode_offset: f64,
}

impl Pie3DData {
    pub fn new(values: &[f64]) -> Result<Self, Plot3DError> {
        Self::with_options::<&str>(values, &[], &[], 0.32)
    }

    pub fn with_options<S: Into<String> + Clone>(
        values: &[f64],
        explode: &[bool],
        labels: &[S],
        thickness: f64,
    ) -> Result<Self, Plot3DError> {
        if values.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        if !explode.is_empty() && explode.len() != values.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        if !labels.is_empty() && labels.len() != values.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        if !thickness.is_finite() || thickness <= 0.0 {
            return Err(Plot3DError::InvalidThickness);
        }
        let total: f64 = values
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .sum();
        if total <= 0.0 || !total.is_finite() {
            return Err(Plot3DError::NonPositiveTotal);
        }
        let mut slices = Vec::with_capacity(values.len());
        let mut cursor = 0.0_f64;
        for (i, &val) in values.iter().enumerate() {
            let clean = if val.is_finite() && val > 0.0 {
                val
            } else {
                0.0
            };
            let frac = clean / total;
            let span = frac * TAU;
            let start = cursor;
            let end = cursor + span;
            cursor = end;
            if frac > 0.0 {
                let label = labels
                    .get(i)
                    .cloned()
                    .map_or_else(|| format!("{:.1}%", frac * 100.0), Into::into);
                slices.push(Pie3DSlice {
                    index: i,
                    value: val,
                    fraction: frac,
                    start_angle_rad: start,
                    end_angle_rad: end,
                    exploded: explode.get(i).copied().unwrap_or(false),
                    label,
                });
            }
        }
        Ok(Self {
            slices: Arc::from(slices),
            thickness: thickness.clamp(0.05, 1.5),
            explode_offset: 0.16,
        })
    }

    pub fn explode_offset(mut self, offset: f64) -> Self {
        if offset.is_finite() && offset >= 0.0 {
            self.explode_offset = offset.clamp(0.0, 0.6);
        }
        self
    }

    pub fn slices(&self) -> &[Pie3DSlice] {
        &self.slices
    }
    pub fn thickness(&self) -> f64 {
        self.thickness
    }
    pub fn offset(&self) -> f64 {
        self.explode_offset
    }
    pub fn sample_bytes(&self) -> usize {
        std::mem::size_of_val(self.slices.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.slices.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self) -> [Point3D; 2] {
        let r = 1.0 + self.explode_offset + 0.05;
        [[-r, -r, 0.0], [r, r, self.thickness]]
    }
}
