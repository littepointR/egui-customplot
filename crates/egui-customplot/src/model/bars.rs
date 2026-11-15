use crate::{BarData, BarOrientation, SeriesStyle, YAxis};
use egui::{AsIdSalt, Color32, IdSalt, Stroke};

#[derive(Clone, Copy, Debug)]
pub struct Bars<'a> {
    pub(crate) data: &'a BarData,
    pub(crate) orientation: BarOrientation,
    pub(crate) style: SeriesStyle,
    pub(crate) border: Stroke,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Bars<'a> {
    pub fn new(data: &'a BarData) -> Self {
        Self {
            data,
            orientation: BarOrientation::Vertical,
            style: SeriesStyle::default(),
            border: Stroke::NONE,
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
    pub fn orientation(mut self, orientation: BarOrientation) -> Self {
        self.orientation = orientation;
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
    /// Optional independent outline. Zero/transparent disables it; invalid widths
    /// reject the outline with a diagnostic, without hiding a valid fill.
    pub fn border(mut self, border: Stroke) -> Self {
        self.border = border;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a BarData {
        self.data
    }
}
