use egui::{Align2, Color32, FontId, Mesh, Painter, Pos2, Shape, Stroke, epaint};

use crate::{
    GeoGlobeData, GeoGlobeTransform, HoveredGlobeItem, data::OFFLINE_LAND_RINGS,
    model::ResolvedPlotStyle,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_geoglobe(
    painter: &Painter,
    data: &GeoGlobeData,
    transform: GeoGlobeTransform,
    show_graticule: bool,
    show_coastlines: bool,
    title: &str,
    style: &ResolvedPlotStyle<'_>,
    hovered: Option<&HoveredGlobeItem>,
    output_paths: &mut usize,
) {
    let center = transform.screen_center();
    let r = transform.radius_px();
    painter.rect_filled(transform.rect(), 0.0, style.background);

    // Atmosphere halo + deep ocean sphere shading
    painter.circle_filled(
        center,
        r * 1.03,
        Color32::from_rgba_unmultiplied(65, 135, 220, 32),
    );
    painter.circle_filled(center, r, Color32::from_rgb(18, 46, 84));
    painter.circle_filled(
        center - egui::vec2(r * 0.22, r * 0.24),
        r * 0.72,
        Color32::from_rgba_unmultiplied(46, 108, 176, 52),
    );

    if show_graticule {
        paint_globe_graticule(painter, transform, output_paths);
    }

    if show_coastlines {
        let land_stroke = Stroke::new(1.15, Color32::from_rgb(138, 198, 146));
        for ring in OFFLINE_LAND_RINGS {
            paint_globe_curve(
                painter,
                transform,
                ring.iter()
                    .chain(ring.first())
                    .map(|&[la, lo]| [la, lo, 0.0]),
                land_stroke,
                output_paths,
            );
        }
    }

    for trk in data.tracks() {
        if trk.extruded && trk.lat_deg.len() >= 2 {
            let fill = trk.color.gamma_multiply(0.28);
            let mut mesh = Mesh::default();
            for i in 0..(trk.lat_deg.len() - 1) {
                if let (Some((g0, _)), Some((g1, _)), Some((t0, _)), Some((t1, _))) = (
                    transform.project(trk.lat_deg[i], trk.lon_deg[i], 0.0),
                    transform.project(trk.lat_deg[i + 1], trk.lon_deg[i + 1], 0.0),
                    transform.project(trk.lat_deg[i], trk.lon_deg[i], trk.alt_m[i]),
                    transform.project(trk.lat_deg[i + 1], trk.lon_deg[i + 1], trk.alt_m[i + 1]),
                ) {
                    let b = mesh.vertices.len() as u32;
                    for p in [g0, g1, t1, t0] {
                        mesh.colored_vertex(p, fill);
                    }
                    mesh.add_triangle(b, b + 1, b + 2);
                    mesh.add_triangle(b, b + 2, b + 3);
                }
            }
            if !mesh.is_empty() {
                painter.add(Shape::mesh(mesh));
            }
            let drop_col = trk.color.gamma_multiply(0.5);
            for i in (0..trk.lat_deg.len()).step_by((trk.lat_deg.len() / 8).max(1)) {
                if let (Some((g, _)), Some((t, _))) = (
                    transform.project(trk.lat_deg[i], trk.lon_deg[i], 0.0),
                    transform.project(trk.lat_deg[i], trk.lon_deg[i], trk.alt_m[i]),
                ) {
                    painter.line_segment([g, t], Stroke::new(1.0, drop_col));
                }
            }
        }
        let trk_stroke = Stroke::new(2.0, trk.color);
        paint_globe_curve(
            painter,
            transform,
            (0..trk.lat_deg.len()).map(|i| [trk.lat_deg[i], trk.lon_deg[i], trk.alt_m[i]]),
            trk_stroke,
            output_paths,
        );
    }

    for m in data.markers() {
        if let Some((pos, _)) = transform.project(m.lat_deg, m.lon_deg, m.alt_m) {
            painter.circle_filled(pos, m.radius_px, m.color);
            painter.circle_stroke(pos, m.radius_px, Stroke::new(1.0, Color32::WHITE));
            if !m.label.is_empty() {
                painter.text(
                    pos + egui::vec2(m.radius_px + 3.0, -3.0),
                    Align2::LEFT_BOTTOM,
                    &m.label,
                    FontId::proportional(9.5),
                    Color32::WHITE,
                );
            }
        }
    }

    painter.circle_stroke(
        center,
        r,
        Stroke::new(1.3, Color32::from_rgb(130, 180, 240)),
    );

    if let Some(hit) = hovered {
        painter.circle_stroke(hit.screen_pos, 6.5, Stroke::new(2.0, Color32::YELLOW));
    }

    if !title.is_empty() {
        let top = transform.rect().top() + 4.0;
        painter.text(
            Pos2::new(transform.rect().center().x, top),
            Align2::CENTER_TOP,
            title,
            FontId::proportional(13.0),
            style.text_color,
        );
    }
}

fn paint_globe_graticule(
    painter: &Painter,
    transform: GeoGlobeTransform,
    output_paths: &mut usize,
) {
    let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(150, 195, 245, 55));
    for lat_i in (-60..=60).step_by(30) {
        let lat = f64::from(lat_i);
        paint_globe_curve(
            painter,
            transform,
            (0..=72).map(|k| [lat, -180.0 + f64::from(k) * 5.0, 0.0]),
            stroke,
            output_paths,
        );
    }
    for lon_i in (-150..=180).step_by(30) {
        let lon = f64::from(lon_i);
        paint_globe_curve(
            painter,
            transform,
            (0..=36).map(|k| [-90.0 + f64::from(k) * 5.0, lon, 0.0]),
            stroke,
            output_paths,
        );
    }
}

fn paint_globe_curve(
    painter: &Painter,
    transform: GeoGlobeTransform,
    pts: impl IntoIterator<Item = [f64; 3]>,
    stroke: Stroke,
    output_paths: &mut usize,
) {
    let mut path: Vec<Pos2> = Vec::new();
    let max_jump = transform.radius_px() * 0.5;
    for [lat, lon, alt] in pts {
        if let Some((pos, _)) = transform.project(lat, lon, alt) {
            if let Some(prev) = path.last()
                && prev.distance(pos) > max_jump
            {
                if path.len() >= 2 {
                    painter.add(epaint::PathShape::line(std::mem::take(&mut path), stroke));
                    *output_paths += 1;
                } else {
                    path.clear();
                }
            }
            path.push(pos);
        } else if path.len() >= 2 {
            painter.add(epaint::PathShape::line(std::mem::take(&mut path), stroke));
            *output_paths += 1;
        } else {
            path.clear();
        }
    }
    if path.len() >= 2 {
        painter.add(epaint::PathShape::line(path, stroke));
        *output_paths += 1;
    }
}
