use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use super::{
    Contours, Line, LineInterpolation, LinePattern, LineSampling, MarkerStyle, PlotLayer,
    SeriesStyle,
};
use crate::{FunctionCurveData, ImplicitCurveData, YAxis};

/// Borrowed descriptor for adaptive explicit / parametric function curves (`fplot` / `ezplot`).
#[derive(Clone, Copy, Debug)]
pub struct FunctionPlot<'a> {
    pub(crate) curve: &'a FunctionCurveData,
    pub(crate) line: Line<'a>,
}

impl<'a> FunctionPlot<'a> {
    pub fn new(curve: &'a FunctionCurveData) -> Self {
        Self {
            curve,
            line: Line::new(curve.data()),
        }
    }

    pub fn curve(self) -> &'a FunctionCurveData {
        self.curve
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.line = self.line.name(name);
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.line = self.line.y_axis(axis);
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.line = self.line.id_salt(id);
        self
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.line = self.line.stroke(stroke);
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.line = self.line.color(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.line = self.line.width(width);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.line = self.line.pattern(pattern);
        self
    }
    pub fn markers(mut self, markers: MarkerStyle) -> Self {
        self.line = self.line.markers(markers);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.line = self.line.style(style);
        self
    }
    pub fn sampling(mut self, sampling: LineSampling) -> Self {
        self.line = self.line.sampling(sampling);
        self
    }
    pub fn interpolation(mut self, interpolation: LineInterpolation) -> Self {
        self.line = self.line.interpolation(interpolation);
        self
    }
}

impl<'a> From<FunctionPlot<'a>> for Line<'a> {
    fn from(value: FunctionPlot<'a>) -> Self {
        value.line
    }
}

impl<'a> From<FunctionPlot<'a>> for PlotLayer<'a> {
    fn from(value: FunctionPlot<'a>) -> Self {
        Self::Series(value.line.into())
    }
}

/// Borrowed descriptor for adaptive implicit curve `F(x, y) = 0` (`fimplicit` / `ezplot`).
#[derive(Clone, Copy, Debug)]
pub struct ImplicitCurve<'a> {
    pub(crate) data: &'a ImplicitCurveData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> ImplicitCurve<'a> {
    pub fn new(data: &'a ImplicitCurveData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
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
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}

impl<'a> From<ImplicitCurve<'a>> for Contours<'a> {
    fn from(value: ImplicitCurve<'a>) -> Self {
        let mut c = Contours::new(value.data.contours())
            .name(value.name)
            .y_axis(value.y_axis)
            .style(value.style);
        if let Some(id) = value.id_salt {
            c = c.id_salt(id);
        }
        c
    }
}

impl<'a> From<ImplicitCurve<'a>> for PlotLayer<'a> {
    fn from(value: ImplicitCurve<'a>) -> Self {
        Self::Contours(value.into())
    }
}
