use egui::IdSalt;

use crate::{
    AlphaShape, Area, Bars, Comet, Contours, ErrorBars, Feather, Fill, FilledContours, Heatmap,
    Line, Patch, Pcolor, PlotData, PlotImage, PolyShape, Quiver, Rectangles, Scatter, Series, Spy,
    Stems, Streamlines, TriPlot, Voronoi, YAxis,
};

mod style;

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
    TriPlot(TriPlot<'a>),
    Voronoi(Voronoi<'a>),
    AlphaShape(AlphaShape<'a>),
    Streamlines(Streamlines<'a>),
    Feather(Feather<'a>),
    Spy(Spy<'a>),
    Comet(Comet<'a>),
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
impl<'a> From<TriPlot<'a>> for PlotLayer<'a> {
    fn from(value: TriPlot<'a>) -> Self {
        Self::TriPlot(value)
    }
}
impl<'a> From<Voronoi<'a>> for PlotLayer<'a> {
    fn from(value: Voronoi<'a>) -> Self {
        Self::Voronoi(value)
    }
}
impl<'a> From<AlphaShape<'a>> for PlotLayer<'a> {
    fn from(value: AlphaShape<'a>) -> Self {
        Self::AlphaShape(value)
    }
}
impl<'a> From<Streamlines<'a>> for PlotLayer<'a> {
    fn from(value: Streamlines<'a>) -> Self {
        Self::Streamlines(value)
    }
}
impl<'a> From<Feather<'a>> for PlotLayer<'a> {
    fn from(value: Feather<'a>) -> Self {
        Self::Feather(value)
    }
}
impl<'a> From<Spy<'a>> for PlotLayer<'a> {
    fn from(value: Spy<'a>) -> Self {
        Self::Spy(value)
    }
}
impl<'a> From<Comet<'a>> for PlotLayer<'a> {
    fn from(value: Comet<'a>) -> Self {
        Self::Comet(value)
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
            Self::TriPlot(s) => s.name,
            Self::Voronoi(s) => s.name,
            Self::AlphaShape(s) => s.name,
            Self::Streamlines(s) => s.name,
            Self::Feather(s) => s.name,
            Self::Spy(s) => s.name,
            Self::Comet(s) => s.name,
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
            Self::TriPlot(s) => s.y_axis,
            Self::Voronoi(s) => s.y_axis,
            Self::AlphaShape(s) => s.y_axis,
            Self::Streamlines(s) => s.y_axis,
            Self::Feather(s) => s.y_axis,
            Self::Spy(s) => s.y_axis,
            Self::Comet(s) => s.y_axis,
        }
    }

    pub(crate) fn id(self, index: usize) -> IdSalt {
        style::layer_id(self, index)
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
