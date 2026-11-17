use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    SeriesStyle, YAxis,
    data::{ControlChartData, OhlcData, Waterfall2DData},
};

/// SPC control chart layer (`controlchart`, `EXT-17`).
#[derive(Clone, Copy, Debug)]
pub struct ControlChart<'a> {
    pub(crate) data: &'a ControlChartData,
    pub(crate) violation_color: Color32,
    pub(crate) limit_color: Color32,
    pub(crate) center_color: Color32,
    pub(crate) show_labels: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> ControlChart<'a> {
    pub fn new(data: &'a ControlChartData) -> Self {
        Self {
            data,
            violation_color: Color32::from_rgb(214, 39, 40),
            limit_color: Color32::from_rgb(196, 78, 82),
            center_color: Color32::from_rgb(85, 168, 104),
            show_labels: true,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn violation_color(mut self, color: Color32) -> Self {
        self.violation_color = color;
        self
    }
    pub fn limit_color(mut self, color: Color32) -> Self {
        self.limit_color = color;
        self
    }
    pub fn center_color(mut self, color: Color32) -> Self {
        self.center_color = color;
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
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

/// Financial OHLC candlestick chart layer (`candle`, `EXT-18`).
#[derive(Clone, Copy, Debug)]
pub struct Candlestick<'a> {
    pub(crate) data: &'a OhlcData,
    pub(crate) up_color: Color32,
    pub(crate) down_color: Color32,
    pub(crate) hollow_up: bool,
    pub(crate) wick_width: f32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Candlestick<'a> {
    pub fn new(data: &'a OhlcData) -> Self {
        Self {
            data,
            up_color: Color32::from_rgb(38, 166, 91),
            down_color: Color32::from_rgb(214, 69, 65),
            hollow_up: true,
            wick_width: 1.4,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn up_color(mut self, color: Color32) -> Self {
        self.up_color = color;
        self
    }
    pub fn down_color(mut self, color: Color32) -> Self {
        self.down_color = color;
        self
    }
    pub fn hollow_up(mut self, hollow: bool) -> Self {
        self.hollow_up = hollow;
        self
    }
    pub fn wick_width(mut self, width: f32) -> Self {
        self.wick_width = width.max(0.5);
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
}

/// Financial OHLC tick-bar chart layer (`highlow`, `EXT-18`).
#[derive(Clone, Copy, Debug)]
pub struct OhlcBars<'a> {
    pub(crate) data: &'a OhlcData,
    pub(crate) up_color: Color32,
    pub(crate) down_color: Color32,
    pub(crate) width: f32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> OhlcBars<'a> {
    pub fn new(data: &'a OhlcData) -> Self {
        Self {
            data,
            up_color: Color32::from_rgb(38, 166, 91),
            down_color: Color32::from_rgb(214, 69, 65),
            width: 1.6,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn up_color(mut self, color: Color32) -> Self {
        self.up_color = color;
        self
    }
    pub fn down_color(mut self, color: Color32) -> Self {
        self.down_color = color;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.5);
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
}

/// 2D cumulative bridge / waterfall chart (`Waterfall2D`, `EXT-18`).
#[derive(Clone, Copy, Debug)]
pub struct Waterfall2D<'a> {
    pub(crate) data: &'a Waterfall2DData,
    pub(crate) positive_color: Color32,
    pub(crate) negative_color: Color32,
    pub(crate) total_color: Color32,
    pub(crate) show_connectors: bool,
    pub(crate) show_values: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Waterfall2D<'a> {
    pub fn new(data: &'a Waterfall2DData) -> Self {
        Self {
            data,
            positive_color: Color32::from_rgb(46, 160, 67),
            negative_color: Color32::from_rgb(218, 54, 51),
            total_color: Color32::from_rgb(31, 119, 180),
            show_connectors: true,
            show_values: true,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn positive_color(mut self, color: Color32) -> Self {
        self.positive_color = color;
        self
    }
    pub fn negative_color(mut self, color: Color32) -> Self {
        self.negative_color = color;
        self
    }
    pub fn total_color(mut self, color: Color32) -> Self {
        self.total_color = color;
        self
    }
    pub fn show_connectors(mut self, show: bool) -> Self {
        self.show_connectors = show;
        self
    }
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
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
}
