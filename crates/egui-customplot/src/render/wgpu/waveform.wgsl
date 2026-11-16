struct WaveUniforms {
    rect: vec4<f32>,
    screen: vec4<f32>,
    x_map: vec4<f32>,
    y_map: vec4<f32>,
    color: vec4<f32>,
    counts: vec4<u32>,
};

@group(0) @binding(0) var<uniform> u: WaveUniforms;
@group(0) @binding(1) var<storage, read> samples: array<f32>;
@group(0) @binding(2) var<storage, read_write> reduced: array<vec2<f32>>;

var<workgroup> wg_min_y: array<f32, 128>;
var<workgroup> wg_max_y: array<f32, 128>;
var<workgroup> wg_min_idx: array<u32, 128>;
var<workgroup> wg_max_idx: array<u32, 128>;
var<workgroup> wg_invalid: array<u32, 128>;

const SENTINEL: vec2<f32> = vec2<f32>(1.0e30, 1.0e30);
const LN10: f32 = 2.302585092994046;

fn is_finite_f32(v: f32) -> bool {
    return (bitcast<u32>(v) & 0x7f800000u) != 0x7f800000u;
}

fn sample_to_screen(idx: u32) -> vec2<f32> {
    let y = samples[idx];
    if !is_finite_f32(y) {
        return SENTINEL;
    }
    let flags = u.counts.w;
    var ny: f32;
    if (flags & 8u) != 0u {
        if y <= 0.0 {
            return SENTINEL;
        }
        ny = (log(y) / LN10 - u.y_map.x) * u.y_map.y;
    } else {
        ny = u.y_map.z + (y - u.y_map.x) * u.y_map.y;
    }
    if (flags & 2u) != 0u {
        ny = 1.0 - ny;
    }

    var nx: f32;
    if (flags & 4u) != 0u {
        let x = u.x_map.x + f32(idx - u.counts.x) * u.x_map.y;
        if x <= 0.0 {
            return SENTINEL;
        }
        nx = (log(x) / LN10 - u.x_map.z) * u.x_map.w;
    } else {
        nx = u.x_map.x + f32(idx - u.counts.x) * u.x_map.y;
    }
    if (flags & 1u) != 0u {
        nx = 1.0 - nx;
    }
    return vec2<f32>(
        u.rect.x + nx * u.rect.z,
        u.rect.y + u.rect.w - ny * u.rect.w,
    );
}

@compute @workgroup_size(128)
fn cs_reduce(
    @builtin(workgroup_id) wg_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let col = wg_id.x;
    let columns = u.counts.z;
    if col >= columns {
        return;
    }
    let start = u.counts.x;
    let total = u.counts.y - start;
    let q = total / columns;
    let r = total % columns;
    let col_start = start + col * q + min(col, r);
    let col_end = start + (col + 1u) * q + min(col + 1u, r);

    let lid = local_id.x;
    var local_min_y = 3.4e38;
    var local_max_y = -3.4e38;
    var local_min_idx = 0xffffffffu;
    var local_max_idx = 0xffffffffu;
    var local_invalid = 0u;
    let log_y = (u.counts.w & 8u) != 0u;

    var i = col_start + lid;
    while i < col_end {
        let y = samples[i];
        let valid = is_finite_f32(y) && (!log_y || y > 0.0);
        if valid {
            if y < local_min_y || (y == local_min_y && i < local_min_idx) {
                local_min_y = y;
                local_min_idx = i;
            }
            if y > local_max_y || (y == local_max_y && i < local_max_idx) {
                local_max_y = y;
                local_max_idx = i;
            }
        } else {
            local_invalid += 1u;
        }
        i += 128u;
    }

    wg_min_y[lid] = local_min_y;
    wg_max_y[lid] = local_max_y;
    wg_min_idx[lid] = local_min_idx;
    wg_max_idx[lid] = local_max_idx;
    wg_invalid[lid] = local_invalid;
    workgroupBarrier();

    var stride = 64u;
    while stride > 0u {
        if lid < stride {
            let r_min_idx = wg_min_idx[lid + stride];
            if r_min_idx != 0xffffffffu {
                let l_min_idx = wg_min_idx[lid];
                let r_min_y = wg_min_y[lid + stride];
                let l_min_y = wg_min_y[lid];
                if l_min_idx == 0xffffffffu || r_min_y < l_min_y || (r_min_y == l_min_y && r_min_idx < l_min_idx) {
                    wg_min_y[lid] = r_min_y;
                    wg_min_idx[lid] = r_min_idx;
                }
            }
            let r_max_idx = wg_max_idx[lid + stride];
            if r_max_idx != 0xffffffffu {
                let l_max_idx = wg_max_idx[lid];
                let r_max_y = wg_max_y[lid + stride];
                let l_max_y = wg_max_y[lid];
                if l_max_idx == 0xffffffffu || r_max_y > l_max_y || (r_max_y == l_max_y && r_max_idx < l_max_idx) {
                    wg_max_y[lid] = r_max_y;
                    wg_max_idx[lid] = r_max_idx;
                }
            }
            wg_invalid[lid] += wg_invalid[lid + stride];
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }

    if lid == 0u {
        let base = col * 4u;
        if col_start >= col_end || wg_min_idx[0] == 0xffffffffu {
            reduced[base] = SENTINEL;
            reduced[base + 1u] = SENTINEL;
            reduced[base + 2u] = SENTINEL;
            reduced[base + 3u] = SENTINEL;
            return;
        }
        let s0 = sample_to_screen(col_start);
        let s3 = sample_to_screen(col_end - 1u);
        if wg_invalid[0] > 0u && s0.x < 1.0e20 && s3.x < 1.0e20 {
            reduced[base] = s0;
            reduced[base + 1u] = SENTINEL;
            reduced[base + 2u] = SENTINEL;
            reduced[base + 3u] = s3;
            return;
        }
        let min_i = wg_min_idx[0];
        let max_i = wg_max_idx[0];
        let lo_ext = min(min_i, max_i);
        let hi_ext = max(min_i, max_i);
        reduced[base] = s0;
        reduced[base + 1u] = sample_to_screen(lo_ext);
        reduced[base + 2u] = sample_to_screen(hi_ext);
        reduced[base + 3u] = s3;
    }
}

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) edge_px: f32,
    @location(1) half_width_px: f32,
};

