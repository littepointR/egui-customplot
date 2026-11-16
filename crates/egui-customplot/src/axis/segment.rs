use std::f32::consts::{PI, TAU};

use egui::Pos2;

use crate::{GraphEdge, PlotPoint, PlotTransform};

pub(crate) struct ProjectedSegment {
    pub shaft: [Pos2; 2],
    pub direction: [f64; 2],
    pub length: f64,
    pub origin_visible: bool,
    pub end_visible: bool,
}

/// Independent vector projection with a conservative cancellation guard. The
/// older reference line path is unchanged. Err means numerically rejected, not
/// merely offscreen. Never silently clamp an unrepresentable vector to a border.
pub(crate) fn project_vector_segment(
    origin: PlotPoint,
    end: PlotPoint,
    transform: PlotTransform,
) -> Result<Option<ProjectedSegment>, ()> {
    let bounds = transform.bounds();
    if origin == end
        || (0..2).any(|i| {
            origin[i].max(end[i]) < bounds.min()[i] || origin[i].min(end[i]) > bounds.max()[i]
        })
    {
        return Ok(None);
    }
    let a = transform.normalized(origin);
    let b = transform.normalized(end);
    let rect = transform.rect();
    let screen_span = [f64::from(rect.width()), f64::from(rect.height())];
    // Reserve 32 ulps of intermediate error, requiring <0.01 logical point.
    // This is an explicit conservative limit, not arbitrary extreme-value LOD.
    if (0..2).any(|i| {
        !a[i].is_finite()
            || !b[i].is_finite()
            || a[i].abs().max(b[i].abs()) * f64::EPSILON * 32.0 * screen_span[i] > 0.01
    }) {
        return Err(());
    }
    let delta = [
        (b[0] - a[0]) * screen_span[0],
        -(b[1] - a[1]) * screen_span[1],
    ];
    let length = delta[0].hypot(delta[1]);
    if !length.is_finite() || length == 0.0 {
        return Err(());
    }
    let Some((start, finish)) = clip_normalized_segment(a, b) else {
        return Ok(None);
    };
    let shaft = [
        transform.position_from_normalized(start),
        transform.position_from_normalized(finish),
    ];
    if shaft.iter().any(|p| !p.is_finite()) {
        return Err(());
    }
    if shaft[0] == shaft[1] {
        return Ok(None); // No representable shaft, so no invented head or hit.
    }
    let inside = |p: PlotPoint| (0..2).all(|i| p[i] >= bounds.min()[i] && p[i] <= bounds.max()[i]);
    Ok(Some(ProjectedSegment {
        shaft,
        direction: delta.map(|d| d / length),
        length,
        origin_visible: inside(origin),
        end_visible: inside(end),
    }))
}

/// Normalized Liang–Barsky clipping, preserving input direction. Callers decide
/// which numerical domain is acceptable before reaching this reference routine.
pub(crate) fn clip_normalized_segment(
    a: PlotPoint,
    b: PlotPoint,
) -> Option<(PlotPoint, PlotPoint)> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let mut enter: f64 = 0.0;
    let mut exit: f64 = 1.0;
    for (p, q) in [
        (-d[0], a[0]),
        (d[0], 1.0 - a[0]),
        (-d[1], a[1]),
        (d[1], 1.0 - a[1]),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let ratio = q / p;
            if p < 0.0 {
                enter = enter.max(ratio);
            } else {
                exit = exit.min(ratio);
            }
            if enter > exit {
                return None;
            }
        }
    }
    let at = |t: f64| std::array::from_fn(|i| (a[i] + t * d[i]).clamp(0.0, 1.0));
    Some((at(enter), at(exit)))
}

