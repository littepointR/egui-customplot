use egui::{Align2, Color32, FontId, Mesh, Painter, Pos2, Shape, Stroke, epaint};

use crate::{
    BubbleCloud, PieChart, PlotDiagnostics, PlotTransform, WordCloud, model::ResolvedSeriesStyle,
};

const DEFAULT_PALETTE: [Color32; 8] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(237, 177, 32),
    Color32::from_rgb(126, 47, 142),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(77, 190, 238),
    Color32::from_rgb(162, 20, 47),
    Color32::from_rgb(90, 120, 155),
];

fn slice_color(palette: Option<&[Color32]>, index: usize) -> Color32 {
    palette.map_or(DEFAULT_PALETTE[index % DEFAULT_PALETTE.len()], |p| {
        p[index % p.len()]
    })
}

pub(super) fn paint_pie_chart(
    painter: &Painter,
    pie: PieChart<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let mut mesh = Mesh::default();
    let mut outlines: Vec<Vec<Pos2>> = Vec::new();
    let mut label_positions: Vec<(Pos2, String)> = Vec::new();
    let inner_r = f64::from(pie.inner_ratio);

    for slice in pie.data.slices() {
        let color = slice_color(pie.palette, slice.index);
        let mid = 0.5 * (slice.start_angle + slice.end_angle);
        let shift = if slice.explode { 0.08_f64 } else { 0.0 };
        let cx = shift * f64::from(mid.cos());
        let cy = shift * f64::from(mid.sin());
        let span = (slice.end_angle - slice.start_angle).max(1e-4);
        let steps = ((span / std::f32::consts::TAU) * 64.0)
            .ceil()
            .clamp(6.0, 64.0) as usize;

        let mut outer_pts = Vec::with_capacity(steps + 1);
        let mut inner_pts = Vec::with_capacity(steps + 1);
        for k in 0..=steps {
            let a = slice.start_angle + span * (k as f32 / steps as f32);
            let ca = f64::from(a.cos());
            let sa = f64::from(a.sin());
            outer_pts.push(transform.position_from_point([cx + ca, cy + sa]));
            if inner_r > 0.0 {
                inner_pts
                    .push(transform.position_from_point([cx + inner_r * ca, cy + inner_r * sa]));
            }
        }
        if inner_r > 0.0 {
            let base = mesh.vertices.len() as u32;
            for k in 0..=steps {
                mesh.colored_vertex(outer_pts[k], color);
                mesh.colored_vertex(inner_pts[k], color);
            }
            for k in 0..steps as u32 {
                let i = base + k * 2;
                mesh.add_triangle(i, i + 1, i + 2);
                mesh.add_triangle(i + 1, i + 3, i + 2);
            }
            let mut ring = outer_pts.clone();
            ring.extend(inner_pts.iter().rev().copied());
            ring.push(outer_pts[0]);
            outlines.push(ring);
        } else {
            let center_pos = transform.position_from_point([cx, cy]);
            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(center_pos, color);
            for p in &outer_pts {
                mesh.colored_vertex(*p, color);
            }
            for k in 0..steps as u32 {
                mesh.add_triangle(base, base + 1 + k, base + 2 + k);
            }
            let mut wedge = Vec::with_capacity(outer_pts.len() + 2);
            wedge.push(center_pos);
            wedge.extend(outer_pts);
            wedge.push(center_pos);
            outlines.push(wedge);
        }
        if pie.show_labels && span >= 0.14 {
            let lr = if inner_r > 0.0 {
                0.5 * (inner_r + 1.0)
            } else {
                0.65
            };
            let lp = transform.position_from_point([
                cx + lr * f64::from(mid.cos()),
                cy + lr * f64::from(mid.sin()),
            ]);
            label_positions.push((lp, slice.label.clone()));
        }
        diagnostics.stats.output_slices += 1;
    }

    if !mesh.is_empty() {
        diagnostics.geometry_capacity_bytes += mesh.vertices.capacity()
            * std::mem::size_of::<epaint::Vertex>()
            + mesh.indices.capacity() * std::mem::size_of::<u32>();
        painter.add(Shape::mesh(mesh));
    }
    if !pie.border.is_empty() && pie.border.width.is_finite() && pie.border.width > 0.0 {
        for poly in outlines {
            painter.add(Shape::line(poly, pie.border));
        }
    }
    if !label_positions.is_empty() {
        let font = FontId::proportional(11.0);
        for (pos, text) in label_positions {
            painter.text(pos, Align2::CENTER_CENTER, text, font.clone(), text_color);
        }
    }
}

