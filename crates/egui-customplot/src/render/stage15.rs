use egui::{Align2, Color32, FontId, Painter, Pos2, Shape, Stroke, Vec2, epaint};

use super::{
    heatmap::paint_heatmap, line::prepare_line, markers::prepare_markers, pattern::apply_pattern,
};
use crate::{
    CwtPlot, Heatmap, LineInterpolation, LinePattern, MarkerShape, MarkerStyle, NicholsPlot,
    NyquistPlot, PlotDiagnostics, PlotPoint, PlotTransform, RootLocus, ZPlanePlot,
    axis::{clip_normalized_segment, clipped_axis_segment},
    model::ResolvedSeriesStyle,
};

const LOCUS_PALETTE: [Color32; 6] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(237, 177, 32),
    Color32::from_rgb(126, 47, 142),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(77, 190, 238),
];

pub(super) fn paint_zplane(
    painter: &Painter,
    zp: ZPlanePlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let b = transform.bounds();
    let axis_stroke = Stroke::new(1.0, text_color.gamma_multiply(0.32));
    if let Some(pair) = clipped_axis_segment([b.min()[0], 0.0], [b.max()[0], 0.0], 0, transform) {
        painter.line_segment(pair, axis_stroke);
    }
    if let Some(pair) = clipped_axis_segment([0.0, b.min()[1]], [0.0, b.max()[1]], 1, transform) {
        painter.line_segment(pair, axis_stroke);
    }

    if zp.show_unit_circle {
        paint_unit_circle(painter, transform, text_color.gamma_multiply(0.55), true);
    }

    let radius = style.marker.radius.max(5.5);
    let zero_pts: Vec<PlotPoint> = zp.data.zeros().iter().map(|z| [z.re, z.im]).collect();
    let pole_pts: Vec<PlotPoint> = zp.data.poles().iter().map(|p| [p.re, p.im]).collect();

    let zm = prepare_markers(
        &zero_pts,
        transform,
        MarkerStyle {
            shape: MarkerShape::CircleHollow,
            radius,
        },
        zp.zero_color,
        None,
        None,
        pixels_per_point,
    );
    diagnostics.output_markers += zm.output_markers;
    if !zm.mesh.is_empty() {
        painter.add(Shape::mesh(zm.mesh));
    }

    let pm = prepare_markers(
        &pole_pts,
        transform,
        MarkerStyle {
            shape: MarkerShape::Cross,
            radius,
        },
        zp.pole_color,
        None,
        None,
        pixels_per_point,
    );
    diagnostics.output_markers += pm.output_markers;
    if !pm.mesh.is_empty() {
        painter.add(Shape::mesh(pm.mesh));
    }

    let font = FontId::proportional(9.5);
    for (roots, color) in [
        (zp.data.zeros(), zp.zero_color),
        (zp.data.poles(), zp.pole_color),
    ] {
        for r in roots {
            if r.multiplicity > 1 && transform.is_valid_point([r.re, r.im]) {
                let p = transform.position_from_point([r.re, r.im]);
                if transform.rect().contains(p) {
                    painter.text(
                        p + Vec2::new(radius + 2.0, -(radius + 1.0)),
                        Align2::LEFT_BOTTOM,
                        format!("{}", r.multiplicity),
                        font.clone(),
                        color,
                    );
                }
            }
        }
    }
}

