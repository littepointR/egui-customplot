use std::sync::Arc;

use egui::{Color32, Stroke};

use super::{PlotStyle, SeriesStyle};

/// Retain in the app and pass an Arc clone to Plot::theme; no host Style mutation.
#[derive(Clone, Debug)]
pub struct PlotTheme {
    pub style: PlotStyle,
    /// Empty palettes fall back to the library palette, never divide by zero.
    pub palette: Arc<[Color32]>,
}

impl PlotTheme {
    pub fn scientific_light() -> Self {
        Self {
            style: PlotStyle {
                background: Some(Color32::WHITE),
                text_color: Some(Color32::from_gray(25)),
                grid_stroke: Some(Stroke::new(1.0, Color32::from_gray(215))),
                minor_grid_stroke: Some(Stroke::new(0.5, Color32::from_gray(235))),
                border_stroke: Some(Stroke::new(1.0, Color32::from_gray(70))),
                crosshair_stroke: Some(Stroke::new(1.0, Color32::from_gray(120))),
                series: SeriesStyle {
                    line_width: Some(1.5),
                    ..Default::default()
                },
            },
            palette: vec![
                Color32::from_rgb(0, 90, 170),
                Color32::from_rgb(205, 75, 0),
                Color32::from_rgb(0, 130, 100),
                Color32::from_rgb(130, 75, 170),
            ]
            .into(),
        }
    }

    pub fn monitoring_dark() -> Self {
        Self {
            style: PlotStyle {
                background: Some(Color32::from_rgb(12, 18, 25)),
                text_color: Some(Color32::from_rgb(220, 230, 240)),
                grid_stroke: Some(Stroke::new(1.0, Color32::from_rgb(45, 57, 70))),
                minor_grid_stroke: Some(Stroke::new(0.5, Color32::from_rgb(28, 38, 48))),
                border_stroke: Some(Stroke::new(1.0, Color32::from_rgb(95, 115, 135))),
                crosshair_stroke: Some(Stroke::new(1.0, Color32::from_rgb(135, 155, 175))),
                series: SeriesStyle {
                    line_width: Some(1.8),
                    ..Default::default()
                },
            },
            palette: vec![
                Color32::from_rgb(70, 195, 255),
                Color32::from_rgb(255, 180, 75),
                Color32::from_rgb(100, 235, 160),
                Color32::from_rgb(210, 145, 255),
            ]
            .into(),
        }
    }
}
