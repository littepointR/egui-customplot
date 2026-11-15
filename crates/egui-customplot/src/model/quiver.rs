use egui::{AsIdSalt, Color32, IdSalt};

use crate::{LinePattern, QuiverData, SeriesStyle, YAxis};

/// Independent origin + displacement arrows; shafts join transformed endpoints
/// with straight screen segments (also on log axes). No automatic vector scale,
/// interpolation, LOD or cache. Point marker fields in SeriesStyle do not apply.
#[derive(Clone, Copy, Debug)]
pub struct Quiver<'a> {
    pub(crate) data: &'a QuiverData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) show_heads: bool,
    pub(crate) head_size: [f32; 2],
}

impl<'a> Quiver<'a> {
    pub fn new(data: &'a QuiverData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            show_heads: true,
            head_size: [10.0, 8.0],
        }
    }
    pub fn data(self) -> &'a QuiverData {
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
    /// Dashes affect shafts only. Open V-shaped heads stay solid.
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn show_heads(mut self, show: bool) -> Self {
        self.show_heads = show;
        self
    }
    /// Length and full width in egui logical points, not data units or physical
    /// pixels. Shrinks proportionally to at most 40% of the full projected shaft
    /// length, not the clipped length. Nonpositive/nonfinite sizes omit heads
    /// (nonfinite/negative sizes also diagnose); shafts remain unchanged.
    pub fn head_size(mut self, length: f32, width: f32) -> Self {
        self.head_size = [length, width];
        self
    }
}
