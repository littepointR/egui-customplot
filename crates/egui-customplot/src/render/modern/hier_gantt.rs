use egui::{Align2, Color32, FontId, Painter, Pos2, Shape, Stroke, epaint};

use super::radial_flow::pick_palette;
use crate::{
    GanttChart, PlotDiagnostics, PlotTransform, SunburstChart, TreemapChart,
    axis::{clipped_data_rect, valid_extent},
    render::mesh::MeshBatch,
};

pub(super) fn paint_treemap_chart(
    painter: &Painter,
    tm: TreemapChart<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let font = FontId::proportional(10.0);
    for r in tm.data.rects() {
        diagnostics.visited_bars += 1;
        let lo = [r.x0, r.y0];
        let hi = [r.x1, r.y1];
        if !valid_extent(lo, hi, transform) {
            continue;
        }
        let Some(rect) = clipped_data_rect(lo, hi, transform) else {
            continue;
        };
        let fill = pick_palette(tm.palette, r.group);
        painter.rect_filled(rect, 0.0, fill);
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.4, tm.border_color),
            egui::StrokeKind::Inside,
        );
        diagnostics.output_bars += 1;
        if tm.show_labels && rect.width() > 32.0 && rect.height() > 16.0 {
            let fg = if fill.r() as u16 + fill.g() as u16 + fill.b() as u16 > 420 {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                &r.label,
                font.clone(),
                fg,
            );
        }
    }
    let _ = text_color;
}

pub(super) fn paint_sunburst_chart(
    painter: &Painter,
    sb: SunburstChart<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let mut batch = MeshBatch::default();
    let mut outlines: Vec<Vec<Pos2>> = Vec::new();
    let font = FontId::proportional(9.5);
    let mut labels: Vec<(Pos2, String, Color32)> = Vec::new();

    for sec in sb.data.sectors() {
        let col = pick_palette(sb.palette, sec.group)
            .gamma_multiply((1.0 - 0.14 * sec.depth as f32).clamp(0.45, 1.0));
        let span = (sec.end_rad - sec.start_rad).max(1e-4);
        let steps = ((span / std::f64::consts::TAU) * 48.0)
            .ceil()
            .clamp(3.0, 48.0) as usize;
        let mut ring = Vec::with_capacity((steps + 1) * 2);
        for k in 0..steps {
            let a0 = sec.start_rad + span * (k as f64 / steps as f64);
            let a1 = sec.start_rad + span * ((k + 1) as f64 / steps as f64);
            let (s0, c0) = a0.sin_cos();
            let (s1, c1) = a1.sin_cos();
            let p_in0 = [sec.r_in * c0, sec.r_in * s0];
            let p_out0 = [sec.r_out * c0, sec.r_out * s0];
            let p_out1 = [sec.r_out * c1, sec.r_out * s1];
            let p_in1 = [sec.r_in * c1, sec.r_in * s1];
            if [p_in0, p_out0, p_out1, p_in1]
                .iter()
                .all(|&p| transform.is_valid_point(p))
            {
                let ni0 = transform.normalized(p_in0);
                let no0 = transform.normalized(p_out0);
                let no1 = transform.normalized(p_out1);
                let ni1 = transform.normalized(p_in1);
                batch.add_flat_triangle(painter, [ni0, no0, no1], col, transform);
                batch.add_flat_triangle(painter, [ni0, no1, ni1], col, transform);
            }
        }
        for k in 0..=steps {
            let a = sec.start_rad + span * (k as f64 / steps as f64);
            let (s, c) = a.sin_cos();
            let pt = [sec.r_out * c, sec.r_out * s];
            if transform.is_valid_point(pt) {
                ring.push(transform.position_from_point(pt));
            }
        }
        for k in (0..=steps).rev() {
            let a = sec.start_rad + span * (k as f64 / steps as f64);
            let (s, c) = a.sin_cos();
            let pt = [sec.r_in * c, sec.r_in * s];
            if transform.is_valid_point(pt) {
                ring.push(transform.position_from_point(pt));
            }
        }
        if ring.len() >= 3 {
            ring.push(ring[0]);
            outlines.push(ring);
        }
        if sb.show_labels && span > 0.35 {
            let mid_a = 0.5 * (sec.start_rad + sec.end_rad);
            let mid_r = 0.5 * (sec.r_in + sec.r_out);
            let pt = [mid_r * mid_a.cos(), mid_r * mid_a.sin()];
            if transform.is_valid_point(pt) {
                let pos = transform.position_from_point(pt);
                if transform.rect().contains(pos) {
                    labels.push((pos, sec.label.clone(), text_color));
                }
            }
        }
    }
    let stats = batch.finish(painter);
    diagnostics.patches.output_meshes += stats.meshes;
    diagnostics.patches.output_triangles += stats.triangles;
    let border = Stroke::new(1.2, sb.border_color);
    for o in outlines {
        painter.add(epaint::PathShape::line(o, border));
    }
    for (pos, txt, col) in labels {
        painter.text(pos, Align2::CENTER_CENTER, txt, font.clone(), col);
    }
}

