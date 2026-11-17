use egui::{AsIdSalt, IdSalt, Stroke};

use crate::{ColorScale, CurvilinearField2D, PcolorShading, YAxis};

/// Curvilinear parametric 2D pseudocolor grid (`pcolor(X, Y, Z)`, `EXT-08`).
#[derive(Clone, Copy, Debug)]
pub struct CurvilinearPcolor<'a> {
    pub(crate) data: &'a CurvilinearField2D,
    pub(crate) colors: ColorScale,
    pub(crate) shading: PcolorShading,
    pub(crate) edge_stroke: Stroke,
    pub(crate) alpha: f32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> CurvilinearPcolor<'a> {
    pub fn new(data: &'a CurvilinearField2D, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            shading: PcolorShading::Flat,
            edge_stroke: Stroke::NONE,
            alpha: 1.0,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a CurvilinearField2D {
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
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
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
