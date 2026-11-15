use std::sync::Arc;

use egui::Color32;

use crate::{AxisScale, PlotBounds, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageError {
    EmptyImage,
    InvalidBounds,
    LengthMismatch,
    PaletteIndexOutOfBounds,
    InvalidDisplayRange,
    CapacityOverflow,
    AllocationFailed,
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyImage => "image width and height must be positive",
            Self::InvalidBounds => "image bounds must be finite with positive span",
            Self::LengthMismatch => "image buffer length does not match dimensions",
            Self::PaletteIndexOutOfBounds => "indexed pixel exceeds palette length",
            Self::InvalidDisplayRange => "grayscale display range must be finite and increasing",
            Self::CapacityOverflow => "image size overflow",
            Self::AllocationFailed => "image edge/pixel allocation failed",
        })
    }
}

impl std::error::Error for ImageError {}

/// Immutable host-owned 2D pixel grid in RGBA8 (unpremultiplied sRGBA) with
/// strictly increasing data-space pixel edges. Row 0 occupies the first Y span.
#[derive(Clone, Debug)]
pub struct ImageData {
    width: usize,
    height: usize,
    x_edges: Arc<[f64]>,
    y_edges: Arc<[f64]>,
    rgba: Arc<[u8]>,
    transparent_pixels: usize,
}

