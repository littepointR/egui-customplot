use egui::{Color32, Mesh, Painter, Pos2, Shape, Stroke, epaint};

use super::{markers::prepare_markers, mesh::MeshBatch};
use crate::{
    BinScatter, BinShape, BubbleChart, Heatmap, Histogram2, MarkerShape, MarkerStyle,
    PlotDiagnostics, PlotPoint, PlotTransform, SwarmChart,
    axis::{clipped_data_rect, valid_extent},
    model::ResolvedSeriesStyle,
};

pub(super) fn paint_bubble_chart(
    painter: &Painter,
    bubble: BubbleChart<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let points = bubble.data.points();
    if points.is_empty() {
        return;
    }
    let base_color = resolved.stroke.color.gamma_multiply(bubble.alpha);
    let mut xy = Vec::with_capacity(points.len());
    let mut radii = Vec::with_capacity(points.len());
    let mut colors = Vec::with_capacity(points.len());
    let scalar_vals = bubble.data.color_values();

    for (i, pt) in points.iter().enumerate() {
        xy.push([pt.x, pt.y]);
        let r = if pt.size.is_finite() && pt.size >= 0.0 {
            bubble.data.pixel_radius(pt.size)
        } else {
            f32::NAN
        };
        radii.push(r);
        let c = if let Some(scale) = bubble.colors {
            let v = scalar_vals.map_or(pt.size, |s| s[i]);
            scale
                .color(v)
                .map_or(Color32::TRANSPARENT, |col| col.gamma_multiply(bubble.alpha))
        } else {
            base_color
        };
        colors.push(c);
    }

    let marker_style = MarkerStyle {
        shape: MarkerShape::Circle,
        radius: 6.0,
    };
    let prepared = prepare_markers(
        &xy,
        transform,
        marker_style,
        base_color,
        Some(&colors),
        Some(&radii),
        pixels_per_point,
    );
    diagnostics.visited_marker_points += prepared.visited_points;
    diagnostics.output_markers += prepared.output_markers;
    diagnostics.stats.output_bubbles += prepared.output_markers;
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    diagnostics.geometry_capacity_bytes += prepared.mesh.vertices.capacity()
        * std::mem::size_of::<epaint::Vertex>()
        + prepared.mesh.indices.capacity() * std::mem::size_of::<u32>();
    if !prepared.mesh.is_empty() {
        painter.add(Shape::mesh(prepared.mesh));
    }

    if let Some(border) = bubble.border
        && !border.is_empty()
        && border.width.is_finite()
        && border.width > 0.0
    {
        for (i, pt) in points.iter().enumerate() {
            let p = [pt.x, pt.y];
            let r = radii[i];
            if !r.is_finite()
                || r <= 0.0
                || colors[i] == Color32::TRANSPARENT
                || !transform.is_valid_point(p)
            {
                continue;
            }
            let pos = transform.position_from_point(p);
            if pos.is_finite() && transform.rect().expand(r).contains(pos) {
                painter.circle_stroke(pos, r, border);
            }
        }
    }
}

pub(super) fn paint_swarm_chart(
    painter: &Painter,
    swarm: SwarmChart<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let pts = swarm.data.points();
    if pts.is_empty() {
        return;
    }
    let xy: Vec<PlotPoint> = pts.iter().map(|p| [p.x, p.y]).collect();
    let colors: Option<Vec<Color32>> = swarm.colors.map(|scale| {
        let scalars = swarm.data.color_values();
        pts.iter()
            .map(|p| {
                let v = scalars
                    .and_then(|s| s.get(p.sample_index).copied())
                    .unwrap_or(p.group_x);
                scale.color(v).unwrap_or(Color32::TRANSPARENT)
            })
            .collect()
    });
    let prepared = prepare_markers(
        &xy,
        transform,
        resolved.marker,
        resolved.stroke.color,
        colors.as_deref(),
        None,
        pixels_per_point,
    );
    diagnostics.visited_marker_points += prepared.visited_points;
    diagnostics.output_markers += prepared.output_markers;
    diagnostics.stats.output_bubbles += prepared.output_markers;
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    if !prepared.mesh.is_empty() {
        painter.add(Shape::mesh(prepared.mesh));
    }
}

