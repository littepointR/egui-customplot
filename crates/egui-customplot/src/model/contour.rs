use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{ColorScale, ContourData, FilledContourData, LinePattern, SeriesStyle, YAxis};

/// Extracted 2D contour isolines (`contour` / `fcontour` / `ezcontour`).
#[derive(Clone, Copy, Debug)]
pub struct Contours<'a> {
    pub(crate) data: &'a ContourData,
    pub(crate) color_scale: Option<ColorScale>,
    pub(crate) show_labels: bool,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Contours<'a> {
    pub fn new(data: &'a ContourData) -> Self {
        Self {
            data,
            color_scale: None,
            show_labels: false,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a ContourData {
        self.data
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
    /// Color each isoline by its scalar level using a shared ColorScale.
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
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
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}

/// Extracted 2D filled contour bands (`contourf` / `ezcontourf`).
#[derive(Clone, Copy, Debug)]
pub struct FilledContours<'a> {
    pub(crate) data: &'a FilledContourData,
    pub(crate) colors: ColorScale,
    pub(crate) border: Stroke,
    pub(crate) border_lines: Option<&'a ContourData>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> FilledContours<'a> {
    pub fn new(data: &'a FilledContourData, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            border: Stroke::NONE,
            border_lines: None,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a FilledContourData {
        self.data
    }
    pub fn color_scale(self) -> ColorScale {
        self.colors
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
    /// Draw matching contour boundary lines over the filled bands.
    pub fn border_lines(mut self, lines: &'a ContourData, stroke: Stroke) -> Self {
        self.border_lines = Some(lines);
        self.border = stroke;
        self
    }
}
