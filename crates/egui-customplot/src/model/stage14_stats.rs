use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    BarOrientation, MarkerShape, SeriesStyle, YAxis,
    data::{BiplotData, DendrogramData, EcdfData, ProbabilityPlotData},
};

/// Probability / QQ plot layer (`probplot`, `normplot`, `wblplot`, `qqplot`, `EXT-13`).
#[derive(Clone, Copy, Debug)]
pub struct ProbabilityPlot<'a> {
    pub(crate) data: &'a ProbabilityPlotData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) ref_color: Option<Color32>,
    pub(crate) show_ref_line: bool,
}

impl<'a> ProbabilityPlot<'a> {
    pub fn new(data: &'a ProbabilityPlotData) -> Self {
        let style = SeriesStyle {
            marker_shape: Some(MarkerShape::Plus),
            marker_radius: Some(4.5),
            ..Default::default()
        };
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style,
            ref_color: None,
            show_ref_line: true,
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
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn ref_color(mut self, color: Color32) -> Self {
        self.ref_color = Some(color);
        self
    }
    pub fn show_ref_line(mut self, show: bool) -> Self {
        self.show_ref_line = show;
        self
    }
    pub fn marker_radius(mut self, radius: f32) -> Self {
        self.style.marker_radius = Some(radius.max(0.5));
        self
    }
    pub fn marker_shape(mut self, shape: MarkerShape) -> Self {
        self.style.marker_shape = Some(shape);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}

/// Kaplan-Meier ECDF / survival step curve with Greenwood confidence band (`EXT-14`).
#[derive(Clone, Copy, Debug)]
pub struct EcdfPlot<'a> {
    pub(crate) data: &'a EcdfData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) show_ci: bool,
    pub(crate) ci_alpha: f32,
    pub(crate) show_censored: bool,
}

impl<'a> EcdfPlot<'a> {
    pub fn new(data: &'a EcdfData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            show_ci: true,
            ci_alpha: 0.18,
            show_censored: true,
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
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
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
    pub fn show_ci(mut self, show: bool) -> Self {
        self.show_ci = show;
        self
    }
    pub fn ci_alpha(mut self, alpha: f32) -> Self {
        self.ci_alpha = alpha.clamp(0.0, 1.0);
        self
    }
    pub fn show_censored(mut self, show: bool) -> Self {
        self.show_censored = show;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}

/// Hierarchical clustering dendrogram tree layer (`dendrogram`, `EXT-15`).
#[derive(Clone, Copy, Debug)]
pub struct DendrogramPlot<'a> {
    pub(crate) data: &'a DendrogramData,
    pub(crate) orientation: BarOrientation,
    pub(crate) color_threshold: Option<f64>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> DendrogramPlot<'a> {
    pub fn new(data: &'a DendrogramData) -> Self {
        Self {
            data,
            orientation: BarOrientation::Vertical,
            color_threshold: None,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn orientation(mut self, orientation: BarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn color_threshold(mut self, threshold: f64) -> Self {
        self.color_threshold = threshold.is_finite().then_some(threshold);
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
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
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}

/// PCA biplot displaying loading arrows and observation score scatter (`biplot`, `EXT-16`).
#[derive(Clone, Copy, Debug)]
pub struct Biplot<'a> {
    pub(crate) data: &'a BiplotData,
    pub(crate) show_scores: bool,
    pub(crate) show_var_labels: bool,
    pub(crate) show_obs_labels: bool,
    pub(crate) loading_color: Option<Color32>,
    pub(crate) score_color: Option<Color32>,
    pub(crate) head_size: [f32; 2],
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> Biplot<'a> {
    pub fn new(data: &'a BiplotData) -> Self {
        Self {
            data,
            show_scores: true,
            show_var_labels: true,
            show_obs_labels: false,
            loading_color: None,
            score_color: None,
            head_size: [10.0, 7.5],
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn show_scores(mut self, show: bool) -> Self {
        self.show_scores = show;
        self
    }
    pub fn show_var_labels(mut self, show: bool) -> Self {
        self.show_var_labels = show;
        self
    }
    pub fn show_obs_labels(mut self, show: bool) -> Self {
        self.show_obs_labels = show;
        self
    }
    pub fn loading_color(mut self, color: Color32) -> Self {
        self.loading_color = Some(color);
        self
    }
    pub fn score_color(mut self, color: Color32) -> Self {
        self.score_color = Some(color);
        self
    }
    pub fn head_size(mut self, length: f32, width: f32) -> Self {
        self.head_size = [length, width];
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}
