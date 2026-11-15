use egui::IdSalt;

use super::PlotLayer;
use crate::{
    SeriesStyle,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

pub(super) fn layer_id(layer: PlotLayer<'_>, index: usize) -> IdSalt {
    let id_salt = match layer {
        PlotLayer::Series(s) => return s.id(index),
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
        }
    }
}