@group(0) @binding(0) var<uniform> ru: WaveUniforms;
@group(0) @binding(1) var<storage, read> pts: array<vec2<f32>>;

fn clip_rect_segment(p0_in: vec2<f32>, p1_in: vec2<f32>, rmin: vec2<f32>, rmax: vec2<f32>) -> vec4<f32> {
    let d = p1_in - p0_in;
    var t0 = 0.0;
    var t1 = 1.0;
    let checks = array<vec2<f32>, 4>(
        vec2<f32>(-d.x, p0_in.x - rmin.x),
        vec2<f32>( d.x, rmax.x - p0_in.x),
        vec2<f32>(-d.y, p0_in.y - rmin.y),
        vec2<f32>( d.y, rmax.y - p0_in.y),
    );
    for (var i = 0u; i < 4u; i += 1u) {
        let p = checks[i].x;
        let q = checks[i].y;
        if abs(p) < 1.0e-12 {
            if q < 0.0 {
                return vec4<f32>(1.0e30, 1.0e30, 1.0e30, 1.0e30);
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                if t > t1 { return vec4<f32>(1.0e30, 1.0e30, 1.0e30, 1.0e30); }
                t0 = max(t0, t);
            } else {
                if t < t0 { return vec4<f32>(1.0e30, 1.0e30, 1.0e30, 1.0e30); }
                t1 = min(t1, t);
            }
        }
    }
    let ca = p0_in + d * t0;
    let cb = p0_in + d * t1;
    return vec4<f32>(ca.x, ca.y, cb.x, cb.y);
}

@vertex
fn vs_line(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.edge_px = 0.0;
    out.half_width_px = 0.0;

    let seg = vid / 6u;
    let corner_idx = vid % 6u;
    let a_raw = pts[seg];
    let b_raw = pts[seg + 1u];
    if a_raw.x > 1.0e20 || b_raw.x > 1.0e20 {
        return out;
    }
    let pad = 0.5 * ru.screen.z + 2.0 / max(ru.screen.w, 0.01);
    let rmin = ru.rect.xy - vec2<f32>(pad);
    let rmax = ru.rect.xy + ru.rect.zw + vec2<f32>(pad);
    let clipped = clip_rect_segment(a_raw, b_raw, rmin, rmax);
    if clipped.x > 1.0e20 {
        return out;
    }
    let a = clipped.xy;
    let b = clipped.zw;
    let d = b - a;
    let len_sq = dot(d, d);
    var dir = vec2<f32>(1.0, 0.0);
    if len_sq < 1.0e-6 {
        if seg != 0u && pts[seg - 1u].x <= 1.0e20 {
            return out;
        }
    } else {
        dir = d * inverseSqrt(len_sq);
    }
    let perp = vec2<f32>(-dir.y, dir.x);
    let ppp = max(ru.screen.w, 0.01);
    let half_w = 0.5 * ru.screen.z;
    let feather = 1.0 / ppp;
    let total_half = half_w + feather;

    // Quad corner mapping: 0=(a,-), 1=(a,+), 2=(b,-), 3=(b,-), 4=(a,+), 5=(b,+)
    let is_end = (corner_idx == 2u) || (corner_idx == 3u) || (corner_idx == 5u);
    let side = select(-1.0, 1.0, (corner_idx == 1u) || (corner_idx == 4u) || (corner_idx == 5u));
    let base_pt = select(a - dir * half_w * 0.5, b + dir * half_w * 0.5, is_end);
    let screen_pt = base_pt + perp * (side * total_half);

    out.pos = vec4<f32>(
        2.0 * screen_pt.x / ru.screen.x - 1.0,
        1.0 - 2.0 * screen_pt.y / ru.screen.y,
        0.0,
        1.0,
    );
    out.edge_px = side * total_half * ppp;
    out.half_width_px = half_w * ppp;
    return out;
}

fn linear_from_gamma(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let lower = srgb / vec3<f32>(12.92);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

@fragment
fn fs_line_gamma(in: VsOut) -> @location(0) vec4<f32> {
    let cov = clamp(in.half_width_px + 0.5 - abs(in.edge_px), 0.0, 1.0);
    let a = ru.color.a * cov;
    return vec4<f32>(ru.color.rgb * a, a);
}

@fragment
fn fs_line_linear(in: VsOut) -> @location(0) vec4<f32> {
    let cov = clamp(in.half_width_px + 0.5 - abs(in.edge_px), 0.0, 1.0);
    let a = ru.color.a * cov;
    return vec4<f32>(linear_from_gamma(ru.color.rgb) * a, a);
}
