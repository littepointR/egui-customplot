use crate::{AxisScale, Point3D, SeriesStyle, Transform3D};

/// Borrowed data-space batches submitted to either 3D depth backend.
pub enum GeometryBatch3D<'a> {
    Polyline(&'a [Point3D]),
    Points(&'a [Point3D]),
    Triangles(&'a [[Point3D; 3]]),
}

pub trait CustomLayer3D: std::fmt::Debug + Send + Sync {
    fn id(&self) -> egui::IdSalt;
    /// Increment when geometry or its generation policy changes for this stable ID.
    fn revision(&self) -> u64;
    fn name(&self) -> &str {
        ""
    }
    fn style(&self) -> SeriesStyle {
        SeriesStyle::default()
    }
    fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]>;
    fn batches(&self, transform: Transform3D, submit: &mut dyn FnMut(GeometryBatch3D<'_>));
    fn pick(
        &self,
        _transform: Transform3D,
        _pointer: egui::Pos2,
        _radius: f32,
    ) -> Option<(usize, Point3D)> {
        None
    }
}
