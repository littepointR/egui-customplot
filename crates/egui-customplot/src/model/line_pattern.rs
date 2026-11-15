/// Screen-space line style. Dash phase restarts at each clipped visible path or
/// domain break. Dash lengths are egui points, not data units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LinePattern {
    #[default]
    Solid,
    /// Each length must be finite and >= 0.5 points. Invalid patterns omit the
    /// stroke (not its markers) and increment numeric_rejections. At most 100,000
    /// visible dash paths are prepared per series; truncation is diagnostic.
    Dashed { length: f32, gap: f32 },
}

impl LinePattern {
    pub(crate) fn is_valid(self) -> bool {
        match self {
            Self::Solid => true,
            Self::Dashed { length, gap } => {
                length.is_finite() && gap.is_finite() && length >= 0.5 && gap >= 0.5
            }
        }
    }
}
