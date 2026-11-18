mod bars_vecs_polys;
mod from_impls;
mod lines_scatter;
mod mesh_tri;
mod scene_opts;
mod stage13;
mod stage16;
mod summary;
mod surfaces;
mod volume_flow;

use egui::IdSalt;

pub use bars_vecs_polys::{Bar3D, Fill3D, Pie3D, Quiver3D, Stem3D};
pub use lines_scatter::{Bubble3D, Comet3D, Line3D, Scatter3D, Swarm3D};
pub use mesh_tri::{Contour3D, MeshSurface3D, MeshSurfaceStyle, TriSurf3D};
pub use scene_opts::{BoxStyle3D, ConstantPlane3D, DepthMode, Text3D};
pub use stage13::{AlphaShape3D, Graph3D, Patch3D};
pub use stage16::{PointCloud3D, VolumeRender3D};
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

/// Ordered 3D scene layers submitted to [`crate::Plot3D::show`].
///
/// 提交给 [`crate::Plot3D::show`] 的三维场景图层枚举（覆盖三维曲线/散点/柱/杆、参数曲面/网格/等值面、体切片/体渲染及八叉树点云）。
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
    Patch(Patch3D<'a>),
    AlphaShape(AlphaShape3D<'a>),
    Graph(Graph3D<'a>),
    VolumeRender(VolumeRender3D<'a>),
    PointCloud(PointCloud3D<'a>),
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
            Self::Patch(s) => s.name,
            Self::AlphaShape(s) => s.name,
            Self::Graph(s) => s.name,
            Self::VolumeRender(s) => s.name,
            Self::PointCloud(s) => s.name,
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
            Self::Patch(s) => s.id_salt,
            Self::AlphaShape(s) => s.id_salt,
            Self::Graph(s) => s.id_salt,
            Self::VolumeRender(s) => s.id_salt,
            Self::PointCloud(s) => s.id_salt,
        };
        custom.unwrap_or_else(|| IdSalt::new(("layer3d", self.name(), index)))
    }

    pub(crate) fn resolved_style(
        self,
        index: usize,
        plot: &ResolvedPlotStyle<'_>,
    ) -> ResolvedSeriesStyle {
        let mut style = self.local_style();
        if let Self::Bubble(s) = self
            && style.color.is_none()
            && let Some(cs) = s.color_scale
        {
            style.color = Some(cs.map().sample(0.5));
        }
        plot.resolve_series(index, style)
    }

    pub(crate) fn local_style(self) -> crate::SeriesStyle {
        match self {
            Self::Line(s) => s.style,
            Self::Scatter(s) => s.style,
            Self::Bubble(s) => s.style,
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
            Self::Patch(s) => s.style,
            Self::AlphaShape(s) => s.style,
            Self::Graph(s) => s.style,
            Self::VolumeRender(_) => crate::SeriesStyle::default(),
            Self::PointCloud(s) => s.style,
        }
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
            Self::Patch(s) => s.data.extent(scales),
            Self::AlphaShape(s) => s.data.extent(scales),
            Self::Graph(s) => s.data.extent(scales),
            Self::VolumeRender(s) => {
                let lo = s.volume.bounds().min();
                let hi = s.volume.bounds().max();
                if (0..3).all(|i| {
                    scales[i].forward(lo[i]).is_some() && scales[i].forward(hi[i]).is_some()
                }) {
                    Some([lo, hi])
                } else {
                    None
                }
            }
            Self::PointCloud(s) => s.data.extent(scales),
        }
    }
}
