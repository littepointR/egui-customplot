use egui::{Align2, Color32, FontId, Painter, Rect, Stroke, StrokeKind};

use crate::{
    ConfusionChart, ConfusionNormalization, PlotDiagnostics, PlotTransform, axis::clipped_data_rect,
};

fn blend_srgb(base: Color32, target: Color32, t: f64) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix =
        |a: u8, b: u8| -> u8 { (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round() as u8 };
    Color32::from_rgb(
        mix(base.r(), target.r()),
        mix(base.g(), target.g()),
        mix(base.b(), target.b()),
    )
}

fn contrast_text(bg: Color32) -> Color32 {
    let lum = 0.299 * f32::from(bg.r()) + 0.587 * f32::from(bg.g()) + 0.114 * f32::from(bg.b());
    if lum > 152.0 {
        Color32::from_rgb(20, 24, 30)
    } else {
        Color32::from_rgb(248, 250, 252)
    }
}

fn paint_box(
    painter: &Painter,
    rect: Rect,
    fill: Color32,
    border: Stroke,
    label: Option<&str>,
    diagnostics: &mut PlotDiagnostics,
) {
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(rect, 0.0, border, StrokeKind::Inside);
    diagnostics.output_heatmap_cells += 1;
    if let Some(txt) = label
        && rect.width() >= 22.0
        && rect.height() >= 14.0
    {
        let size = (rect.height() * 0.34).clamp(9.0, 13.0);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            txt,
            FontId::proportional(size),
            contrast_text(fill),
        );
    }
}

pub(super) fn paint_confusion_chart(
    painter: &Painter,
    chart: ConfusionChart<'_>,
    transform: PlotTransform,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = chart.data;
    let n = data.n_classes();
    let base = Color32::from_rgb(245, 247, 250);
    let border = Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 128, 140, 110));
    let diag_denom = data.max_diag().max(1) as f64;
    let off_denom = data.max_off_diag().max(1) as f64;

    for row in 0..n {
        for col in 0..n {
            let lo = [col as f64 + 0.5, row as f64 + 0.5];
            let hi = [col as f64 + 1.5, row as f64 + 1.5];
            let Some(rect) = clipped_data_rect(lo, hi, transform) else {
                continue;
            };
            let count = data.count(row, col);
            let frac = if row == col {
                count as f64 / diag_denom
            } else {
                count as f64 / off_denom
            };
            let target = if row == col {
                chart.diagonal_color
            } else {
                chart.off_diagonal_color
            };
            let fill = blend_srgb(base, target, 0.12 + 0.88 * frac);
            let txt = chart.show_cell_labels.then(|| {
                let v = data.display_value(row, col, chart.normalization);
                match chart.normalization {
                    ConfusionNormalization::Absolute => format!("{count}"),
                    ConfusionNormalization::RowNormalized
                    | ConfusionNormalization::ColumnNormalized
                    | ConfusionNormalization::TotalNormalized => format!("{v:.1}%"),
                }
            });
            paint_box(painter, rect, fill, border, txt.as_deref(), diagnostics);
        }
    }

    if !chart.show_summaries {
        return;
    }
    let nf = n as f64;
    for row in 0..n {
        let recall = data.row_recall(row);
        for (sub_c, is_tpr) in [(0usize, true), (1usize, false)] {
            let x0 = nf + 0.75 + sub_c as f64 * 0.9;
            let lo = [x0, row as f64 + 0.5];
            let hi = [x0 + 0.9, row as f64 + 1.5];
            let Some(rect) = clipped_data_rect(lo, hi, transform) else {
                continue;
            };
            let frac = recall.map_or(0.0, |r| if is_tpr { r } else { 1.0 - r });
            let target = if is_tpr {
                chart.diagonal_color
            } else {
                chart.off_diagonal_color
            };
            let fill = blend_srgb(base, target, 0.08 + 0.85 * frac);
            let txt = chart
                .show_cell_labels
                .then(|| recall.map_or_else(|| "-".into(), |_| format!("{:.1}%", frac * 100.0)));
            paint_box(painter, rect, fill, border, txt.as_deref(), diagnostics);
        }
    }
    for col in 0..n {
        let prec = data.col_precision(col);
        for (sub_r, is_ppv) in [(0usize, true), (1usize, false)] {
            let y0 = nf + 0.75 + sub_r as f64 * 0.9;
            let lo = [col as f64 + 0.5, y0];
            let hi = [col as f64 + 1.5, y0 + 0.9];
            let Some(rect) = clipped_data_rect(lo, hi, transform) else {
                continue;
            };
            let frac = prec.map_or(0.0, |p| if is_ppv { p } else { 1.0 - p });
            let target = if is_ppv {
                chart.diagonal_color
            } else {
                chart.off_diagonal_color
            };
            let fill = blend_srgb(base, target, 0.08 + 0.85 * frac);
            let txt = chart
                .show_cell_labels
                .then(|| prec.map_or_else(|| "-".into(), |_| format!("{:.1}%", frac * 100.0)));
            paint_box(painter, rect, fill, border, txt.as_deref(), diagnostics);
        }
    }
    if let Some(corner) =
        clipped_data_rect([nf + 0.75, nf + 0.75], [nf + 2.55, nf + 2.55], transform)
    {
        painter.rect_stroke(corner, 0.0, border, StrokeKind::Inside);
        if corner.width() >= 34.0 && corner.height() >= 18.0 {
            let acc = data.accuracy() * 100.0;
            painter.text(
                corner.center(),
                Align2::CENTER_CENTER,
                format!("Acc\n{acc:.1}%"),
                FontId::proportional((corner.height() * 0.24).clamp(9.0, 12.0)),
                text_color,
            );
        }
    }
}