/// Shared screen-space polyline and trimmed tip for straight, multi-edge, bidirectional,
/// and self-loop graph edges across both rendering and edge hover picking.
pub(crate) fn graph_edge_screen_polyline(
    edge: GraphEdge,
    u_pos: Pos2,
    v_pos: Pos2,
    centroid: Pos2,
    directed: bool,
    target_radius: f32,
) -> (Vec<Pos2>, Option<(Pos2, [f32; 2])>) {
    if !u_pos.is_finite() || !v_pos.is_finite() {
        return (Vec::new(), None);
    }
    if edge.source == edge.target {
        let delta = u_pos - centroid;
        let out_dir = if delta.length() > 1e-3 {
            delta.normalized()
        } else {
            egui::vec2(0.0, -1.0)
        };
        let loop_r = target_radius * 1.45 + 9.0 + edge.parallel_index as f32 * 5.0;
        let center = u_pos + out_dir * (target_radius + loop_r * 0.72);
        let base_ang = out_dir.y.atan2(out_dir.x) + PI;
        let start_ang = base_ang + 0.35;
        let end_ang = base_ang + TAU - 0.35;
        let steps = 22usize;
        let mut pts = Vec::with_capacity(steps + 3);
        pts.push(u_pos);
        for s in 0..=steps {
            let a = start_ang + (end_ang - start_ang) * (s as f32 / steps as f32);
            pts.push(Pos2::new(
                center.x + loop_r * a.cos(),
                center.y + loop_r * a.sin(),
            ));
        }
        let tip = if directed && pts.len() >= 2 {
            let last = pts[pts.len() - 1];
            let dir_to_u = (u_pos - last).normalized();
            let trimmed = u_pos - dir_to_u * target_radius;
            pts.push(trimmed);
            Some((trimmed, [dir_to_u.x, dir_to_u.y]))
        } else {
            pts.push(u_pos);
            None
        };
        return (pts, tip);
    }
    let chord = v_pos - u_pos;
    let len = chord.length();
    if len <= 1e-3 {
        return (Vec::new(), None);
    }
    let dir = chord / len;
    if edge.parallel_count > 1 || edge.bidirectional_pair {
        let n = egui::vec2(-dir.y, dir.x);
        let centered =
            edge.parallel_index as f32 - 0.5 * (edge.parallel_count.saturating_sub(1)) as f32;
        let shift = if edge.bidirectional_pair {
            centered + 0.75 * (edge.parallel_count.max(1) as f32 * 0.5)
        } else {
            centered
        };
        let bend = shift * (len * 0.16).clamp(16.0, 38.0);
        let ctrl = Pos2::new(
            0.5 * (u_pos.x + v_pos.x) + n.x * bend,
            0.5 * (u_pos.y + v_pos.y) + n.y * bend,
        );
        let steps = 16usize;
        let mut pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let mt = 1.0 - t;
            let p = Pos2::new(
                mt * mt * u_pos.x + 2.0 * mt * t * ctrl.x + t * t * v_pos.x,
                mt * mt * u_pos.y + 2.0 * mt * t * ctrl.y + t * t * v_pos.y,
            );
            pts.push(p);
        }
        let tip = if directed && pts.len() >= 2 {
            let prev = pts[pts.len() - 2];
            let d_end = v_pos - prev;
            let d_len = d_end.length();
            if d_len > 1e-3 {
                let d_unit = d_end / d_len;
                let trimmed = v_pos - d_unit * target_radius.min(d_len * 0.65);
                *pts.last_mut().unwrap() = trimmed;
                Some((trimmed, [d_unit.x, d_unit.y]))
            } else {
                None
            }
        } else {
            None
        };
        return (pts, tip);
    }
    let end = if directed && len > target_radius + 2.0 {
        v_pos - dir * target_radius
    } else {
        v_pos
    };
    (vec![u_pos, end], directed.then_some((end, [dir.x, dir.y])))
}

pub(crate) fn point_polyline_distance(p: Pos2, pts: &[Pos2]) -> f32 {
    if pts.len() < 2 {
        return f32::INFINITY;
    }
    let mut best = f32::INFINITY;
    for seg in pts.windows(2) {
        let a = seg[0];
        let b = seg[1];
        let ab = b - a;
        let len2 = ab.length_sq();
        let proj = if len2 <= 1e-6 {
            a
        } else {
            let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
            a + ab * t
        };
        best = best.min(p.distance(proj));
    }
    best
}