pub(super) fn paint_word_cloud(
    painter: &Painter,
    cloud: WordCloud<'_>,
    transform: PlotTransform,
    _resolved: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    let origin = transform.position_from_point([0.0, 0.0]);
    let unit = transform.position_from_point([1.0, 0.0]);
    let zoom =
        ((unit.x - origin.x).abs() / (transform.rect().width().max(1.0) * 0.35)).clamp(0.65, 1.5);
    for w in cloud.data.words() {
        if !transform.is_valid_point(w.center) {
            continue;
        }
        let pos = transform.position_from_point(w.center);
        if !transform.rect().contains(pos) {
            continue;
        }
        let px = (cloud.base_font_px * w.font_scale * zoom).clamp(9.0, 56.0);
        let color = slice_color(cloud.palette, w.color_index);
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            &w.text,
            FontId::proportional(px),
            color,
        );
        diagnostics.stats.output_clouds += 1;
    }
}

pub(super) fn paint_bubble_cloud(
    painter: &Painter,
    cloud: BubbleCloud<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let mut mesh = Mesh::default();
    let mut outlines: Vec<(Pos2, f32)> = Vec::new();
    let mut labels: Vec<(Pos2, String, f32)> = Vec::new();
    let segs = 32usize;

    for b in cloud.data.bubbles() {
        if !transform.is_valid_point(b.center) {
            continue;
        }
        let center = transform.position_from_point(b.center);
        let edge_x = transform.position_from_point([b.center[0] + b.radius, b.center[1]]);
        let edge_y = transform.position_from_point([b.center[0], b.center[1] + b.radius]);
        let rx = (edge_x.x - center.x).abs();
        let ry = (edge_y.y - center.y).abs();
        let r_px = rx.min(ry);
        if !r_px.is_finite() || r_px < 1.0 {
            continue;
        }
        let color = slice_color(cloud.palette, b.group).gamma_multiply(cloud.alpha);
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(center, color);
        for k in 0..=segs {
            let a = (k as f32 / segs as f32) * std::f32::consts::TAU;
            mesh.colored_vertex(center + egui::vec2(r_px * a.cos(), r_px * a.sin()), color);
        }
        for k in 0..segs as u32 {
            mesh.add_triangle(base, base + 1 + k, base + 2 + k);
        }
        outlines.push((center, r_px));
        if cloud.show_labels && r_px >= 16.0 {
            labels.push((center, b.label.clone(), (r_px * 0.42).clamp(9.0, 18.0)));
        }
        diagnostics.stats.output_clouds += 1;
    }
    if !mesh.is_empty() {
        diagnostics.geometry_capacity_bytes += mesh.vertices.capacity()
            * std::mem::size_of::<epaint::Vertex>()
            + mesh.indices.capacity() * std::mem::size_of::<u32>();
        painter.add(Shape::mesh(mesh));
    }
    if !cloud.border.is_empty() && cloud.border.width.is_finite() && cloud.border.width > 0.0 {
        for (c, r) in outlines {
            painter.circle_stroke(c, r, Stroke::new(cloud.border.width, cloud.border.color));
        }
    }
    for (c, text, px) in labels {
        painter.text(
            c,
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(px),
            text_color,
        );
    }
}
