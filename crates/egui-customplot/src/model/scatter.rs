use egui::{AsIdSalt, Color32, IdSalt};

use super::{MarkerShape, MarkerStyle};
use crate::PlotData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesError {
    LengthMismatch,
    InvalidRadius,
}

impl std::fmt::Display for SeriesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LengthMismatch => "per-point style length must match sample length",
            Self::InvalidRadius => "marker radius must be finite and nonnegative",
        })
    }
}
impl std::error::Error for SeriesError {}

/// Batched markers. Per-point style slices are borrowed, never cloned by show.
#[derive(Clone, Copy, Debug)]
pub struct Scatter<'a> {
    pub(crate) data: &'a PlotData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) color: Option<Color32>,
    pub(crate) style: MarkerStyle,
    pub(crate) colors: Option<&'a [Color32]>,
    pub(crate) radii: Option<&'a [f32]>,
}

impl<'a> Scatter<'a> {
    pub fn new(data: &'a PlotData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color: None,
            style: MarkerStyle::default(),
            colors: None,
            radii: None,
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
    /// Invalid scalar radii are rejected by rendering/picking and counted in diagnostics.
    pub fn radius(mut self, radius: f32) -> Self {
        self.style.radius = radius;
        self
    }
    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.style.shape = shape;
        self
    }
    pub fn colors(mut self, colors: &'a [Color32]) -> Result<Self, SeriesError> {
        if colors.len() != self.data.len() {
            return Err(SeriesError::LengthMismatch);
        }
        self.colors = Some(colors);
        Ok(self)
    }
    pub fn radii(mut self, radii: &'a [f32]) -> Result<Self, SeriesError> {
        if radii.len() != self.data.len() {
            return Err(SeriesError::LengthMismatch);
        }
        if radii.iter().any(|r| !r.is_finite() || *r < 0.0) {
            return Err(SeriesError::InvalidRadius);
        }
        self.radii = Some(radii);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_point_style_validation_rejects_mismatches_and_invalid_radii() {
        let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
        assert_eq!(
            Scatter::new(&data).colors(&[]).unwrap_err(),
            SeriesError::LengthMismatch
        );
        assert_eq!(
            Scatter::new(&data).radii(&[1.0]).unwrap_err(),
            SeriesError::LengthMismatch
        );
        for radius in [f32::NAN, f32::INFINITY, -1.0] {
            assert_eq!(
                Scatter::new(&data).radii(&[radius, 1.0]).unwrap_err(),
                SeriesError::InvalidRadius
            );
        }
        assert!(Scatter::new(&data).radii(&[0.0, 2.0]).is_ok());
    }
}
