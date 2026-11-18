//! Depth-tested rasterization of projected triangles, segments and markers.
use super::super::scene::{Marker3D, Seg3D, Tri3D};
use egui::{Pos2, Rect};

fn to_buf(p: Pos2, area: Rect, w: usize, h: usize) -> [f32; 2] {
    [
        (p.x - area.left()) / area.width() * w as f32,
        (p.y - area.top()) / area.height() * h as f32,
    ]
}

#[inline]
fn clamp_box(
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    w: usize,
    h: usize,
) -> Option<[usize; 4]> {
    if w == 0
        || h == 0
        || !min_x.is_finite()
        || !max_x.is_finite()
        || !min_y.is_finite()
        || !max_y.is_finite()
    {
        return None;
    }
    let (wf, hf) = (w as f32, h as f32);
    if max_x < 0.0 || min_x >= wf || max_y < 0.0 || min_y >= hf {
        return None;
    }
    let x0 = (min_x.floor().max(0.0) as usize).min(w - 1);
    let x1 = (max_x.ceil().max(0.0) as usize).min(w - 1);
    let y0 = (min_y.floor().max(0.0) as usize).min(h - 1);
    let y1 = (max_y.ceil().max(0.0) as usize).min(h - 1);
    (x0 <= x1 && y0 <= y1).then_some([x0, x1, y0, y1])
}

pub(super) fn raster_tri(
    depth: &mut [f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    area: Rect,
    t: &Tri3D,
    opaque: bool,
) {
    let a = to_buf(t.pos[0], area, w, h);
    let b = to_buf(t.pos[1], area, w, h);
    let c = to_buf(t.pos[2], area, w, h);
    let area2 = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if !area2.is_finite() || area2.abs() < 1e-4 {
        return;
    }
    let sign = area2.signum();
    let inv = 1.0 / area2;
    let Some([x0, x1, y0, y1]) = clamp_box(
        a[0].min(b[0]).min(c[0]),
        a[0].max(b[0]).max(c[0]),
        a[1].min(b[1]).min(c[1]),
        a[1].max(b[1]).max(c[1]),
        w,
        h,
    ) else {
        return;
    };
    let c0 = t.colors[0].to_srgba_unmultiplied();
    let c1 = t.colors[1].to_srgba_unmultiplied();
    let c2 = t.colors[2].to_srgba_unmultiplied();
    let uniform = c0 == c1 && c1 == c2;

    for py in y0..=y1 {
        let fy = py as f32 + 0.5;
        for px in x0..=x1 {
            let fx = px as f32 + 0.5;
            let e0 = ((c[0] - b[0]) * (fy - b[1]) - (c[1] - b[1]) * (fx - b[0])) * sign;
            let e1 = ((a[0] - c[0]) * (fy - c[1]) - (a[1] - c[1]) * (fx - c[0])) * sign;
            let e2 = ((b[0] - a[0]) * (fy - a[1]) - (b[1] - a[1]) * (fx - a[0])) * sign;
            if e0 >= -0.01 && e1 >= -0.01 && e2 >= -0.01 {
                let l0 = (e0 * sign * inv).clamp(0.0, 1.0);
                let l1 = (e1 * sign * inv).clamp(0.0, 1.0);
                let l2 = (e2 * sign * inv).clamp(0.0, 1.0);
                let sum = (l0 + l1 + l2).max(1e-6);
                let (w0, w1, w2) = (l0 / sum, l1 / sum, l2 / sum);
                let z = w0 * t.depth[0] + w1 * t.depth[1] + w2 * t.depth[2];
                let col = if uniform {
                    c0
                } else {
                    std::array::from_fn(|k| {
                        (w0 * f32::from(c0[k]) + w1 * f32::from(c1[k]) + w2 * f32::from(c2[k]))
                            .round()
                            .clamp(0.0, 255.0) as u8
                    })
                };
                let idx = py * w + px;
                if opaque {
                    if z <= depth[idx] {
                        depth[idx] = z;
                        rgba[idx] = col;
                    }
                } else if z <= depth[idx] + 1e-4 {
                    rgba[idx] = blend_over(rgba[idx], col);
                }
            }
        }
    }
}

pub(super) fn raster_seg(
    depth: &mut [f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    area: Rect,
    s: &Seg3D,
) {
    let scale_x = w as f32 / area.width();
    let scale_y = h as f32 / area.height();
    let radius = s.stroke.width * 0.5;
    if radius <= 0.0 {
        return;
    }
    let radius = radius.max(0.5 / scale_x.min(scale_y));
    let a = to_buf(s.p0, area, w, h);
    let b = to_buf(s.p1, area, w, h);
    let rx = radius * scale_x;
    let ry = radius * scale_y;
    let Some([left, right, top, bottom]) = clamp_box(
        a[0].min(b[0]) - rx,
        a[0].max(b[0]) + rx,
        a[1].min(b[1]) - ry,
        a[1].max(b[1]) + ry,
        w,
        h,
    ) else {
        return;
    };
    let dir = s.p1 - s.p0;
    let length_sq = dir.length_sq();
    let col = s.stroke.color.to_srgba_unmultiplied();
    for y in top..=bottom {
        let fy = y as f32 + 0.5;
        // Limit each scanline to the section of the segment within one radius.
        let (x0, x1) = if (b[1] - a[1]).abs() > f32::EPSILON {
            let x_at = |v: f32| a[0] + (b[0] - a[0]) * ((v - a[1]) / (b[1] - a[1])).clamp(0., 1.);
            let (x, z) = (x_at(fy - ry), x_at(fy + ry));
            (
                left.max((x.min(z) - rx).floor() as usize),
                right.min((x.max(z) + rx).ceil() as usize),
            )
        } else {
            (left, right)
        };
        for x in x0..=x1 {
            let point = area.min + egui::vec2((x as f32 + 0.5) / scale_x, fy / scale_y);
            let t = if length_sq > 0.0 {
                ((point - s.p0).dot(dir) / length_sq).clamp(0., 1.)
            } else {
                0.
            };
            if point.distance_sq(s.p0 + dir * t) > radius * radius {
                continue;
            }
            let idx = y * w + x;
            let z = s.depth[0] + (s.depth[1] - s.depth[0]) * t - 2e-4;
            if z <= depth[idx] {
                depth[idx] = z;
                rgba[idx] = if col[3] >= 250 {
                    col
                } else {
                    blend_over(rgba[idx], col)
                };
            }
        }
    }
}

pub(super) fn raster_marker(
    depth: &mut [f32],
    rgba: &mut [[u8; 4]],
    w: usize,
    h: usize,
    area: Rect,
    m: &Marker3D,
) {
    let c = to_buf(m.pos, area, w, h);
    let rx = m.radius * w as f32 / area.width();
    let ry = m.radius * h as f32 / area.height();
    if !(rx > 0.0 && ry > 0.0) {
        return;
    }
    let Some([x0, x1, y0, y1]) = clamp_box(c[0] - rx, c[0] + rx, c[1] - ry, c[1] + ry, w, h) else {
        return;
    };
    let col = m.color.to_srgba_unmultiplied();
    let z = m.depth - 3e-4;
    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = (px as f32 + 0.5 - c[0]) / rx;
            let dy = (py as f32 + 0.5 - c[1]) / ry;
            if match m.shape {
                crate::MarkerShape::Square => dx.abs().max(dy.abs()) <= 0.9,
                crate::MarkerShape::Diamond => dx.abs() + dy.abs() <= 1.0,
                _ => dx * dx + dy * dy <= 1.0,
            } {
                let idx = py * w + px;
                if z <= depth[idx] {
                    if col[3] >= 250 {
                        depth[idx] = z;
                        rgba[idx] = col;
                    } else {
                        rgba[idx] = blend_over(rgba[idx], col);
                    }
                }
            }
        }
    }
}

