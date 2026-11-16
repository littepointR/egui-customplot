use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape};

use super::super::{
    markers::{marker_outline, push_marker},
    pattern::apply_pattern,
};
use crate::{
    ColorMap, ColorScale, GeoBubbles, GeoDensity, GeoDiagnostics, GeoLayer, GeoLine, GeoPlotState,
    GeoScatter, MarkerShape,
    axis::{GeoTransform, project_geo_polyline},
    model::{ResolvedPlotStyle, ResolvedSeriesStyle},
};

const MAX_QUADS_PER_MESH: usize = 8_192;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_geo_layers(
    painter: &Painter,
    state: &GeoPlotState,
    layers: &[GeoLayer<'_>],
    transform: GeoTransform,
    pixels_per_point: f32,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut GeoDiagnostics,
) {
    let fringe = 1.0 / pixels_per_point.max(0.01);
    for (idx, layer) in layers.iter().enumerate() {
        let id = layer.id(idx);
        if !state.layer_visible(id) {
            continue;
        }
        let resolved = layer.resolved_style(idx, style);
        match *layer {
            GeoLayer::Density(d) => paint_density(painter, d, transform, diagnostics),
            GeoLayer::Line(l) => paint_line(painter, l, transform, resolved, fringe, diagnostics),
            GeoLayer::Scatter(s) => {
                paint_scatter(painter, s, transform, resolved, fringe, diagnostics)
            }
            GeoLayer::Bubbles(b) => paint_bubbles(
                painter,
                state,
                b,
                transform,
                style,
                resolved,
                fringe,
                diagnostics,
            ),
        }
    }
}

fn paint_density(
    painter: &Painter,
    density: GeoDensity<'_>,
    transform: GeoTransform,
    diagnostics: &mut GeoDiagnostics,
) {
    let grid = density.data.evaluate_viewport(transform);
    diagnostics.visited_items += grid.visited_samples;
    if grid.max_density <= 0.0 {
        return;
    }
    let vp = transform.viewport();
    let scale = density.color_scale.unwrap_or_else(|| {
        ColorScale::new([0.0, grid.max_density])
            .expect("density scale")
            .color_map(ColorMap::Viridis)
    });
    let mut mesh = Mesh::default();
    let mut quads = 0usize;
    let cw = vp.width() / grid.cols as f32;
    let ch = vp.height() / grid.rows as f32;
    for row in 0..grid.rows {
        let y0 = vp.top() + row as f32 * ch;
        let y1 = y0 + ch + 0.35;
        for col in 0..grid.cols {
            let val = grid.values[row * grid.cols + col];
            if val <= grid.max_density * 1e-3 {
                continue;
            }
            let x0 = vp.left() + col as f32 * cw;
            let x1 = x0 + cw + 0.35;
            let frac = (val / grid.max_density).clamp(0.0, 1.0) as f32;
            let base = scale.color(val).unwrap_or(Color32::from_rgb(255, 160, 40));
            let color = base.gamma_multiply(density.alpha * (0.25 + 0.75 * frac));
            mesh.add_colored_rect(
                Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1)),
                color,
            );
            diagnostics.output_density_cells += 1;
            quads += 1;
            if quads >= MAX_QUADS_PER_MESH {
                painter.add(Shape::mesh(std::mem::take(&mut mesh)));
                quads = 0;
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(Shape::mesh(mesh));
    }
}

fn paint_line(
    painter: &Painter,
    line: GeoLine<'_>,
    transform: GeoTransform,
    resolved: ResolvedSeriesStyle,
    fringe: f32,
    diagnostics: &mut GeoDiagnostics,
) {
    let pts = line.data.points();
    diagnostics.visited_items += pts.len();
    let raw_paths = project_geo_polyline(pts, transform, &mut diagnostics.dateline_splits);
    let prepared = apply_pattern(raw_paths, resolved.pattern);
    for seg in prepared.paths {
        if seg.len() >= 2 {
            painter.add(Shape::line(seg, resolved.stroke));
            diagnostics.output_paths += 1;
        }
    }
    if line.show_markers {
        let outline = marker_outline(resolved.marker.shape);
        let mut mesh = Mesh::default();
        for &pt in pts {
            if let Some(pos) = transform.project(pt)
                && push_marker(
                    &mut mesh,
                    pos,
                    resolved.marker.radius,
                    resolved.stroke.color,
                    &outline,
                    fringe,
                )
            {
                diagnostics.output_markers += 1;
            }
        }
        if !mesh.is_empty() {
            painter.add(Shape::mesh(mesh));
        }
    }
}

fn paint_scatter(
    painter: &Painter,
    scatter: GeoScatter<'_>,
    transform: GeoTransform,
    resolved: ResolvedSeriesStyle,
    fringe: f32,
    diagnostics: &mut GeoDiagnostics,
) {
    let outline = marker_outline(resolved.marker.shape);
    let mut mesh = Mesh::default();
    let mut count = 0usize;
    for pt in scatter.data.points() {
        diagnostics.visited_items += 1;
        let Some(pos) = transform.project(pt.point) else {
            continue;
        };
        let radius = pt.size.unwrap_or(resolved.marker.radius);
        let color = if let Some([r, g, b, a]) = pt.rgba {
            Color32::from_rgba_unmultiplied(r, g, b, a)
        } else if let (Some(scale), Some(v)) = (scatter.color_scale, pt.color_value) {
            scale.color(v).unwrap_or(resolved.stroke.color)
        } else {
            resolved.stroke.color
        }
        .gamma_multiply(scatter.alpha);
        if color != Color32::TRANSPARENT && radius > 0.0 {
            if push_marker(&mut mesh, pos, radius, color, &outline, fringe) {
                diagnostics.output_markers += 1;
                count += 1;
            }
            if count >= MAX_QUADS_PER_MESH {
                painter.add(Shape::mesh(std::mem::take(&mut mesh)));
                count = 0;
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(Shape::mesh(mesh));
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_bubbles(
    painter: &Painter,
    state: &GeoPlotState,
    bubbles: GeoBubbles<'_>,
    transform: GeoTransform,
    plot_style: &ResolvedPlotStyle<'_>,
    resolved: ResolvedSeriesStyle,
    fringe: f32,
    diagnostics: &mut GeoDiagnostics,
) {
    let outline = marker_outline(MarkerShape::Circle);
    let cats = bubbles.data.categories();
    let mut mesh = Mesh::default();
    let mut borders: Vec<(Pos2, f32)> = Vec::new();
    for pt in bubbles.data.points() {
        diagnostics.visited_items += 1;
        if !pt.point.is_valid_domain() || !pt.size.is_finite() || pt.size < 0.0 {
            continue;
        }
        if let Some(cat_idx) = pt.category
            && let Some(cat_name) = cats.get(cat_idx)
            && !state.category_visible(cat_name)
        {
            continue;
        }
        let Some(pos) = transform.project(pt.point) else {
            continue;
        };
        let radius = bubbles.data.pixel_radius(pt.size);
        let base_color = if let Some(cat_idx) = pt.category {
            plot_style.palette[cat_idx % plot_style.palette.len()]
        } else {
            resolved.stroke.color
        };
        let fill = base_color.gamma_multiply(bubbles.alpha);
        if push_marker(&mut mesh, pos, radius, fill, &outline, fringe) {
            diagnostics.output_bubbles += 1;
            if bubbles.border.is_some() {
                borders.push((pos, radius));
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(Shape::mesh(mesh));
    }
    if let Some(border) = bubbles.border
        && border.width > 0.0
        && border.color != Color32::TRANSPARENT
    {
        for (pos, radius) in borders {
            painter.circle_stroke(pos, radius, border);
        }
    }
}
