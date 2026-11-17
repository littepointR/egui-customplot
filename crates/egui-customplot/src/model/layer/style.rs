use egui::IdSalt;

use super::PlotLayer;
use crate::{
    SeriesStyle,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

pub(super) fn layer_id(layer: PlotLayer<'_>, index: usize) -> IdSalt {
    let id_salt = match layer {
        PlotLayer::Series(s) => return s.id(index),
        PlotLayer::StreamLine(s) => s.id_salt,
        PlotLayer::WaveformLine(s) => s.id_salt,
        PlotLayer::Heatmap(s) => s.id_salt,
        PlotLayer::Bars(s) => s.id_salt,
        PlotLayer::ErrorBars(s) => s.id_salt,
        PlotLayer::Stems(s) => s.id_salt,
        PlotLayer::Quiver(s) => s.id_salt,
        PlotLayer::Area(s) => s.id_salt,
        PlotLayer::Fill(s) => s.id_salt,
        PlotLayer::Patch(s) => s.id_salt,
        PlotLayer::Rectangles(s) => s.id_salt,
        PlotLayer::PolyShape(s) => s.id_salt,
        PlotLayer::Contours(s) => s.id_salt,
        PlotLayer::FilledContours(s) => s.id_salt,
        PlotLayer::Pcolor(s) => s.id_salt,
        PlotLayer::Image(s) => s.id_salt,
        PlotLayer::TriPlot(s) => s.id_salt,
        PlotLayer::Voronoi(s) => s.id_salt,
        PlotLayer::AlphaShape(s) => s.id_salt,
        PlotLayer::Streamlines(s) => s.id_salt,
        PlotLayer::Feather(s) => s.id_salt,
        PlotLayer::Spy(s) => s.id_salt,
        PlotLayer::Comet(s) => s.id_salt,
        PlotLayer::BubbleChart(s) => s.id_salt,
        PlotLayer::SwarmChart(s) => s.id_salt,
        PlotLayer::Histogram2(s) => s.id_salt,
        PlotLayer::BinScatter(s) => s.id_salt,
        PlotLayer::BoxChart(s) => s.id_salt,
        PlotLayer::ViolinPlot(s) => s.id_salt,
        PlotLayer::Raincloud(s) => s.id_salt,
        PlotLayer::PieChart(s) => s.id_salt,
        PlotLayer::WordCloud(s) => s.id_salt,
        PlotLayer::BubbleCloud(s) => s.id_salt,
        PlotLayer::CategoricalHeatmap(s) => s.id_salt,
        PlotLayer::ParallelPlot(s) => s.id_salt,
        PlotLayer::Graph(s) => s.id_salt,
        PlotLayer::ConfusionChart(s) => s.id_salt,
        PlotLayer::CurvilinearPcolor(s) => s.id_salt,
        PlotLayer::ProbabilityPlot(s) => s.id_salt,
        PlotLayer::EcdfPlot(s) => s.id_salt,
        PlotLayer::DendrogramPlot(s) => s.id_salt,
        PlotLayer::Biplot(s) => s.id_salt,
        PlotLayer::ControlChart(s) => s.id_salt,
        PlotLayer::Candlestick(s) => s.id_salt,
        PlotLayer::OhlcBars(s) => s.id_salt,
        PlotLayer::Waterfall2D(s) => s.id_salt,
        PlotLayer::ZPlanePlot(s) => s.id_salt,
        PlotLayer::NyquistPlot(s) => s.id_salt,
        PlotLayer::NicholsPlot(s) => s.id_salt,
        PlotLayer::RootLocus(s) => s.id_salt,
        PlotLayer::CwtPlot(s) => s.id_salt,
        PlotLayer::RadarChart(s) => s.id_salt,
        PlotLayer::SankeyChart(s) => s.id_salt,
        PlotLayer::TreemapChart(s) => s.id_salt,
        PlotLayer::SunburstChart(s) => s.id_salt,
        PlotLayer::GanttChart(s) => s.id_salt,
    };
    id_salt.unwrap_or_else(|| {
        if layer.name().is_empty() {
            IdSalt::new(("series-index", index))
        } else {
            IdSalt::new(("series-name", layer.name()))
        }
    })
}

impl<'a> PlotLayer<'a> {
    pub(crate) fn resolved_style(
        self,
        index: usize,
        plot: &ResolvedPlotStyle<'_>,
    ) -> ResolvedSeriesStyle {
        match self {
            Self::Series(s) => s.resolved_style(index, plot),
            Self::StreamLine(s) => plot.resolve_series(index, s.style),
            Self::WaveformLine(s) => plot.resolve_series(index, s.style),
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
            Self::TriPlot(s) => {
                let mut style = s.style;
                if style.color.is_none()
                    && let Some((_, scale)) = s.vertex_scalars
                {
                    style.color = Some(scale.map().sample(0.5));
                }
                plot.resolve_series(index, style)
            }
            Self::Voronoi(s) => plot.resolve_series(index, s.style),
            Self::AlphaShape(s) => plot.resolve_series(index, s.style),
            Self::Streamlines(s) => plot.resolve_series(index, s.style),
            Self::Feather(s) => plot.resolve_series(index, s.style),
            Self::Spy(s) => plot.resolve_series(index, s.style),
            Self::Comet(s) => plot.resolve_series(index, s.style),
            Self::BubbleChart(s) => plot.resolve_series(index, s.style),
            Self::SwarmChart(s) => plot.resolve_series(index, s.style),
            Self::Histogram2(_)
            | Self::BinScatter(_)
            | Self::CategoricalHeatmap(_)
            | Self::ConfusionChart(_)
            | Self::CurvilinearPcolor(_) => plot.resolve_series(index, SeriesStyle::default()),
            Self::BoxChart(s) => plot.resolve_series(index, s.style),
            Self::ViolinPlot(s) => plot.resolve_series(index, s.style),
            Self::Raincloud(s) => plot.resolve_series(index, s.style),
            Self::PieChart(s) => plot.resolve_series(index, s.style),
            Self::WordCloud(s) => plot.resolve_series(index, s.style),
            Self::BubbleCloud(s) => plot.resolve_series(index, s.style),
            Self::ParallelPlot(s) => plot.resolve_series(index, s.style),
            Self::Graph(s) => plot.resolve_series(index, s.style),
            Self::ProbabilityPlot(s) => plot.resolve_series(index, s.style),
            Self::EcdfPlot(s) => plot.resolve_series(index, s.style),
            Self::DendrogramPlot(s) => plot.resolve_series(index, s.style),
            Self::Biplot(s) => plot.resolve_series(index, s.style),
            Self::ControlChart(s) => plot.resolve_series(index, s.style),
            Self::Candlestick(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.up_color),
                    ..Default::default()
                },
            ),
            Self::OhlcBars(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.up_color),
                    ..Default::default()
                },
            ),
            Self::Waterfall2D(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.positive_color),
                    ..Default::default()
                },
            ),
            Self::ZPlanePlot(s) => plot.resolve_series(index, s.style),
            Self::NyquistPlot(s) => plot.resolve_series(index, s.style),
            Self::NicholsPlot(s) => plot.resolve_series(index, s.style),
            Self::RootLocus(s) => plot.resolve_series(index, s.style),
            Self::CwtPlot(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: Some(s.colors.map().sample(0.5)),
                    ..Default::default()
                },
            ),
            Self::RadarChart(s) => plot.resolve_series(index, s.style),
            Self::SankeyChart(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: s.palette.and_then(|p| p.first().copied()),
                    ..Default::default()
                },
            ),
            Self::TreemapChart(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: s.palette.and_then(|p| p.first().copied()),
                    ..Default::default()
                },
            ),
            Self::SunburstChart(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: s.palette.and_then(|p| p.first().copied()),
                    ..Default::default()
                },
            ),
            Self::GanttChart(s) => plot.resolve_series(
                index,
                SeriesStyle {
                    color: s.palette.and_then(|p| p.first().copied()),
                    ..Default::default()
                },
            ),
        }
    }
}
