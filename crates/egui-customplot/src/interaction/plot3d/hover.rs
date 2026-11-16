use egui::{IdSalt, Pos2, Rect};

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
                for (pi, &p) in c.data.points().iter().enumerate() {
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
            Layer3D::Fill(fill) => {
                for (pi, poly) in fill.data.polygons().iter().enumerate() {
                    for &[ia, ib, ic] in poly.triangles.iter() {
                        consider_tri_region(
                            &mut best_region,
                            id,
                            li,
                            pi,
                            || format!("fill3[{pi}]"),
                            [poly.vertices[ia], poly.vertices[ib], poly.vertices[ic]],
                            transform,
                            pointer,
                        );
                    }
                }
            }
            Layer3D::Pie(pie) => {
                let h = pie.data.thickness();
                let expl = pie.data.offset();
                for slice in pie.data.slices() {
                    let mid = 0.5 * (slice.start_angle_rad + slice.end_angle_rad);
                    let shift = if slice.exploded {
                        [expl * mid.cos(), expl * mid.sin()]
                    } else {
                        [0.0, 0.0]
                    };
                    let c_top = [shift[0], shift[1], h];
                    let steps = (((slice.end_angle_rad - slice.start_angle_rad)
                        / (std::f64::consts::PI / 18.0))
                        .ceil() as usize)
                        .clamp(4, 36);
                    let rim = |i: usize| -> (Point3D, Point3D) {
                        let a = slice.start_angle_rad
                            + (slice.end_angle_rad - slice.start_angle_rad)
                                * (i as f64 / steps as f64);
                        let x = shift[0] + a.cos();
                        let y = shift[1] + a.sin();
                        ([x, y, 0.0], [x, y, h])
                    };
                    for i in 0..steps {
                        let (bi0, ti0) = rim(i);
                        let (bi1, ti1) = rim(i + 1);
                        for tri in [[c_top, ti0, ti1], [bi0, bi1, ti1], [bi0, ti1, ti0]] {
                            consider_tri_region(
                                &mut best_region,
                                id,
                                li,
                                slice.index,
                                || slice.label.clone(),
                                tri,
                                transform,
                                pointer,
                            );
                        }
                    }
                }
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

#[allow(clippy::too_many_arguments)]
fn consider_pt(
    best: &mut Option<Hovered3DPoint>,
    layer_id: IdSalt,
    layer_index: usize,
    point_index: usize,
    pos3: Point3D,
    transform: Transform3D,
    pointer: Pos2,
    radius: f32,
) {
    let Some(bounds_pt) = transform.data_to_unit(pos3) else {
        return;
    };
    if bounds_pt.iter().any(|&v| !(-1e-4..=1.0001).contains(&v)) {
        return;
    }
    let Some((screen, depth)) = transform.project_unit(bounds_pt) else {
        return;
    };
    let dist = screen.distance(pointer);
    if dist <= radius
        && best.as_ref().is_none_or(|cur| {
            depth < cur.depth - 1e-4
                || ((depth - cur.depth).abs() <= 1e-4 && dist < cur.screen_distance)
        })
    {
        *best = Some(Hovered3DPoint {
            layer_id,
            layer_index,
            point_index,
            position: pos3,
            depth,
            screen_distance: dist,
        });
    }
}

fn projected_box_aabb(lo: Point3D, hi: Point3D, transform: Transform3D) -> Option<(Rect, f32)> {
    let mut rect = Rect::NOTHING;
    let mut depth_sum = 0.0_f32;
    let mut count = 0u32;
    for ix in [lo[0], hi[0]] {
        for iy in [lo[1], hi[1]] {
            for iz in [lo[2], hi[2]] {
                let (p, d) = transform.project_data([ix, iy, iz])?;
                rect.extend_with(p);
                depth_sum += d;
                count += 1;
            }
        }
    }
    (count == 8).then(|| (rect, depth_sum / 8.0))
}

#[allow(clippy::too_many_arguments)]
fn consider_tri_region(
    best: &mut Option<Hovered3DRegion>,
    layer_id: IdSalt,
    layer_index: usize,
    item_index: usize,
    make_label: impl FnOnce() -> String,
    tri: [Point3D; 3],
    transform: Transform3D,
    pointer: Pos2,
) {
    if let (Some((pa, da)), Some((pb, db)), Some((pc, dc))) = (
        transform.project_data(tri[0]),
        transform.project_data(tri[1]),
        transform.project_data(tri[2]),
    ) && point_in_tri(pointer, pa, pb, pc)
    {
        let d = (da + db + dc) / 3.0;
        if best.as_ref().is_none_or(|cur| d < cur.depth) {
            *best = Some(Hovered3DRegion {
                layer_id,
                layer_index,
                item_index,
                label: make_label(),
                depth: d,
            });
        }
    }
}

fn point_in_tri(p: Pos2, a: Pos2, b: Pos2, c: Pos2) -> bool {
    let s1 = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
    let s2 = (c.x - b.x) * (p.y - b.y) - (c.y - b.y) * (p.x - b.x);
    let s3 = (a.x - c.x) * (p.y - c.y) - (a.y - c.y) * (p.x - c.x);
    let has_neg = s1 < 0.0 || s2 < 0.0 || s3 < 0.0;
    let has_pos = s1 > 0.0 || s2 > 0.0 || s3 > 0.0;
    !(has_neg && has_pos)
}