impl ImageData {
    pub fn rgba8(
        width: usize,
        height: usize,
        bounds: PlotBounds,
        rgba: impl Into<Arc<[u8]>>,
    ) -> Result<Self, ImageError> {
        let rgba: Arc<[u8]> = rgba.into();
        let expected = pixel_byte_len(width, height, 4)?;
        if rgba.len() != expected {
            return Err(ImageError::LengthMismatch);
        }
        let (x_edges, y_edges) = uniform_edges(width, height, bounds)?;
        let transparent_pixels = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] == 0)
            .count();
        Ok(Self {
            width,
            height,
            x_edges: Arc::from(x_edges),
            y_edges: Arc::from(y_edges),
            rgba,
            transparent_pixels,
        })
    }

    pub fn rgb8(
        width: usize,
        height: usize,
        bounds: PlotBounds,
        rgb: &[u8],
    ) -> Result<Self, ImageError> {
        let expected = pixel_byte_len(width, height, 3)?;
        if rgb.len() != expected {
            return Err(ImageError::LengthMismatch);
        }
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(pixel_byte_len(width, height, 4)?)
            .map_err(|_| ImageError::AllocationFailed)?;
        for px in rgb.as_chunks::<3>().0 {
            rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
        Self::rgba8(width, height, bounds, rgba)
    }

    /// Expand `[x_first, x_last]` / `[y_first, y_last]` pixel centers by half a pixel.
    pub fn from_pixel_centers(
        width: usize,
        height: usize,
        x_centers: [f64; 2],
        y_centers: [f64; 2],
        rgba: impl Into<Arc<[u8]>>,
    ) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::EmptyImage);
        }
        let expand = |range: [f64; 2], n: usize| -> Result<[f64; 2], ImageError> {
            if !range[0].is_finite() || !range[1].is_finite() {
                return Err(ImageError::InvalidBounds);
            }
            if n == 1 {
                if range[0] != range[1] {
                    return Err(ImageError::InvalidBounds);
                }
                return Ok([range[0] - 0.5, range[0] + 0.5]);
            }
            let step = (range[1] - range[0]) / (n - 1) as f64;
            if !step.is_finite() || step <= 0.0 {
                return Err(ImageError::InvalidBounds);
            }
            Ok([range[0] - 0.5 * step, range[1] + 0.5 * step])
        };
        let x = expand(x_centers, width)?;
        let y = expand(y_centers, height)?;
        let bounds =
            PlotBounds::new([x[0], y[0]], [x[1], y[1]]).map_err(|_| ImageError::InvalidBounds)?;
        Self::rgba8(width, height, bounds, rgba)
    }

    pub fn indexed(
        width: usize,
        height: usize,
        bounds: PlotBounds,
        indices: &[usize],
        palette: &[Color32],
    ) -> Result<Self, ImageError> {
        let count = pixel_byte_len(width, height, 1)?;
        if indices.len() != count {
            return Err(ImageError::LengthMismatch);
        }
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(pixel_byte_len(width, height, 4)?)
            .map_err(|_| ImageError::AllocationFailed)?;
        for &idx in indices {
            let color = *palette
                .get(idx)
                .ok_or(ImageError::PaletteIndexOutOfBounds)?;
            rgba.extend_from_slice(&color.to_srgba_unmultiplied());
        }
        Self::rgba8(width, height, bounds, rgba)
    }

    /// Grayscale `imshow(I, [low, high])`: clamps finite values into `[low, high]`,
    /// omitting `NaN` / `Inf` pixels as transparent (`alpha = 0`).
    pub fn grayscale(
        width: usize,
        height: usize,
        bounds: PlotBounds,
        values: &[f64],
        display_range: Option<[f64; 2]>,
    ) -> Result<Self, ImageError> {
        let count = pixel_byte_len(width, height, 1)?;
        if values.len() != count {
            return Err(ImageError::LengthMismatch);
        }
        let [lo, hi] = match display_range {
            Some([lo, hi]) => {
                if !lo.is_finite() || !hi.is_finite() || hi <= lo {
                    return Err(ImageError::InvalidDisplayRange);
                }
                [lo, hi]
            }
            None => {
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for &v in values {
                    if v.is_finite() {
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
                if !lo.is_finite() {
                    [0.0, 1.0]
                } else if hi <= lo {
                    [lo, lo + 1.0]
                } else {
                    [lo, hi]
                }
            }
        };
        let span = hi - lo;
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(pixel_byte_len(width, height, 4)?)
            .map_err(|_| ImageError::AllocationFailed)?;
        for &v in values {
            if !v.is_finite() {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let g = (((v.clamp(lo, hi) - lo) / span) * 255.0).round() as u8;
                rgba.extend_from_slice(&[g, g, g, 255]);
            }
        }
        Self::rgba8(width, height, bounds, rgba)
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn x_edges(&self) -> &[f64] {
        &self.x_edges
    }
    pub fn y_edges(&self) -> &[f64] {
        &self.y_edges
    }
    pub fn rgba_bytes(&self) -> &[u8] {
        &self.rgba
    }
    pub fn transparent_pixels(&self) -> usize {
        self.transparent_pixels
    }
    pub fn pixel(&self, col: usize, row: usize) -> Option<[u8; 4]> {
        (col < self.width && row < self.height).then(|| {
            let i = (row * self.width + col) * 4;
            [
                self.rgba[i],
                self.rgba[i + 1],
                self.rgba[i + 2],
                self.rgba[i + 3],
            ]
        })
    }
    pub fn pixel_bounds(&self, col: usize, row: usize) -> Option<PlotBounds> {
        (col < self.width && row < self.height).then(|| {
            PlotBounds::new(
                [self.x_edges[col], self.y_edges[row]],
                [self.x_edges[col + 1], self.y_edges[row + 1]],
            )
            .expect("validated pixel edges")
        })
    }
    pub fn sample_bytes(&self) -> usize {
        self.rgba.len() + (self.x_edges.len() + self.y_edges.len()) * 8
    }
    pub(crate) fn allocations(&self) -> [(usize, usize); 3] {
        [
            (self.x_edges.as_ptr() as usize, self.x_edges.len() * 8),
            (self.y_edges.as_ptr() as usize, self.y_edges.len() * 8),
            (self.rgba.as_ptr() as usize, self.rgba.len()),
        ]
    }
    pub(crate) fn domain_start(&self, axis: usize, scale: AxisScale) -> usize {
        if scale == AxisScale::Linear {
            return 0;
        }
        let edges = if axis == 0 {
            &self.x_edges
        } else {
            &self.y_edges
        };
        edges.partition_point(|v| *v <= 0.0).min(edges.len() - 1)
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        let sx = self.domain_start(0, scales[0]);
        let sy = self.domain_start(1, scales[1]);
        if sx == self.width || sy == self.height {
            return None;
        }
        Some((
            [self.x_edges[sx], self.y_edges[sy]],
            [self.x_edges[self.width], self.y_edges[self.height]],
        ))
    }
    pub(crate) fn invalid_domain_pixels(&self, scales: [AxisScale; 2]) -> usize {
        self.width * self.height
            - (self.width - self.domain_start(0, scales[0]))
                * (self.height - self.domain_start(1, scales[1]))
    }
}

fn pixel_byte_len(width: usize, height: usize, channels: usize) -> Result<usize, ImageError> {
    if width == 0 || height == 0 {
        return Err(ImageError::EmptyImage);
    }
    width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(channels))
        .ok_or(ImageError::CapacityOverflow)
}

fn uniform_edges(
    width: usize,
    height: usize,
    bounds: PlotBounds,
) -> Result<(Vec<f64>, Vec<f64>), ImageError> {
    let build = |axis: usize, cells: usize| -> Result<Vec<f64>, ImageError> {
        let len = cells.checked_add(1).ok_or(ImageError::CapacityOverflow)?;
        let mut edges = Vec::new();
        edges
            .try_reserve_exact(len)
            .map_err(|_| ImageError::AllocationFailed)?;
        for i in 0..=cells {
            edges.push(if i == cells {
                bounds.max()[axis]
            } else {
                bounds.min()[axis] + bounds.span()[axis] * (i as f64 / cells as f64)
            });
        }
        if edges.windows(2).any(|w| w[0] >= w[1]) {
            return Err(ImageError::InvalidBounds);
        }
        Ok(edges)
    };
    Ok((build(0, width)?, build(1, height)?))
}
