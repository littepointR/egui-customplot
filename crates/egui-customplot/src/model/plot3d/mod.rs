mod bars_vecs_polys;
mod lines_scatter;

use std::collections::HashSet;

use egui::{Color32, IdSalt, Stroke};

pub use bars_vecs_polys::{Bar3D, Fill3D, Pie3D, Quiver3D, Stem3D};
pub use lines_scatter::{Bubble3D, Comet3D, Line3D, Scatter3D, Swarm3D};

use crate::{
    AxisScale, PlotDiagnostics, Point3D,
    data::union_bounds3,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

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

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Layer3D<'a> {
    Line(Line3D<'a>),
    Scatter(Scatter3D<'a>),
    Bubble(Bubble3D<'a>),
    Swarm(Swarm3D<'a>),
    Comet(Comet3D<'a>),
    Bar(Bar3D<'a>),
    Stem(Stem3D<'a>),
    Pie(Pie3D<'a>),
    Quiver(Quiver3D<'a>),
    Fill(Fill3D<'a>),
}

impl<'a> From<Line3D<'a>> for Layer3D<'a> {
    fn from(v: Line3D<'a>) -> Self {
        Self::Line(v)
    }
}
impl<'a> From<Scatter3D<'a>> for Layer3D<'a> {
    fn from(v: Scatter3D<'a>) -> Self {
        Self::Scatter(v)
    }
}
impl<'a> From<Bubble3D<'a>> for Layer3D<'a> {
    fn from(v: Bubble3D<'a>) -> Self {
        Self::Bubble(v)
    }
}
impl<'a> From<Swarm3D<'a>> for Layer3D<'a> {
    fn from(v: Swarm3D<'a>) -> Self {
        Self::Swarm(v)
    }
}
impl<'a> From<Comet3D<'a>> for Layer3D<'a> {
    fn from(v: Comet3D<'a>) -> Self {
        Self::Comet(v)
    }
}
impl<'a> From<Bar3D<'a>> for Layer3D<'a> {
    fn from(v: Bar3D<'a>) -> Self {
        Self::Bar(v)
    }
}
impl<'a> From<Stem3D<'a>> for Layer3D<'a> {
    fn from(v: Stem3D<'a>) -> Self {
        Self::Stem(v)
    }
}
impl<'a> From<Pie3D<'a>> for Layer3D<'a> {
    fn from(v: Pie3D<'a>) -> Self {
        Self::Pie(v)
    }
}
impl<'a> From<Quiver3D<'a>> for Layer3D<'a> {
    fn from(v: Quiver3D<'a>) -> Self {
        Self::Quiver(v)
    }
}
impl<'a> From<Fill3D<'a>> for Layer3D<'a> {
    fn from(v: Fill3D<'a>) -> Self {
        Self::Fill(v)
    }
}

