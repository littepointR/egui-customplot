struct ScatterUniforms {
    rect: vec4<f32>,      // (left, top, width, height)
    screen: vec4<f32>,    // (screen_w_pts, screen_h_pts, default_radius, pixels_per_point)
    axis_min: vec4<f32>,  // (axis_min_x, inv_span_x, axis_min_y, inv_span_y)
    ref_origin: vec4<f32>,// (ref_rel_norm_x, ref_rel_norm_y, shape_id_f32, flags_f32)
    default_color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> su: ScatterUniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local_px: vec2<f32>,
    @location(1) radius_px: f32,
    @location(2) color: vec4<f32>,
};

const LN10: f32 = 2.302585092994046;

fn unpack_rgba(c: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(c & 255u),
        f32((c >> 8u) & 255u),
        f32((c >> 16u) & 255u),
        f32((c >> 24u) & 255u),
    ) / 255.0;
}

@vertex
fn vs_scatter(
    @builtin(vertex_index) vid: u32,
    @builtin(instance_index) iid: u32,
    @location(0) pos: vec2<f32>,
    @location(1) radius_override: f32,
    @location(2) rgba_packed: u32,
) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.local_px = vec2<f32>(0.0);
    out.radius_px = 0.0;
    out.color = vec4<f32>(0.0);

    let packed = u32(su.ref_origin.w + 0.5);
    let flags = packed & 15u;
    let stride = max(packed >> 4u, 1u);
    if stride > 1u && (iid % stride) != 0u {
        return out;
    }

    let r = select(radius_override, su.screen.z, radius_override < 0.0);
    let col = select(unpack_rgba(rgba_packed), su.default_color, radius_override < 0.0);
    if r <= 0.0 || col.a <= 0.0 {
        return out;
    }

    var nx: f32;
    if (flags & 4u) != 0u {
        if pos.x <= 0.0 { return out; }
        nx = (log(pos.x) / LN10 - su.axis_min.x) * su.axis_min.y;
    } else {
        nx = su.ref_origin.x + pos.x * su.axis_min.y;
    }
    var ny: f32;
    if (flags & 8u) != 0u {
        if pos.y <= 0.0 { return out; }
        ny = (log(pos.y) / LN10 - su.axis_min.z) * su.axis_min.w;
    } else {
        ny = su.ref_origin.y + pos.y * su.axis_min.w;
    }
    if (flags & 1u) != 0u { nx = 1.0 - nx; }
    if (flags & 2u) != 0u { ny = 1.0 - ny; }

    let center = vec2<f32>(
        su.rect.x + nx * su.rect.z,
        su.rect.y + su.rect.w - ny * su.rect.w,
    );
    let ppp = max(su.screen.w, 0.01);
    let half_extent = r + 1.0 / ppp;
    if center.x + half_extent < su.rect.x
        || center.x - half_extent > su.rect.x + su.rect.z
        || center.y + half_extent < su.rect.y
        || center.y - half_extent > su.rect.y + su.rect.w
    {
        return out;
    }
    var corner = vec2<f32>(0.0);
    switch vid {
        case 0u: { corner = vec2<f32>(-1.0, -1.0); }
        case 1u: { corner = vec2<f32>( 1.0, -1.0); }
        case 2u: { corner = vec2<f32>(-1.0,  1.0); }
        case 3u: { corner = vec2<f32>(-1.0,  1.0); }
        case 4u: { corner = vec2<f32>( 1.0, -1.0); }
        default: { corner = vec2<f32>( 1.0,  1.0); }
    }
    let screen_pt = center + corner * half_extent;
    out.pos = vec4<f32>(
        2.0 * screen_pt.x / su.screen.x - 1.0,
        1.0 - 2.0 * screen_pt.y / su.screen.y,
        0.0,
        1.0,
    );
    out.local_px = corner * (half_extent * ppp);
    out.radius_px = r * ppp;
    out.color = col;
    return out;
}

fn marker_sdf(p: vec2<f32>, r: f32, shape_id: u32) -> f32 {
    switch shape_id {
        case 1u: { // Square
            let d = abs(p) - vec2<f32>(r);
            return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
        }
        case 2u: { // Diamond
            return (abs(p.x) + abs(p.y)) * 0.70710677 - r * 0.70710677;
        }
        case 3u, 4u: { // TriangleUp / TriangleDown
            let py = select(p.y, -p.y, shape_id == 4u);
            let k = sqrt(3.0);
            var q = vec2<f32>(abs(p.x) / r, -py / r);
            q.x = q.x - 0.8660254;
            q.y = q.y + 0.5;
            if (q.x + k * q.y) > 0.0 {
                q = vec2<f32>(q.x - k * q.y, -k * q.x - q.y) * 0.5;
            }
            q.x -= clamp(q.x, -1.7320508, 0.0);
            return (-length(q) * sign(q.y)) * r;
        }
        case 5u: { // Plus
            let t = r * 0.3;
            let dh = max(abs(p.x) - r, abs(p.y) - t);
            let dv = max(abs(p.x) - t, abs(p.y) - r);
            return min(dh, dv);
        }
        case 6u: { // Cross
            let rot = vec2<f32>(p.x - p.y, p.x + p.y) * 0.70710677;
            let t = r * 0.3;
            let dh = max(abs(rot.x) - r, abs(rot.y) - t);
            let dv = max(abs(rot.x) - t, abs(rot.y) - r);
            return min(dh, dv);
        }
        default: { // Circle
            return length(p) - r;
        }
    }
}

fn linear_from_gamma(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let lower = srgb / vec3<f32>(12.92);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

@fragment
fn fs_scatter_gamma(in: VsOut) -> @location(0) vec4<f32> {
    if in.radius_px <= 0.0 { discard; }
    let shape_id = u32(su.ref_origin.z + 0.5);
    let d = marker_sdf(in.local_px, in.radius_px, shape_id);
    let cov = clamp(0.5 - d, 0.0, 1.0);
    if cov <= 0.0 { discard; }
    let a = in.color.a * cov;
    return vec4<f32>(in.color.rgb * a, a);
}

@fragment
fn fs_scatter_linear(in: VsOut) -> @location(0) vec4<f32> {
    if in.radius_px <= 0.0 { discard; }
    let shape_id = u32(su.ref_origin.z + 0.5);
    let d = marker_sdf(in.local_px, in.radius_px, shape_id);
    let cov = clamp(0.5 - d, 0.0, 1.0);
    if cov <= 0.0 { discard; }
    let a = in.color.a * cov;
    return vec4<f32>(linear_from_gamma(in.color.rgb) * a, a);
}
