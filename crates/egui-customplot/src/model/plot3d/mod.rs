mod bars_vecs_polys;
mod lines_scatter;
mod mesh_tri;
mod scene_opts;
mod summary;
mod surfaces;
mod volume_flow;

use egui::IdSalt;

pub use bars_vecs_polys::{Bar3D, Fill3D, Pie3D, Quiver3D, Stem3D};
pub use lines_scatter::{Bubble3D, Comet3D, Line3D, Scatter3D, Swarm3D};
pub use mesh_tri::{Contour3D, MeshSurface3D, MeshSurfaceStyle, TriSurf3D};
pub use scene_opts::{BoxStyle3D, ConstantPlane3D, DepthMode, Text3D};
pub(crate) use summary::summarize_plot3d;
pub use surfaces::{LightSource3D, Ribbon3D, Surface3D, SurfaceShading};
pub use volume_flow::{
    ConePlot3D, Isocaps3D, Isosurface3D, StreamParticles3D, StreamRibbon3D, StreamTube3D,
    Streamline3D, TetraMesh3D, VolumeSlice3D,
};

use crate::{
    AxisScale, Point3D,
    data::union_bounds3,
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

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
    Surface(Surface3D<'a>),
    MeshSurface(MeshSurface3D<'a>),
    Contour(Contour3D<'a>),
    Ribbon(Ribbon3D<'a>),
    TriSurf(TriSurf3D<'a>),
    VolumeSlice(VolumeSlice3D<'a>),
    Isosurface(Isosurface3D<'a>),
    Isocaps(Isocaps3D<'a>),
    TetraMesh(TetraMesh3D<'a>),
    Streamline(Streamline3D<'a>),
    StreamParticles(StreamParticles3D<'a>),
    StreamRibbon(StreamRibbon3D<'a>),
    StreamTube(StreamTube3D<'a>),
    ConePlot(ConePlot3D<'a>),
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
impl<'a> From<Surface3D<'a>> for Layer3D<'a> {
    fn from(v: Surface3D<'a>) -> Self {
        Self::Surface(v)
    }
}
impl<'a> From<MeshSurface3D<'a>> for Layer3D<'a> {
    fn from(v: MeshSurface3D<'a>) -> Self {
        Self::MeshSurface(v)
    }
}
impl<'a> From<Contour3D<'a>> for Layer3D<'a> {
    fn from(v: Contour3D<'a>) -> Self {
        Self::Contour(v)
    }
}
impl<'a> From<Ribbon3D<'a>> for Layer3D<'a> {
    fn from(v: Ribbon3D<'a>) -> Self {
        Self::Ribbon(v)
    }
}
impl<'a> From<TriSurf3D<'a>> for Layer3D<'a> {
    fn from(v: TriSurf3D<'a>) -> Self {
        Self::TriSurf(v)
    }
}
impl<'a> From<VolumeSlice3D<'a>> for Layer3D<'a> {
    fn from(v: VolumeSlice3D<'a>) -> Self {
        Self::VolumeSlice(v)
    }
}
impl<'a> From<Isosurface3D<'a>> for Layer3D<'a> {
    fn from(v: Isosurface3D<'a>) -> Self {
        Self::Isosurface(v)
    }
}
impl<'a> From<Isocaps3D<'a>> for Layer3D<'a> {
    fn from(v: Isocaps3D<'a>) -> Self {
        Self::Isocaps(v)
    }
}
impl<'a> From<TetraMesh3D<'a>> for Layer3D<'a> {
    fn from(v: TetraMesh3D<'a>) -> Self {
        Self::TetraMesh(v)
    }
}
impl<'a> From<Streamline3D<'a>> for Layer3D<'a> {
    fn from(v: Streamline3D<'a>) -> Self {
        Self::Streamline(v)
    }
}
impl<'a> From<StreamParticles3D<'a>> for Layer3D<'a> {
    fn from(v: StreamParticles3D<'a>) -> Self {
        Self::StreamParticles(v)
    }
}
impl<'a> From<StreamRibbon3D<'a>> for Layer3D<'a> {
    fn from(v: StreamRibbon3D<'a>) -> Self {
        Self::StreamRibbon(v)
    }
}
impl<'a> From<StreamTube3D<'a>> for Layer3D<'a> {
    fn from(v: StreamTube3D<'a>) -> Self {
        Self::StreamTube(v)
    }
}
impl<'a> From<ConePlot3D<'a>> for Layer3D<'a> {
    fn from(v: ConePlot3D<'a>) -> Self {
        Self::ConePlot(v)
    }
}

impl<'a> Layer3D<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Self::Line(s) => {
                if s.primitive {
                    ""
                } else {
                    s.name
                }
            }
            Self::Scatter(s) => s.name,
            Self::Bubble(s) => s.name,
            Self::Swarm(s) => s.name,
            Self::Comet(s) => s.name,
            Self::Bar(s) => s.name,
            Self::Stem(s) => s.name,
            Self::Pie(s) => s.name,
            Self::Quiver(s) => s.name,
            Self::Fill(s) => s.name,
            Self::Surface(s) => {
                if s.primitive {
                    ""
                } else {
                    s.name
                }
            }
            Self::MeshSurface(s) => s.name,
            Self::Contour(s) => s.name,
            Self::Ribbon(s) => s.name,
            Self::TriSurf(s) => s.name,
            Self::VolumeSlice(s) => s.name,
            Self::Isosurface(s) => s.name,
            Self::Isocaps(s) => s.name,
            Self::TetraMesh(s) => s.name,
            Self::Streamline(s) => s.name,
            Self::StreamParticles(s) => s.name,
            Self::StreamRibbon(s) => s.name,
            Self::StreamTube(s) => s.name,
            Self::ConePlot(s) => s.name,
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
            Self::Surface(s) => s.id_salt,
            Self::MeshSurface(s) => s.id_salt,
            Self::Contour(s) => s.id_salt,
            Self::Ribbon(s) => s.id_salt,
            Self::TriSurf(s) => s.id_salt,
            Self::VolumeSlice(s) => s.id_salt,
            Self::Isosurface(s) => s.id_salt,
            Self::Isocaps(s) => s.id_salt,
            Self::TetraMesh(s) => s.id_salt,
            Self::Streamline(s) => s.id_salt,
            Self::StreamParticles(s) => s.id_salt,
            Self::StreamRibbon(s) => s.id_salt,
            Self::StreamTube(s) => s.id_salt,
            Self::ConePlot(s) => s.id_salt,
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
            Self::Surface(s) => s.style,
            Self::MeshSurface(s) => s.style,
            Self::Contour(s) => s.style,
            Self::Ribbon(s) => s.style,
            Self::TriSurf(s) => s.style,
            Self::VolumeSlice(s) => s.style,
            Self::Isosurface(s) => s.style,
            Self::Isocaps(s) => s.style,
            Self::TetraMesh(s) => s.style,
            Self::Streamline(s) => s.style,
            Self::StreamParticles(s) => s.style,
            Self::StreamRibbon(s) => s.style,
            Self::StreamTube(s) => s.style,
            Self::ConePlot(s) => s.style,
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
            Self::Surface(s) => {
                let mut b = s.data.extent(scales);
                if s.floor_contours
                    && let Some(fz) = s.floor_z
                    && scales[2].forward(fz).is_some()
                    && let Some([lo, _]) = b
                {
                    b = union_bounds3(b, Some([[lo[0], lo[1], fz], [lo[0], lo[1], fz]]));
                }
                b
            }
            Self::MeshSurface(s) => {
                let mut b = s.data.extent(scales);
                for opt in [s.floor_contours.then_some(s.floor_z).flatten(), s.base_z] {
                    if let Some(z_extra) = opt
                        && scales[2].forward(z_extra).is_some()
                        && let Some([lo, _]) = b
                    {
                        b = union_bounds3(
                            b,
                            Some([[lo[0], lo[1], z_extra], [lo[0], lo[1], z_extra]]),
                        );
                    }
                }
                b
            }
            Self::Contour(s) => s.data.extent(scales),
            Self::Ribbon(s) => s.data.extent(scales),
            Self::TriSurf(s) => s
                .data
                .and_then(|m| m.extent(scales))
                .or_else(|| s.implicit.and_then(|imp| imp.extent(scales))),
            Self::VolumeSlice(s) => s.data.extent(scales),
            Self::Isosurface(s) => s.data.extent(scales),
            Self::Isocaps(s) => s.data.extent(scales),
            Self::TetraMesh(s) => s.data.extent(scales),
            Self::Streamline(s) => s.data.extent(scales),
            Self::StreamParticles(s) => s.data.extent(scales),
            Self::StreamRibbon(s) => s.data.extent(scales),
            Self::StreamTube(s) => s.data.extent(scales),
            Self::ConePlot(s) => s.data.extent(scales),
        }
    }
}
