use egui::{Color32, IdSalt};

use super::{Line, Scatter};
use crate::PlotData;

/// Explicit mixed-series API; the original Plot::show remains a line shortcut.
#[derive(Clone, Copy, Debug)]
pub enum Series<'a> {
    Line(Line<'a>),
    Scatter(Scatter<'a>),
}

impl<'a> From<Line<'a>> for Series<'a> {
    fn from(line: Line<'a>) -> Self {
        Self::Line(line)
    }
}
impl<'a> From<Scatter<'a>> for Series<'a> {
    fn from(points: Scatter<'a>) -> Self {
        Self::Scatter(points)
    }
}

impl<'a> Series<'a> {
    pub fn data(self) -> &'a PlotData {
        match self {
            Self::Line(s) => s.data,
            Self::Scatter(s) => s.data,
        }
    }
    pub fn name(self) -> &'a str {
        match self {
            Self::Line(s) => s.name,
            Self::Scatter(s) => s.name,
        }
    }
    pub(crate) fn id(self, index: usize) -> IdSalt {
        let id = match self {
            Self::Line(s) => s.id_salt,
            Self::Scatter(s) => s.id_salt,
        };
        id.unwrap_or_else(|| {
            if self.name().is_empty() {
                IdSalt::new(("series-index", index))
            } else {
                IdSalt::new(("series-name", self.name()))
            }
        })
    }
    pub(crate) fn color(self, index: usize) -> Color32 {
        let color = match self {
            Self::Line(s) => s.stroke.map(|s| s.color),
            Self::Scatter(s) => s.color,
        };
        color.unwrap_or(
            [
                Color32::from_rgb(70, 160, 235),
                Color32::from_rgb(235, 145, 65),
                Color32::from_rgb(90, 190, 130),
            ][index % 3],
        )
    }
}
