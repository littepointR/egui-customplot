use egui::{Color32, Rect};

/// Local export work limits, not a process-wide allocation or time guarantee.
#[derive(Clone, Copy, Debug)]
pub struct ExportOptions {
    /// Output pixels per captured logical point. Does not re-layout fonts/LOD.
    pub pixels_per_point: f32,
    /// PNG physical-resolution metadata; does not change pixel dimensions.
    pub dpi: f32,
    /// Composited behind the capture; does not erase painted plot backgrounds.
    pub background: Color32,
    pub max_pixels: usize,
    pub max_output_bytes: usize,
    /// Sum of clipped triangle bounding-box pixel visits, including overdraw.
    pub max_raster_steps: u64,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            pixels_per_point: 1.0,
            dpi: 96.0,
            background: Color32::TRANSPARENT,
            max_pixels: 4_194_304,
            max_output_bytes: 64 * 1024 * 1024,
            max_raster_steps: 200_000_000,
        }
    }
}

impl ExportOptions {
    pub(super) fn dimensions(self, rect: Rect) -> Result<[usize; 2], ExportError> {
        if !rect.is_finite()
            || !rect.is_positive()
            || !self.pixels_per_point.is_finite()
            || self.pixels_per_point <= 0.0
            || !self.dpi.is_finite()
            || self.dpi <= 0.0
            || self.dpi as f64 / 0.0254 > u32::MAX as f64
        {
            return Err(ExportError::InvalidDimensions);
        }
        let size = rect.size() * self.pixels_per_point;
        if !size.is_finite() || size.max_elem() > u32::MAX as f32 {
            return Err(ExportError::InvalidDimensions);
        }
        let dimensions = [size.x.ceil() as usize, size.y.ceil() as usize];
        if dimensions.contains(&0)
            || dimensions[0]
                .checked_mul(dimensions[1])
                .is_none_or(|pixels| pixels > self.max_pixels)
        {
            return Err(ExportError::LimitExceeded("output pixels"));
        }
        if additive(self.background) {
            return Err(ExportError::UnsupportedAdditiveColor);
        }
        Ok(dimensions)
    }
}

pub(super) fn additive(color: Color32) -> bool {
    color.to_array()[..3].iter().any(|c| *c > color.a())
}

#[derive(Debug)]
pub enum ExportError {
    InvalidDimensions,
    InvalidGeometry,
    LimitExceeded(&'static str),
    AllocationFailed,
    UnsupportedTexture(egui::TextureId),
    UnsupportedCallback,
    UnsupportedAdditiveColor,
    Encoding(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "plot export failed: {self:?}")
    }
}
impl std::error::Error for ExportError {}
