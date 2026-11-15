use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{FeatherData, LinePattern, SeriesStyle, StreamlineData, YAxis};

/// 2D streamline / streamslice layer (`streamline` / `streamslice`, `ML-071` / `ML-072`).
#[derive(Clone, Copy, Debug)]
pub struct Streamlines<'a> {
    pub(crate) data: &'a StreamlineData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) show_arrows: bool,
    pub(crate) arrow_size: [f32; 2],
}

impl<'a> Streamlines<'a> {
    pub fn new(data: &'a StreamlineData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            show_arrows: true,
            arrow_size: [8.0, 6.0],
        }
    }

    pub fn data(self) -> &'a StreamlineData {
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
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn show_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
        self
    }
    pub fn arrow_size(mut self, length: f32, width: f32) -> Self {
        self.arrow_size = [length, width];
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}

/// 2D feather plot along a baseline (`feather`, `ML-058`).
#[derive(Clone, Copy, Debug)]
pub struct Feather<'a> {
    pub(crate) data: &'a FeatherData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) baseline_stroke: Option<Stroke>,
    pub(crate) show_heads: bool,
    pub(crate) head_size: [f32; 2],
}

impl<'a> Feather<'a> {
    pub fn new(data: &'a FeatherData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            baseline_stroke: None,
            show_heads: true,
            head_size: [9.0, 7.0],
        }
    }

    pub fn data(self) -> &'a FeatherData {
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
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn baseline_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.baseline_stroke = Some(stroke.into());
        self
    }
    pub fn show_heads(mut self, show: bool) -> Self {
        self.show_heads = show;
        self
    }
    pub fn head_size(mut self, length: f32, width: f32) -> Self {
        self.head_size = [length, width];
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}
