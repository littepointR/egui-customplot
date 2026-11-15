use egui::{AsIdSalt, IdSalt, Stroke};

use crate::{ColorScale, ScalarField2D, YAxis};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PcolorShading {
    #[default]
    Flat,
    Interp,
}

/// Vertex-centered pseudocolor grid (`pcolor`). `Flat` colors cell `(c, r)`
/// from node `(c, r)` so the last row/column bound the outer geometry.
#[derive(Clone, Copy, Debug)]
pub struct Pcolor<'a> {
    pub(crate) data: &'a ScalarField2D,
    pub(crate) colors: ColorScale,
    pub(crate) shading: PcolorShading,
    pub(crate) edge_stroke: Stroke,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Pcolor<'a> {
    pub fn new(data: &'a ScalarField2D, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            shading: PcolorShading::Flat,
            edge_stroke: Stroke::NONE,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a ScalarField2D {
        self.data
    }
    pub fn color_scale(self) -> ColorScale {
        self.colors
    }
    pub fn shading(mut self, shading: PcolorShading) -> Self {
        self.shading = shading;
        self
    }
    pub fn edge_stroke(mut self, stroke: Stroke) -> Self {
        self.edge_stroke = stroke;
        self
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
}
