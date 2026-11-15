use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use super::{Line, LineInterpolation, LinePattern, MarkerStyle, PlotLayer, SeriesStyle};
use crate::{AnimatedLine2D, CometPlayback, PlotData, YAxis};

/// Borrowed descriptor for incremental `animatedline` (`ML-078`).
#[derive(Clone, Copy, Debug)]
pub struct AnimatedLine<'a> {
    pub(crate) line: Line<'a>,
}

impl<'a> AnimatedLine<'a> {
    pub fn new(source: &'a AnimatedLine2D) -> Self {
        Self {
            line: Line::new(source.data()),
        }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.line = self.line.name(name);
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.line = self.line.id_salt(id);
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.line = self.line.y_axis(axis);
        self
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.line = self.line.stroke(stroke);
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.line = self.line.color(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.line = self.line.width(width);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.line = self.line.pattern(pattern);
        self
    }
    pub fn markers(mut self, markers: MarkerStyle) -> Self {
        self.line = self.line.markers(markers);
        self
    }
    pub fn interpolation(mut self, interpolation: LineInterpolation) -> Self {
        self.line = self.line.interpolation(interpolation);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.line = self.line.style(style);
        self
    }
}

impl<'a> From<AnimatedLine<'a>> for Line<'a> {
    fn from(value: AnimatedLine<'a>) -> Self {
        value.line
    }
}

impl<'a> From<AnimatedLine<'a>> for PlotLayer<'a> {
    fn from(value: AnimatedLine<'a>) -> Self {
        Self::Series(value.line.into())
    }
}

/// 2D animated comet trajectory (`comet` / `ML-079`).
#[derive(Clone, Copy, Debug)]
pub struct Comet<'a> {
    pub(crate) data: &'a PlotData,
    pub(crate) playback: &'a CometPlayback,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) head_color: Option<Color32>,
    pub(crate) head_radius: f32,
    pub(crate) tail_color: Option<Color32>,
    pub(crate) body_color: Option<Color32>,
    pub(crate) show_full_track: bool,
}

impl<'a> Comet<'a> {
    pub fn new(data: &'a PlotData, playback: &'a CometPlayback) -> Self {
        Self {
            data,
            playback,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            head_color: None,
            head_radius: 5.0,
            tail_color: None,
            body_color: None,
            show_full_track: true,
        }
    }

    pub fn data(self) -> &'a PlotData {
        self.data
    }
    pub fn playback(self) -> &'a CometPlayback {
        self.playback
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
    pub fn head_color(mut self, color: Color32) -> Self {
        self.head_color = Some(color);
        self
    }
    pub fn head_radius(mut self, radius: f32) -> Self {
        self.head_radius = radius;
        self
    }
    pub fn tail_color(mut self, color: Color32) -> Self {
        self.tail_color = Some(color);
        self
    }
    pub fn body_color(mut self, color: Color32) -> Self {
        self.body_color = Some(color);
        self
    }
    pub fn show_full_track(mut self, show: bool) -> Self {
        self.show_full_track = show;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}
