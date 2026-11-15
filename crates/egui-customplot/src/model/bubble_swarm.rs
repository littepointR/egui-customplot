use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{
    BinScatterData, BivariateNormalization, BubbleData, ColorScale, Histogram2Data, MarkerShape,
    SeriesStyle, SwarmData, YAxis,
};

/// Proportional-area scatter bubble layer (`bubblechart`, `ML-015`).
#[derive(Clone, Copy, Debug)]
pub struct BubbleChart<'a> {
    pub(crate) data: &'a BubbleData,
    pub(crate) colors: Option<ColorScale>,
    pub(crate) style: SeriesStyle,
    pub(crate) alpha: f32,
    pub(crate) border: Option<Stroke>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> BubbleChart<'a> {
    pub fn new(data: &'a BubbleData) -> Self {
        Self {
            data,
            colors: None,
            style: SeriesStyle::default(),
            alpha: 0.62,
            border: None,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
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
    pub fn color_scale(mut self, colors: ColorScale) -> Self {
        self.colors = Some(colors);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn border(mut self, border: Stroke) -> Self {
        self.border = Some(border);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a BubbleData {
        self.data
    }
}

/// Jittered categorical beeswarm scatter layer (`swarmchart`, `ML-017`).
#[derive(Clone, Copy, Debug)]
pub struct SwarmChart<'a> {
    pub(crate) data: &'a SwarmData,
    pub(crate) colors: Option<ColorScale>,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> SwarmChart<'a> {
    pub fn new(data: &'a SwarmData) -> Self {
        Self {
            data,
            colors: None,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
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
    pub fn color_scale(mut self, colors: ColorScale) -> Self {
        self.colors = Some(colors);
        self
    }
    pub fn radius(mut self, radius: f32) -> Self {
        self.style.marker_radius = Some(radius);
        self
    }
    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.style.marker_shape = Some(shape);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a SwarmData {
        self.data
    }
}

/// 2D binned histogram layer (`histogram2`, `ML-022`).
#[derive(Clone, Copy, Debug)]
pub struct Histogram2<'a> {
    pub(crate) data: &'a Histogram2Data,
    pub(crate) colors: ColorScale,
    pub(crate) normalization: BivariateNormalization,
    pub(crate) show_counts: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Histogram2<'a> {
    pub fn new(data: &'a Histogram2Data, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            normalization: BivariateNormalization::Count,
            show_counts: false,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn normalization(mut self, norm: BivariateNormalization) -> Self {
        self.normalization = norm;
        self
    }
    pub fn show_counts(mut self, show: bool) -> Self {
        self.show_counts = show;
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
    pub fn data(self) -> &'a Histogram2Data {
        self.data
    }
    pub fn color_scale(self) -> ColorScale {
        self.colors
    }
}

/// 2D density-binned scatter layer (`binscatter`, `ML-019`).
#[derive(Clone, Copy, Debug)]
pub struct BinScatter<'a> {
    pub(crate) data: &'a BinScatterData,
    pub(crate) colors: ColorScale,
    pub(crate) border: Stroke,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> BinScatter<'a> {
    pub fn new(data: &'a BinScatterData, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            border: Stroke::NONE,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn border(mut self, border: Stroke) -> Self {
        self.border = border;
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
    pub fn data(self) -> &'a BinScatterData {
        self.data
    }
    pub fn color_scale(self) -> ColorScale {
        self.colors
    }
}