pub(super) fn paint_nyquist(
    painter: &Painter,
    nyq: NyquistPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if nyq.show_unit_circle {
        paint_unit_circle(painter, transform, text_color.gamma_multiply(0.32), true);
    }
    let crit = prepare_markers(
        &[[-1.0, 0.0]],
        transform,
        MarkerStyle {
            shape: MarkerShape::Plus,
            radius: 5.5,
        },
        nyq.critical_color,
        None,
        None,
        pixels_per_point,
    );
    if !crit.mesh.is_empty() {
        painter.add(Shape::mesh(crit.mesh));
    }

    let pts = nyq.data.nyquist_curve().points();
    let prep = prepare_line(pts, transform, LineInterpolation::Linear);
    diagnostics.visited_segments += prep.visited_segments;
    for path in &prep.paths {
        painter.add(epaint::PathShape::line(path.clone(), style.stroke));
    }
    if nyq.show_arrows {
        paint_path_chevrons(painter, &prep.paths, style.stroke.color);
    }

    if nyq.show_mirror {
        let mirror: Vec<PlotPoint> = pts.iter().map(|&[re, im]| [re, -im]).collect();
        let m_prep = prepare_line(&mirror, transform, LineInterpolation::Linear);
        let dashed = apply_pattern(
            m_prep.paths,
            LinePattern::Dashed {
                length: 5.0,
                gap: 4.0,
            },
        );
        let m_stroke = Stroke::new(
            (style.stroke.width * 0.85).max(1.0),
            style.stroke.color.gamma_multiply(0.55),
        );
        for path in dashed.paths {
            painter.add(epaint::PathShape::line(path, m_stroke));
        }
    }
}

pub(super) fn paint_nichols(
    painter: &Painter,
    nichols: NicholsPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if nichols.show_grid {
        let grid_stroke = Stroke::new(1.0, text_color.gamma_multiply(0.22));
        for &m_db in &[-6.0_f64, -3.0, 0.0, 3.0, 6.0, 12.0] {
            let m = 10.0_f64.powf(m_db / 20.0);
            let contour: Vec<PlotPoint> = if (m - 1.0).abs() < 1e-4 {
                (-12..=12)
                    .map(|k| {
                        let v = (k as f64) * 0.45;
                        let ph = v.atan2(-0.5).to_degrees();
                        let ph = if ph > 0.0 { ph - 360.0 } else { ph };
                        [ph, 20.0 * (-0.5_f64).hypot(v).log10()]
                    })
                    .collect()
            } else {
                let m2 = m * m;
                let cx = -m2 / (m2 - 1.0);
                let rad = m / (m2 - 1.0).abs();
                let mut prev_deg = -180.0_f64;
                (0..=64)
                    .map(|k| {
                        let th = std::f64::consts::TAU * (k as f64 / 64.0);
                        let u = cx + rad * th.cos();
                        let v = rad * th.sin();
                        let mut deg = v.atan2(u).to_degrees();
                        if deg > 0.0 {
                            deg -= 360.0;
                        }
                        while deg - prev_deg > 180.0 {
                            deg -= 360.0;
                        }
                        while deg - prev_deg < -180.0 {
                            deg += 360.0;
                        }
                        prev_deg = deg;
                        [deg, 20.0 * u.hypot(v).max(1e-12).log10()]
                    })
                    .collect()
            };
            let prep = prepare_line(&contour, transform, LineInterpolation::Linear);
            for path in prep.paths {
                painter.add(epaint::PathShape::line(path, grid_stroke));
            }
        }
    }

    let crit = prepare_markers(
        &[[-180.0, 0.0]],
        transform,
        MarkerStyle {
            shape: MarkerShape::Plus,
            radius: 5.5,
        },
        Color32::from_rgb(214, 39, 40),
        None,
        None,
        pixels_per_point,
    );
    if !crit.mesh.is_empty() {
        painter.add(Shape::mesh(crit.mesh));
    }

    let prep = prepare_line(
        nichols.data.nichols_curve().points(),
        transform,
        LineInterpolation::Linear,
    );
    diagnostics.visited_segments += prep.visited_segments;
    for path in &prep.paths {
        painter.add(epaint::PathShape::line(path.clone(), style.stroke));
    }
    if nichols.show_arrows {
        paint_path_chevrons(painter, &prep.paths, style.stroke.color);
    }
}

