use egui::{AsIdSalt, Color32, IdSalt};

use crate::{BarOrientation, BoxChartData, RaincloudData, SeriesStyle, ViolinData, YAxis};

/// Box-and-whisker distribution layer (`boxchart`, `ML-024`).
#[derive(Clone, Copy, Debug)]
pub struct BoxChart<'a> {
    pub(crate) data: &'a BoxChartData,
    pub(crate) style: SeriesStyle,
    pub(crate) fill_alpha: f32,
    pub(crate) notched: bool,
    pub(crate) show_mean: bool,
    pub(crate) show_outliers: bool,
    pub(crate) orientation: BarOrientation,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> BoxChart<'a> {
    pub fn new(data: &'a BoxChartData) -> Self {
        Self {
            data,
            style: SeriesStyle::default(),
            fill_alpha: 0.35,
            notched: false,
            show_mean: false,
            show_outliers: true,
            orientation: BarOrientation::Vertical,
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
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.fill_alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn notched(mut self, notched: bool) -> Self {
        self.notched = notched;
        self
    }
    pub fn show_mean(mut self, show: bool) -> Self {
        self.show_mean = show;
        self
    }
    pub fn show_outliers(mut self, show: bool) -> Self {
        self.show_outliers = show;
        self
    }
    pub fn orientation(mut self, orientation: BarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a BoxChartData {
        self.data
    }
}

/// Kernel-density violin layer (`violinplot`, `ML-025`).
#[derive(Clone, Copy, Debug)]
pub struct ViolinPlot<'a> {
    pub(crate) data: &'a ViolinData,
    pub(crate) style: SeriesStyle,
    pub(crate) fill_alpha: f32,
    pub(crate) show_box: Option<bool>,
    pub(crate) show_median: bool,
    pub(crate) orientation: BarOrientation,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> ViolinPlot<'a> {
    pub fn new(data: &'a ViolinData) -> Self {
        Self {
            data,
            style: SeriesStyle::default(),
            fill_alpha: 0.45,
            show_box: None,
            show_median: true,
            orientation: BarOrientation::Vertical,
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
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.fill_alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn show_box(mut self, show: bool) -> Self {
        self.show_box = Some(show);
        self
    }
    pub fn show_median(mut self, show: bool) -> Self {
        self.show_median = show;
        self
    }
    pub fn orientation(mut self, orientation: BarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a ViolinData {
        self.data
    }
}

/// Half-violin + boxplot + jittered raindrops layer (`raincloudplot`, `ML-026`).
#[derive(Clone, Copy, Debug)]
pub struct Raincloud<'a> {
    pub(crate) data: &'a RaincloudData,
    pub(crate) style: SeriesStyle,
    pub(crate) fill_alpha: f32,
    pub(crate) drop_radius: f32,
    pub(crate) orientation: BarOrientation,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Raincloud<'a> {
    pub fn new(data: &'a RaincloudData) -> Self {
        Self {
            data,
            style: SeriesStyle::default(),
            fill_alpha: 0.45,
            drop_radius: 2.6,
            orientation: BarOrientation::Vertical,
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
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.fill_alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn drop_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() {
            self.drop_radius = radius.clamp(1.0, 12.0);
        }
        self
    }
    pub fn orientation(mut self, orientation: BarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn data(self) -> &'a RaincloudData {
        self.data
    }
}
