use super::Layer3D;
use crate::{
    AlphaShape3D, Bar3D, Bubble3D, Comet3D, ConePlot3D, Contour3D, Fill3D, Graph3D, Isocaps3D,
    Isosurface3D, Line3D, MeshSurface3D, Patch3D, Pie3D, PointCloud3D, Quiver3D, Ribbon3D,
    Scatter3D, Stem3D, StreamParticles3D, StreamRibbon3D, StreamTube3D, Streamline3D, Surface3D,
    Swarm3D, TetraMesh3D, TriSurf3D, VolumeRender3D, VolumeSlice3D,
};

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
impl<'a> From<Patch3D<'a>> for Layer3D<'a> {
    fn from(v: Patch3D<'a>) -> Self {
        Self::Patch(v)
    }
}
impl<'a> From<AlphaShape3D<'a>> for Layer3D<'a> {
    fn from(v: AlphaShape3D<'a>) -> Self {
        Self::AlphaShape(v)
    }
}
impl<'a> From<Graph3D<'a>> for Layer3D<'a> {
    fn from(v: Graph3D<'a>) -> Self {
        Self::Graph(v)
    }
}
impl<'a> From<VolumeRender3D<'a>> for Layer3D<'a> {
    fn from(v: VolumeRender3D<'a>) -> Self {
        Self::VolumeRender(v)
    }
}
impl<'a> From<PointCloud3D<'a>> for Layer3D<'a> {
    fn from(v: PointCloud3D<'a>) -> Self {
        Self::PointCloud(v)
    }
}
