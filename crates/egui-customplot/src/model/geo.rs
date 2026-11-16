use std::collections::HashSet;

use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{
    ColorScale, GeoBubbleData, GeoDensityData, GeoPathData, GeoScatterData, LinePattern,
    MarkerShape, MarkerStyle, PlotDiagnostics, SeriesStyle,
    axis::{GeoBounds, GeoPoint},
    interaction::GeoPlotState,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

/// Geographic polyline layer (`geoplot` / `ML-041`).
#[derive(Clone, Copy, Debug)]
pub struct GeoLine<'a> {
    pub data: &'a GeoPathData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub style: SeriesStyle,
    pub show_markers: bool,
}

impl<'a> GeoLine<'a> {
    pub fn new(data: &'a GeoPathData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
            show_markers: false,
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
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
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self.show_markers = true;
        self
    }
}

/// Geographic scatter layer (`geoscatter` / `ML-042`).
#[derive(Clone, Copy, Debug)]
pub struct GeoScatter<'a> {
    pub data: &'a GeoScatterData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> GeoScatter<'a> {
    pub fn new(data: &'a GeoScatterData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            alpha: 1.0,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(4.5),
                ..Default::default()
            },
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn marker(mut self, shape: MarkerShape, radius: f32) -> Self {
        self.style.marker_shape = Some(shape);
        self.style.marker_radius = Some(radius.max(0.5));
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
}

/// Geographic spatial kernel density layer (`geodensityplot` / `ML-043`).
#[derive(Clone, Copy, Debug)]
pub struct GeoDensity<'a> {
    pub data: &'a GeoDensityData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub alpha: f32,
}

impl<'a> GeoDensity<'a> {
    pub fn new(data: &'a GeoDensityData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            alpha: 0.72,
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
}

/// Geographic proportional bubble layer (`geobubble` / `ML-044`).
#[derive(Clone, Copy, Debug)]
pub struct GeoBubbles<'a> {
    pub data: &'a GeoBubbleData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub alpha: f32,
    pub border: Option<Stroke>,
    pub style: SeriesStyle,
}

impl<'a> GeoBubbles<'a> {
    pub fn new(data: &'a GeoBubbleData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            alpha: 0.65,
            border: Some(Stroke::new(1.0, Color32::from_black_alpha(120))),
            style: SeriesStyle::default(),
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn border(mut self, stroke: Option<Stroke>) -> Self {
        self.border = stroke;
        self
    }
}

/// Heterogeneous layer sum type for [`crate::GeoPlot`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum GeoLayer<'a> {
    Line(GeoLine<'a>),
    Scatter(GeoScatter<'a>),
    Density(GeoDensity<'a>),
    Bubbles(GeoBubbles<'a>),
}

impl<'a> From<GeoLine<'a>> for GeoLayer<'a> {
    fn from(v: GeoLine<'a>) -> Self {
        Self::Line(v)
    }
}
impl<'a> From<GeoScatter<'a>> for GeoLayer<'a> {
    fn from(v: GeoScatter<'a>) -> Self {
        Self::Scatter(v)
    }
}
impl<'a> From<GeoDensity<'a>> for GeoLayer<'a> {
    fn from(v: GeoDensity<'a>) -> Self {
        Self::Density(v)
    }
}
impl<'a> From<GeoBubbles<'a>> for GeoLayer<'a> {
    fn from(v: GeoBubbles<'a>) -> Self {
        Self::Bubbles(v)
    }
}

impl<'a> GeoLayer<'a> {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Line(l) => l.name,
            Self::Scatter(s) => s.name,
            Self::Density(d) => d.name,
            Self::Bubbles(b) => b.name,
        }
    }

    pub(crate) fn id(&self, index: usize) -> IdSalt {
        let custom = match self {
            Self::Line(l) => l.id_salt,
            Self::Scatter(s) => s.id_salt,
            Self::Density(d) => d.id_salt,
            Self::Bubbles(b) => b.id_salt,
        };
        custom.unwrap_or_else(|| IdSalt::new(("geo-layer", index, self.name())))
    }

    pub(crate) fn resolved_style(
        &self,
        index: usize,
        plot: &ResolvedPlotStyle<'_>,
    ) -> ResolvedSeriesStyle {
        let local = match self {
            Self::Line(l) => l.style,
            Self::Scatter(s) => s.style,
            Self::Density(_) => SeriesStyle::default(),
            Self::Bubbles(b) => b.style,
        };
        plot.resolve_series(index, local)
    }
}

pub(crate) fn summarize_geo(
    layers: &[GeoLayer<'_>],
    state: &GeoPlotState,
) -> (PlotDiagnostics, HashSet<IdSalt>, GeoBounds) {
    let mut diag = PlotDiagnostics::default();
    let mut present = HashSet::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut sample_pts: Vec<GeoPoint> = Vec::new();

    for (idx, layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        present.insert(id);
        let (alloc, pts, non_fin, dom_inv) = match *layer {
            GeoLayer::Line(l) => (
                l.data.allocation(),
                l.data.points().len(),
                l.data.non_finite_points(),
                l.data.domain_invalid_points(),
            ),
            GeoLayer::Scatter(s) => (
                s.data.allocation(),
                s.data.points().len(),
                s.data.non_finite_points(),
                s.data.domain_invalid_points(),
            ),
            GeoLayer::Density(d) => (
                d.data.allocation(),
                d.data.input_len(),
                d.data.non_finite_points(),
                d.data.domain_invalid_points(),
            ),
            GeoLayer::Bubbles(b) => (
                b.data.allocation(),
                b.data.points().len(),
                b.data.non_finite_points(),
                b.data.domain_invalid_points(),
            ),
        };
        diag.geo.input_points += pts;
        diag.geo.non_finite_points += non_fin;
        diag.geo.domain_invalid_points += dom_inv;
        if seen.insert(alloc) {
            diag.geo.unique_bytes += alloc.1;
        }
        if state.layer_visible(id) {
            diag.visible_series += 1;
            match *layer {
                GeoLayer::Line(l) => sample_pts.extend(
                    l.data
                        .points()
                        .iter()
                        .copied()
                        .filter(|p| p.is_valid_domain()),
                ),
                GeoLayer::Scatter(s) => sample_pts.extend(
                    s.data
                        .points()
                        .iter()
                        .map(|p| p.point)
                        .filter(|p| p.is_valid_domain()),
                ),
                GeoLayer::Density(d) => {
                    sample_pts.extend(d.data.sample_points());
                }
                GeoLayer::Bubbles(b) => sample_pts.extend(
                    b.data
                        .points()
                        .iter()
                        .filter(|pt| {
                            pt.point.is_valid_domain() && pt.size.is_finite() && pt.size >= 0.0
                        })
                        .map(|pt| pt.point),
                ),
            }
        }
    }
    let bounds = GeoBounds::from_points(sample_pts).unwrap_or_else(GeoBounds::world);
    (diag, present, bounds)
}
