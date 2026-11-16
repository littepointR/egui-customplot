use egui::{Color32, IdSalt, Stroke};

use crate::{
    ColorScale, CompassData, LinePattern, MarkerShape, MarkerStyle, NegativeRadiusMode,
    PolarBubbleData, PolarCurveData, PolarHistNormalization, PolarHistogramData, SeriesStyle,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

#[derive(Clone, Copy, Debug)]
pub struct PolarLine<'a> {
    pub data: &'a PolarCurveData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub style: SeriesStyle,
    pub show_markers: bool,
}

impl<'a> PolarLine<'a> {
    pub fn new(data: &'a PolarCurveData) -> Self {
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
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
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

#[derive(Clone, Copy, Debug)]
pub struct PolarScatter<'a> {
    pub data: &'a PolarCurveData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub colors: Option<&'a [Color32]>,
    pub radii: Option<&'a [f32]>,
    pub style: SeriesStyle,
}

impl<'a> PolarScatter<'a> {
    pub fn new(data: &'a PolarCurveData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            colors: None,
            radii: None,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(4.0),
                ..Default::default()
            },
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn colors(mut self, colors: &'a [Color32]) -> Self {
        self.colors = Some(colors);
        self
    }
    pub fn radii(mut self, radii: &'a [f32]) -> Self {
        self.radii = Some(radii);
        self
    }
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PolarBubble<'a> {
    pub data: &'a PolarBubbleData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub alpha: f32,
    pub border: Option<Stroke>,
    pub style: SeriesStyle,
}

impl<'a> PolarBubble<'a> {
    pub fn new(data: &'a PolarBubbleData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            alpha: 0.72,
            border: Some(Stroke::new(1.0, Color32::from_white_alpha(180))),
            style: SeriesStyle::default(),
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
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
    pub fn border(mut self, stroke: Option<Stroke>) -> Self {
        self.border = stroke;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PolarHistogram<'a> {
    pub data: &'a PolarHistogramData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub normalization: PolarHistNormalization,
    pub border: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> PolarHistogram<'a> {
    pub fn new(data: &'a PolarHistogramData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            normalization: PolarHistNormalization::Count,
            border: Some(Stroke::new(1.0, Color32::from_black_alpha(120))),
            alpha: 0.78,
            style: SeriesStyle::default(),
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn normalization(mut self, normalization: PolarHistNormalization) -> Self {
        self.normalization = normalization;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn border(mut self, stroke: Option<Stroke>) -> Self {
        self.border = stroke;
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompassPlot<'a> {
    pub data: &'a CompassData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub head_size: [f32; 2],
    pub show_heads: bool,
    pub style: SeriesStyle,
}

impl<'a> CompassPlot<'a> {
    pub fn new(data: &'a CompassData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            head_size: [10.0, 8.0],
            show_heads: true,
            style: SeriesStyle::default(),
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
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
    pub fn head_size(mut self, size: [f32; 2]) -> Self {
        self.head_size = size;
        self
    }
    pub fn show_heads(mut self, show: bool) -> Self {
        self.show_heads = show;
        self
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum PolarLayer<'a> {
    Line(PolarLine<'a>),
    Scatter(PolarScatter<'a>),
    Bubble(PolarBubble<'a>),
    Histogram(PolarHistogram<'a>),
    Compass(CompassPlot<'a>),
}

impl<'a> From<PolarLine<'a>> for PolarLayer<'a> {
    fn from(value: PolarLine<'a>) -> Self {
        Self::Line(value)
    }
}
impl<'a> From<PolarScatter<'a>> for PolarLayer<'a> {
    fn from(value: PolarScatter<'a>) -> Self {
        Self::Scatter(value)
    }
}
impl<'a> From<PolarBubble<'a>> for PolarLayer<'a> {
    fn from(value: PolarBubble<'a>) -> Self {
        Self::Bubble(value)
    }
}
impl<'a> From<PolarHistogram<'a>> for PolarLayer<'a> {
    fn from(value: PolarHistogram<'a>) -> Self {
        Self::Histogram(value)
    }
}
impl<'a> From<CompassPlot<'a>> for PolarLayer<'a> {
    fn from(value: CompassPlot<'a>) -> Self {
        Self::Compass(value)
    }
}

impl<'a> PolarLayer<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Self::Line(s) => s.name,
            Self::Scatter(s) => s.name,
            Self::Bubble(s) => s.name,
            Self::Histogram(s) => s.name,
            Self::Compass(s) => s.name,
        }
    }

    pub(crate) fn id(self, index: usize) -> IdSalt {
        let explicit = match self {
            Self::Line(s) => s.id_salt,
            Self::Scatter(s) => s.id_salt,
            Self::Bubble(s) => s.id_salt,
            Self::Histogram(s) => s.id_salt,
            Self::Compass(s) => s.id_salt,
        };
        explicit.unwrap_or_else(|| {
            if self.name().is_empty() {
                IdSalt::new(("polar-index", index))
            } else {
                IdSalt::new(("polar-name", self.name()))
            }
        })
    }

    pub(crate) fn resolved_style(
        self,
        index: usize,
        plot: &ResolvedPlotStyle<'_>,
    ) -> ResolvedSeriesStyle {
        let style = match self {
            Self::Line(s) => s.style,
            Self::Scatter(s) => s.style,
            Self::Bubble(s) => {
                let mut st = s.style;
                if st.color.is_none()
                    && let Some(cs) = s.color_scale
                {
                    st.color = Some(cs.map().sample(0.5));
                }
                st
            }
            Self::Histogram(s) => s.style,
            Self::Compass(s) => s.style,
        };
        plot.resolve_series(index, style)
    }

    pub(crate) fn r_extent(self, neg_mode: NegativeRadiusMode) -> Option<[f64; 2]> {
        match self {
            Self::Line(s) => s.data.r_extent(neg_mode),
            Self::Scatter(s) => s.data.r_extent(neg_mode),
            Self::Bubble(s) => s.data.r_extent(neg_mode),
            Self::Histogram(s) => Some([0.0, s.data.max_value(s.normalization).max(1e-3)]),
            Self::Compass(s) => (!s.data.is_empty()).then(|| [0.0, s.data.max_rho().max(1e-3)]),
        }
    }
}
