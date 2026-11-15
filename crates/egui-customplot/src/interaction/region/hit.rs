use egui::Color32;

use crate::{
    Area, FilledContours, Patch, PatchColoring, Pcolor, PcolorShading, PlotPoint, PlotTransform,
    data::QueryStats, model::AreaSource,
};

pub(super) fn pick_area(
    area: Area<'_>,
    point: PlotPoint,
    p_norm: [f64; 2],
    transform: PlotTransform,
    stroke_color: Color32,
) -> Option<usize> {
    if area.resolved_fill(stroke_color).a() == 0 || !area.baseline_valid(transform.scales()) {
        return None;
    }
    match area.source {
        AreaSource::Band(band) => {
            let x = band.x();
            if x.len() < 2 {
                return None;
            }
            let check_segment = |i: usize| -> bool {
                let lo0 = [x[i], band.lower()[i]];
                let lo1 = [x[i + 1], band.lower()[i + 1]];
                let hi0 = [x[i], band.upper()[i]];
                let hi1 = [x[i + 1], band.upper()[i + 1]];
                if ![lo0, lo1, hi0, hi1]
                    .into_iter()
                    .all(|pt| transform.is_valid_point(pt))
                {
                    return false;
                }
                let n_lo0 = transform.normalized(lo0);
                let n_lo1 = transform.normalized(lo1);
                let n_hi0 = transform.normalized(hi0);
                let n_hi1 = transform.normalized(hi1);
                let (xmin, xmax) = (n_lo0[0].min(n_lo1[0]), n_lo0[0].max(n_lo1[0]));
                if p_norm[0] < xmin || p_norm[0] > xmax {
                    return false;
                }
                let dx = n_lo1[0] - n_lo0[0];
                let t = if dx.abs() > 0.0 {
                    ((p_norm[0] - n_lo0[0]) / dx).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let v_lo = n_lo0[1] + t * (n_lo1[1] - n_lo0[1]);
                let v_hi = n_hi0[1] + t * (n_hi1[1] - n_hi0[1]);
                let (ymin, ymax) = (v_lo.min(v_hi), v_lo.max(v_hi));
                p_norm[1] >= ymin && p_norm[1] <= ymax
            };
            if band.is_x_monotonic() {
                if point[0] < x[0] || point[0] > x[x.len() - 1] {
                    return None;
                }
                let start = x.partition_point(|vx| *vx < point[0]).saturating_sub(1);
                let end = x.partition_point(|vx| *vx <= point[0]).min(x.len() - 2);
                (start..=end).find(|&i| check_segment(i))
            } else {
                (0..x.len() - 1).find(|&i| check_segment(i))
            }
        }
        AreaSource::Curve { data, baseline } => {
            let pts = data.points();
            if pts.len() < 2 {
                return None;
            }
            let base_v = transform.normalized([point[0], baseline])[1];
            if !base_v.is_finite() {
                return None;
            }
            let check_range = |range: std::ops::Range<usize>| -> Option<usize> {
                let start = range.start.saturating_sub(1);
                let end = range.end.min(pts.len()).saturating_sub(1);
                for i in start..end {
                    let (a, b) = (pts[i], pts[i + 1]);
                    if !transform.is_valid_point(a) || !transform.is_valid_point(b) {
                        continue;
                    }
                    let na = transform.normalized(a);
                    let nb = transform.normalized(b);
                    let (xmin, xmax) = (na[0].min(nb[0]), na[0].max(nb[0]));
                    if p_norm[0] < xmin || p_norm[0] > xmax {
                        continue;
                    }
                    let dx = nb[0] - na[0];
                    let t = if dx.abs() > 0.0 {
                        ((p_norm[0] - na[0]) / dx).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let ny = na[1] + t * (nb[1] - na[1]);
                    let (ymin, ymax) = (ny.min(base_v), ny.max(base_v));
                    if p_norm[1] >= ymin && p_norm[1] <= ymax {
                        return Some(i);
                    }
                }
                None
            };
            if let Some(index) = data.line_index() {
                let mut stats = QueryStats::default();
                for range in
                    index.candidates(pts, transform.scales(), [point[0], point[0]], &mut stats)
                {
                    if let hit @ Some(_) = check_range(range) {
                        return hit;
                    }
                }
                None
            } else {
                check_range(0..pts.len())
            }
        }
    }
}

pub(super) fn pick_patch(
    patch: Patch<'_>,
    p_norm: [f64; 2],
    transform: PlotTransform,
    default_color: Color32,
) -> Option<(usize, Option<f64>)> {
    let verts = patch.data.vertices();
    let scales = transform.scales();
    for face in (0..patch.data.face_count()).rev() {
        let mut scalar = None;
        let face_visible = match patch.coloring {
            PatchColoring::Uniform => default_color.a() > 0,
            PatchColoring::FaceColors(colors) => colors[face].a() > 0,
            PatchColoring::FaceScalars { values, scale } => {
                if let Some(c) = scale.color(values[face])
                    && c.a() > 0
                {
                    scalar = Some(values[face]);
                    true
                } else {
                    false
                }
            }
            PatchColoring::VertexColors(_) => true,
            PatchColoring::VertexScalars { values, scale } => patch
                .data
                .face(face)
                .iter()
                .all(|&idx| scale.color(values[idx]).is_some()),
        };
        if !face_visible {
            continue;
        }
        for &[i0, i1, i2] in patch.data.face_triangles(face, scales) {
            let (v0, v1, v2) = (verts[i0], verts[i1], verts[i2]);
            if !transform.is_valid_point(v0)
                || !transform.is_valid_point(v1)
                || !transform.is_valid_point(v2)
            {
                continue;
            }
            match patch.coloring {
                PatchColoring::VertexColors(colors) => {
                    if [i0, i1, i2].into_iter().all(|i| colors[i].a() == 0) {
                        continue;
                    }
                }
                PatchColoring::VertexScalars { values, scale }
                    if [i0, i1, i2]
                        .into_iter()
                        .any(|i| scale.color(values[i]).is_none()) =>
                {
                    continue;
                }
                _ => {}
            }
            if let Some([w0, w1, w2]) = triangle_barycentric(
                p_norm,
                transform.normalized(v0),
                transform.normalized(v1),
                transform.normalized(v2),
            ) {
                if let PatchColoring::VertexScalars { values, .. } = patch.coloring {
                    scalar = Some(w0 * values[i0] + w1 * values[i1] + w2 * values[i2]);
                }
                return Some((face, scalar));
            }
        }
    }
    None
}

pub(super) fn pick_filled_contour(
    fc: FilledContours<'_>,
    p_norm: [f64; 2],
    transform: PlotTransform,
) -> Option<(usize, f64, f64)> {
    for (idx, band) in fc.data.bands().iter().enumerate().rev() {
        if fc.colors.color(band.lower).is_none_or(|c| c.a() == 0) {
            continue;
        }
        for &[i0, i1, i2] in &band.triangles {
            let v0 = band.vertices[i0 as usize];
            let v1 = band.vertices[i1 as usize];
            let v2 = band.vertices[i2 as usize];
            if !transform.is_valid_point(v0)
                || !transform.is_valid_point(v1)
                || !transform.is_valid_point(v2)
            {
                continue;
            }
            if triangle_barycentric(
                p_norm,
                transform.normalized(v0),
                transform.normalized(v1),
                transform.normalized(v2),
            )
            .is_some()
            {
                return Some((idx, band.lower, band.upper));
            }
        }
    }
    None
}

pub(super) fn pcolor_cell_scalar(
    pc: Pcolor<'_>,
    col: usize,
    row: usize,
    p_norm: [f64; 2],
    transform: PlotTransform,
) -> Option<f64> {
    if col + 1 >= pc.data.cols() || row + 1 >= pc.data.rows() {
        return None;
    }
    match pc.shading {
        PcolorShading::Flat => {
            let v = pc.data.node_value(col, row)?;
            pc.colors.color(v).and_then(|c| (c.a() > 0).then_some(v))
        }
        PcolorShading::Interp => {
            let z00 = pc.data.node_value(col, row)?;
            let z10 = pc.data.node_value(col + 1, row)?;
            let z11 = pc.data.node_value(col + 1, row + 1)?;
            let z01 = pc.data.node_value(col, row + 1)?;
            if [z00, z10, z11, z01]
                .into_iter()
                .any(|z| pc.colors.color(z).is_none())
            {
                return None;
            }
            let x = pc.data.x_nodes();
            let y = pc.data.y_nodes();
            let n0 = transform.normalized([x[col], y[row]]);
            let n1 = transform.normalized([x[col + 1], y[row + 1]]);
            let tx = if (n1[0] - n0[0]).abs() > 0.0 {
                ((p_norm[0] - n0[0]) / (n1[0] - n0[0])).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let ty = if (n1[1] - n0[1]).abs() > 0.0 {
                ((p_norm[1] - n0[1]) / (n1[1] - n0[1])).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let v0 = z00 + tx * (z10 - z00);
            let v1 = z01 + tx * (z11 - z01);
            Some(v0 + ty * (v1 - v0))
        }
    }
}

fn triangle_barycentric(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<[f64; 3]> {
    let denom = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if !denom.is_finite() || denom == 0.0 {
        return None;
    }
    let w0 = ((b[0] - p[0]) * (c[1] - p[1]) - (b[1] - p[1]) * (c[0] - p[0])) / denom;
    let w1 = ((c[0] - p[0]) * (a[1] - p[1]) - (c[1] - p[1]) * (a[0] - p[0])) / denom;
    let w2 = 1.0 - w0 - w1;
    (w0 >= -1e-9 && w1 >= -1e-9 && w2 >= -1e-9).then_some([w0, w1, w2])
}
