use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    ColorScale, MarkerShape, SeriesStyle, YAxis,
    data::{CwtData, FrequencyResponseData, PoleZeroData, RootLocusData},
};

/// Discrete/continuous zero-pole diagram layer (`zplane`, `EXT-20`).
#[derive(Clone, Copy, Debug)]
pub struct ZPlanePlot<'a> {
    pub(crate) data: &'a PoleZeroData,
    pub(crate) zero_color: Color32,
    pub(crate) pole_color: Color32,
    pub(crate) show_unit_circle: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> ZPlanePlot<'a> {
    pub fn new(data: &'a PoleZeroData) -> Self {
        let style = SeriesStyle {
            marker_shape: Some(MarkerShape::Circle),
            marker_radius: Some(5.5),
            ..Default::default()
        };
        Self {
            data,
            zero_color: Color32::from_rgb(0, 114, 189),
            pole_color: Color32::from_rgb(217, 83, 25),
            show_unit_circle: true,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style,
        }
    }
    pub fn zero_color(mut self, color: Color32) -> Self {
        self.zero_color = color;
        self
    }
    pub fn pole_color(mut self, color: Color32) -> Self {
        self.pole_color = color;
        self
    }
    pub fn show_unit_circle(mut self, show: bool) -> Self {
        self.show_unit_circle = show;
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

/// Complex Nyquist stability locus layer (`nyquist`, `EXT-21`).
#[derive(Clone, Copy, Debug)]
pub struct NyquistPlot<'a> {
    pub(crate) data: &'a FrequencyResponseData,
    pub(crate) show_mirror: bool,
    pub(crate) show_unit_circle: bool,
    pub(crate) show_arrows: bool,
    pub(crate) critical_color: Color32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> NyquistPlot<'a> {
    pub fn new(data: &'a FrequencyResponseData) -> Self {
        Self {
            data,
            show_mirror: true,
            show_unit_circle: true,
            show_arrows: true,
            critical_color: Color32::from_rgb(214, 39, 40),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn show_mirror(mut self, show: bool) -> Self {
        self.show_mirror = show;
        self
    }
    pub fn show_unit_circle(mut self, show: bool) -> Self {
        self.show_unit_circle = show;
        self
    }
    pub fn show_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
        self
    }
    pub fn critical_color(mut self, color: Color32) -> Self {
        self.critical_color = color;
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

/// Nichols log-magnitude vs phase plot layer with optional closed-loop M/N contours (`nichols`, `EXT-21`).
#[derive(Clone, Copy, Debug)]
pub struct NicholsPlot<'a> {
    pub(crate) data: &'a FrequencyResponseData,
    pub(crate) show_grid: bool,
    pub(crate) show_arrows: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> NicholsPlot<'a> {
    pub fn new(data: &'a FrequencyResponseData) -> Self {
        Self {
            data,
            show_grid: true,
            show_arrows: true,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }
    pub fn show_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
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

/// Evans root locus plot layer (`rlocus`, `EXT-21`).
#[derive(Clone, Copy, Debug)]
pub struct RootLocus<'a> {
    pub(crate) data: &'a RootLocusData,
    pub(crate) show_imaginary_axis: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> RootLocus<'a> {
    pub fn new(data: &'a RootLocusData) -> Self {
        Self {
            data,
            show_imaginary_axis: true,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn show_imaginary_axis(mut self, show: bool) -> Self {
        self.show_imaginary_axis = show;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
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

/// Continuous wavelet scalogram layer with Cone-of-Influence curves (`cwt`, `EXT-22`).
#[derive(Clone, Copy, Debug)]
pub struct CwtPlot<'a> {
    pub(crate) data: &'a CwtData,
    pub(crate) colors: ColorScale,
    pub(crate) show_coi: bool,
    pub(crate) coi_color: Color32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> CwtPlot<'a> {
    pub fn new(data: &'a CwtData, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            show_coi: true,
            coi_color: Color32::WHITE,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn show_coi(mut self, show: bool) -> Self {
        self.show_coi = show;
        self
    }
    pub fn coi_color(mut self, color: Color32) -> Self {
        self.coi_color = color;
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
