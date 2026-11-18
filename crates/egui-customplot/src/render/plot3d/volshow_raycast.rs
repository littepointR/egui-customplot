use egui::{Pos2, Rect};

use crate::{AxisScale, Plot3DDiagnostics, Transform3D, VolumeRender3D, data::VolumeRenderMode};

pub(super) fn volume_screen_aabb(
    transform: Transform3D,
    w: usize,
    h: usize,
    area: Rect,
) -> Option<[usize; 4]> {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut any = false;
    for iz in [0.0_f64, 1.0] {
        for iy in [0.0_f64, 1.0] {
            for ix in [0.0_f64, 1.0] {
                if let Some((pos, _)) = transform.project_unit([ix, iy, iz]) {
                    min_x = min_x.min(pos.x);
                    max_x = max_x.max(pos.x);
                    min_y = min_y.min(pos.y);
                    max_y = max_y.max(pos.y);
                    any = true;
                }
            }
        }
    }
    if w == 0 || h == 0 {
        return None;
    }
    if !any {
        return Some([0, w - 1, 0, h - 1]);
    }
    if !min_x.is_finite()
        || !max_x.is_finite()
        || !min_y.is_finite()
        || !max_y.is_finite()
        || max_x < area.left()
        || min_x > area.right()
        || max_y < area.top()
        || min_y > area.bottom()
    {
        return None;
    }
    let sx = w as f32 / area.width().max(1.0);
    let sy = h as f32 / area.height().max(1.0);
    let x0 = (((min_x - area.left()) * sx).floor() as isize - 1).clamp(0, w as isize - 1) as usize;
    let x1 = (((max_x - area.left()) * sx).ceil() as isize + 1).clamp(0, w as isize - 1) as usize;
    let y0 = (((min_y - area.top()) * sy).floor() as isize - 1).clamp(0, h as isize - 1) as usize;
    let y1 = (((max_y - area.top()) * sy).ceil() as isize + 1).clamp(0, h as isize - 1) as usize;
    (x0 <= x1 && y0 <= y1).then_some([x0, x1, y0, y1])
}

