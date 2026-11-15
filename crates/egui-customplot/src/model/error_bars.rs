use crate::{ErrorBarData, SeriesStyle, YAxis};
use egui::{AsIdSalt, Color32, IdSalt};

/// Independent horizontal/vertical whiskers, without connecting center lines.
/// Caps are logical points; data error magnitudes remain in data coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ErrorBars<'a> {
    pub(crate) data: &'a ErrorBarData,
    pub(crate) cap_width: f32,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> ErrorBars<'a> {
    pub fn new(data: &'a ErrorBarData) -> Self {
        Self {
            data,
            cap_width: 8.0,
            style: SeriesStyle::default(),
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
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn cap_width(mut self, width: f32) -> Self {
        self.cap_width = width;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a ErrorBarData {
        self.data
    }
}
