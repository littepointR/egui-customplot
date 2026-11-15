use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use super::{LinePattern, MarkerStyle};
use crate::PlotData;

/// A borrowed series description; samples remain in the application model.
#[derive(Clone, Copy, Debug)]
pub struct Line<'a> {
    pub(crate) data: &'a PlotData,
    pub(crate) stroke: Option<Stroke>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) markers: Option<MarkerStyle>,
    pub(crate) pattern: LinePattern,
}

impl<'a> Line<'a> {
    pub fn new(data: &'a PlotData) -> Self {
        Self {
            data,
            stroke: None,
            name: "",
            id_salt: None,
            markers: None,
            pattern: LinePattern::Solid,
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
        self.markers = Some(markers);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.pattern = pattern;
        self
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }
    pub fn color(self, color: Color32) -> Self {
        self.stroke(Stroke::new(self.stroke.map_or(1.5, |s| s.width), color))
    }
    pub fn width(self, width: f32) -> Self {
        self.stroke(Stroke::new(
            width,
            self.stroke.map_or(Color32::LIGHT_BLUE, |s| s.color),
        ))
    }
}
