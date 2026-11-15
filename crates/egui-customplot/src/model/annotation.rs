use std::sync::Arc;

use egui::{Align2, Color32, Stroke};

use crate::PlotPoint;

/// Non-interactive data-coordinate overlays, painted in submission order after
/// series and before the legend. They never affect auto bounds or point picking.
#[derive(Clone, Debug)]
pub enum Annotation {
    Text {
        point: PlotPoint,
        text: Arc<str>,
        color: Option<Color32>,
        anchor: Align2,
    },
    XLine {
        value: f64,
        label: Arc<str>,
        stroke: Option<Stroke>,
    },
    YLine {
        value: f64,
        label: Arc<str>,
        stroke: Option<Stroke>,
    },
    /// Ordered interval, intersected with the view before coordinate mapping.
    /// Infinite endpoints and endpoints outside a log domain are allowed.
    XRegion {
        range: [f64; 2],
        fill: Color32,
    },
    YRegion {
        range: [f64; 2],
        fill: Color32,
    },
}

impl Annotation {
    pub fn text(point: PlotPoint, text: impl Into<Arc<str>>) -> Self {
        Self::Text {
            point,
            text: text.into(),
            color: None,
            anchor: Align2::LEFT_BOTTOM,
        }
    }

    pub fn x_line(value: f64, label: impl Into<Arc<str>>) -> Self {
        Self::XLine {
            value,
            label: label.into(),
            stroke: None,
        }
    }

    pub fn y_line(value: f64, label: impl Into<Arc<str>>) -> Self {
        Self::YLine {
            value,
            label: label.into(),
            stroke: None,
        }
    }

    pub fn x_region(range: [f64; 2], fill: Color32) -> Self {
        Self::XRegion { range, fill }
    }

    pub fn y_region(range: [f64; 2], fill: Color32) -> Self {
        Self::YRegion { range, fill }
    }
}
