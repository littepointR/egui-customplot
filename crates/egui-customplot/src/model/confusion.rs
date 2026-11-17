use egui::{AsIdSalt, Color32, IdSalt};

use crate::{ConfusionMatrixData, ConfusionNormalization, YAxis};

/// Confusion matrix chart layer (`confusionchart`, `EXT-06`).
#[derive(Clone, Copy, Debug)]
pub struct ConfusionChart<'a> {
    pub(crate) data: &'a ConfusionMatrixData,
    pub(crate) normalization: ConfusionNormalization,
    pub(crate) show_summaries: bool,
    pub(crate) show_cell_labels: bool,
    pub(crate) diagonal_color: Color32,
    pub(crate) off_diagonal_color: Color32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> ConfusionChart<'a> {
    pub fn new(data: &'a ConfusionMatrixData) -> Self {
        Self {
            data,
            normalization: ConfusionNormalization::Absolute,
            show_summaries: true,
            show_cell_labels: true,
            diagonal_color: Color32::from_rgb(0, 114, 189),
            off_diagonal_color: Color32::from_rgb(217, 83, 25),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }

    pub fn normalization(mut self, normalization: ConfusionNormalization) -> Self {
        self.normalization = normalization;
        self
    }
    pub fn show_summaries(mut self, show: bool) -> Self {
        self.show_summaries = show;
        self
    }
    pub fn show_cell_labels(mut self, show: bool) -> Self {
        self.show_cell_labels = show;
        self
    }
    pub fn diagonal_color(mut self, color: Color32) -> Self {
        self.diagonal_color = color;
        self
    }
    pub fn off_diagonal_color(mut self, color: Color32) -> Self {
        self.off_diagonal_color = color;
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
    pub fn data(self) -> &'a ConfusionMatrixData {
        self.data
    }
}
