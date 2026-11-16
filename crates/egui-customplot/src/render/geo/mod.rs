mod layers;

use egui::{Align2, Color32, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

use crate::{
    AxisScale, GeoAttribution, GeoBasemapStyle, GeoDiagnostics, GeoLayer, GeoOfflinePolicy,
    GeoPlotState, GeoTileCache,
    axis::{GeoTransform, format_lat, format_lon, geo_graticule_ticks, wrap_lon_180},
    data::{OFFLINE_LAND_RINGS, triangulate_indexed_polygon, visible_tiles_for_bounds},
    model::ResolvedPlotStyle,
};

const MAX_TILE_SAMPLE_RES: u16 = 32;
const MAX_QUADS_PER_MESH: usize = 8_192;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_geo(
    painter: &Painter,
    ui: &egui::Ui,
    state: &GeoPlotState,
    layers: &[GeoLayer<'_>],
    transform: GeoTransform,
    basemap: GeoBasemapStyle,
    mut tile_cache: Option<&mut GeoTileCache>,
    attribution: Option<&GeoAttribution>,
    title: &str,
    show_grid: bool,
    pixels_per_point: f32,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut GeoDiagnostics,
) {
    let vp = transform.viewport();
    let ocean = match basemap {
        GeoBasemapStyle::OfflineVectorDark => Color32::from_rgb(20, 32, 48),
        GeoBasemapStyle::OfflineVectorLight | GeoBasemapStyle::HostTiles => {
            Color32::from_rgb(225, 238, 248)
        }
        GeoBasemapStyle::None => style.background,
    };
    painter.rect_filled(vp, 0.0, ocean);

    let map_painter = painter.with_clip_rect(vp);
    let mut offline_notice = false;
    match basemap {
        GeoBasemapStyle::None => {}
        GeoBasemapStyle::OfflineVectorLight => {
            paint_offline_coastlines(&map_painter, transform, false, diagnostics);
        }
        GeoBasemapStyle::OfflineVectorDark => {
            paint_offline_coastlines(&map_painter, transform, true, diagnostics);
        }
        GeoBasemapStyle::HostTiles => {
            if let Some(cache) = tile_cache.as_deref_mut() {
                offline_notice = paint_host_tiles(&map_painter, transform, cache, diagnostics);
            } else {
                diagnostics.offline_fallback_tiles += 1;
                offline_notice = true;
                paint_offline_coastlines(&map_painter, transform, false, diagnostics);
            }
        }
    }

    if show_grid {
        paint_graticule(&map_painter, painter, transform, style);
    }

    layers::paint_geo_layers(
        &map_painter,
        state,
        layers,
        transform,
        pixels_per_point,
        style,
        diagnostics,
    );

    painter.rect_stroke(vp, 0.0, style.border_stroke, StrokeKind::Inside);

    if !title.is_empty() {
        let title_font = egui::TextStyle::Heading.resolve(ui.style());
        painter.text(
            Pos2::new(vp.center().x, vp.top() - 6.0),
            Align2::CENTER_BOTTOM,
            title,
            title_font,
            style.text_color,
        );
    }

    let resolved_attr = attribution
        .or_else(|| tile_cache.as_ref().map(|c| c.attribution()))
        .filter(|a| !a.text().trim().is_empty());
    let attr_text = if basemap == GeoBasemapStyle::HostTiles && offline_notice {
        match resolved_attr {
            Some(a) => format!("{} (offline fallback active)", a.text()),
            None => "Offline fallback coastlines (no tile cache)".to_owned(),
        }
    } else if let Some(a) = resolved_attr {
        if a.license().is_empty() {
            a.text().to_owned()
        } else {
            format!("{} [{}]", a.text(), a.license())
        }
    } else if basemap != GeoBasemapStyle::None {
        GeoAttribution::builtin_offline().text().to_owned()
    } else {
        String::new()
    };

    if !attr_text.is_empty() && vp.width() >= 120.0 {
        let font = egui::TextStyle::Small.resolve(ui.style());
        let bg = Color32::from_rgba_unmultiplied(15, 18, 24, 160);
        let badge_pos = Pos2::new(vp.right() - 6.0, vp.bottom() - 4.0);
        let est_w = (attr_text.chars().count() as f32 * 5.6 + 10.0).min(vp.width() - 12.0);
        let badge_rect = Rect::from_min_max(
            Pos2::new(
                (badge_pos.x - est_w).max(vp.left() + 4.0),
                badge_pos.y - 16.0,
            ),
            Pos2::new(vp.right() - 2.0, vp.bottom() - 2.0),
        );
        painter.rect_filled(badge_rect, 2.0, bg);
        painter.with_clip_rect(badge_rect).text(
            Pos2::new(badge_rect.right() - 4.0, badge_rect.center().y),
            Align2::RIGHT_CENTER,
            attr_text,
            font,
            Color32::from_rgb(235, 240, 245),
        );
    }
}

fn paint_offline_coastlines(
    painter: &Painter,
    transform: GeoTransform,
    dark: bool,
    diagnostics: &mut GeoDiagnostics,
) {
    let bounds = transform.bounds();
    let half_span = bounds.lon_span * 0.5;
    let vp = transform.viewport();
    let (land_fill, coast_stroke) = if dark {
        (
            Color32::from_rgb(44, 58, 72),
            Stroke::new(1.0, Color32::from_rgb(92, 118, 138)),
        )
    } else {
        (
            Color32::from_rgb(236, 234, 224),
            Stroke::new(1.0, Color32::from_rgb(164, 162, 148)),
        )
    };
    let y_min = transform.projection().lat_to_norm_y(bounds.lat_max);
    let y_max = transform.projection().lat_to_norm_y(bounds.lat_min);

    let to_screen = |lat: f64, rel_lon: f64| -> Pos2 {
        let u = (rel_lon + half_span) / bounds.lon_span.max(1e-9);
        let wy = transform.projection().lat_to_norm_y(lat);
        let v = 1.0 - (wy - y_min) / (y_max - y_min).max(1e-9);
        transform.pos_from_norm([u, v])
    };

    for &ring in OFFLINE_LAND_RINGS {
        for shift in [-360.0_f64, 0.0, 360.0] {
            let mut min_rel = f64::INFINITY;
            let mut max_rel = f64::NEG_INFINITY;
            let mut verts = Vec::with_capacity(ring.len());
            let mut screen = Vec::with_capacity(ring.len());
            for &[lat, lon] in ring {
                let rel = (lon + shift) - bounds.center_lon;
                min_rel = min_rel.min(rel);
                max_rel = max_rel.max(rel);
                let p = to_screen(lat, rel);
                verts.push([f64::from(p.x), f64::from(p.y)]);
                screen.push(p);
            }
            if max_rel < -half_span || min_rel > half_span {
                continue;
            }
            let outer: Vec<usize> = (0..verts.len()).collect();
            if let Some(tris) =
                triangulate_indexed_polygon(&verts, &outer, &[], [AxisScale::Linear; 2])
            {
                let mut mesh = Mesh::default();
                for p in &screen {
                    mesh.colored_vertex(*p, land_fill);
                }
                for [a, b, c] in tris {
                    mesh.add_triangle(a as u32, b as u32, c as u32);
                }
                if !mesh.is_empty() {
                    painter.add(Shape::mesh(mesh));
                    diagnostics.output_basemap_polygons += 1;
                }
            }
            if screen.len() >= 3 {
                painter.add(Shape::closed_line(screen, coast_stroke));
            }
        }
    }
    let _ = vp;
}

fn paint_host_tiles(
    painter: &Painter,
    transform: GeoTransform,
    cache: &mut GeoTileCache,
    diagnostics: &mut GeoDiagnostics,
) -> bool {
    let bounds = transform.bounds();
    let half_span = bounds.lon_span * 0.5;
    let desired_z = transform.zoom_level_equivalent().round().clamp(0.0, 6.0) as u8;
    let z = cache.best_zoom_for(desired_z);
    let keys = visible_tiles_for_bounds(bounds, z, 64);
    let vp = transform.viewport();
    let n = (1_u32 << z) as f64;
    let mut missed_rects: Vec<Rect> = Vec::new();
    let mut batch = Mesh::default();
    let mut batch_quads = 0usize;

    for key in keys {
        let tb = key.tile_bounds();
        let lon_w = (f64::from(key.x) / n) * 360.0 - 180.0;
        let lon_e = (f64::from(key.x + 1) / n) * 360.0 - 180.0;
        let mut tile_rects = Vec::new();
        for shift in [-360.0_f64, 0.0, 360.0] {
            let rw = (lon_w + shift) - bounds.center_lon;
            let re = (lon_e + shift) - bounds.center_lon;
            if re < -half_span || rw > half_span {
                continue;
            }
            let u0 = ((rw + half_span) / bounds.lon_span) as f32;
            let u1 = ((re + half_span) / bounds.lon_span) as f32;
            let x0 = vp.left() + u0 * vp.width();
            let x1 = vp.left() + u1 * vp.width();
            let y_top = transform
                .project_unclipped(crate::GeoPoint::new(tb.lat_max, bounds.center_lon))
                .map_or(vp.top(), |p| p.y);
            let y_bot = transform
                .project_unclipped(crate::GeoPoint::new(tb.lat_min, bounds.center_lon))
                .map_or(vp.bottom(), |p| p.y);
            let rect = Rect::from_min_max(Pos2::new(x0, y_top), Pos2::new(x1, y_bot));
            if rect.is_positive() {
                tile_rects.push(rect);
            }
        }
        if tile_rects.is_empty() {
            continue;
        }
        if let Some(tile) = cache.get(key) {
            diagnostics.tile_cache_hits += 1;
            let step_x = (tile.width() / MAX_TILE_SAMPLE_RES).max(1) as usize;
            let step_y = (tile.height() / MAX_TILE_SAMPLE_RES).max(1) as usize;
            let w = usize::from(tile.width());
            let h = usize::from(tile.height());
            for tr in tile_rects {
                for py in (0..h).step_by(step_y) {
                    let ny0 = py as f32 / h as f32;
                    let ny1 = (py + step_y).min(h) as f32 / h as f32;
                    for px in (0..w).step_by(step_x) {
                        let nx0 = px as f32 / w as f32;
                        let nx1 = (px + step_x).min(w) as f32 / w as f32;
                        let [r, g, b, a] = tile.pixel(px, py);
                        if a == 0 {
                            continue;
                        }
                        let cell = Rect::from_min_max(
                            Pos2::new(tr.left() + nx0 * tr.width(), tr.top() + ny0 * tr.height()),
                            Pos2::new(tr.left() + nx1 * tr.width(), tr.top() + ny1 * tr.height()),
                        );
                        batch.add_colored_rect(cell, Color32::from_rgba_unmultiplied(r, g, b, a));
                        batch_quads += 1;
                        if batch_quads >= MAX_QUADS_PER_MESH {
                            painter.add(Shape::mesh(std::mem::take(&mut batch)));
                            batch_quads = 0;
                        }
                    }
                }
            }
        } else {
            diagnostics.tile_cache_misses += 1;
            diagnostics.offline_fallback_tiles += 1;
            missed_rects.extend(tile_rects);
        }
    }
    if !batch.is_empty() {
        painter.add(Shape::mesh(batch));
    }

    let policy = cache.offline_policy();
    for mr in &missed_rects {
        let clipped = mr.intersect(vp);
        if !clipped.is_positive() {
            continue;
        }
        match policy {
            GeoOfflinePolicy::FallbackToVector => {
                let sub = painter.with_clip_rect(clipped);
                paint_offline_coastlines(&sub, transform, false, diagnostics);
            }
            GeoOfflinePolicy::BlankWithNotice => {
                painter.rect_stroke(
                    clipped,
                    0.0,
                    Stroke::new(1.0, Color32::from_gray(150)),
                    StrokeKind::Inside,
                );
            }
        }
    }
    !missed_rects.is_empty()
}

fn paint_graticule(
    map_painter: &Painter,
    label_painter: &Painter,
    transform: GeoTransform,
    style: &ResolvedPlotStyle<'_>,
) {
    let bounds = transform.bounds();
    let vp = transform.viewport();
    let (lats, rel_lons) = geo_graticule_ticks(bounds, 5);
    let font = egui::TextStyle::Small.resolve(&egui::Style::default());
    let stroke = style.grid_stroke;

    for lat in lats {
        let Some(py) = transform.project_unclipped(crate::GeoPoint::new(lat, bounds.center_lon))
        else {
            continue;
        };
        if py.y < vp.top() || py.y > vp.bottom() {
            continue;
        }
        map_painter.line_segment(
            [Pos2::new(vp.left(), py.y), Pos2::new(vp.right(), py.y)],
            stroke,
        );
        label_painter.text(
            Pos2::new(vp.left() - 4.0, py.y),
            Align2::RIGHT_CENTER,
            format_lat(lat),
            font.clone(),
            style.text_color,
        );
    }

    for rel in rel_lons {
        let lon = wrap_lon_180(bounds.center_lon + rel);
        let u = (rel + bounds.lon_span * 0.5) / bounds.lon_span.max(1e-9);
        let px = vp.left() + u as f32 * vp.width();
        if px < vp.left() - 1.0 || px > vp.right() + 1.0 {
            continue;
        }
        map_painter.line_segment(
            [Pos2::new(px, vp.top()), Pos2::new(px, vp.bottom())],
            stroke,
        );
        label_painter.text(
            Pos2::new(px, vp.bottom() + 4.0),
            Align2::CENTER_TOP,
            format_lon(lon),
            font.clone(),
            style.text_color,
        );
    }
}
