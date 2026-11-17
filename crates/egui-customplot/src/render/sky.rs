use std::f64::consts::TAU;

use egui::{Align2, Color32, FontId, Mesh, Painter, Pos2, Shape, Stroke, Vec2, epaint};

use crate::{
    ColorScale, MarkerShape, PolarDiagnostics, SkyPlotData, SkyPlotState, SkyTransform,
    model::ResolvedPlotStyle,
    render::markers::{marker_outline, push_shaped_marker},
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sky(
    painter: &Painter,
    state: &SkyPlotState,
    data: &SkyPlotData,
    transform: SkyTransform,
    mask_elevation_deg: f64,
    color_scale: Option<ColorScale>,
    marker_shape: MarkerShape,
    marker_radius: f32,
    show_labels: bool,
    show_grid: bool,
    title: &str,
    pixels_per_point: f32,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut PolarDiagnostics,
    visible_series: &mut usize,
) {
    let center = transform.center();
    let max_r = transform.radius_px();
    painter.circle_filled(center, max_r, style.background);

    if mask_elevation_deg > transform.min_elevation_deg() + 1e-6
        && mask_elevation_deg < 90.0
        && let Some(r_frac) = transform.radius_fraction(mask_elevation_deg)
    {
        let r_mask = (r_frac as f32 * max_r).clamp(0.0, max_r);
        let fill = style.text_color.gamma_multiply(0.11);
        let mask_mesh = annulus_mesh(center, r_mask, max_r, fill);
        if !mask_mesh.is_empty() {
            painter.add(Shape::mesh(mask_mesh));
            diagnostics.output_sectors += 1;
        }
        painter.circle_stroke(
            center,
            r_mask,
            Stroke::new(1.0, style.text_color.gamma_multiply(0.45)),
        );
    }

    if show_grid {
        paint_sky_grid(painter, transform, style);
    }

    let mut groups_seen = std::collections::HashSet::new();
    for track in data.tracks() {
        diagnostics.input_points += track.azimuth_deg.len();
        if !state.group_visible(track.group) {
            continue;
        }
        groups_seen.insert(track.group);
        let base_color = style
            .resolve_series(track.group, Default::default())
            .stroke
            .color;
        paint_sky_track(
            painter,
            track,
            transform,
            color_scale,
            base_color,
            diagnostics,
        );
    }

    let mut marker_mesh = Mesh::default();
    let outline = marker_outline(marker_shape);
    let fringe = 1.0 / pixels_per_point.max(0.01);
    for sat in data.satellites() {
        diagnostics.input_points += 1;
        if !state.group_visible(sat.group) {
            continue;
        }
        groups_seen.insert(sat.group);
        diagnostics.visited_items += 1;
        let Some(pos) = transform.project(sat.azimuth_deg, sat.elevation_deg) else {
            diagnostics.clipped_points += 1;
            continue;
        };
        let mut color = style
            .resolve_series(sat.group, Default::default())
            .stroke
            .color;
        if sat.elevation_deg < mask_elevation_deg {
            color = color.gamma_multiply(0.38);
        }
        if push_shaped_marker(
            &mut marker_mesh,
            pos,
            marker_radius,
            color,
            marker_shape,
            &outline,
            fringe,
        ) {
            diagnostics.output_markers += 1;
        }
        if show_labels && !sat.label.is_empty() {
            painter.text(
                pos + Vec2::new(marker_radius + 3.0, -2.0),
                Align2::LEFT_CENTER,
                &sat.label,
                FontId::proportional(10.5),
                if sat.elevation_deg < mask_elevation_deg {
                    style.text_color.gamma_multiply(0.55)
                } else {
                    style.text_color
                },
            );
        }
    }
    if !marker_mesh.is_empty() {
        painter.add(Shape::mesh(marker_mesh));
    }
    *visible_series = groups_seen.len();

    painter.circle_stroke(center, max_r, style.border_stroke);
    if !title.is_empty() {
        painter.text(
            Pos2::new(center.x, center.y - max_r - 22.0),
            Align2::CENTER_BOTTOM,
            title,
            FontId::proportional(14.0),
            style.text_color,
        );
    }
}

fn paint_sky_grid(painter: &Painter, transform: SkyTransform, style: &ResolvedPlotStyle<'_>) {
    let center = transform.center();
    let max_r = transform.radius_px();
    let min_el = transform.min_elevation_deg();
    let label_color = style.text_color.gamma_multiply(0.82);
    for el in [15.0_f64, 30.0, 45.0, 60.0, 75.0] {
        if el <= min_el + 1.0 || el >= 89.0 {
            continue;
        }
        let Some(frac) = transform.radius_fraction(el) else {
            continue;
        };
        let r = frac as f32 * max_r;
        painter.circle_stroke(center, r, style.grid_stroke);
        if let Some(pos) = transform.project(15.0, el) {
            painter.text(
                pos + Vec2::new(3.0, -3.0),
                Align2::LEFT_BOTTOM,
                format!("{el:.0}°"),
                FontId::proportional(10.0),
                label_color,
            );
        }
    }
    let min_label_pos = Pos2::new(center.x, center.y - max_r + 10.0);
    painter.text(
        min_label_pos,
        Align2::CENTER_TOP,
        format!("{min_el:.0}°"),
        FontId::proportional(10.0),
        label_color,
    );

    for k in 0..12 {
        let az = k as f64 * 30.0;
        let rad = az.to_radians();
        let dir = Vec2::new(rad.sin() as f32, -rad.cos() as f32);
        painter.line_segment([center, center + dir * max_r], style.grid_stroke);
        let label = match k {
            0 => "0° N".to_owned(),
            3 => "90° E".to_owned(),
            6 => "180° S".to_owned(),
            9 => "270° W".to_owned(),
            _ => format!("{az:.0}°"),
        };
        painter.text(
            center + dir * (max_r + 12.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.5),
            label_color,
        );
    }
}

fn paint_sky_track(
    painter: &Painter,
    track: &crate::SkyTrack,
    transform: SkyTransform,
    color_scale: Option<ColorScale>,
    base_color: Color32,
    diagnostics: &mut PolarDiagnostics,
) {
    let mut prev: Option<(Pos2, Option<f64>)> = None;
    let mut run: Vec<Pos2> = Vec::new();
    let scalar_track = color_scale.zip(track.scalars.as_deref());
    for i in 0..track.azimuth_deg.len() {
        diagnostics.visited_items += 1;
        let az = track.azimuth_deg[i];
        let el = track.elevation_deg[i];
        let Some(pos) = transform.project(az, el) else {
            diagnostics.clipped_points += 1;
            if run.len() >= 2 {
                painter.add(Shape::line(
                    std::mem::take(&mut run),
                    Stroke::new(1.8, base_color),
                ));
                diagnostics.output_paths += 1;
            } else {
                run.clear();
            }
            prev = None;
            continue;
        };
        if let Some((cs, scalars)) = scalar_track {
            let s = scalars[i];
            if let Some((prev_pos, prev_s)) = prev {
                let mid_val = match prev_s {
                    Some(a) if a.is_finite() && s.is_finite() => 0.5 * (a + s),
                    _ => s,
                };
                let c = cs.color(mid_val).unwrap_or(base_color);
                painter.line_segment([prev_pos, pos], Stroke::new(2.0, c));
                diagnostics.output_paths += 1;
            }
            prev = Some((pos, Some(s)));
        } else {
            run.push(pos);
        }
    }
    if run.len() >= 2 {
        painter.add(Shape::line(run, Stroke::new(1.8, base_color)));
        diagnostics.output_paths += 1;
    }
}

fn annulus_mesh(center: Pos2, inner_r: f32, outer_r: f32, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    if outer_r <= inner_r || color == Color32::TRANSPARENT {
        return mesh;
    }
    let steps = 64usize;
    for i in 0..=steps {
        let ang = i as f64 / steps as f64 * TAU;
        let dir = Vec2::new(ang.sin() as f32, -ang.cos() as f32);
        mesh.vertices.push(epaint::Vertex {
            pos: center + dir * inner_r,
            uv: epaint::WHITE_UV,
            color,
        });
        mesh.vertices.push(epaint::Vertex {
            pos: center + dir * outer_r,
            uv: epaint::WHITE_UV,
            color,
        });
    }
    for i in 0..steps as u32 {
        let a = i * 2;
        mesh.indices
            .extend_from_slice(&[a, a + 1, a + 3, a, a + 3, a + 2]);
    }
    mesh
}
