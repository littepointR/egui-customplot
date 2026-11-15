use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{BubbleCloudData, PieData, SeriesStyle, WordCloudData, YAxis};

/// Pie / donut / legacy pie chart layer (`piechart`, `donutchart`, `pie`, `ML-027`, `ML-028`, `LG-001`).
#[derive(Clone, Copy, Debug)]
pub struct PieChart<'a> {
    pub(crate) data: &'a PieData,
    pub(crate) inner_ratio: f32,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) show_labels: bool,
    pub(crate) border: Stroke,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> PieChart<'a> {
    pub fn new(data: &'a PieData) -> Self {
        Self {
            data,
            inner_ratio: 0.0,
            palette: None,
            show_labels: true,
            border: Stroke::new(1.2, Color32::WHITE),
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn donut(data: &'a PieData) -> Self {
        Self::new(data).inner_ratio(0.5)
    }
    pub fn inner_ratio(mut self, ratio: f32) -> Self {
        if ratio.is_finite() {
            self.inner_ratio = ratio.clamp(0.0, 0.9);
        }
        self
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
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
    pub fn data(self) -> &'a PieData {
        self.data
    }
}

/// Packed text word-cloud layer (`wordcloud`, `ML-029`).
#[derive(Clone, Copy, Debug)]
pub struct WordCloud<'a> {
    pub(crate) data: &'a WordCloudData,
    pub(crate) base_font_px: f32,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> WordCloud<'a> {
    pub fn new(data: &'a WordCloudData) -> Self {
        Self {
            data,
            base_font_px: 18.0,
            palette: None,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn base_font_px(mut self, px: f32) -> Self {
        if px.is_finite() {
            self.base_font_px = px.clamp(8.0, 64.0);
        }
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
    pub fn data(self) -> &'a WordCloudData {
        self.data
    }
}

/// Circle-packed labeled bubble cloud layer (`bubblecloud`, `ML-030`).
#[derive(Clone, Copy, Debug)]
pub struct BubbleCloud<'a> {
    pub(crate) data: &'a BubbleCloudData,
    pub(crate) palette: Option<&'a [Color32]>,
    pub(crate) show_labels: bool,
    pub(crate) alpha: f32,
    pub(crate) border: Stroke,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> BubbleCloud<'a> {
    pub fn new(data: &'a BubbleCloudData) -> Self {
        Self {
            data,
            palette: None,
            show_labels: true,
            alpha: 0.78,
            border: Stroke::new(1.0, Color32::WHITE),
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        if !palette.is_empty() {
            self.palette = Some(palette);
        }
        self
    }
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
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
    pub fn data(self) -> &'a BubbleCloudData {
        self.data
    }
}
