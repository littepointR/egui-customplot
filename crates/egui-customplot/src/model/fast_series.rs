use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use super::{LineInterpolation, LinePattern, LineSampling, RenderBackend, SeriesStyle};
use crate::{IndexedStream, UniformWaveform, YAxis};

/// Borrowed descriptor over an [`IndexedStream`] for zero-copy streaming plots.
#[derive(Clone, Copy, Debug)]
pub struct StreamLine<'a> {
    pub(crate) data: &'a IndexedStream,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) style: SeriesStyle,
    pub(crate) sampling: LineSampling,
    pub(crate) backend: RenderBackend,
    pub(crate) interpolation: LineInterpolation,
    pub(crate) y_axis: YAxis,
}

impl<'a> StreamLine<'a> {
    pub fn new(data: &'a IndexedStream) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
            sampling: LineSampling::Auto,
            backend: RenderBackend::Auto,
            interpolation: LineInterpolation::Linear,
            y_axis: YAxis::Left,
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        let stroke = stroke.into();
        self.style.color = Some(stroke.color);
        self.style.line_width = Some(stroke.width);
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn sampling(mut self, sampling: LineSampling) -> Self {
        self.sampling = sampling;
        self
    }
    pub fn backend(mut self, backend: RenderBackend) -> Self {
        self.backend = backend;
        self
    }
    pub fn interpolation(mut self, interpolation: LineInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }
}

/// Borrowed descriptor over a [`UniformWaveform`] for $O(1)$-sliced waveform plots.
#[derive(Clone, Copy, Debug)]
pub struct WaveformLine<'a> {
    pub(crate) data: &'a UniformWaveform,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) style: SeriesStyle,
    pub(crate) sampling: LineSampling,
    pub(crate) backend: RenderBackend,
    pub(crate) interpolation: LineInterpolation,
    pub(crate) y_axis: YAxis,
}

impl<'a> WaveformLine<'a> {
    pub fn new(data: &'a UniformWaveform) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
            sampling: LineSampling::Auto,
            backend: RenderBackend::Auto,
            interpolation: LineInterpolation::Linear,
            y_axis: YAxis::Left,
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        let stroke = stroke.into();
        self.style.color = Some(stroke.color);
        self.style.line_width = Some(stroke.width);
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn sampling(mut self, sampling: LineSampling) -> Self {
        self.sampling = sampling;
        self
    }
    pub fn backend(mut self, backend: RenderBackend) -> Self {
        self.backend = backend;
        self
    }
    pub fn interpolation(mut self, interpolation: LineInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }
}