pub(super) fn paint_histogram2(
    painter: &Painter,
    hist: Histogram2<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let Ok(grid) = hist.data.to_grid(hist.normalization) else {
        diagnostics.numeric_rejections += 1;
        return;
    };
    let heatmap = Heatmap::new(&grid, hist.colors);
    let before = diagnostics.output_heatmap_cells;
    super::heatmap::paint_heatmap(painter, heatmap, transform, diagnostics);
    let drawn = diagnostics.output_heatmap_cells.saturating_sub(before);
    diagnostics.stats.output_bins += drawn;

    if hist.show_counts && drawn > 0 && drawn <= 144 {
        let font = egui::FontId::proportional(10.0);
        let nx = hist.data.nx();
        let ny = hist.data.ny();
        for row in 0..ny {
            for col in 0..nx {
                let count = hist.data.counts()[row * nx + col];
                if count == 0 {
                    continue;
                }
                if let Some(bounds) = grid.cell_bounds(col, row)
                    && let Some(rect) = crate::axis::grid_cell_rect(bounds, transform)
                {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        count.to_string(),
                        font.clone(),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

pub(super) fn paint_bin_scatter(
    painter: &Painter,
    bins: BinScatter<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let cells = bins.data.cells();
    if cells.is_empty() {
        return;
    }
    let mut mesh = Mesh::default();
    let mut batch = MeshBatch::default();
    let use_hex = bins.data.shape() == BinShape::Hexagon;
    for cell in cells {
        let Some(color) = bins.colors.color(cell.count as f64) else {
            continue;
        };
        let lo = [cell.x0, cell.y0];
        let hi = [cell.x1, cell.y1];
        if !valid_extent(lo, hi, transform) {
            continue;
        }
        if !use_hex {
            if let Some(rect) = clipped_data_rect(lo, hi, transform) {
                mesh.add_colored_rect(rect, color);
                diagnostics.stats.output_bins += 1;
                if !bins.border.is_empty()
                    && bins.border.width.is_finite()
                    && bins.border.width > 0.0
                {
                    painter.rect_stroke(rect, 0.0, bins.border, egui::StrokeKind::Inside);
                }
            }
        } else {
            let rx = (cell.x1 - cell.x0) * 0.56;
            let ry = (cell.y1 - cell.y0) * 0.62;
            let mut ring = Vec::with_capacity(6);
            for k in 0..6 {
                let a = (k as f64) * std::f64::consts::FRAC_PI_3 + std::f64::consts::FRAC_PI_6;
                let pt = [cell.cx + rx * a.cos(), cell.cy + ry * a.sin()];
                if transform.is_valid_point(pt) {
                    ring.push(transform.normalized(pt));
                }
            }
            if ring.len() == 6 {
                let center_n = transform.normalized([cell.cx, cell.cy]);
                for k in 0..6 {
                    let next = (k + 1) % 6;
                    batch.add_flat_triangle(
                        painter,
                        [center_n, ring[k], ring[next]],
                        color,
                        transform,
                    );
                }
                diagnostics.stats.output_bins += 1;
                if !bins.border.is_empty()
                    && bins.border.width.is_finite()
                    && bins.border.width > 0.0
                {
                    let mut screen_pts: Vec<Pos2> = ring
                        .iter()
                        .map(|&n| transform.position_from_normalized(n))
                        .collect();
                    if let Some(&first) = screen_pts.first() {
                        screen_pts.push(first);
                        painter.add(Shape::line(
                            screen_pts,
                            Stroke::new(bins.border.width, bins.border.color),
                        ));
                    }
                }
            }
        }
    }
    if !mesh.is_empty() {
        diagnostics.geometry_capacity_bytes += mesh.vertices.capacity()
            * std::mem::size_of::<epaint::Vertex>()
            + mesh.indices.capacity() * std::mem::size_of::<u32>();
        painter.add(Shape::mesh(mesh));
    }
    let stats = batch.finish(painter);
    diagnostics.geometry_capacity_bytes += stats.bytes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}
