use egui::{AsIdSalt, Color32, IdSalt};

use crate::{MarkerShape, SeriesStyle, SpyData, YAxis};

/// Sparsity pattern layer (`spy`, `ML-020`).
#[derive(Clone, Copy, Debug)]
pub struct Spy<'a> {
    pub(crate) data: &'a SpyData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> Spy<'a> {
    pub fn new(data: &'a SpyData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Square),
                marker_radius: Some(2.5),
                ..Default::default()
            },
        }
    }

    pub fn data(self) -> &'a SpyData {
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
    pub fn marker_shape(mut self, shape: MarkerShape) -> Self {
        self.style.marker_shape = Some(shape);
        self
    }
    pub fn marker_size(mut self, radius: f32) -> Self {
        self.style.marker_radius = Some(radius);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}
