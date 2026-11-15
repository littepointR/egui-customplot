use egui::{AsIdSalt, IdSalt};

use crate::{ColorScale, GridData, YAxis};

/// Nearest-cell (piecewise constant) colors; no interpolation or silent LOD.
#[derive(Clone, Copy, Debug)]
pub struct Heatmap<'a> {
    pub(crate) data: &'a GridData,
    pub(crate) colors: ColorScale,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Heatmap<'a> {
    pub fn new(data: &'a GridData, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
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
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn data(self) -> &'a GridData {
        self.data
    }
    pub fn color_scale(self) -> ColorScale {
        self.colors
    }
}
