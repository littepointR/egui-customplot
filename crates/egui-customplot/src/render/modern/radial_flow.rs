use egui::{Align2, Color32, FontId, Painter, Shape, Stroke, epaint};

use crate::{
    PlotDiagnostics, PlotPoint, PlotTransform, RadarChart, SankeyChart,
    axis::{clipped_axis_segment, clipped_data_rect, valid_extent},
    model::ResolvedSeriesStyle,
    render::mesh::MeshBatch,
};

pub(super) const MODERN_PALETTE: [Color32; 8] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(237, 177, 32),
    Color32::from_rgb(126, 47, 142),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(77, 190, 238),
    Color32::from_rgb(162, 20, 47),
    Color32::from_rgb(95, 115, 140),
];

pub(super) fn pick_palette(palette: Option<&[Color32]>, idx: usize) -> Color32 {
    palette.map_or(MODERN_PALETTE[idx % MODERN_PALETTE.len()], |p| {
        p[idx % p.len()]
    })
}

pub(super) fn paint_radar_chart(
    painter: &Painter,
    radar: RadarChart<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = radar.data;
    let dirs = data.spoke_dirs();
    let grid_col = text_color.gamma_multiply(0.25);
    let grid_stroke = Stroke::new(1.0, grid_col);
    let mut ring_pts: Vec<PlotPoint> = Vec::with_capacity(dirs.len() + 1);

    for r_i in 1..=radar.rings {
        let frac = r_i as f64 / radar.rings as f64;
        ring_pts.clear();
        for &d in dirs {
            ring_pts.push([d[0] * frac, d[1] * frac]);
        }
        if let Some(&d0) = dirs.first() {
            ring_pts.push([d0[0] * frac, d0[1] * frac]);
        }
        let screen: Vec<_> = ring_pts
            .iter()
            .filter(|&&p| transform.is_valid_point(p))
            .map(|&p| transform.position_from_point(p))
            .collect();
        if screen.len() >= 2 {
            painter.add(epaint::PathShape::line(screen, grid_stroke));
        }
    }

    let font = FontId::proportional(10.0);
    for (k, &d) in dirs.iter().enumerate() {
        if transform.is_valid_point([0.0, 0.0]) && transform.is_valid_point(d) {
            let p0 = transform.position_from_point([0.0, 0.0]);
            let p1 = transform.position_from_point(d);
            painter.line_segment([p0, p1], grid_stroke);
            if radar.show_labels {
                let lp = transform.position_from_point([d[0] * 1.14, d[1] * 1.14]);
                if transform.rect().expand(24.0).contains(lp) {
                    painter.text(
                        lp,
                        Align2::CENTER_CENTER,
                        &data.axes_labels()[k],
                        font.clone(),
                        text_color,
                    );
                }
            }
        }
    }

    let width = style.stroke.width.max(1.8);
    for (si, ser) in data.series().iter().enumerate() {
        diagnostics.visited_marker_points += ser.vertices.len();
        let col = pick_palette(radar.palette, si);
        if ser.vertices.len() < 3 {
            continue;
        }
        let mut center_acc = [0.0_f64; 2];
        let valid = ser.vertices.iter().all(|&p| transform.is_valid_point(p));
        if !valid {
            continue;
        }
        for &v in ser.vertices.iter() {
            center_acc[0] += v[0];
            center_acc[1] += v[1];
        }
        let n_v = ser.vertices.len() as f64;
        let centroid = transform.normalized([center_acc[0] / n_v, center_acc[1] / n_v]);
        let fill_col = col.gamma_multiply(radar.fill_alpha);
        if fill_col != Color32::TRANSPARENT {
            let mut batch = MeshBatch::default();
            for i in 0..ser.vertices.len() {
                let a = transform.normalized(ser.vertices[i]);
                let b = transform.normalized(ser.vertices[(i + 1) % ser.vertices.len()]);
                batch.add_flat_triangle(painter, [centroid, a, b], fill_col, transform);
            }
            let stats = batch.finish(painter);
            diagnostics.patches.output_meshes += stats.meshes;
            diagnostics.patches.output_triangles += stats.triangles;
        }
        let mut poly: Vec<_> = ser
            .vertices
            .iter()
            .map(|&p| transform.position_from_point(p))
            .collect();
        poly.push(poly[0]);
        for &p in &poly[..ser.vertices.len()] {
            painter.circle_filled(p, 3.2, col);
        }
        painter.add(epaint::PathShape::line(poly, Stroke::new(width, col)));
        diagnostics.output_paths += 1;
    }
    let _ = (clipped_axis_segment, Shape::Noop);
}

pub(super) fn paint_sankey_chart(
    painter: &Painter,
    sankey: SankeyChart<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = sankey.data;
    let mut batch = MeshBatch::default();
    for rib in data.ribbons() {
        let col = pick_palette(sankey.palette, rib.source).gamma_multiply(sankey.ribbon_alpha);
        let n = rib.top_curve.len().min(rib.bottom_curve.len());
        for i in 0..n.saturating_sub(1) {
            let t0 = rib.top_curve[i];
            let t1 = rib.top_curve[i + 1];
            let b0 = rib.bottom_curve[i];
            let b1 = rib.bottom_curve[i + 1];
            if [t0, t1, b0, b1]
                .iter()
                .all(|&p| transform.is_valid_point(p))
            {
                let nt0 = transform.normalized(t0);
                let nt1 = transform.normalized(t1);
                let nb0 = transform.normalized(b0);
                let nb1 = transform.normalized(b1);
                batch.add_flat_triangle(painter, [nb0, nb1, nt1], col, transform);
                batch.add_flat_triangle(painter, [nb0, nt1, nt0], col, transform);
            }
        }
    }
    let stats = batch.finish(painter);
    diagnostics.patches.output_meshes += stats.meshes;
    diagnostics.patches.output_triangles += stats.triangles;

    let font = FontId::proportional(10.0);
    for nd in data.nodes() {
        diagnostics.visited_bars += 1;
        let col = pick_palette(sankey.palette, nd.index);
        if valid_extent([nd.x0, nd.y0], [nd.x1, nd.y1], transform)
            && let Some(rect) = clipped_data_rect([nd.x0, nd.y0], [nd.x1, nd.y1], transform)
        {
            painter.rect_filled(rect, 0.0, col);
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, text_color.gamma_multiply(0.4)),
                egui::StrokeKind::Inside,
            );
            diagnostics.output_bars += 1;
            if sankey.show_labels {
                let (pos, align) = if nd.x0 > 0.7 {
                    (
                        egui::pos2(rect.left() - 4.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                    )
                } else {
                    (
                        egui::pos2(rect.right() + 4.0, rect.center().y),
                        Align2::LEFT_CENTER,
                    )
                };
                painter.text(pos, align, &nd.label, font.clone(), text_color);
            }
        }
    }
}
