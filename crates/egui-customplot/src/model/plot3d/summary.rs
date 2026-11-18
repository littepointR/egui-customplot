use std::collections::HashSet;

use egui::IdSalt;

use crate::{AxisScale, Comet3D, Line3D, PlotDiagnostics, Point3D, Scatter3D, data::union_bounds3};

use super::Layer3D;

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
            Layer3D::Custom(_) => ((0, 0), 0, 0, 0, 0),
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
            Layer3D::Surface(s) => (
                s.data.allocation(),
                s.data.points().len(),
                (s.data.rows() - 1) * (s.data.cols() - 1),
                s.data.non_finite_points(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::MeshSurface(s) => (
                s.data.allocation(),
                s.data.points().len(),
                (s.data.rows() - 1) * (s.data.cols() - 1),
                s.data.non_finite_points(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Contour(s) => (
                s.data.allocation(),
                s.data.lines().iter().map(|l| l.points.len()).sum(),
                s.data.lines().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::Ribbon(s) => (
                s.data.allocation(),
                s.data.strips().iter().map(|st| st.points().len()).sum(),
                s.data.strips().len(),
                s.data
                    .strips()
                    .iter()
                    .map(|st| st.non_finite_points())
                    .sum(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::TriSurf(s) => match (s.data, s.implicit) {
                (Some(m), _) => (
                    m.allocation(),
                    m.vertices().len(),
                    m.triangles().len(),
                    0,
                    m.domain_invalid(scales),
                ),
                (None, Some(imp)) => (imp.allocation(), 0, 0, 0, imp.domain_invalid(scales)),
                (None, None) => ((0, 0), 0, 0, 0, 0),
            },
            Layer3D::VolumeSlice(s) => (
                s.data.allocation(),
                s.data.slices().iter().map(|sl| sl.points().len()).sum(),
                s.data
                    .slices()
                    .iter()
                    .map(|sl| (sl.rows() - 1) * (sl.cols() - 1))
                    .sum(),
                s.data
                    .slices()
                    .iter()
                    .map(|sl| sl.non_finite_points())
                    .sum(),
                s.data.domain_invalid(scales),
            ),
            Layer3D::Isosurface(s) => match s.data.mesh() {
                Some(m) => (
                    m.allocation(),
                    m.vertices().len(),
                    m.triangles().len(),
                    0,
                    s.data.domain_invalid(scales),
                ),
                None => ((0, 0), 0, 0, 0, 0),
            },
            Layer3D::Isocaps(s) => match s.data.mesh() {
                Some(m) => (
                    m.allocation(),
                    m.vertices().len(),
                    m.triangles().len(),
                    0,
                    s.data.domain_invalid(scales),
                ),
                None => ((0, 0), 0, 0, 0, 0),
            },
            Layer3D::TetraMesh(s) => (
                s.data.allocation(),
                s.data.vertices().len(),
                if s.boundary_only {
                    s.data.boundary_faces().len()
                } else {
                    s.data.all_faces().len()
                },
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::Streamline(s) => (
                s.data.allocation(),
                s.data.lines().iter().map(|l| l.points.len()).sum(),
                s.data.lines().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::StreamParticles(s) => (
                s.data.allocation(),
                s.data.streamlines().lines().len() * s.data.particles_per_line(),
                s.data.streamlines().lines().len() * s.data.particles_per_line(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::StreamRibbon(s) => (
                s.data.allocation(),
                s.data.strips().iter().map(|st| st.points().len()).sum(),
                s.data.strips().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::StreamTube(s) => (
                s.data.allocation(),
                s.data.tubes().iter().map(|tb| tb.points().len()).sum(),
                s.data.tubes().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::ConePlot(s) => (
                s.data.allocation(),
                s.data.cones().len(),
                s.data.cones().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::Patch(s) => (
                s.data.allocation(),
                s.data.vertices().len(),
                s.data.triangles().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::AlphaShape(s) => (
                s.data.allocation(),
                s.data.mesh().map_or(0, |m| m.vertices().len()),
                s.data.boundary_faces().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::Graph(s) => (
                s.data.allocation(),
                s.data.node_count(),
                s.data.edges().len(),
                0,
                s.data.domain_invalid(scales),
            ),
            Layer3D::VolumeRender(s) => (
                (
                    s.volume.values().as_ptr() as usize,
                    s.volume.values().len() * 8,
                ),
                s.volume.values().len(),
                1,
                s.volume.non_finite(),
                0,
            ),
            Layer3D::PointCloud(s) => (
                s.data.allocation(),
                s.data.len(),
                s.data.len(),
                s.data.non_finite(),
                s.data.domain_invalid(scales),
            ),
        };
        diag.plot3d.input_points += pts;
        diag.plot3d.input_primitives += prims;
        diag.plot3d.non_finite_items += non_fin;
        diag.plot3d.domain_invalid_items += dom_inv;
        if alloc.1 > 0 && seen.insert(alloc) {
            diag.plot3d.unique_bytes += alloc.1;
        }
        if visible_fn(id) {
            diag.visible_series += 1;
            range = union_bounds3(range, layer.extent(scales));
        }
    }
    (diag, present, range)
}
