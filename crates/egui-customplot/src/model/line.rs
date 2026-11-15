use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use super::{LinePattern, MarkerStyle, SeriesStyle};
use crate::PlotData;

/// A borrowed series description; samples remain in the application model.
#[derive(Clone, Copy, Debug)]
pub struct Line<'a> {
    pub(crate) data: &'a PlotData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) style: SeriesStyle,
}

impl<'a> Line<'a> {
    pub fn new(data: &'a PlotData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
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
    pub fn markers(mut self, markers: MarkerStyle) -> Self {
        self.style.line_markers = Some(true);
        self.style.marker_shape = Some(markers.shape);
        self.style.marker_radius = Some(markers.radius);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        let stroke = stroke.into();
        self.style.color = Some(stroke.color);
        self.style.line_width = Some(stroke.width);
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn show_markers(mut self, show: bool) -> Self {
        self.style.line_markers = Some(show);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}
