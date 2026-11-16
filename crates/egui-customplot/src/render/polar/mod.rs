use std::f64::consts::TAU;

use egui::{Align2, Color32, Mesh, Painter, Pos2, Shape, epaint};

use crate::{
    NegativeRadiusMode, PolarDiagnostics, PolarLayer, PolarRegionAnnotation, PolarState,
    PolarTransform, model::ResolvedPlotStyle,
};

mod layers;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_polar(
    painter: &Painter,
    state: &PolarState,
    layers: &[PolarLayer<'_>],
    regions: &[PolarRegionAnnotation],
    transform: PolarTransform,
    neg_mode: NegativeRadiusMode,
    title: &str,
    show_grid: bool,
    pixels_per_point: f32,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut PolarDiagnostics,
) {
    let center = transform.center();
    let max_r = transform.pixel_radius();
    paint_polar_backdrop(painter, transform, show_grid, style);
    paint_polar_regions(painter, regions, transform, style, diagnostics);
    layers::paint_polar_layers(
        painter,
        state,
        layers,
        transform,
        neg_mode,
        pixels_per_point,
        style,
        diagnostics,
    );
    paint_polar_border_and_labels(painter, transform, title, center, max_r, style);
}

fn paint_polar_backdrop(
    painter: &Painter,
    transform: PolarTransform,
    show_grid: bool,
    style: &ResolvedPlotStyle<'_>,
) {
    let center = transform.center();
    let max_r = transform.pixel_radius();
    if transform.is_full_circle() {
        painter.circle_filled(center, max_r, style.background);
    } else {
        let [t0, t1] = transform.theta_lim();
        let mesh = sector_mesh(transform, t0, t1 - t0, 0.0, max_r, style.background);
        if !mesh.is_empty() {
            painter.add(Shape::mesh(mesh));
        }
    }
    if !show_grid {
        return;
    }
    let grid_stroke = style.grid_stroke;
    for tick in transform.radial_ticks(5) {
        let norm = transform.normalized_radius(tick.value);
        if !(0.0..=1.0).contains(&norm) || norm <= 1e-3 {
            continue;
        }
        let rad = norm as f32 * max_r;
        if transform.is_full_circle() {
            painter.circle_stroke(center, rad, grid_stroke);
        } else {
            let [t0, t1] = transform.theta_lim();
            let arc = arc_points(transform, t0, t1 - t0, rad, 48);
            if arc.len() >= 2 {
                painter.add(Shape::line(arc, grid_stroke));
            }
        }
    }
    for (ang, _) in transform.angular_ticks() {
        let s = transform.screen_angle(ang);
        let tip = Pos2::new(
            center.x + max_r * s.cos() as f32,
            center.y + max_r * s.sin() as f32,
        );
        painter.line_segment([center, tip], grid_stroke);
    }
}

fn paint_polar_regions(
    painter: &Painter,
    regions: &[PolarRegionAnnotation],
    transform: PolarTransform,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut PolarDiagnostics,
) {
    let [r_lo, r_hi] = transform.r_lim();
    let span_r = (r_hi - r_lo).max(1e-12);
    let max_r = transform.pixel_radius();
    for reg in regions {
        if !reg.is_valid() {
            diagnostics.region_rejections += 1;
            continue;
        }
        let [req_r0, req_r1] = reg.r_range().unwrap_or([r_lo, r_hi]);
        let clipped_r0 = req_r0.max(r_lo);
        let clipped_r1 = req_r1.min(r_hi);
        if clipped_r1 <= clipped_r0 {
            continue;
        }
        let inner_px = ((clipped_r0 - r_lo) / span_r) as f32 * max_r;
        let outer_px = ((clipped_r1 - r_lo) / span_r) as f32 * max_r;
        let (theta_start, theta_span) = if let Some([a, b]) = reg.theta_range() {
            let diff = (b - a).abs();
            if diff >= TAU - 1e-6 {
                let [t0, t1] = transform.theta_lim();
                (t0, t1 - t0)
            } else {
                let wrap = (b - a).rem_euclid(TAU);
                if wrap <= 1e-9 {
                    diagnostics.region_rejections += 1;
                    continue;
                }
                (a, wrap)
            }
        } else {
            let [t0, t1] = transform.theta_lim();
            (t0, t1 - t0)
        };
        let mesh = sector_mesh(
            transform,
            theta_start,
            theta_span,
            inner_px,
            outer_px,
            reg.fill_color(),
        );
        if mesh.is_empty() {
            continue;
        }
        painter.add(Shape::mesh(mesh));
        diagnostics.output_regions += 1;
        if let Some(border) = reg.border_stroke()
            && border.width > 0.0
            && border.color != Color32::TRANSPARENT
        {
            let outer_arc = arc_points(transform, theta_start, theta_span, outer_px, 36);
            if outer_arc.len() >= 2 {
                painter.add(Shape::line(outer_arc, border));
            }
            if inner_px > 0.5 {
                let inner_arc = arc_points(transform, theta_start, theta_span, inner_px, 36);
                if inner_arc.len() >= 2 {
                    painter.add(Shape::line(inner_arc, border));
                }
            }
            if theta_span < TAU - 1e-4 {
                for th in [theta_start, theta_start + theta_span] {
                    if transform.contains_theta(th) {
                        let s = transform.screen_angle(th);
                        let p0 = Pos2::new(
                            transform.center().x + inner_px * s.cos() as f32,
                            transform.center().y + inner_px * s.sin() as f32,
                        );
                        let p1 = Pos2::new(
                            transform.center().x + outer_px * s.cos() as f32,
                            transform.center().y + outer_px * s.sin() as f32,
                        );
                        painter.line_segment([p0, p1], border);
                    }
                }
            }
        }
        if !reg.label_text().is_empty() {
            let mid_t = theta_start + 0.5 * theta_span;
            if transform.contains_theta(mid_t) {
                let s = transform.screen_angle(mid_t);
                let mid_r = 0.5 * (inner_px + outer_px);
                let pos = Pos2::new(
                    transform.center().x + mid_r * s.cos() as f32,
                    transform.center().y + mid_r * s.sin() as f32,
                );
                painter.text(
                    pos,
                    Align2::CENTER_CENTER,
                    reg.label_text(),
                    egui::FontId::proportional(11.0),
                    style.text_color,
                );
            }
        }
    }
}

fn paint_polar_border_and_labels(
    painter: &Painter,
    transform: PolarTransform,
    title: &str,
    center: Pos2,
    max_r: f32,
    style: &ResolvedPlotStyle<'_>,
) {
    if transform.is_full_circle() {
        painter.circle_stroke(center, max_r, style.border_stroke);
    } else {
        let [t0, t1] = transform.theta_lim();
        let mut pts = arc_points(transform, t0, t1 - t0, max_r, 64);
        pts.push(center);
        if let Some(&first) = pts.first() {
            pts.push(first);
        }
        painter.add(Shape::line(pts, style.border_stroke));
    }
    let font = egui::FontId::proportional(11.0);
    for (ang, label) in transform.angular_ticks() {
        let s = transform.screen_angle(ang);
        let pos = Pos2::new(
            center.x + (max_r + 14.0) * s.cos() as f32,
            center.y + (max_r + 14.0) * s.sin() as f32,
        );
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            style.text_color,
        );
    }
    let label_ang = transform.screen_angle(0.25);
    for tick in transform.radial_ticks(5) {
        let norm = transform.normalized_radius(tick.value);
        if !(0.0..=1.0).contains(&norm) {
            continue;
        }
        let rad = norm as f32 * max_r;
        let pos = Pos2::new(
            center.x + rad * label_ang.cos() as f32 + 3.0,
            center.y + rad * label_ang.sin() as f32 - 4.0,
        );
        painter.text(
            pos,
            Align2::LEFT_BOTTOM,
            tick.label,
            egui::FontId::proportional(10.0),
            style.text_color.gamma_multiply(0.8),
        );
    }
    if !title.is_empty() {
        painter.text(
            Pos2::new(center.x, center.y - max_r - 22.0),
            Align2::CENTER_BOTTOM,
            title,
            egui::FontId::proportional(14.0),
            style.text_color,
        );
    }
}

