use egui::{Align2, Color32, FontId, Painter, Shape, Stroke, epaint};

use super::{
    line::prepare_line, markers::prepare_markers, mesh::MeshBatch, pattern::apply_pattern,
    quiver::paint_quiver_data,
};
use crate::{
    BarOrientation, Biplot, DendrogramPlot, EcdfPlot, LineInterpolation, LinePattern, MarkerShape,
    MarkerStyle, PlotDiagnostics, PlotPoint, PlotTransform, ProbabilityPlot,
    axis::{clip_normalized_segment, clipped_axis_segment},
    model::ResolvedSeriesStyle,
};

const CLUSTER_PALETTE: [Color32; 6] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(237, 177, 32),
    Color32::from_rgb(126, 47, 142),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(77, 190, 238),
];

pub(super) fn paint_probability_plot(
    painter: &Painter,
    prob: ProbabilityPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    if prob.show_ref_line {
        let [a, b] = prob.data.ref_line();
        if transform.is_valid_point(a) && transform.is_valid_point(b) {
            let na = transform.normalized(a);
            let nb = transform.normalized(b);
            if let Some((ca, cb)) = clip_normalized_segment(na, nb) {
                let pa = transform.position_from_normalized(ca);
                let pb = transform.position_from_normalized(cb);
                let ref_color = prob
                    .ref_color
                    .unwrap_or_else(|| Color32::from_rgb(214, 39, 40));
                let dashed = apply_pattern(
                    vec![vec![pa, pb]],
                    LinePattern::Dashed {
                        length: 7.0,
                        gap: 4.0,
                    },
                );
                let stroke = Stroke::new(style.stroke.width.max(1.4), ref_color);
                for seg in dashed.paths {
                    painter.add(epaint::PathShape::line(seg, stroke));
                }
            }
        }
    }

    let markers = prepare_markers(
        prob.data.points(),
        transform,
        style.marker,
        style.stroke.color,
        None,
        None,
        pixels_per_point,
    );
    diagnostics.visited_marker_points += markers.visited_points;
    diagnostics.numeric_rejections += markers.numeric_rejections;
    if !markers.mesh.is_empty() {
        diagnostics.output_markers += markers.output_markers;
        painter.add(Shape::mesh(markers.mesh));
    }
}

pub(super) fn paint_ecdf_plot(
    painter: &Painter,
    ecdf: EcdfPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = ecdf.data;
    if ecdf.show_ci && ecdf.ci_alpha > 0.0 && data.ci_lower().len() >= 2 {
        let fill = style.stroke.color.gamma_multiply(ecdf.ci_alpha);
        let lo = data.ci_lower();
        let hi = data.ci_upper();
        let n = lo.len().min(hi.len());
        let mut batch = MeshBatch::default();
        for i in 0..n.saturating_sub(1) {
            let pts = [lo[i], lo[i + 1], hi[i + 1], hi[i]];
            if pts.iter().any(|&p| !transform.is_valid_point(p)) {
                continue;
            }
            let npts = pts.map(|p| transform.normalized(p));
            batch.add_flat_triangle(painter, [npts[0], npts[1], npts[2]], fill, transform);
            batch.add_flat_triangle(painter, [npts[0], npts[2], npts[3]], fill, transform);
        }
        let stats = batch.finish(painter);
        diagnostics.patches.output_meshes += stats.meshes;
        diagnostics.patches.output_triangles += stats.triangles;
    }

    let prepared = prepare_line(data.step_points(), transform, LineInterpolation::Linear);
    diagnostics.visited_segments += prepared.visited_segments;
    diagnostics.numeric_rejections += prepared.numeric_rejections;
    let patterned = apply_pattern(prepared.paths, style.pattern);
    diagnostics.pattern_truncated |= patterned.truncated;
    for path in patterned.paths {
        painter.add(epaint::PathShape::line(path, style.stroke));
    }

    if ecdf.show_censored && !data.censored_marks().is_empty() {
        let cens_style = MarkerStyle {
            shape: MarkerShape::Plus,
            radius: (style.marker.radius + 1.2).max(4.0),
        };
        let markers = prepare_markers(
            data.censored_marks(),
            transform,
            cens_style,
            style.stroke.color,
            None,
            None,
            pixels_per_point,
        );
        if !markers.mesh.is_empty() {
            diagnostics.output_markers += markers.output_markers;
            painter.add(Shape::mesh(markers.mesh));
        }
    }
}

