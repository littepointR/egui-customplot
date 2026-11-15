use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{AreaBandData, AxisScale, LinePattern, PlotData, PlotPoint, SeriesStyle, YAxis};

#[derive(Clone, Copy, Debug)]
pub(crate) enum AreaSource<'a> {
    Curve { data: &'a PlotData, baseline: f64 },
    Band(&'a AreaBandData),
}

/// Filled area between a curve and baseline or between lower/upper stacked bands.
#[derive(Clone, Copy, Debug)]
pub struct Area<'a> {
    pub(crate) source: AreaSource<'a>,
    pub(crate) fill_color: Option<Color32>,
    pub(crate) show_stroke: bool,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Area<'a> {
    pub fn new(data: &'a PlotData) -> Self {
        Self {
            source: AreaSource::Curve {
                data,
                baseline: 0.0,
            },
            fill_color: None,
            show_stroke: true,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }

    pub fn from_band(band: &'a AreaBandData) -> Self {
        Self {
            source: AreaSource::Band(band),
            fill_color: None,
            show_stroke: true,
            style: SeriesStyle::default(),
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    /// Scalar baseline for single-curve `Area::new`. Ignored for `Area::from_band`.
    /// Default zero is invalid on a log Y-axis; provide a positive baseline.
    pub fn baseline(mut self, baseline: f64) -> Self {
        if let AreaSource::Curve {
            baseline: target, ..
        } = &mut self.source
        {
            *target = baseline;
        }
        self
    }
    pub fn fill_color(mut self, color: Color32) -> Self {
        self.fill_color = Some(color);
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
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        let stroke = stroke.into();
        self.style.color = Some(stroke.color);
        self.style.line_width = Some(stroke.width);
        self
    }
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn show_stroke(mut self, show: bool) -> Self {
        self.show_stroke = show;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }

    pub(crate) fn resolved_fill(self, stroke_color: Color32) -> Color32 {
        self.fill_color
            .unwrap_or_else(|| stroke_color.gamma_multiply(0.35))
    }

    pub(crate) fn baseline_valid(self, scales: [AxisScale; 2]) -> bool {
        match self.source {
            AreaSource::Curve { baseline, .. } => {
                baseline.is_finite() && scales[1].forward(baseline).is_some()
            }
            AreaSource::Band(_) => true,
        }
    }

    pub(crate) fn extent(self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        match self.source {
            AreaSource::Curve { data, baseline } => {
                if !self.baseline_valid(scales) {
                    return None;
                }
                let (mut lo, mut hi) = data.extent(scales)?;
                lo[1] = lo[1].min(baseline);
                hi[1] = hi[1].max(baseline);
                Some((lo, hi))
            }
            AreaSource::Band(band) => band.extent(scales),
        }
    }
}
