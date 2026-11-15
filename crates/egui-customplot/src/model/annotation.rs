use std::sync::Arc;

use egui::{Align2, Color32, Stroke};

use crate::PlotPoint;

/// Coordinate space for `EX-027` annotations: plot/figure normalized `[0, 1]^2`
/// (origin bottom-left) or explicit data coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationCoord {
    Data,
    Normalized,
}

/// Non-interactive overlays, painted in submission order after series and before
/// the legend. They never affect auto bounds or point picking.
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
    Line {
        coord: AnnotationCoord,
        start: PlotPoint,
        end: PlotPoint,
        stroke: Option<Stroke>,
    },
    Arrow {
        coord: AnnotationCoord,
        start: PlotPoint,
        end: PlotPoint,
        stroke: Option<Stroke>,
        head_size: [f32; 2],
    },
    DoubleArrow {
        coord: AnnotationCoord,
        start: PlotPoint,
        end: PlotPoint,
        stroke: Option<Stroke>,
        head_size: [f32; 2],
    },
    TextArrow {
        coord: AnnotationCoord,
        tail: PlotPoint,
        tip: PlotPoint,
        text: Arc<str>,
        stroke: Option<Stroke>,
        text_color: Option<Color32>,
        head_size: [f32; 2],
    },
    TextBox {
        coord: AnnotationCoord,
        bounds: [PlotPoint; 2],
        text: Arc<str>,
        fill: Color32,
        stroke: Option<Stroke>,
        text_color: Option<Color32>,
    },
    Rectangle {
        coord: AnnotationCoord,
        bounds: [PlotPoint; 2],
        fill: Color32,
        stroke: Option<Stroke>,
    },
    Ellipse {
        coord: AnnotationCoord,
        bounds: [PlotPoint; 2],
        fill: Color32,
        stroke: Option<Stroke>,
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

    pub fn line(coord: AnnotationCoord, start: PlotPoint, end: PlotPoint) -> Self {
        Self::Line {
            coord,
            start,
            end,
            stroke: None,
        }
    }

    pub fn arrow(coord: AnnotationCoord, start: PlotPoint, end: PlotPoint) -> Self {
        Self::Arrow {
            coord,
            start,
            end,
            stroke: None,
            head_size: [10.0, 8.0],
        }
    }

    pub fn double_arrow(coord: AnnotationCoord, start: PlotPoint, end: PlotPoint) -> Self {
        Self::DoubleArrow {
            coord,
            start,
            end,
            stroke: None,
            head_size: [10.0, 8.0],
        }
    }

    pub fn text_arrow(
        coord: AnnotationCoord,
        tail: PlotPoint,
        tip: PlotPoint,
        text: impl Into<Arc<str>>,
    ) -> Self {
        Self::TextArrow {
            coord,
            tail,
            tip,
            text: text.into(),
            stroke: None,
            text_color: None,
            head_size: [10.0, 8.0],
        }
    }

    pub fn textbox(
        coord: AnnotationCoord,
        bounds: [PlotPoint; 2],
        text: impl Into<Arc<str>>,
    ) -> Self {
        Self::TextBox {
            coord,
            bounds,
            text: text.into(),
            fill: Color32::TRANSPARENT,
            stroke: None,
            text_color: None,
        }
    }

    pub fn rectangle(coord: AnnotationCoord, bounds: [PlotPoint; 2]) -> Self {
        Self::Rectangle {
            coord,
            bounds,
            fill: Color32::TRANSPARENT,
            stroke: None,
        }
    }

    pub fn ellipse(coord: AnnotationCoord, bounds: [PlotPoint; 2]) -> Self {
        Self::Ellipse {
            coord,
            bounds,
            fill: Color32::TRANSPARENT,
            stroke: None,
        }
    }

    pub fn with_stroke(mut self, s: impl Into<Stroke>) -> Self {
        let s = s.into();
        match &mut self {
            Self::XLine { stroke, .. }
            | Self::YLine { stroke, .. }
            | Self::Line { stroke, .. }
            | Self::Arrow { stroke, .. }
            | Self::DoubleArrow { stroke, .. }
            | Self::TextArrow { stroke, .. }
            | Self::TextBox { stroke, .. }
            | Self::Rectangle { stroke, .. }
            | Self::Ellipse { stroke, .. } => *stroke = Some(s),
            _ => {}
        }
        self
    }

    pub fn with_fill(mut self, color: Color32) -> Self {
        match &mut self {
            Self::XRegion { fill, .. }
            | Self::YRegion { fill, .. }
            | Self::TextBox { fill, .. }
            | Self::Rectangle { fill, .. }
            | Self::Ellipse { fill, .. } => *fill = color,
            _ => {}
        }
        self
    }

    pub fn with_color(mut self, c: Color32) -> Self {
        match &mut self {
            Self::Text { color, .. } => *color = Some(c),
            Self::TextArrow { text_color, .. } | Self::TextBox { text_color, .. } => {
                *text_color = Some(c)
            }
            _ => {}
        }
        self
    }

    pub fn with_head_size(mut self, length: f32, width: f32) -> Self {
        match &mut self {
            Self::Arrow { head_size, .. }
            | Self::DoubleArrow { head_size, .. }
            | Self::TextArrow { head_size, .. } => *head_size = [length, width],
            _ => {}
        }
        self
    }
}
