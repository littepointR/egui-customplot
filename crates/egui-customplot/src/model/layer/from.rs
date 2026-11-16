use super::PlotLayer;
use crate::{
    AlphaShape, Area, Bars, Comet, Contours, ErrorBars, Feather, Fill, FilledContours, Heatmap,
    Line, Patch, Pcolor, PlotImage, PolyShape, Quiver, Rectangles, Scatter, Series, Spy, Stems,
    StreamLine, Streamlines, TriPlot, Voronoi, WaveformLine,
};

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
impl<'a> From<StreamLine<'a>> for PlotLayer<'a> {
    fn from(value: StreamLine<'a>) -> Self {
        Self::StreamLine(value)
    }
}
impl<'a> From<WaveformLine<'a>> for PlotLayer<'a> {
    fn from(value: WaveformLine<'a>) -> Self {
        Self::WaveformLine(value)
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