fn blend_over(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let sa = f32::from(src[3]) / 255.0;
    let da = f32::from(dst[3]) / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 1e-4 {
        return [0; 4];
    }
    let mix = |s: u8, d: u8| -> u8 {
        ((f32::from(s) * sa + f32::from(d) * da * (1.0 - sa)) / out_a).clamp(0.0, 255.0) as u8
    };
    [
        mix(src[0], dst[0]),
        mix(src[1], dst[1]),
        mix(src[2], dst[2]),
        (out_a * 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Color32, Stroke, pos2, vec2};
    #[test]
    fn logical_stroke_and_marker_footprints_scale_with_target_density() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(32., 32.));
        for density in [1, 3] {
            let w = 32 * density;
            let mut depth = vec![f32::INFINITY; w * w];
            let mut rgba = vec![[0; 4]; w * w];
            raster_seg(
                &mut depth,
                &mut rgba,
                w,
                w,
                rect,
                &Seg3D {
                    p0: pos2(1., 8.),
                    p1: pos2(30., 8.),
                    depth: [1.; 2],
                    stroke: Stroke::new(4., Color32::WHITE),
                },
            );
            assert_eq!(
                (0..w).filter(|y| rgba[y * w + 16 * density][3] > 0).count(),
                4 * density
            );
            for shape in [
                crate::MarkerShape::Circle,
                crate::MarkerShape::Square,
                crate::MarkerShape::Diamond,
            ] {
                rgba.fill([0; 4]);
                depth.fill(f32::INFINITY);
                raster_marker(
                    &mut depth,
                    &mut rgba,
                    w,
                    w,
                    rect,
                    &Marker3D {
                        pos: pos2(16., 16.),
                        depth: 1.,
                        radius: 8.,
                        color: Color32::WHITE,
                        shape,
                    },
                );
                let corner = (22 * density) * w + 22 * density;
                assert_eq!(rgba[corner][3] > 0, shape == crate::MarkerShape::Square);
            }
        }
    }
}
