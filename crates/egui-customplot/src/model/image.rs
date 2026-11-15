use egui::{AsIdSalt, IdSalt};

use crate::{ImageData, YAxis};

/// Direct RGBA/indexed/grayscale image layer (`image` / `imshow`).
#[derive(Clone, Copy, Debug)]
pub struct PlotImage<'a> {
    pub(crate) data: &'a ImageData,
    pub(crate) opacity: f32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> PlotImage<'a> {
    pub fn new(data: &'a ImageData) -> Self {
        Self {
            data,
            opacity: 1.0,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a ImageData {
        self.data
    }
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
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