pub(super) fn paint_dendrogram_plot(
    painter: &Painter,
    dendro: DendrogramPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) {
    let base_color = style.stroke.color;
    let width = style.stroke.width.max(1.3);
    let orient = dendro.orientation;
    let map = |leaf: f64, dist: f64| -> PlotPoint {
        match orient {
            BarOrientation::Vertical => [leaf, dist],
            BarOrientation::Horizontal => [dist, leaf],
        }
    };
    for branch in dendro.data.branches() {
        let color = if let Some(thresh) = dendro.color_threshold {
            if branch.y_top <= thresh {
                CLUSTER_PALETTE[branch.index % CLUSTER_PALETTE.len()]
            } else {
                base_color
            }
        } else {
            base_color
        };
        let stroke = Stroke::new(width, color);
        let segs = [
            (
                map(branch.x_left, branch.y_left),
                map(branch.x_left, branch.y_top),
                usize::from(orient == BarOrientation::Vertical),
            ),
            (
                map(branch.x_left, branch.y_top),
                map(branch.x_right, branch.y_top),
                usize::from(orient == BarOrientation::Horizontal),
            ),
            (
                map(branch.x_right, branch.y_top),
                map(branch.x_right, branch.y_right),
                usize::from(orient == BarOrientation::Vertical),
            ),
        ];
        for (a, b, axis) in segs {
            if let Some(pair) = clipped_axis_segment(a, b, axis, transform) {
                painter.line_segment(pair, stroke);
                diagnostics.network.output_edges += 1;
            }
        }
    }
}

pub(super) fn paint_biplot(
    painter: &Painter,
    biplot: Biplot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = biplot.data;
    // Origin crosshairs
    let zero_stroke = Stroke::new(1.0, text_color.gamma_multiply(0.25));
    let bounds = transform.bounds();
    if let Some(pair) =
        clipped_axis_segment([bounds.min()[0], 0.0], [bounds.max()[0], 0.0], 0, transform)
    {
        painter.line_segment(pair, zero_stroke);
    }
    if let Some(pair) =
        clipped_axis_segment([0.0, bounds.min()[1]], [0.0, bounds.max()[1]], 1, transform)
    {
        painter.line_segment(pair, zero_stroke);
    }

    if biplot.show_scores && !data.scores().is_empty() {
        let score_color = biplot
            .score_color
            .unwrap_or_else(|| Color32::from_rgb(217, 83, 25));
        let score_marker = MarkerStyle {
            shape: MarkerShape::Circle,
            radius: style.marker.radius.max(3.2),
        };
        let markers = prepare_markers(
            data.scores().points(),
            transform,
            score_marker,
            score_color,
            None,
            None,
            pixels_per_point,
        );
        diagnostics.visited_marker_points += markers.visited_points;
        if !markers.mesh.is_empty() {
            diagnostics.output_markers += markers.output_markers;
            painter.add(Shape::mesh(markers.mesh));
        }
        if biplot.show_obs_labels && data.n_obs() <= 48 {
            let font = FontId::proportional(9.5);
            for (pt, label) in data.scores().points().iter().zip(data.obs_labels().iter()) {
                if transform.is_valid_point(*pt) {
                    let pos = transform.position_from_point(*pt);
                    if transform.rect().contains(pos) {
                        painter.text(
                            pos + egui::vec2(4.0, -4.0),
                            Align2::LEFT_BOTTOM,
                            label,
                            font.clone(),
                            score_color,
                        );
                    }
                }
            }
        }
    }

    let mut vec_style = style;
    vec_style.stroke.color = biplot.loading_color.unwrap_or(style.stroke.color);
    let q_stats = paint_quiver_data(
        painter,
        data.loadings_quiver(),
        true,
        biplot.head_size,
        transform,
        vec_style,
    );
    diagnostics.visited_quiver_arrows += q_stats.visited;
    diagnostics.output_quiver_shafts += q_stats.output_shafts;
    diagnostics.output_quiver_heads += q_stats.output_heads;

    if biplot.show_var_labels && data.n_vars() <= 48 {
        let font = FontId::proportional(10.5);
        for (arrow, label) in data
            .loadings_quiver()
            .arrows()
            .iter()
            .zip(data.var_labels().iter())
        {
            if let Some(end) = arrow.endpoint()
                && transform.is_valid_point(end)
            {
                let o = transform.position_from_point(arrow.origin);
                let p = transform.position_from_point(end);
                if transform.rect().contains(p) {
                    let dir = (p - o).normalized();
                    let offset = if dir.length_sq() > 0.1 {
                        dir * 8.0
                    } else {
                        egui::vec2(6.0, -6.0)
                    };
                    painter.text(
                        p + offset,
                        Align2::CENTER_CENTER,
                        label,
                        font.clone(),
                        vec_style.stroke.color,
                    );
                }
            }
        }
    }
}
