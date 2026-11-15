use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{AxisScale, LinePattern, MarkerStyle, PlotData, PlotPoint, SeriesStyle, YAxis};

/// Which coordinate varies between a scalar baseline and a raw XY endpoint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StemOrientation {
    #[default]
    Vertical,
    Horizontal,
}

impl StemOrientation {
    pub(crate) fn axis(self) -> usize {
        match self {
            Self::Vertical => 1,
            Self::Horizontal => 0,
        }
    }
}

/// Independent rods from a scalar baseline to raw XY endpoints, with markers
/// enabled by default. Does not connect neighboring samples or retain view state.
/// The baseline and endpoint must both be valid in the coordinate domain.
#[derive(Clone, Copy, Debug)]
pub struct Stems<'a> {
    pub(crate) data: &'a PlotData,
    pub(crate) baseline: f64,
    pub(crate) orientation: StemOrientation,
    pub(crate) show_baseline: bool,
    pub(crate) baseline_stroke: Option<Stroke>,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
}

impl<'a> Stems<'a> {
    pub fn new(data: &'a PlotData) -> Self {
        Self {
            data,
            baseline: 0.0,
            orientation: StemOrientation::Vertical,
            show_baseline: true,
            baseline_stroke: None,
            style: SeriesStyle {
                line_markers: Some(true),
                ..Default::default()
            },
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
        }
    }
    pub fn data(self) -> &'a PlotData {
        self.data
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
    /// Y for vertical stems and X for horizontal stems, in data coordinates.
    /// Default zero is invalid on a log value axis; use a positive baseline.
    pub fn baseline(mut self, baseline: f64) -> Self {
        self.baseline = baseline;
        self
    }
    /// Does not swap the endpoint's raw X and Y coordinates.
    pub fn orientation(mut self, orientation: StemOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// The baseline spans valid raw endpoints, not the viewport. It is not a
    /// pickable sample. Hiding it does not remove the rods' baseline from bounds.
    pub fn show_baseline(mut self, show: bool) -> Self {
        self.show_baseline = show;
        self
    }
    /// Baseline style is independent of the rod's dash pattern; it stays solid.
    pub fn baseline_stroke(mut self, stroke: Stroke) -> Self {
        self.baseline_stroke = Some(stroke);
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
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn markers(mut self, markers: MarkerStyle) -> Self {
        self.style.line_markers = Some(true);
        self.style.marker_shape = Some(markers.shape);
        self.style.marker_radius = Some(markers.radius);
        self
    }
    pub fn show_markers(mut self, show: bool) -> Self {
        self.style.line_markers = Some(show);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }

    pub(crate) fn baseline_valid(self, scales: [AxisScale; 2]) -> bool {
        scales[self.orientation.axis()]
            .forward(self.baseline)
            .is_some()
    }

    pub(crate) fn extent(self, scales: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        if !self.baseline_valid(scales) {
            return None;
        }
        let (mut min, mut max) = self.data.extent(scales)?;
        let axis = self.orientation.axis();
        min[axis] = min[axis].min(self.baseline);
        max[axis] = max[axis].max(self.baseline);
        Some((min, max))
    }
}
