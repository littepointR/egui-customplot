use egui::{Align2, Color32, FontId, Painter, Pos2, Shape, Stroke};

use super::markers::prepare_markers;
use crate::{
    CategoricalHeatmap, Heatmap, MarkerShape, MarkerStyle, ParallelPlot, PlotDiagnostics,
    PlotPoint, PlotTransform,
    axis::{clip_normalized_segment, clipped_axis_segment, grid_cell_rect},
    model::ResolvedSeriesStyle,
};

const PARALLEL_PALETTE: [Color32; 8] = [
    Color32::from_rgb(0, 114, 189),
    Color32::from_rgb(217, 83, 25),
    Color32::from_rgb(237, 177, 32),
    Color32::from_rgb(126, 47, 142),
    Color32::from_rgb(119, 172, 48),
    Color32::from_rgb(77, 190, 238),
    Color32::from_rgb(162, 20, 47),
    Color32::from_rgb(100, 115, 140),
];

pub(super) fn paint_categorical_heatmap(
    painter: &Painter,
    cat: CategoricalHeatmap<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let heatmap = Heatmap::new(cat.data.grid(), cat.colors);
    super::heatmap::paint_heatmap(painter, heatmap, transform, diagnostics);

    if !cat.show_cell_labels {
        return;
    }
    let grid = cat.data.grid();
    let nx = cat.data.x_categories().len();
    let ny = cat.data.y_categories().len();
    if nx * ny > 144 {
        return;
    }
    let font = FontId::proportional(10.5);
    for row in 0..ny {
        for col in 0..nx {
            let Some(val) = grid.value(col, row) else {
                continue;
            };
            if !val.is_finite() {
                continue;
            }
            if let Some(bounds) = grid.cell_bounds(col, row)
                && let Some(rect) = grid_cell_rect(bounds, transform)
            {
                let txt = if (val - val.round()).abs() < 1e-6 {
                    format!("{val:.0}")
                } else {
                    format!("{val:.2}")
                };
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    txt,
                    font.clone(),
                    text_color,
                );
            }
        }
    }
}

pub(super) fn paint_parallel_plot(
    painter: &Painter,
    par: ParallelPlot<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    let n_vars = par.data.n_vars();
    let n_rows = par.data.n_rows();
    if n_vars == 0 || n_rows == 0 {
        return;
    }
    if par.show_vertical_axes {
        let guide = Stroke::new(1.0, Color32::from_gray(135).gamma_multiply(0.65));
        let [y0, y1] = [transform.bounds().min()[1], transform.bounds().max()[1]];
        for col in 0..n_vars {
            if let Some(seg) =
                clipped_axis_segment([col as f64, y0], [col as f64, y1], 1, transform)
            {
                painter.line_segment(seg, guide);
            }
        }
    }

    let stroke_w = resolved.stroke.width.clamp(0.75, 4.0);
    let mut marker_pts: Vec<PlotPoint> = Vec::new();
    let mut marker_cols: Vec<Color32> = Vec::new();

    for row in 0..n_rows {
        let group = par.data.row_group(row);
        let base_col = par
            .palette
            .map_or(PARALLEL_PALETTE[group % PARALLEL_PALETTE.len()], |p| {
                p[group % p.len()]
            });
        let color = if par.style.color.is_some() && par.data.group_names().is_empty() {
            resolved.stroke.color.gamma_multiply(par.alpha)
        } else {
            base_col.gamma_multiply(par.alpha)
        };
        let stroke = Stroke::new(stroke_w, color);
        let values = par.data.normalized_row(row);

        let mut run: Vec<Pos2> = Vec::with_capacity(n_vars);
        for col in 0..n_vars {
            let p = [col as f64, values[col]];
            if !transform.is_valid_point(p) {
                if run.len() >= 2 {
                    painter.add(Shape::line(std::mem::take(&mut run), stroke));
                } else {
                    run.clear();
                }
                continue;
            }
            if par.show_markers {
                marker_pts.push(p);
                marker_cols.push(color);
            }
            if col > 0 {
                let prev = [(col - 1) as f64, values[col - 1]];
                if transform.is_valid_point(prev) {
                    let a = transform.normalized(prev);
                    let b = transform.normalized(p);
                    if let Some((sa, sb)) = clip_normalized_segment(a, b) {
                        let pa = transform.position_from_normalized(sa);
                        let pb = transform.position_from_normalized(sb);
                        if run.is_empty() {
                            run.push(pa);
                        }
                        run.push(pb);
                        diagnostics.stats.output_parallel_segments += 1;
                        continue;
                    }
                }
                if run.len() >= 2 {
                    painter.add(Shape::line(std::mem::take(&mut run), stroke));
                } else {
                    run.clear();
                }
            } else {
                let pa = transform.position_from_point(p);
                if pa.is_finite() {
                    run.push(pa);
                }
            }
        }
        if run.len() >= 2 {
            painter.add(Shape::line(run, stroke));
        }
    }

    if par.show_markers && !marker_pts.is_empty() {
        let prepared = prepare_markers(
            &marker_pts,
            transform,
            MarkerStyle {
                shape: MarkerShape::Circle,
                radius: 2.8,
            },
            resolved.stroke.color,
            Some(&marker_cols),
            None,
            pixels_per_point,
        );
        if !prepared.mesh.is_empty() {
            painter.add(Shape::mesh(prepared.mesh));
        }
    }
}