#[inline]
fn shade_rgb(
    shading: bool,
    grad: [f64; 3],
    light_dir: [f64; 3],
    ambient: f32,
    rgb: [f32; 3],
) -> [f32; 3] {
    if !shading {
        return rgb;
    }
    let len = (grad[0] * grad[0] + grad[1] * grad[1] + grad[2] * grad[2])
        .sqrt()
        .max(1e-12);
    let ndotl = ((grad[0] * light_dir[0] + grad[1] * light_dir[1] + grad[2] * light_dir[2]) / len)
        .abs() as f32;
    let shade = ambient + (1.0 - ambient) * ndotl;
    [rgb[0] * shade, rgb[1] * shade, rgb[2] * shade]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn raycast_into_buffer(
    vr: VolumeRender3D<'_>,
    transform: Transform3D,
    depth_buf: Option<&[f32]>,
    rgba: &mut [[u8; 4]],
    mut hit_t_buf: Option<&mut [f32]>,
    w: usize,
    h: usize,
    plot_area: Rect,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let Some([vmin, vmax]) = vr.volume.value_range() else {
        return;
    };
    let Some([x0, x1, y0, y1]) = volume_screen_aabb(transform, w, h, plot_area) else {
        return;
    };
    let inv_span = 1.0 / (vmax - vmin).max(1e-12);
    let cw = plot_area.width() / w as f32;
    let ch = plot_area.height() / h as f32;
    let steps = vr.sample_steps.max(8);
    let light_dir = [0.42_f64, 0.58, 0.70];
    let linear_axes = transform
        .mappings()
        .iter()
        .all(|m| m.domain() == AxisScale::Linear && m.function_key().is_none());

    for py in y0..=y1 {
        let sy = plot_area.top() + (py as f32 + 0.5) * ch;
        for px in x0..=x1 {
            let sx = plot_area.left() + (px as f32 + 0.5) * cw;
            let Some((u0, ud, t0, mut t1)) = transform.screen_ray_unit(Pos2::new(sx, sy)) else {
                continue;
            };
            let idx = py * w + px;
            if let Some(d_arr) = depth_buf {
                t1 = t1.min(f64::from(d_arr[idx]));
            }
            if t1 <= t0 + 1e-6 {
                continue;
            }
            let dt = (t1 - t0) / steps as f64;
            let linear_ray = if linear_axes {
                let u_a = [
                    u0[0] + (t0 + 0.5 * dt) * ud[0],
                    u0[1] + (t0 + 0.5 * dt) * ud[1],
                    u0[2] + (t0 + 0.5 * dt) * ud[2],
                ];
                let u_b = [
                    u_a[0] + dt * ud[0],
                    u_a[1] + dt * ud[1],
                    u_a[2] + dt * ud[2],
                ];
                transform
                    .unit_to_data(u_a)
                    .zip(transform.unit_to_data(u_b))
                    .map(|(a, b)| (a, [b[0] - a[0], b[1] - a[1], b[2] - a[2]]))
            } else {
                None
            };
            let data_at = |s: usize| -> Option<[f64; 3]> {
                if let Some((dp0, dstep)) = linear_ray {
                    let sf = s as f64;
                    Some([
                        dp0[0] + sf * dstep[0],
                        dp0[1] + sf * dstep[1],
                        dp0[2] + sf * dstep[2],
                    ])
                } else {
                    let t = t0 + (s as f64 + 0.5) * dt;
                    transform.unit_to_data([
                        u0[0] + t * ud[0],
                        u0[1] + t * ud[1],
                        u0[2] + t * ud[2],
                    ])
                }
            };
            let mut hit_t = f32::INFINITY;
            let sample_rgba = match vr.mode {
                VolumeRenderMode::DirectAlpha => {
                    let mut acc = [0.0_f32; 4];
                    let op = vr.opacity_scale * (32.0 / steps as f32);
                    for s in 0..steps {
                        let Some(dp) = data_at(s) else { continue };
                        let Some((val, grad)) = vr.volume.sample_and_grad_fast(dp, vr.shading)
                        else {
                            continue;
                        };
                        let [r0, g0, b0, a_raw] = vr.transfer.sample_norm((val - vmin) * inv_span);
                        let a = (a_raw * op).clamp(0.0, 1.0);
                        if a <= 1e-4 {
                            continue;
                        }
                        if hit_t.is_infinite() {
                            hit_t = (t0 + (s as f64 + 0.5) * dt) as f32;
                        }
                        let [r, g, b] = shade_rgb(vr.shading, grad, light_dir, 0.55, [r0, g0, b0]);
                        let rem = 1.0 - acc[3];
                        acc[0] += rem * r * a;
                        acc[1] += rem * g * a;
                        acc[2] += rem * b * a;
                        acc[3] += rem * a;
                        if acc[3] >= 0.97 {
                            break;
                        }
                    }
                    (acc[3] > 0.01).then(|| {
                        let inv = 1.0 / acc[3].max(1e-5);
                        [
                            (acc[0] * inv).clamp(0.0, 1.0),
                            (acc[1] * inv).clamp(0.0, 1.0),
                            (acc[2] * inv).clamp(0.0, 1.0),
                            acc[3].clamp(0.0, 1.0),
                        ]
                    })
                }
                VolumeRenderMode::MaximumIntensity => {
                    let mut best_v = f64::NEG_INFINITY;
                    let mut best_s = 0usize;
                    for s in 0..steps {
                        if let Some(dp) = data_at(s)
                            && let Some((val, _)) = vr.volume.sample_and_grad_fast(dp, false)
                            && val > best_v
                        {
                            best_v = val;
                            best_s = s;
                        }
                    }
                    if best_v.is_finite() {
                        hit_t = (t0 + (best_s as f64 + 0.5) * dt) as f32;
                        Some(vr.transfer.sample_norm((best_v - vmin) * inv_span))
                    } else {
                        None
                    }
                }
                VolumeRenderMode::IsosurfaceRay { threshold } => {
                    let mut prev_s: Option<f64> = None;
                    let mut hit = None;
                    for s in 0..steps {
                        let Some(dp) = data_at(s) else {
                            prev_s = None;
                            continue;
                        };
                        let Some((val, grad)) = vr.volume.sample_and_grad_fast(dp, vr.shading)
                        else {
                            prev_s = None;
                            continue;
                        };
                        if let Some(pv) = prev_s
                            && (pv - threshold) * (val - threshold) <= 0.0
                        {
                            let [r0, g0, b0, _] =
                                vr.transfer.sample_norm((threshold - vmin) * inv_span);
                            let [r, g, b] =
                                shade_rgb(vr.shading, grad, light_dir, 0.45, [r0, g0, b0]);
                            hit_t = (t0 + (s as f64 + 0.5) * dt) as f32;
                            hit = Some([r, g, b, vr.opacity_scale.clamp(0.0, 1.0)]);
                            break;
                        }
                        prev_s = Some(val);
                    }
                    hit
                }
            };
            if let Some([r, g, b, a]) = sample_rgba
                && a > 0.01
            {
                let bg = rgba[idx];
                let bg_a = f32::from(bg[3]) / 255.0;
                let rem = 1.0 - a;
                let out_a = (a + bg_a * rem).clamp(0.0, 1.0);
                let mix = |fg: f32, bg_u8: u8| -> u8 {
                    let val = (fg * a + (f32::from(bg_u8) / 255.0) * bg_a * rem) / out_a.max(1e-5);
                    (val * 255.0).round().clamp(0.0, 255.0) as u8
                };
                rgba[idx] = [
                    mix(r, bg[0]),
                    mix(g, bg[1]),
                    mix(b, bg[2]),
                    (out_a * 255.0).round() as u8,
                ];
                if let Some(ht) = hit_t_buf.as_deref_mut() {
                    ht[idx] = ht[idx].min(hit_t);
                }
                diagnostics.zbuffer_pixels += 1;
            }
        }
    }
}