pub(super) fn sector_mesh(
    transform: PolarTransform,
    start_theta: f64,
    span_theta: f64,
    inner_px: f32,
    outer_px: f32,
    color: Color32,
) -> Mesh {
    let mut mesh = Mesh::default();
    if color == Color32::TRANSPARENT || outer_px <= inner_px || span_theta <= 1e-6 {
        return mesh;
    }
    let steps = ((span_theta.abs() * f64::from(outer_px) * 0.15).ceil() as usize).clamp(8, 72);
    let center = transform.center();
    for i in 0..steps {
        let a0 = start_theta + span_theta * (i as f64 / steps as f64);
        let a1 = start_theta + span_theta * ((i + 1) as f64 / steps as f64);
        let mid = 0.5 * (a0 + a1);
        if !transform.contains_theta(mid) {
            continue;
        }
        let s0 = transform.screen_angle(a0);
        let s1 = transform.screen_angle(a1);
        let p_in0 = Pos2::new(
            center.x + inner_px * s0.cos() as f32,
            center.y + inner_px * s0.sin() as f32,
        );
        let p_in1 = Pos2::new(
            center.x + inner_px * s1.cos() as f32,
            center.y + inner_px * s1.sin() as f32,
        );
        let p_out0 = Pos2::new(
            center.x + outer_px * s0.cos() as f32,
            center.y + outer_px * s0.sin() as f32,
        );
        let p_out1 = Pos2::new(
            center.x + outer_px * s1.cos() as f32,
            center.y + outer_px * s1.sin() as f32,
        );
        let base = mesh.vertices.len() as u32;
        if inner_px <= 0.25 {
            mesh.vertices.push(vtx(center, color));
            mesh.vertices.push(vtx(p_out0, color));
            mesh.vertices.push(vtx(p_out1, color));
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
        } else {
            mesh.vertices.push(vtx(p_in0, color));
            mesh.vertices.push(vtx(p_out0, color));
            mesh.vertices.push(vtx(p_out1, color));
            mesh.vertices.push(vtx(p_in1, color));
            mesh.indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    mesh
}

pub(super) fn arc_points(
    transform: PolarTransform,
    start_theta: f64,
    span_theta: f64,
    radius_px: f32,
    steps: usize,
) -> Vec<Pos2> {
    let center = transform.center();
    let mut out = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let th = start_theta + span_theta * (i as f64 / steps as f64);
        if transform.contains_theta(th) || i == 0 || i == steps {
            let s = transform.screen_angle(th);
            out.push(Pos2::new(
                center.x + radius_px * s.cos() as f32,
                center.y + radius_px * s.sin() as f32,
            ));
        }
    }
    out
}

fn vtx(pos: Pos2, color: Color32) -> epaint::Vertex {
    epaint::Vertex {
        pos,
        uv: epaint::WHITE_UV,
        color,
    }
}
