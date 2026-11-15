use egui::IdSalt;

use super::{Line, ResolvedPlotStyle, ResolvedSeriesStyle, Scatter};
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
    pub(crate) fn resolved_style(
        self,
        index: usize,
        plot: &ResolvedPlotStyle<'_>,
    ) -> ResolvedSeriesStyle {
        plot.resolve_series(
            index,
            match self {
                Self::Line(s) => s.style,
                Self::Scatter(s) => s.style,
            },
        )
    }
}