pub(super) fn paint_root_locus(
    painter: &Painter,
    rl: RootLocus<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if rl.show_imaginary_axis {
        let b = transform.bounds();
        let axis_col = text_color.gamma_multiply(0.32);
        if let Some(pair) = clipped_axis_segment([0.0, b.min()[1]], [0.0, b.max()[1]], 1, transform)
        {
            painter.line_segment(pair, Stroke::new(1.0, axis_col));
        }
        if let Some(pair) = clipped_axis_segment([b.min()[0], 0.0], [b.max()[0], 0.0], 0, transform)
        {
            painter.line_segment(pair, Stroke::new(1.0, axis_col));
        }
    }

    let w = style.stroke.width.max(1.6);
    for (idx, branch) in rl.data.branches().iter().enumerate() {
        let col = LOCUS_PALETTE[idx % LOCUS_PALETTE.len()];
        let prep = prepare_line(branch.points(), transform, LineInterpolation::Linear);
        diagnostics.visited_segments += prep.visited_segments;
        for path in prep.paths {
            painter.add(epaint::PathShape::line(path, Stroke::new(w, col)));
        }
    }

    let pm = prepare_markers(
        rl.data.open_loop_poles(),
        transform,
        MarkerStyle {
            shape: MarkerShape::Cross,
            radius: 5.5,
        },
        Color32::from_rgb(214, 39, 40),
        None,
        None,
        pixels_per_point,
    );
    if !pm.mesh.is_empty() {
        painter.add(Shape::mesh(pm.mesh));
    }
    let zm = prepare_markers(
        rl.data.open_loop_zeros(),
        transform,
        MarkerStyle {
            shape: MarkerShape::CircleHollow,
            radius: 5.5,
        },
        Color32::from_rgb(0, 114, 189),
        None,
        None,
        pixels_per_point,
    );
    if !zm.mesh.is_empty() {
        painter.add(Shape::mesh(zm.mesh));
    }
}

pub(super) fn paint_cwt(
    painter: &Painter,
    cwt: CwtPlot<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    paint_heatmap(
        painter,
        Heatmap::new(cwt.data.grid(), cwt.colors),
        transform,
        diagnostics,
    );
    if !cwt.show_coi {
        return;
    }
    let stroke = Stroke::new(1.6, cwt.coi_color);
    let pat = LinePattern::Dashed {
        length: 5.0,
        gap: 4.0,
    };
    for curve in [cwt.data.coi_left(), cwt.data.coi_right()] {
        let prep = prepare_line(curve.points(), transform, LineInterpolation::Linear);
        for path in apply_pattern(prep.paths, pat).paths {
            painter.add(epaint::PathShape::line(path, stroke));
        }
    }
}

fn paint_unit_circle(painter: &Painter, transform: PlotTransform, color: Color32, dashed: bool) {
    let steps = 72usize;
    let pts: Vec<PlotPoint> = (0..=steps)
        .map(|k| {
            let th = std::f64::consts::TAU * (k as f64 / steps as f64);
            [th.cos(), th.sin()]
        })
        .collect();
    let prep = prepare_line(&pts, transform, LineInterpolation::Linear);
    let paths = if dashed {
        apply_pattern(
            prep.paths,
            LinePattern::Dashed {
                length: 5.0,
                gap: 4.0,
            },
        )
        .paths
    } else {
        prep.paths
    };
    let stroke = Stroke::new(1.1, color);
    for p in paths {
        painter.add(epaint::PathShape::line(p, stroke));
    }
    let _ = clip_normalized_segment;
}

fn paint_path_chevrons(painter: &Painter, paths: &[Vec<Pos2>], color: Color32) {
    let stroke = Stroke::new(1.5, color);
    for path in paths {
        if path.len() < 4 {
            continue;
        }
        let mid = path.len() / 2;
        let a = path[mid.saturating_sub(1)];
        let b = path[mid];
        let dir = (b - a).normalized();
        if dir.length_sq() < 0.25 {
            continue;
        }
        let perp = Vec2::new(-dir.y, dir.x);
        painter.line_segment([b, b - dir * 7.0 + perp * 4.0], stroke);
        painter.line_segment([b, b - dir * 7.0 - perp * 4.0], stroke);
    }
}
