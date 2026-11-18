//! Borrowed extension point for an ordered 2D layer.
use crate::{AxisScale, PlotPoint, PlotTransform, SeriesStyle, YAxis};
use egui::{IdSalt, Painter, Pos2, Stroke};

/// Per-batch drawing context shared by interactive painting and snapshot replay.
pub struct LayerPaintContext<'a> {
    pub painter: &'a Painter,
    pub transform: PlotTransform,
    pub stroke: Stroke,
    pub pixels_per_point: f32,
    /// Emit capture-compatible egui shapes here when the interactive path uses a GPU callback.
    pub exporting: bool,
}

/// Host-owned data and geometry. Each visible layer is repainted in submission order.
/// Cache implementations belong to the host and must account for data, style and transform changes.
pub trait CustomPlotLayer: std::fmt::Debug + Send + Sync {
    fn id(&self) -> IdSalt;
    fn name(&self) -> &str {
        ""
    }
    fn y_axis(&self) -> YAxis {
        YAxis::Left
    }
    fn style(&self) -> SeriesStyle {
        SeriesStyle::default()
    }
    /// Finite min/max of drawable samples for the supplied axis domains.
    fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)>;
    fn paint(&self, context: &LayerPaintContext<'_>);
    /// Return the raw index and data coordinates of the nearest eligible anchor.
    fn pick(
        &self,
        _transform: PlotTransform,
        _pointer: Pos2,
        _radius: f32,
    ) -> Option<(usize, PlotPoint)> {
        None
    }
}
