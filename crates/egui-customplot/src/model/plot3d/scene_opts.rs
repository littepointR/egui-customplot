use egui::{Color32, Stroke};

use crate::Point3D;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthMode {
    #[default]
    SortedPainter,
    ZBuffer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoxStyle3D {
    #[default]
    BackPlanes,
    FullBox,
    AxesOnly,
}

/// 3D infinite plane `normal · [x, y, z] = offset` clipped to the axis box (`constantplane`, `EX-032`).
#[derive(Clone, Debug, PartialEq)]
pub struct ConstantPlane3D {
    pub normal: [f64; 3],
    pub offset: f64,
    pub fill: Color32,
    pub border: Option<Stroke>,
    pub grid_divisions: usize,
    pub label: String,
}

impl ConstantPlane3D {
    pub fn new(normal: [f64; 3], offset: f64) -> Self {
        Self {
            normal,
            offset,
            fill: Color32::from_rgba_unmultiplied(70, 145, 220, 72),
            border: Some(Stroke::new(1.2, Color32::from_rgb(45, 110, 180))),
            grid_divisions: 0,
            label: String::new(),
        }
    }
    pub fn x_plane(x: f64) -> Self {
        Self::new([1.0, 0.0, 0.0], x)
    }
    pub fn y_plane(y: f64) -> Self {
        Self::new([0.0, 1.0, 0.0], y)
    }
    pub fn z_plane(z: f64) -> Self {
        Self::new([0.0, 0.0, 1.0], z)
    }
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }
    pub fn border(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.border = stroke.into();
        self
    }
    pub fn grid_divisions(mut self, n: usize) -> Self {
        self.grid_divisions = n.min(32);
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

/// 3D data-space text label (`EX-026` in 3D axes).
#[derive(Clone, Debug, PartialEq)]
pub struct Text3D {
    pub position: Point3D,
    pub text: String,
    pub color: Option<Color32>,
}

impl Text3D {
    pub fn new(position: Point3D, text: impl Into<String>) -> Self {
        Self {
            position,
            text: text.into(),
            color: None,
        }
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}
