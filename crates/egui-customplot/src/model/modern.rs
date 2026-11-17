use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    SeriesStyle, YAxis,
    data::{GanttData, RadarData, SankeyData, SunburstData, TreemapData},
};

/// Multi-axis spider / radar chart layer (`RadarChart`, `EXT-26`).
#[derive(Clone, Copy, Debug)]
pub struct RadarChart<'a> {
    pub(crate) data: &'a RadarData,
    pub(crate) fill_alpha: f32,
    pub(crate) show_labels: bool,
    pub(crate) rings: usize,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
}

impl<'a> RadarChart<'a> {
    pub fn new(data: &'a RadarData) -> Self {
        Self {
            data,
            fill_alpha: 0.22,
            show_labels: true,
            rings: 4,
            palette: None,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
        }
    }
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.fill_alpha = alpha.clamp(0.0, 1.0);
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn rings(mut self, rings: usize) -> Self {
        self.rings = rings.clamp(2, 10);
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
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

/// Alluvial / flow-conserving Sankey diagram layer (`SankeyChart`, `EXT-26`).
#[derive(Clone, Copy, Debug)]
pub struct SankeyChart<'a> {
    pub(crate) data: &'a SankeyData,
    pub(crate) ribbon_alpha: f32,
    pub(crate) show_labels: bool,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> SankeyChart<'a> {
    pub fn new(data: &'a SankeyData) -> Self {
        Self {
            data,
            ribbon_alpha: 0.45,
            show_labels: true,
            palette: None,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn ribbon_alpha(mut self, alpha: f32) -> Self {
        self.ribbon_alpha = alpha.clamp(0.05, 1.0);
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
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

/// Hierarchical squarified treemap layer (`TreemapChart`, `EXT-26`).
#[derive(Clone, Copy, Debug)]
pub struct TreemapChart<'a> {
    pub(crate) data: &'a TreemapData,
    pub(crate) show_labels: bool,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) border_color: Color32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> TreemapChart<'a> {
    pub fn new(data: &'a TreemapData) -> Self {
        Self {
            data,
            show_labels: true,
            palette: None,
            border_color: Color32::WHITE,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
        self
    }
    pub fn border_color(mut self, color: Color32) -> Self {
        self.border_color = color;
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

/// Concentric hierarchical ring chart layer (`SunburstChart`, `EXT-26`).
#[derive(Clone, Copy, Debug)]
pub struct SunburstChart<'a> {
    pub(crate) data: &'a SunburstData,
    pub(crate) show_labels: bool,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) border_color: Color32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> SunburstChart<'a> {
    pub fn new(data: &'a SunburstData) -> Self {
        Self {
            data,
            show_labels: true,
            palette: None,
            border_color: Color32::WHITE,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
        self
    }
    pub fn border_color(mut self, color: Color32) -> Self {
        self.border_color = color;
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

/// Schedule & dependency Gantt bar chart layer (`GanttChart`, `EXT-26`).
#[derive(Clone, Copy, Debug)]
pub struct GanttChart<'a> {
    pub(crate) data: &'a GanttData,
    pub(crate) show_dependencies: bool,
    pub(crate) show_progress: bool,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> GanttChart<'a> {
    pub fn new(data: &'a GanttData) -> Self {
        Self {
            data,
            show_dependencies: true,
            show_progress: true,
            palette: None,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn show_dependencies(mut self, show: bool) -> Self {
        self.show_dependencies = show;
        self
    }
    pub fn show_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
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
