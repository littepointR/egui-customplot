use egui::IdSalt;

use super::{ResolvedPlotStyle, ResolvedSeriesStyle, SeriesStyle};
use crate::{
    Area, Bars, Contours, ErrorBars, Fill, FilledContours, Heatmap, Line, Patch, Pcolor, PlotData,
    PlotImage, PolyShape, Quiver, Rectangles, Scatter, Series, Stems, YAxis,
};

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
    Area(Area<'a>),
    Fill(Fill<'a>),
    Patch(Patch<'a>),
    Rectangles(Rectangles<'a>),
    PolyShape(PolyShape<'a>),
    Contours(Contours<'a>),
    FilledContours(FilledContours<'a>),
    Pcolor(Pcolor<'a>),
    Image(PlotImage<'a>),
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

impl<'a> From<Area<'a>> for PlotLayer<'a> {
    fn from(value: Area<'a>) -> Self {
        Self::Area(value)
    }
}

impl<'a> From<Fill<'a>> for PlotLayer<'a> {
    fn from(value: Fill<'a>) -> Self {
        Self::Fill(value)
    }
}

impl<'a> From<Patch<'a>> for PlotLayer<'a> {
    fn from(value: Patch<'a>) -> Self {
        Self::Patch(value)
    }
}

impl<'a> From<Rectangles<'a>> for PlotLayer<'a> {
    fn from(value: Rectangles<'a>) -> Self {
        Self::Rectangles(value)
    }
}

impl<'a> From<PolyShape<'a>> for PlotLayer<'a> {
    fn from(value: PolyShape<'a>) -> Self {
        Self::PolyShape(value)
    }
}

impl<'a> From<Contours<'a>> for PlotLayer<'a> {
    fn from(value: Contours<'a>) -> Self {
        Self::Contours(value)
    }
}

impl<'a> From<FilledContours<'a>> for PlotLayer<'a> {
    fn from(value: FilledContours<'a>) -> Self {
        Self::FilledContours(value)
    }
}

impl<'a> From<Pcolor<'a>> for PlotLayer<'a> {
    fn from(value: Pcolor<'a>) -> Self {
        Self::Pcolor(value)
    }
}

impl<'a> From<PlotImage<'a>> for PlotLayer<'a> {
    fn from(value: PlotImage<'a>) -> Self {
        Self::Image(value)
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
            Self::Area(s) => s.name,
            Self::Fill(s) => s.name,
            Self::Patch(s) => s.name,
            Self::Rectangles(s) => s.name,
            Self::PolyShape(s) => s.name,
            Self::Contours(s) => s.name,
            Self::FilledContours(s) => s.name,
            Self::Pcolor(s) => s.name,
            Self::Image(s) => s.name,
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
            Self::Area(s) => s.y_axis,
            Self::Fill(s) => s.y_axis,
            Self::Patch(s) => s.y_axis,
            Self::Rectangles(s) => s.y_axis,
            Self::PolyShape(s) => s.y_axis,
            Self::Contours(s) => s.y_axis,
            Self::FilledContours(s) => s.y_axis,
            Self::Pcolor(s) => s.y_axis,
            Self::Image(s) => s.y_axis,
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
            Self::Area(s) => s.id_salt,
            Self::Fill(s) => s.id_salt,
            Self::Patch(s) => s.id_salt,
            Self::Rectangles(s) => s.id_salt,
            Self::PolyShape(s) => s.id_salt,
            Self::Contours(s) => s.id_salt,
            Self::FilledContours(s) => s.id_salt,
            Self::Pcolor(s) => s.id_salt,
            Self::Image(s) => s.id_salt,
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
            Self::Area(s) => plot.resolve_series(index, s.style),
            Self::Fill(s) => plot.resolve_series(index, s.style),
            Self::Patch(s) => {
                let mut style = s.style;
                if style.color.is_none() {
                    style.color = match s.coloring {
                        crate::PatchColoring::Uniform => None,
                        crate::PatchColoring::FaceColors(c)
                        | crate::PatchColoring::VertexColors(c) => c.first().copied(),
                        crate::PatchColoring::FaceScalars { scale, .. }
                        | crate::PatchColoring::VertexScalars { scale, .. } => {
                            Some(scale.map().sample(0.5))
                        }
                    };
                }
                plot.resolve_series(index, style)
            }
            Self::Rectangles(s) => plot.resolve_series(index, s.style),
            Self::PolyShape(s) => plot.resolve_series(index, s.style),
            Self::Contours(s) => {
                let mut style = s.style;
                if style.color.is_none()
                    && let Some(scale) = s.color_scale
                {
                    style.color = Some(scale.map().sample(0.5));
                }
                plot.resolve_series(index, style)
            }
            Self::FilledContours(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.colors.map().sample(0.5)),
                    ..Default::default()
                },
            ),
            Self::Pcolor(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.colors.map().sample(0.5)),
                    ..Default::default()
                },
            ),
            Self::Image(_) => plot.resolve_series(index, SeriesStyle::default()),
        }
    }

    pub(crate) fn point_data(self) -> Option<&'a PlotData> {
        match self {
            Self::Series(s) => Some(s.data()),
            Self::Stems(s) => Some(s.data),
            Self::Area(s) => match s.source {
                super::AreaSource::Curve { data, .. } => Some(data),
                super::AreaSource::Band(_) => None,
            },
            _ => None,
        }
    }
}
