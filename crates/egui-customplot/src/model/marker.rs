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
    Asterisk,
    Pentagram,
    Hexagram,
    CircleHollow,
    SquareHollow,
    DiamondHollow,
    TriangleUpHollow,
    TriangleDownHollow,
}

impl MarkerShape {
    pub const ALL: [Self; 15] = [
        Self::Circle,
        Self::Square,
        Self::Diamond,
        Self::TriangleUp,
        Self::TriangleDown,
        Self::Plus,
        Self::Cross,
        Self::Asterisk,
        Self::Pentagram,
        Self::Hexagram,
        Self::CircleHollow,
        Self::SquareHollow,
        Self::DiamondHollow,
        Self::TriangleUpHollow,
        Self::TriangleDownHollow,
    ];

    pub const fn is_hollow(self) -> bool {
        matches!(
            self,
            Self::CircleHollow
                | Self::SquareHollow
                | Self::DiamondHollow
                | Self::TriangleUpHollow
                | Self::TriangleDownHollow
        )
    }

    pub const fn to_filled(self) -> Self {
        match self {
            Self::CircleHollow => Self::Circle,
            Self::SquareHollow => Self::Square,
            Self::DiamondHollow => Self::Diamond,
            Self::TriangleUpHollow => Self::TriangleUp,
            Self::TriangleDownHollow => Self::TriangleDown,
            other => other,
        }
    }

    pub const fn to_hollow(self) -> Self {
        match self {
            Self::Circle | Self::CircleHollow => Self::CircleHollow,
            Self::Square | Self::SquareHollow => Self::SquareHollow,
            Self::Diamond | Self::DiamondHollow => Self::DiamondHollow,
            Self::TriangleUp | Self::TriangleUpHollow => Self::TriangleUpHollow,
            Self::TriangleDown | Self::TriangleDownHollow => Self::TriangleDownHollow,
            other => other,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MarkerStyle {
    pub shape: MarkerShape,
    /// Radius in egui points, independent of axis scaling and DPI.
    pub radius: f32,
}

impl MarkerStyle {
    pub const fn new(shape: MarkerShape, radius: f32) -> Self {
        Self { shape, radius }
    }

    pub const fn hollow(shape: MarkerShape, radius: f32) -> Self {
        Self {
            shape: shape.to_hollow(),
            radius,
        }
    }
}

impl Default for MarkerStyle {
    fn default() -> Self {
        Self {
            shape: MarkerShape::Circle,
            radius: 3.5,
        }
    }
}
