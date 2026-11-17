use egui::{IdSalt, Pos2, Rect};

use super::hover::{Hovered3DPoint, Hovered3DRegion};
use crate::{Point3D, Transform3D};

#[allow(clippy::too_many_arguments)]
pub(super) fn consider_pt(
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

pub(super) fn projected_box_aabb(
    lo: Point3D,
    hi: Point3D,
    transform: Transform3D,
) -> Option<(Rect, f32)> {
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
pub(super) fn consider_tri_region(
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
