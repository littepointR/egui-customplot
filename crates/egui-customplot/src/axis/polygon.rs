use egui::{Color32, Pos2};

use super::clip_normalized_segment;
use crate::{PlotPoint, PlotTransform};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClipVertex {
    pub norm: [f64; 2],
    pub rgba: [f32; 4],
}

impl ClipVertex {
    pub fn from_color(norm: [f64; 2], color: Color32) -> Self {
        Self {
            norm,
            rgba: color.to_array().map(f32::from),
        }
    }

    pub fn color(self) -> Color32 {
        Color32::from_rgba_premultiplied(
            self.rgba[0].round().clamp(0.0, 255.0) as u8,
            self.rgba[1].round().clamp(0.0, 255.0) as u8,
            self.rgba[2].round().clamp(0.0, 255.0) as u8,
            self.rgba[3].round().clamp(0.0, 255.0) as u8,
        )
    }
}

/// Clip a normalized-space triangle against [0, 1]^2 using Sutherland-Hodgman.
/// Returns Err(()) on non-finite coordinates or potential cancellation overflow.
pub(crate) fn clip_normalized_triangle(
    tri: [ClipVertex; 3],
    screen_span: [f64; 2],
) -> Result<Vec<ClipVertex>, ()> {
    if tri
        .iter()
        .any(|v| !v.norm[0].is_finite() || !v.norm[1].is_finite())
    {
        return Err(());
    }
    let min_u = tri[0].norm[0].min(tri[1].norm[0]).min(tri[2].norm[0]);
    let max_u = tri[0].norm[0].max(tri[1].norm[0]).max(tri[2].norm[0]);
    let min_v = tri[0].norm[1].min(tri[1].norm[1]).min(tri[2].norm[1]);
    let max_v = tri[0].norm[1].max(tri[1].norm[1]).max(tri[2].norm[1]);
    if max_u <= 0.0 || min_u >= 1.0 || max_v <= 0.0 || min_v >= 1.0 {
        return Ok(Vec::new());
    }
    for (axis, (lo, hi)) in [(min_u, max_u), (min_v, max_v)].into_iter().enumerate() {
        let max_abs = lo.abs().max(hi.abs());
        if max_abs * f64::EPSILON * 32.0 * screen_span[axis] > 0.01 {
            return Err(());
        }
    }
    if min_u >= 0.0 && max_u <= 1.0 && min_v >= 0.0 && max_v <= 1.0 {
        return Ok(tri.to_vec());
    }
    let mut poly = tri.to_vec();
    for plane in 0..4 {
        if poly.is_empty() {
            return Ok(poly);
        }
        poly = clip_plane(&poly, plane);
    }
    if poly.len() < 3 {
        poly.clear();
    }
    Ok(poly)
}

fn clip_plane(poly: &[ClipVertex], plane: usize) -> Vec<ClipVertex> {
    let mut out = Vec::with_capacity(poly.len() + 2);
    let mut prev = *poly.last().expect("non-empty polygon");
    let mut prev_in = inside(prev.norm, plane);
    for &curr in poly {
        let curr_in = inside(curr.norm, plane);
        if curr_in != prev_in {
            out.push(intersect(prev, curr, plane));
        }
        if curr_in {
            out.push(curr);
        }
        prev = curr;
        prev_in = curr_in;
    }
    out
}

fn inside(norm: [f64; 2], plane: usize) -> bool {
    match plane {
        0 => norm[0] >= 0.0,
        1 => norm[0] <= 1.0,
        2 => norm[1] >= 0.0,
        _ => norm[1] <= 1.0,
    }
}

fn intersect(a: ClipVertex, b: ClipVertex, plane: usize) -> ClipVertex {
    let (axis, edge) = match plane {
        0 => (0, 0.0),
        1 => (0, 1.0),
        2 => (1, 0.0),
        _ => (1, 1.0),
    };
    let denom = b.norm[axis] - a.norm[axis];
    let t = if denom == 0.0 {
        0.0
    } else {
        ((edge - a.norm[axis]) / denom).clamp(0.0, 1.0)
    };
    let tf = t as f32;
    let mut norm = [
        a.norm[0] + t * (b.norm[0] - a.norm[0]),
        a.norm[1] + t * (b.norm[1] - a.norm[1]),
    ];
    norm[axis] = edge;
    let rgba = std::array::from_fn(|i| a.rgba[i] + tf * (b.rgba[i] - a.rgba[i]));
    ClipVertex { norm, rgba }
}

/// Project an arbitrary true boundary segment, clipping in normalized space.
pub(crate) fn project_boundary_segment(
    a: PlotPoint,
    b: PlotPoint,
    transform: PlotTransform,
) -> Result<Option<[Pos2; 2]>, ()> {
    if !transform.is_valid_point(a) || !transform.is_valid_point(b) {
        return Ok(None);
    }
    let na = transform.normalized(a);
    let nb = transform.normalized(b);
    let rect = transform.rect();
    let span = [f64::from(rect.width()), f64::from(rect.height())];
    for i in 0..2 {
        if !na[i].is_finite() || !nb[i].is_finite() {
            return Err(());
        }
        if na[i].max(nb[i]) < 0.0 || na[i].min(nb[i]) > 1.0 {
            return Ok(None);
        }
        if na[i].abs().max(nb[i].abs()) * f64::EPSILON * 32.0 * span[i] > 0.01 {
            return Err(());
        }
    }
    let Some((sa, sb)) = clip_normalized_segment(na, nb) else {
        return Ok(None);
    };
    let pa = transform.position_from_normalized(sa);
    let pb = transform.position_from_normalized(sb);
    if !pa.is_finite() || !pb.is_finite() {
        return Err(());
    }
    Ok((pa != pb).then_some([pa, pb]))
}

/// Even-odd ray-cast inside normalized view space so straight screen edges match
/// rendered mesh triangles on both linear and logarithmic axes.
pub(crate) fn point_in_ring_transformed(
    point: PlotPoint,
    ring: &[PlotPoint],
    transform: PlotTransform,
) -> bool {
    if ring.len() < 3 || !transform.is_valid_point(point) {
        return false;
    }
    let p = transform.normalized(point);
    if !p[0].is_finite() || !p[1].is_finite() {
        return false;
    }
    let mut inside = false;
    let mut prev = ring[ring.len() - 1];
    if !transform.is_valid_point(prev) {
        return false;
    }
    let mut na = transform.normalized(prev);
    for &curr in ring {
        if !transform.is_valid_point(curr) {
            return false;
        }
        let nb = transform.normalized(curr);
        let straddles = (na[1] > p[1]) != (nb[1] > p[1]);
        if straddles {
            let x_cross = na[0] + (p[1] - na[1]) * (nb[0] - na[0]) / (nb[1] - na[1]);
            if p[0] < x_cross {
                inside = !inside;
            }
        }
        prev = curr;
        na = nb;
    }
    let _ = prev;
    inside
}
