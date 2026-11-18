use egui::{Align2, Color32, FontId, Painter, Pos2, Shape, Stroke, Vec2, epaint};

use crate::{
    SmithGridKind, SmithPlotData, SmithPlotState, SmithTransform, interaction::HoveredSmithPoint,
    model::ResolvedPlotStyle,
};

const R_CIRCLES: [f64; 6] = [0.0, 0.2, 0.5, 1.0, 2.0, 5.0];
const X_ARCS: [f64; 5] = [0.2, 0.5, 1.0, 2.0, 5.0];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_smith(
    painter: &Painter,
    state: &SmithPlotState,
    data: &SmithPlotData,
    transform: SmithTransform,
    grid_kind: SmithGridKind,
    show_grid: bool,
    style: &ResolvedPlotStyle<'_>,
    hovered: Option<&HoveredSmithPoint>,
    visible_series: &mut usize,
) {
    let unit_center = transform.gamma_to_pos([0.0, 0.0]);
    let unit_edge = transform.gamma_to_pos([1.0, 0.0]);
    let unit_radius = (unit_edge.x - unit_center.x).abs().max(2.0);
    painter.circle_filled(unit_center, unit_radius, style.background);

    if show_grid {
        paint_smith_grid(painter, transform, grid_kind, style);
    }
    painter.circle_stroke(unit_center, unit_radius, style.border_stroke);

    for &vswr in data.vswr_circles() {
        let rho = ((vswr - 1.0) / (vswr + 1.0)).clamp(0.0, 1.0);
        let r_px = unit_radius * rho as f32;
        paint_dashed_circle(
            painter,
            unit_center,
            r_px,
            Stroke::new(1.0, style.text_color.gamma_multiply(0.45)),
        );
    }

    for (ti, trace) in data.traces().iter().enumerate() {
        if !state.trace_visible(ti) {
            continue;
        }
        *visible_series += 1;
        let resolved = style.resolve_series(ti, Default::default());
        let pts: Vec<Pos2> = trace
            .gamma
            .iter()
            .map(|&g| transform.gamma_to_pos(g))
            .filter(|p| p.is_finite())
            .collect();
        if pts.len() >= 2 {
            painter.add(epaint::PathShape::line(pts.clone(), resolved.stroke));
        }
        if let Some(&first) = pts.first() {
            painter.circle_filled(first, 3.5, resolved.stroke.color);
        }
        if let Some(&last) = pts.last() {
            painter.circle_stroke(last, 4.0, Stroke::new(1.6, resolved.stroke.color));
        }
    }

    if let Some(hit) = hovered {
        painter.circle_stroke(hit.screen_pos, 6.5, Stroke::new(2.0, style.text_color));
    }
}

fn paint_smith_grid(
    painter: &Painter,
    transform: SmithTransform,
    kind: SmithGridKind,
    style: &ResolvedPlotStyle<'_>,
) {
    let imp_col = style.grid_stroke.color;
    let adm_col = Color32::from_rgb(76, 145, 195).gamma_multiply(0.42);
    let imp_stroke = Stroke::new(1.0, imp_col);
    let adm_stroke = Stroke::new(1.0, adm_col);

    if matches!(
        kind,
        SmithGridKind::Impedance | SmithGridKind::ImpedanceAdmittance
    ) {
        let p_l = transform.gamma_to_pos([-1.0, 0.0]);
        let p_r = transform.gamma_to_pos([1.0, 0.0]);
        painter.line_segment([p_l, p_r], imp_stroke);
        for &r in &R_CIRCLES {
            let pts: Vec<Pos2> = SmithTransform::constant_resistance_circle(r, 72)
                .into_iter()
                .map(|g| transform.gamma_to_pos(g))
                .collect();
            painter.add(Shape::closed_line(pts, imp_stroke));
            if r > 0.0 && r <= 2.0 {
                let lp = transform.gamma_to_pos([(r - 1.0) / (r + 1.0), 0.0]);
                painter.text(
                    lp + egui::vec2(2.0, -2.0),
                    Align2::LEFT_BOTTOM,
                    format!("{r}"),
                    FontId::proportional(9.0),
                    style.text_color.gamma_multiply(0.55),
                );
            }
        }
        for &x in &X_ARCS {
            for sign in [1.0_f64, -1.0_f64] {
                let pts: Vec<Pos2> = SmithTransform::constant_reactance_arc(sign * x, 48)
                    .into_iter()
                    .map(|g| transform.gamma_to_pos(g))
                    .collect();
                painter.add(epaint::PathShape::line(pts, imp_stroke));
            }
        }
    }

    if matches!(
        kind,
        SmithGridKind::Admittance | SmithGridKind::ImpedanceAdmittance
    ) {
        let p_l = transform.gamma_to_pos([-1.0, 0.0]);
        let p_r = transform.gamma_to_pos([1.0, 0.0]);
        painter.line_segment([p_l, p_r], adm_stroke);
        for &g_val in &R_CIRCLES[1..] {
            let pts: Vec<Pos2> = SmithTransform::constant_resistance_circle(g_val, 72)
                .into_iter()
                .map(|g| transform.gamma_to_pos([-g[0], -g[1]]))
                .collect();
            painter.add(Shape::closed_line(pts, adm_stroke));
        }
        for &b in &X_ARCS {
            for sign in [1.0_f64, -1.0_f64] {
                let pts: Vec<Pos2> = SmithTransform::constant_reactance_arc(sign * b, 48)
                    .into_iter()
                    .map(|g| transform.gamma_to_pos([-g[0], -g[1]]))
                    .collect();
                painter.add(epaint::PathShape::line(pts, adm_stroke));
            }
        }
    }
}

fn paint_dashed_circle(painter: &Painter, center: Pos2, radius: f32, stroke: Stroke) {
    let steps = 48usize;
    for k in (0..steps).step_by(2) {
        let a0 = std::f32::consts::TAU * (k as f32 / steps as f32);
        let a1 = std::f32::consts::TAU * ((k + 1) as f32 / steps as f32);
        let p0 = center + Vec2::new(a0.cos(), a0.sin()) * radius;
        let p1 = center + Vec2::new(a1.cos(), a1.sin()) * radius;
        painter.line_segment([p0, p1], stroke);
    }
}
