use egui::IdSalt;

use super::{ResolvedPlotStyle, ResolvedSeriesStyle, SeriesStyle};
use crate::{Bars, ErrorBars, Heatmap, Line, PlotData, Quiver, Scatter, Series, Stems, YAxis};

/// Ordered heterogeneous layers. This extends the plotting API without changing
/// the existing point-series enum or Series::data() contract.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum PlotLayer<'a> {
    Series(Series<'a>),
    Heatmap(Heatmap<'a>),
    Bars(Bars<'a>),
    ErrorBars(ErrorBars<'a>),
    Stems(Stems<'a>),
    Quiver(Quiver<'a>),
}

impl<'a> From<Series<'a>> for PlotLayer<'a> {
    fn from(value: Series<'a>) -> Self {
        Self::Series(value)
    }
}
impl<'a> From<Line<'a>> for PlotLayer<'a> {
    fn from(value: Line<'a>) -> Self {
        Self::Series(value.into())
    }
}
impl<'a> From<Scatter<'a>> for PlotLayer<'a> {
    fn from(value: Scatter<'a>) -> Self {
        Self::Series(value.into())
    }
}
impl<'a> From<Heatmap<'a>> for PlotLayer<'a> {
    fn from(value: Heatmap<'a>) -> Self {
        Self::Heatmap(value)
    }
}
impl<'a> From<Bars<'a>> for PlotLayer<'a> {
    fn from(value: Bars<'a>) -> Self {
        Self::Bars(value)
    }
}
impl<'a> From<ErrorBars<'a>> for PlotLayer<'a> {
    fn from(value: ErrorBars<'a>) -> Self {
        Self::ErrorBars(value)
    }
}
impl<'a> From<Stems<'a>> for PlotLayer<'a> {
    fn from(value: Stems<'a>) -> Self {
        Self::Stems(value)
    }
}
impl<'a> From<Quiver<'a>> for PlotLayer<'a> {
    fn from(value: Quiver<'a>) -> Self {
        Self::Quiver(value)
    }
}

impl<'a> PlotLayer<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Self::Series(s) => s.name(),
            Self::Heatmap(s) => s.name,
            Self::Bars(s) => s.name,
            Self::ErrorBars(s) => s.name,
            Self::Stems(s) => s.name,
            Self::Quiver(s) => s.name,
        }
    }
    pub fn y_axis(self) -> YAxis {
        match self {
            Self::Series(s) => s.y_axis(),
            Self::Heatmap(s) => s.y_axis,
            Self::Bars(s) => s.y_axis,
            Self::ErrorBars(s) => s.y_axis,
            Self::Stems(s) => s.y_axis,
            Self::Quiver(s) => s.y_axis,
        }
    }
    pub(crate) fn id(self, index: usize) -> IdSalt {
        let id_salt = match self {
            Self::Series(s) => return s.id(index),
            Self::Heatmap(s) => s.id_salt,
            Self::Bars(s) => s.id_salt,
            Self::ErrorBars(s) => s.id_salt,
            Self::Stems(s) => s.id_salt,
            Self::Quiver(s) => s.id_salt,
        };
        id_salt.unwrap_or_else(|| {
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
        match self {
            Self::Series(s) => s.resolved_style(index, plot),
            Self::Heatmap(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.colors.map().sample(0.5)),
                    ..Default::default()
                },
            ),
            Self::Bars(s) => plot.resolve_series(index, s.style),
            Self::ErrorBars(s) => plot.resolve_series(index, s.style),
            Self::Stems(s) => plot.resolve_series(index, s.style),
            Self::Quiver(s) => plot.resolve_series(index, s.style),
        }
    }

    pub(crate) fn point_data(self) -> Option<&'a PlotData> {
        match self {
            Self::Series(s) => Some(s.data()),
            Self::Stems(s) => Some(s.data),
            _ => None,
        }
    }
}
