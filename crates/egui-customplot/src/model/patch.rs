use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{ColorScale, PatchData, SeriesError, SeriesStyle, YAxis};

#[derive(Clone, Copy, Debug)]
pub enum PatchColoring<'a> {
    Uniform,
    FaceColors(&'a [Color32]),
    FaceScalars {
        values: &'a [f64],
        scale: ColorScale,
    },
    VertexColors(&'a [Color32]),
    VertexScalars {
        values: &'a [f64],
        scale: ColorScale,
    },
}

/// General 2D faces sharing vertices with uniform, per-face or Gouraud coloring (`patch`).
#[derive(Clone, Copy, Debug)]
pub struct Patch<'a> {
    pub(crate) data: &'a PatchData,
    pub(crate) coloring: PatchColoring<'a>,
    pub(crate) border: Stroke,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Patch<'a> {
    pub fn new(data: &'a PatchData) -> Self {
        Self {
            data,
            coloring: PatchColoring::Uniform,
            border: Stroke::NONE,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a PatchData {
        self.data
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn border(mut self, border: Stroke) -> Self {
        self.border = border;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn face_colors(mut self, colors: &'a [Color32]) -> Result<Self, SeriesError> {
        if colors.len() != self.data.face_count() {
            return Err(SeriesError::LengthMismatch);
        }
        self.coloring = PatchColoring::FaceColors(colors);
        Ok(self)
    }
    pub fn face_scalars(
        mut self,
        values: &'a [f64],
        scale: ColorScale,
    ) -> Result<Self, SeriesError> {
        if values.len() != self.data.face_count() {
            return Err(SeriesError::LengthMismatch);
        }
        self.coloring = PatchColoring::FaceScalars { values, scale };
        Ok(self)
    }
    pub fn vertex_colors(mut self, colors: &'a [Color32]) -> Result<Self, SeriesError> {
        if colors.len() != self.data.vertices().len() {
            return Err(SeriesError::LengthMismatch);
        }
        self.coloring = PatchColoring::VertexColors(colors);
        Ok(self)
    }
    pub fn vertex_scalars(
        mut self,
        values: &'a [f64],
        scale: ColorScale,
    ) -> Result<Self, SeriesError> {
        if values.len() != self.data.vertices().len() {
            return Err(SeriesError::LengthMismatch);
        }
        self.coloring = PatchColoring::VertexScalars { values, scale };
        Ok(self)
    }
}