impl<'a> Layer3D<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Self::Line(s) => s.name,
            Self::Scatter(s) => s.name,
            Self::Bubble(s) => s.name,
            Self::Swarm(s) => s.name,
            Self::Comet(s) => s.name,
            Self::Bar(s) => s.name,
            Self::Stem(s) => s.name,
            Self::Pie(s) => s.name,
            Self::Quiver(s) => s.name,
            Self::Fill(s) => s.name,
        }
    }

    pub(crate) fn id(self, index: usize) -> IdSalt {
        let custom = match self {
            Self::Line(s) => s.id_salt,
            Self::Scatter(s) => s.id_salt,
            Self::Bubble(s) => s.id_salt,
            Self::Swarm(s) => s.id_salt,
            Self::Comet(s) => s.id_salt,
            Self::Bar(s) => s.id_salt,
            Self::Stem(s) => s.id_salt,
            Self::Pie(s) => s.id_salt,
            Self::Quiver(s) => s.id_salt,
            Self::Fill(s) => s.id_salt,
        };
        custom.unwrap_or_else(|| IdSalt::new(("layer3d", self.name(), index)))
    }

    pub(crate) fn resolved_style(
        self,
        index: usize,
        plot: &ResolvedPlotStyle<'_>,
    ) -> ResolvedSeriesStyle {
        let mut st = match self {
            Self::Line(s) => s.style,
            Self::Scatter(s) => s.style,
            Self::Bubble(s) => {
                let mut base = s.style;
                if base.color.is_none()
                    && let Some(cs) = s.color_scale
                {
                    base.color = Some(cs.map().sample(0.5));
                }
                base
            }
            Self::Swarm(s) => s.style,
            Self::Comet(s) => s.style,
            Self::Bar(s) => s.style,
            Self::Stem(s) => s.style,
            Self::Pie(s) => s.style,
            Self::Quiver(s) => s.style,
            Self::Fill(s) => s.style,
        };
        if let Some(c) = st.color {
            st.color = Some(c);
        }
        plot.resolve_series(index, st)
    }

    pub(crate) fn extent(self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        match self {
            Self::Line(s) => s.data.extent(scales),
            Self::Scatter(s) => s.data.extent(scales),
            Self::Bubble(s) => s.data.extent(scales),
            Self::Swarm(s) => s.data.extent(scales),
            Self::Comet(s) => s.data.extent(scales),
            Self::Bar(s) => s.data.extent(scales),
            Self::Stem(s) => s.data.extent(scales),
            Self::Pie(s) => {
                let [lo, hi] = s.data.extent();
                if (0..3).all(|i| {
                    scales[i].forward(lo[i]).is_some() && scales[i].forward(hi[i]).is_some()
                }) {
                    Some([lo, hi])
                } else {
                    None
                }
            }
            Self::Quiver(s) => s.data.extent(scales),
            Self::Fill(s) => s.data.extent(scales),
        }
    }
}

pub(crate) fn summarize_plot3d(
    layers: &[Layer3D<'_>],
    scales: [AxisScale; 3],
    visible_fn: impl Fn(IdSalt) -> bool,
) -> (PlotDiagnostics, HashSet<IdSalt>, Option<[Point3D; 2]>) {
    let mut diag = PlotDiagnostics::default();
    let mut present = HashSet::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut range: Option<[Point3D; 2]> = None;

    for (idx, &layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        present.insert(id);
        let (alloc, pts, prims, non_fin, dom_inv) = match layer {
            Layer3D::Line(Line3D { data: s, .. })
            | Layer3D::Scatter(Scatter3D { data: s, .. })
            | Layer3D::Comet(Comet3D { data: s, .. }) => (
                s.allocation(),
                s.len(),
                0,
                s.non_finite_points(),
                s.domain_invalid(scales),
            ),
            Layer3D::Bubble(s) => (
                s.data.allocation(),
                s.data.points().len(),
                s.data.points().len(),
                s.data.non_finite_points(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Swarm(s) => (
                s.data.allocation(),
                s.data.len(),
                s.data.len(),
                s.data.non_finite_points(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Bar(s) => (
                s.data.allocation(),
                0,
                s.data.boxes().len(),
                s.data.non_finite_items(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Stem(s) => (
                s.data.allocation(),
                s.data.points().len(),
                s.data.points().len(),
                s.data.non_finite_points(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Pie(s) => (
                s.data.allocation(),
                0,
                s.data.slices().len(),
                0,
                if scales.contains(&AxisScale::Log10) {
                    s.data.slices().len()
                } else {
                    0
                },
            ),
            Layer3D::Quiver(s) => (
                s.data.allocation(),
                0,
                s.data.arrows().len(),
                s.data.non_finite_arrows(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Fill(s) => (
                s.data.allocation(),
                0,
                s.data.polygons().len(),
                0,
                s.data.domain_invalid(scales),
            ),
        };
        diag.plot3d.input_points += pts;
        diag.plot3d.input_primitives += prims;
        diag.plot3d.non_finite_items += non_fin;
        diag.plot3d.domain_invalid_items += dom_inv;
        if seen.insert(alloc) {
            diag.plot3d.unique_bytes += alloc.1;
        }
        if visible_fn(id) {
            diag.visible_series += 1;
            range = union_bounds3(range, layer.extent(scales));
        }
    }
    (diag, present, range)
}
