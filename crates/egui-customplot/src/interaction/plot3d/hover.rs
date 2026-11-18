use egui::{IdSalt, Pos2};

use super::Plot3DState;
use crate::{Layer3D, PlotDiagnostics, Point3D, Transform3D};

#[derive(Clone, Debug, PartialEq)]
pub struct Hovered3DPoint {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub point_index: usize,
    pub position: Point3D,
    pub depth: f32,
    pub screen_distance: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hovered3DBar {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub row: usize,
    pub col: usize,
    pub value: f64,
    pub depth: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hovered3DRegion {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub item_index: usize,
    pub label: String,
    pub depth: f32,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Plot3DHoverPick {
    pub point: Option<Hovered3DPoint>,
    pub bar: Option<Hovered3DBar>,
    pub region: Option<Hovered3DRegion>,
    pub text: Option<String>,
}

/// Depth-aware 3D hover picking: selects foreground (smallest `depth`) within `pick_radius`.
pub(crate) fn pick_plot3d_hover(
    state: &Plot3DState,
    layers: &[Layer3D<'_>],
    transform: Transform3D,
    pointer: Pos2,
    pick_radius: f32,
    diagnostics: &mut PlotDiagnostics,
) -> Plot3DHoverPick {
    diagnostics.plot3d.pick_queries += 1;
    let mut best_pt: Option<Hovered3DPoint> = None;

    for (li, &layer) in layers.iter().enumerate() {
        let id = layer.id(li);
        if !state.layer_visible(id) {
            continue;
        }
        match layer {
            Layer3D::Custom(s) => {
                if let Some((index, p)) = s.pick(transform, pointer, pick_radius) {
                    consider_pt(
                        &mut best_pt,
                        id,
                        li,
                        index,
                        p,
                        transform,
                        pointer,
                        pick_radius,
                    );
                }
            }
            Layer3D::Line(s) => {
                if s.primitive {
                    continue;
                }
                for (pi, &p) in s.data.points().iter().enumerate() {
                    consider_pt(&mut best_pt, id, li, pi, p, transform, pointer, pick_radius);
                }
            }
            Layer3D::Scatter(s) => {
                let rad = pick_radius.max(s.style.marker_radius.unwrap_or(4.0));
                for (pi, &p) in s.data.points().iter().enumerate() {
                    consider_pt(&mut best_pt, id, li, pi, p, transform, pointer, rad);
                }
            }
            Layer3D::Bubble(b) => {
                for (pi, pt) in b.data.points().iter().enumerate() {
                    let rad = pick_radius.max(b.data.pixel_radius(pt.size));
                    consider_pt(
                        &mut best_pt,
                        id,
                        li,
                        pi,
                        pt.position,
                        transform,
                        pointer,
                        rad,
                    );
                }
            }
            Layer3D::Swarm(s) => {
                let rad = pick_radius.max(s.style.marker_radius.unwrap_or(3.5));
                for (pi, pt) in s.data.points().iter().enumerate() {
                    consider_pt(
                        &mut best_pt,
                        id,
                        li,
                        pt.raw_index,
                        pt.position,
                        transform,
                        pointer,
                        rad,
                    );
                    let _ = pi;
                }
            }
            Layer3D::Comet(c) => {
                for (pi, &p) in c.visible_points().iter().enumerate() {
                    consider_pt(&mut best_pt, id, li, pi, p, transform, pointer, pick_radius);
                }
            }
            Layer3D::Stem(s) => {
                for (pi, &p) in s.data.points().iter().enumerate() {
                    consider_pt(&mut best_pt, id, li, pi, p, transform, pointer, pick_radius);
                }
            }
            Layer3D::Quiver(q) => {
                let scale = q.data.scale_factor();
                for (ai, &arr) in q.data.arrows().iter().enumerate() {
                    consider_pt(
                        &mut best_pt,
                        id,
                        li,
                        ai,
                        arr.origin,
                        transform,
                        pointer,
                        pick_radius,
                    );
                    if let Some(end) = arr.endpoint(scale) {
                        consider_pt(
                            &mut best_pt,
                            id,
                            li,
                            ai,
                            end,
                            transform,
                            pointer,
                            pick_radius,
                        );
                    }
                }
            }
            Layer3D::Bar(_) | Layer3D::Pie(_) | Layer3D::Fill(_) => {}
            Layer3D::Surface(_)
            | Layer3D::MeshSurface(_)
            | Layer3D::Contour(_)
            | Layer3D::Ribbon(_)
            | Layer3D::TriSurf(_) => {
                super::surface_hover::pick_surface_points(
                    layer,
                    id,
                    li,
                    transform,
                    pointer,
                    pick_radius,
                    &mut best_pt,
                );
            }
            Layer3D::Streamline(_) | Layer3D::StreamParticles(_) | Layer3D::ConePlot(_) => {
                super::volume_hover::pick_volume_points(
                    layer,
                    id,
                    li,
                    transform,
                    pointer,
                    pick_radius,
                    &mut best_pt,
                );
            }
            Layer3D::VolumeSlice(_)
            | Layer3D::Isosurface(_)
            | Layer3D::Isocaps(_)
            | Layer3D::TetraMesh(_)
            | Layer3D::StreamRibbon(_)
            | Layer3D::StreamTube(_)
            | Layer3D::Patch(_)
            | Layer3D::AlphaShape(_)
            | Layer3D::VolumeRender(_) => {}
            Layer3D::Graph(_) | Layer3D::PointCloud(_) => {
                super::stage13_hover::pick_stage13_points(
                    layer,
                    id,
                    li,
                    transform,
                    pointer,
                    pick_radius,
                    &mut best_pt,
                );
            }
        }
    }

    let mut best_bar: Option<Hovered3DBar> = None;
    for (li, &layer) in layers.iter().enumerate() {
        let id = layer.id(li);
        if !state.layer_visible(id) {
            continue;
        }
        if let Layer3D::Bar(b) = layer {
            for bx in b.data.boxes() {
                if let Some((screen_box, center_depth)) =
                    projected_box_aabb(bx.min, bx.max, transform)
                    && screen_box.contains(pointer)
                    && best_bar.as_ref().is_none_or(|cur| center_depth < cur.depth)
                {
                    best_bar = Some(Hovered3DBar {
                        layer_id: id,
                        layer_index: li,
                        row: bx.row,
                        col: bx.col,
                        value: bx.value,
                        depth: center_depth,
                    });
                }
            }
        }
    }

    let mut best_region: Option<Hovered3DRegion> = None;
    for (li, &layer) in layers.iter().enumerate() {
        let id = layer.id(li);
        if !state.layer_visible(id) {
            continue;
        }
        match layer {
            Layer3D::Fill(_)
            | Layer3D::Pie(_)
            | Layer3D::Surface(_)
            | Layer3D::Ribbon(_)
            | Layer3D::TriSurf(_) => {
                super::surface_hover::pick_extra_regions(
                    layer,
                    id,
                    li,
                    transform,
                    pointer,
                    &mut best_region,
                );
            }
            Layer3D::VolumeSlice(_)
            | Layer3D::Isosurface(_)
            | Layer3D::Isocaps(_)
            | Layer3D::TetraMesh(_)
            | Layer3D::StreamRibbon(_)
            | Layer3D::StreamTube(_)
            | Layer3D::ConePlot(_) => {
                super::volume_hover::pick_volume_regions(
                    layer,
                    id,
                    li,
                    transform,
                    pointer,
                    &mut best_region,
                );
            }
            Layer3D::Patch(_)
            | Layer3D::AlphaShape(_)
            | Layer3D::Graph(_)
            | Layer3D::VolumeRender(_) => {
                super::stage13_hover::pick_stage13_regions(
                    layer,
                    id,
                    li,
                    transform,
                    pointer,
                    &mut best_region,
                );
            }
            _ => {}
        }
    }

    if let Some(pt) = best_pt
        && best_bar.as_ref().is_none_or(|b| pt.depth <= b.depth)
        && best_region.as_ref().is_none_or(|r| pt.depth <= r.depth)
    {
        let name = layers[pt.layer_index].name();
        let text = if name.is_empty() {
            format!(
                "[{}] x={:.3}  y={:.3}  z={:.3}",
                pt.point_index, pt.position[0], pt.position[1], pt.position[2]
            )
        } else {
            format!(
                "{name}\n[{}] x={:.3}  y={:.3}  z={:.3}",
                pt.point_index, pt.position[0], pt.position[1], pt.position[2]
            )
        };
        return Plot3DHoverPick {
            point: Some(pt),
            text: Some(text),
            ..Default::default()
        };
    }
    if let Some(bar) = best_bar
        && best_region.as_ref().is_none_or(|r| bar.depth <= r.depth)
    {
        let text = format!("bar[{}, {}] = {:.4}", bar.row, bar.col, bar.value);
        return Plot3DHoverPick {
            bar: Some(bar),
            text: Some(text),
            ..Default::default()
        };
    }
    if let Some(reg) = best_region {
        let text = reg.label.clone();
        return Plot3DHoverPick {
            region: Some(reg),
            text: Some(text),
            ..Default::default()
        };
    }
    Plot3DHoverPick::default()
}

pub(super) use super::pick_geom::{consider_pt, consider_tri_region, projected_box_aabb};
