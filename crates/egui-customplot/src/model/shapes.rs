use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{RectData, SeriesStyle, YAxis};

/// Data-space rectangles, rounded rectangles and ellipses (`rectangle`).
#[derive(Clone, Copy, Debug)]
pub struct Rectangles<'a> {
    pub(crate) data: &'a RectData,
    pub(crate) border: Stroke,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Rectangles<'a> {
    pub fn new(data: &'a RectData) -> Self {
        Self {
            data,
            border: Stroke::NONE,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a RectData {
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
}
