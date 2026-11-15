#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerShape {
    #[default]
    Circle,
    Square,
    Diamond,
    TriangleUp,
    TriangleDown,
    Plus,
    Cross,
}

#[derive(Clone, Copy, Debug)]
pub struct MarkerStyle {
    pub shape: MarkerShape,
    /// Radius in egui points, independent of axis scaling and DPI.
    pub radius: f32,
}

impl Default for MarkerStyle {
    fn default() -> Self {
        Self {
            shape: MarkerShape::Circle,
            radius: 3.5,
        }
    }
}
