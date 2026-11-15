use egui::{AsIdSalt, Color32, IdSalt};

use crate::{CategoricalHeatmapData, ColorScale, ParallelData, SeriesStyle, YAxis};

/// Categorical/table-aggregated heatmap layer (`ML-031`).
#[derive(Clone, Copy, Debug)]
pub struct CategoricalHeatmap<'a> {
    pub(crate) data: &'a CategoricalHeatmapData,
    pub(crate) colors: ColorScale,
    pub(crate) show_cell_labels: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> CategoricalHeatmap<'a> {
    pub fn new(data: &'a CategoricalHeatmapData, colors: ColorScale) -> Self {
        Self {
            data,
            colors,
            show_cell_labels: false,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn show_cell_labels(mut self, show: bool) -> Self {
        self.show_cell_labels = show;
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
    pub fn data(self) -> &'a CategoricalHeatmapData {
        self.data
    }
    pub fn color_scale(self) -> ColorScale {
        self.colors
    }
}

/// Parallel coordinates multivariate polyline layer (`parallelplot`, `ML-032`).
#[derive(Clone, Copy, Debug)]
pub struct ParallelPlot<'a> {
    pub(crate) data: &'a ParallelData,
    pub(crate) style: SeriesStyle,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) alpha: f32,
    pub(crate) show_markers: bool,
    pub(crate) show_vertical_axes: bool,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> ParallelPlot<'a> {
    pub fn new(data: &'a ParallelData) -> Self {
        Self {
            data,
            style: SeriesStyle::default(),
            palette: None,
            alpha: 0.72,
            show_markers: false,
            show_vertical_axes: true,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.05, 1.0);
        }
        self
    }
    pub fn show_markers(mut self, show: bool) -> Self {
        self.show_markers = show;
        self
    }
    pub fn show_vertical_axes(mut self, show: bool) -> Self {
        self.show_vertical_axes = show;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
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
    pub fn data(self) -> &'a ParallelData {
        self.data
    }
}