pub(super) fn paint_gantt_chart(
    painter: &Painter,
    gantt: GanttChart<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let tasks = gantt.data.tasks();
    let half_h = 0.32_f64;
    let dep_col = text_color.gamma_multiply(0.55);
    let dep_stroke = Stroke::new(1.2, dep_col);

    if gantt.show_dependencies {
        for &[a, b] in gantt.data.dependencies() {
            let ta = &tasks[a];
            let tb = &tasks[b];
            let ya = a as f64 + 1.0;
            let yb = b as f64 + 1.0;
            if [ta.end, ya, tb.start, yb].iter().all(|v| v.is_finite())
                && transform.is_valid_point([ta.end, ya])
                && transform.is_valid_point([tb.start, yb])
            {
                let p0 = transform.position_from_point([ta.end, ya]);
                let p3 = transform.position_from_point([tb.start, yb]);
                let mid_x = p0.x + 8.0;
                let pts = vec![p0, Pos2::new(mid_x, p0.y), Pos2::new(mid_x, p3.y), p3];
                painter.add(epaint::PathShape::line(pts, dep_stroke));
            }
        }
    }

    for (idx, task) in tasks.iter().enumerate() {
        diagnostics.visited_bars += 1;
        let y = idx as f64 + 1.0;
        let col = pick_palette(gantt.palette, task.group);
        if task.milestone {
            let pt = [task.end, y];
            if transform.is_valid_point(pt) {
                let c = transform.position_from_point(pt);
                let r = 6.5;
                let pts = vec![
                    c + egui::vec2(0.0, -r),
                    c + egui::vec2(r, 0.0),
                    c + egui::vec2(0.0, r),
                    c + egui::vec2(-r, 0.0),
                ];
                painter.add(Shape::convex_polygon(
                    pts,
                    col,
                    Stroke::new(1.0, Color32::WHITE),
                ));
                diagnostics.output_bars += 1;
            }
            continue;
        }
        let lo = [task.start, y - half_h];
        let hi = [task.end, y + half_h];
        if !valid_extent(lo, hi, transform) {
            continue;
        }
        if let Some(rect) = clipped_data_rect(lo, hi, transform) {
            painter.rect_filled(rect, 3.0, col.gamma_multiply(0.45));
            if gantt.show_progress && task.progress > 0.0 {
                let p_hi = [
                    task.start + (task.end - task.start) * task.progress,
                    y + half_h,
                ];
                if let Some(p_rect) = clipped_data_rect(lo, p_hi, transform) {
                    painter.rect_filled(p_rect, 3.0, col);
                }
            }
            painter.rect_stroke(rect, 3.0, Stroke::new(1.0, col), egui::StrokeKind::Inside);
            diagnostics.output_bars += 1;
        }
    }
}
